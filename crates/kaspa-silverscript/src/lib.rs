//! Python bindings for the SilverScript compiler (`kaspa.silverscript`).
//!
//! This is a separate extension module from the `kaspa` core: it links
//! rusty-kaspa @ tn12 (kaspa-txscript 1.1.1-toc.1) via `silverscript-lang`,
//! whereas the core links @ cfafeb4c0 (2.0.1). The two never share Rust types —
//! they interoperate purely through script `bytes`.
//!
//! Exposed surface (Tier 0 + Tier 1 of the design inventory):
//!   - `compile(source, constructor_args=[], *, allow_entrypoint_return, record_debug_infos) -> CompiledContract`
//!   - `CompiledContract.{contract_name, compiler_version, without_selector, script, abi, state_layout}`
//!   - `CompiledContract.build_sig_script(function_name, args=[]) -> bytes`
//!   - `CompiledContract.build_sig_script_for_covenant_decl(function_name, args=[], *, is_leader=False) -> bytes`
//!   - `FunctionAbiEntry`, `FunctionInputAbi`, `SilverScriptError`

use pyo3::prelude::*;
use pyo3::types::{PyBool, PyByteArray, PyBytes, PyDict, PyInt, PyList, PyString, PyTuple};
use pyo3_stub_gen::define_stub_info_gatherer;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyfunction, gen_stub_pymethods};

use silverscript_lang::ast::{Expr, ExprKind, StateFieldExpr};
use silverscript_lang::compiler::{CompileOptions, CovenantDeclCallOptions, compile_contract};
use silverscript_lang::errors::CompilerError;

// abi3 (the limited API) can't subclass a native exception via `#[pyclass]`, so
// use `create_exception!` (C-API based, works under abi3 and the full API alike).
// It isn't captured by pyo3-stub-gen, so the `stub-gen` bin appends it to the
// generated `.pyi`.
pyo3::create_exception!(
    silverscript,
    SilverScriptError,
    pyo3::exceptions::PyException,
    "Raised when SilverScript compilation or signature-script construction fails."
);

fn map_err(err: CompilerError) -> PyErr {
    match err.span() {
        Some(span) => {
            SilverScriptError::new_err(format!("{err} (at bytes {}..{})", span.start, span.end))
        }
        None => SilverScriptError::new_err(err.to_string()),
    }
}

/// Owned, `'static` representation of a Python argument value. We convert
/// Python → this once, store it, and rebuild `Expr`s from it on demand — which
/// sidesteps `CompiledContract<'i>` borrowing the source string.
#[derive(Clone)]
enum Value {
    Int(i64),
    Bool(bool),
    Str(String),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    Struct(Vec<(String, Value)>),
}

fn py_to_value(obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    // bool must precede int: in Python, bool is a subclass of int.
    if obj.cast::<PyBool>().is_ok() {
        return Ok(Value::Bool(obj.extract::<bool>()?));
    }
    if obj.cast::<PyInt>().is_ok() {
        return Ok(Value::Int(obj.extract::<i64>()?));
    }
    if let Ok(s) = obj.cast::<PyString>() {
        return Ok(Value::Str(s.to_str()?.to_owned()));
    }
    if let Ok(b) = obj.cast::<PyBytes>() {
        return Ok(Value::Bytes(b.as_bytes().to_vec()));
    }
    if let Ok(b) = obj.cast::<PyByteArray>() {
        return Ok(Value::Bytes(b.to_vec()));
    }
    if let Ok(list) = obj.cast::<PyList>() {
        let mut items = Vec::with_capacity(list.len());
        for item in list.iter() {
            items.push(py_to_value(&item)?);
        }
        return Ok(Value::List(items));
    }
    if let Ok(tuple) = obj.cast::<PyTuple>() {
        let mut items = Vec::with_capacity(tuple.len());
        for item in tuple.iter() {
            items.push(py_to_value(&item)?);
        }
        return Ok(Value::List(items));
    }
    if let Ok(dict) = obj.cast::<PyDict>() {
        let mut fields = Vec::with_capacity(dict.len());
        for (key, value) in dict.iter() {
            let key = key
                .cast::<PyString>()
                .map_err(|_| SilverScriptError::new_err("struct argument keys must be strings"))?;
            fields.push((key.to_str()?.to_owned(), py_to_value(&value)?));
        }
        return Ok(Value::Struct(fields));
    }
    Err(SilverScriptError::new_err(
        "unsupported argument type (expected int, bool, str, bytes, list/tuple, or dict)",
    ))
}

/// Build an owned (`'static`) literal `Expr` — no borrows into the source, so
/// the result is freely usable as a constructor or call argument.
fn value_to_expr(value: &Value) -> Expr<'static> {
    match value {
        Value::Int(i) => Expr::int(*i),
        Value::Bool(b) => Expr::bool(*b),
        Value::Str(s) => Expr::string(s.clone()),
        Value::Bytes(b) => Expr::bytes(b.clone()),
        Value::List(items) => {
            let exprs: Vec<Expr<'static>> = items.iter().map(value_to_expr).collect();
            Expr::from(exprs)
        }
        Value::Struct(fields) => {
            let entries = fields
                .iter()
                .map(|(name, value)| StateFieldExpr {
                    name: name.clone(),
                    expr: value_to_expr(value),
                    span: Default::default(),
                    name_span: Default::default(),
                })
                .collect();
            Expr::new(ExprKind::StateObject(entries), Default::default())
        }
    }
}

/// Convert an optional Python `list`/`tuple` of argument values into `Value`s.
fn collect_args(obj: Option<&Bound<'_, PyAny>>) -> PyResult<Vec<Value>> {
    let Some(obj) = obj else {
        return Ok(Vec::new());
    };
    let items: Vec<Bound<'_, PyAny>> = if let Ok(list) = obj.cast::<PyList>() {
        list.iter().collect()
    } else if let Ok(tuple) = obj.cast::<PyTuple>() {
        tuple.iter().collect()
    } else {
        return Err(SilverScriptError::new_err(
            "arguments must be a list or tuple",
        ));
    };
    items.iter().map(py_to_value).collect()
}

/// A single input parameter of a contract entrypoint.
#[gen_stub_pyclass]
#[pyclass(name = "FunctionInputAbi", module = "kaspa.silverscript", frozen)]
#[derive(Clone)]
struct FunctionInputAbi {
    #[pyo3(get)]
    name: String,
    /// SilverScript type, e.g. `"int"`, `"byte[32]"`, `"pubkey"`.
    #[pyo3(get)]
    type_name: String,
}

#[gen_stub_pymethods]
#[pymethods]
impl FunctionInputAbi {
    fn __repr__(&self) -> String {
        format!(
            "FunctionInputAbi(name={:?}, type_name={:?})",
            self.name, self.type_name
        )
    }
}

/// A single callable entrypoint in a compiled contract's ABI.
#[gen_stub_pyclass]
#[pyclass(name = "FunctionAbiEntry", module = "kaspa.silverscript", frozen)]
#[derive(Clone)]
struct FunctionAbiEntry {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    inputs: Vec<FunctionInputAbi>,
}

#[gen_stub_pymethods]
#[pymethods]
impl FunctionAbiEntry {
    fn __repr__(&self) -> String {
        format!(
            "FunctionAbiEntry(name={:?}, inputs={} input(s))",
            self.name,
            self.inputs.len()
        )
    }
}

/// A compiled SilverScript contract: the locking script plus the metadata
/// needed to build unlocking (signature) scripts for its entrypoints.
#[gen_stub_pyclass]
#[pyclass(name = "CompiledContract", module = "kaspa.silverscript", frozen)]
struct CompiledContract {
    contract_name: String,
    compiler_version: String,
    without_selector: bool,
    script: Vec<u8>,
    abi: Vec<FunctionAbiEntry>,
    state_layout: (usize, usize),
    // Retained so `build_sig_script*` can recompile (the native
    // `CompiledContract` borrows the source string and can't be stored).
    source: String,
    constructor_args: Vec<Value>,
    options: CompileOptions,
}

impl CompiledContract {
    fn sig_script(
        &self,
        function_name: &str,
        args: Vec<Value>,
        covenant: Option<bool>,
    ) -> PyResult<Vec<u8>> {
        let ctor: Vec<Expr<'static>> = self.constructor_args.iter().map(value_to_expr).collect();
        let compiled = compile_contract(&self.source, &ctor, self.options).map_err(map_err)?;
        let call_args: Vec<Expr<'static>> = args.iter().map(value_to_expr).collect();
        let bytes = match covenant {
            None => compiled.build_sig_script(function_name, call_args),
            Some(is_leader) => compiled.build_sig_script_for_covenant_decl(
                function_name,
                call_args,
                CovenantDeclCallOptions { is_leader },
            ),
        }
        .map_err(map_err)?;
        Ok(bytes)
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl CompiledContract {
    /// The contract name from the SilverScript source.
    #[getter]
    fn contract_name(&self) -> &str {
        &self.contract_name
    }

    /// The compiler version that produced this contract.
    #[getter]
    fn compiler_version(&self) -> &str {
        &self.compiler_version
    }

    /// Whether the contract has a single entrypoint (no function selector).
    #[getter]
    fn without_selector(&self) -> bool {
        self.without_selector
    }

    /// The compiled locking script (redeem script) bytes.
    #[getter]
    fn script<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.script)
    }

    /// The contract ABI: one entry per callable entrypoint.
    #[getter]
    fn abi(&self) -> Vec<FunctionAbiEntry> {
        self.abi.clone()
    }

    /// `(start, len)` byte offsets of the contract state within the script.
    #[getter]
    fn state_layout(&self) -> (usize, usize) {
        self.state_layout
    }

    /// Build the signature (unlocking) script for `function_name`.
    ///
    /// `args` are native Python values (int/bool/str/bytes/list/dict) matching
    /// the function's ABI input types.
    #[pyo3(signature = (function_name, args=None))]
    fn build_sig_script<'py>(
        &self,
        py: Python<'py>,
        function_name: &str,
        args: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let args = collect_args(args.as_ref())?;
        let bytes = self.sig_script(function_name, args, None)?;
        Ok(PyBytes::new(py, &bytes))
    }

    /// Build the signature script for a covenant declaration entrypoint.
    #[pyo3(signature = (function_name, args=None, *, is_leader=false))]
    fn build_sig_script_for_covenant_decl<'py>(
        &self,
        py: Python<'py>,
        function_name: &str,
        args: Option<Bound<'py, PyAny>>,
        is_leader: bool,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let args = collect_args(args.as_ref())?;
        let bytes = self.sig_script(function_name, args, Some(is_leader))?;
        Ok(PyBytes::new(py, &bytes))
    }

    fn __repr__(&self) -> String {
        format!(
            "CompiledContract(name={:?}, script={} bytes, entrypoints={})",
            self.contract_name,
            self.script.len(),
            self.abi.len()
        )
    }
}

/// Compile SilverScript `source` into a `CompiledContract`.
///
/// `constructor_args` are native Python values matching the contract's
/// constructor parameters.
#[gen_stub_pyfunction(module = "kaspa.silverscript")]
#[pyfunction]
#[pyo3(signature = (source, constructor_args=None, *, allow_entrypoint_return=false, record_debug_infos=false))]
fn compile(
    source: String,
    constructor_args: Option<Bound<'_, PyAny>>,
    allow_entrypoint_return: bool,
    record_debug_infos: bool,
) -> PyResult<CompiledContract> {
    let constructor_args = collect_args(constructor_args.as_ref())?;
    let options = CompileOptions {
        allow_entrypoint_return,
        record_debug_infos,
    };

    // Compile once to extract owned metadata, then drop the borrowing native
    // artifact before moving `source`/`constructor_args` into the pyclass.
    let (contract_name, compiler_version, without_selector, script, abi, state_layout) = {
        let ctor: Vec<Expr<'static>> = constructor_args.iter().map(value_to_expr).collect();
        let compiled = compile_contract(&source, &ctor, options).map_err(map_err)?;
        let abi = compiled
            .abi
            .iter()
            .map(|entry| FunctionAbiEntry {
                name: entry.name.clone(),
                inputs: entry
                    .inputs
                    .iter()
                    .map(|input| FunctionInputAbi {
                        name: input.name.clone(),
                        type_name: input.type_name.clone(),
                    })
                    .collect(),
            })
            .collect();
        (
            compiled.contract_name.clone(),
            compiled.compiler_version.clone(),
            compiled.without_selector,
            compiled.script.clone(),
            abi,
            (compiled.state_layout.start, compiled.state_layout.len),
        )
    };

    Ok(CompiledContract {
        contract_name,
        compiler_version,
        without_selector,
        script,
        abi,
        state_layout,
        source,
        constructor_args,
        options,
    })
}

#[pymodule]
fn silverscript(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    m.add_class::<CompiledContract>()?;
    m.add_class::<FunctionAbiEntry>()?;
    m.add_class::<FunctionInputAbi>()?;
    m.add("SilverScriptError", m.py().get_type::<SilverScriptError>())?;
    Ok(())
}

define_stub_info_gatherer!(stub_info);
