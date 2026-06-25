//! Python bindings for the SilverScript compiler (`kaspa.experimental.silverscript`).
//!
//! A separate extension module from the core `kaspa`, since SilverScript pins a different rusty-kaspa dep commit.

use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyByteArray, PyBytes, PyDict, PyInt, PyList, PyString, PyTuple};
use pyo3_stub_gen::define_stub_info_gatherer;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyfunction, gen_stub_pymethods};

use kaspa_python_sdk_core::create_py_exception;
use silverscript_lang::ast::{Expr, ExprKind, StateFieldExpr};
use silverscript_lang::compiler::{CompileOptions, CovenantDeclCallOptions, compile_contract};
use silverscript_lang::errors::CompilerError;

// Shared `create_py_exception!` macro: `#[pyclass(extends = PyException)]` so
// pyo3-stub-gen captures it in the `.pyi` automatically.
create_py_exception!(
    /// Raised when SilverScript compilation or signature-script construction fails.
    PySilverScriptError,
    "SilverScriptError",
    "kaspa.experimental.silverscript"
);

fn map_err(err: CompilerError) -> PyErr {
    match err.span() {
        Some(span) => {
            PySilverScriptError::new_err(format!("{err} (at bytes {}..{})", span.start, span.end))
        }
        None => PySilverScriptError::new_err(err.to_string()),
    }
}

/// Owned, `'static` form of a Python argument. Converted once, then rebuilt into
/// `Expr`s on demand — sidesteps `CompiledContract<'i>` borrowing the source.
#[derive(Clone)]
enum Value {
    Int(i64),
    Bool(bool),
    Str(String),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    Struct(Vec<(String, Value)>),
}

/// Max argument nesting depth. Bounds `py_to_value` recursion so a deeply nested
/// value raises `SilverScriptError` instead of overflowing the native stack.
const MAX_ARG_DEPTH: usize = 128;

fn py_to_value(obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    py_to_value_at(obj, 0)
}

fn py_to_value_at(obj: &Bound<'_, PyAny>, depth: usize) -> PyResult<Value> {
    if depth >= MAX_ARG_DEPTH {
        return Err(PySilverScriptError::new_err(format!(
            "argument nesting too deep (exceeds {MAX_ARG_DEPTH} levels)"
        )));
    }
    // bool must precede int: in Python, bool is a subclass of int.
    if obj.cast::<PyBool>().is_ok() {
        return Ok(Value::Bool(obj.extract::<bool>()?));
    }
    if obj.cast::<PyInt>().is_ok() {
        // Remap pyo3's OverflowError so callers only ever see SilverScriptError.
        let int = obj.extract::<i64>().map_err(|_| {
            PySilverScriptError::new_err(
                "integer argument out of range (must fit in a signed 64-bit integer)",
            )
        })?;
        return Ok(Value::Int(int));
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
            items.push(py_to_value_at(&item, depth + 1)?);
        }
        return Ok(Value::List(items));
    }
    if let Ok(tuple) = obj.cast::<PyTuple>() {
        let mut items = Vec::with_capacity(tuple.len());
        for item in tuple.iter() {
            items.push(py_to_value_at(&item, depth + 1)?);
        }
        return Ok(Value::List(items));
    }
    if let Ok(dict) = obj.cast::<PyDict>() {
        let mut fields = Vec::with_capacity(dict.len());
        for (key, value) in dict.iter() {
            let key = key.cast::<PyString>().map_err(|_| {
                PySilverScriptError::new_err("struct argument keys must be strings")
            })?;
            fields.push((key.to_str()?.to_owned(), py_to_value_at(&value, depth + 1)?));
        }
        return Ok(Value::Struct(fields));
    }
    Err(PySilverScriptError::new_err(
        "unsupported argument type (expected int, bool, str, bytes, list/tuple, or dict)",
    ))
}

/// Build an owned (`'static`) literal `Expr` from a `Value` — no source borrows.
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
        return Err(PySilverScriptError::new_err(
            "arguments must be a list or tuple",
        ));
    };
    items.iter().map(py_to_value).collect()
}

/// A single input parameter of a contract entrypoint.
#[gen_stub_pyclass]
#[pyclass(
    name = "FunctionInputAbi",
    module = "kaspa.experimental.silverscript",
    frozen
)]
#[derive(Clone)]
pub struct PyFunctionInputAbi {
    #[pyo3(get)]
    name: String,
    /// SilverScript type, e.g. `"int"`, `"byte[32]"`, `"pubkey"`.
    #[pyo3(get)]
    type_name: String,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyFunctionInputAbi {
    pub fn __repr__(&self) -> String {
        format!(
            "FunctionInputAbi(name={:?}, type_name={:?})",
            self.name, self.type_name
        )
    }
}

/// A single callable entrypoint in a compiled contract's ABI.
#[gen_stub_pyclass]
#[pyclass(
    name = "FunctionAbiEntry",
    module = "kaspa.experimental.silverscript",
    frozen
)]
#[derive(Clone)]
pub struct PyFunctionAbiEntry {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    inputs: Vec<PyFunctionInputAbi>,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyFunctionAbiEntry {
    pub fn __repr__(&self) -> String {
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
#[pyclass(
    name = "CompiledContract",
    module = "kaspa.experimental.silverscript",
    frozen
)]
pub struct PyCompiledContract {
    contract_name: String,
    compiler_version: String,
    without_selector: bool,
    script: Vec<u8>,
    abi: Vec<PyFunctionAbiEntry>,
    state_layout: (usize, usize),
    // Retained so `build_sig_script*` can recompile (the native
    // `CompiledContract` borrows the source string and can't be stored).
    source: String,
    constructor_args: Vec<Value>,
    options: CompileOptions,
}

impl PyCompiledContract {
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
impl PyCompiledContract {
    /// The contract name from the SilverScript source.
    #[getter]
    pub fn contract_name(&self) -> &str {
        &self.contract_name
    }

    /// The compiler version that produced this contract.
    #[getter]
    pub fn compiler_version(&self) -> &str {
        &self.compiler_version
    }

    /// Whether the contract has a single entrypoint (no function selector).
    #[getter]
    pub fn without_selector(&self) -> bool {
        self.without_selector
    }

    /// The compiled locking script (redeem script) bytes.
    #[getter]
    pub fn script<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.script)
    }

    /// The contract ABI: one entry per callable entrypoint.
    #[getter]
    pub fn abi(&self) -> Vec<PyFunctionAbiEntry> {
        self.abi.clone()
    }

    /// `(start, len)`: byte offset and length of the contract state within the script.
    #[getter]
    pub fn state_layout(&self) -> (usize, usize) {
        self.state_layout
    }

    /// Build the signature (unlocking) script for an entrypoint.
    ///
    /// Args:
    ///     function_name: The entrypoint to call.
    ///     args: Native Python values (int, bool, str, bytes, list/tuple, or
    ///         dict) matching the entrypoint's ABI input types. Omit or pass
    ///         None for an entrypoint that takes no arguments.
    ///
    /// Returns:
    ///     bytes: The signature (unlocking) script.
    ///
    /// Raises:
    ///     SilverScriptError: If the entrypoint is unknown or an argument is
    ///         invalid (wrong type, out of range, or too deeply nested).
    #[pyo3(signature = (function_name, args=None))]
    pub fn build_sig_script<'py>(
        &self,
        py: Python<'py>,
        function_name: &str,
        args: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let args = collect_args(args.as_ref())?;
        let bytes = self.sig_script(function_name, args, None)?;
        Ok(PyBytes::new(py, &bytes))
    }

    /// Build the signature (unlocking) script for a covenant declaration entrypoint.
    ///
    /// Args:
    ///     function_name: The covenant entrypoint to call.
    ///     args: Native Python values matching the entrypoint's ABI input
    ///         types. Omit or pass None for an entrypoint that takes no
    ///         arguments.
    ///     is_leader: Select the leader path for covenants that distinguish a
    ///         leader from delegates (default: False).
    ///
    /// Returns:
    ///     bytes: The signature (unlocking) script.
    ///
    /// Raises:
    ///     SilverScriptError: If the entrypoint is unknown or an argument is
    ///         invalid (wrong type, out of range, or too deeply nested).
    #[pyo3(signature = (function_name, args=None, *, is_leader=false))]
    pub fn build_sig_script_for_covenant_decl<'py>(
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

    pub fn __repr__(&self) -> String {
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
/// **Experimental:** SilverScript and these bindings are under active
/// development; the API and the compiler's output may change in breaking ways
/// between releases. See the `kaspa.experimental.silverscript` module docs.
///
/// Args:
///     source: The SilverScript contract source.
///     constructor_args: Native Python values matching the contract's
///         constructor parameters. Omit or pass None for a contract with no
///         constructor parameters.
///     allow_entrypoint_return: Permit entrypoints that return a value
///         (default: False).
///     record_debug_infos: Record debug information during compilation
///         (default: False).
///
/// Returns:
///     CompiledContract: The compiled contract.
///
/// Raises:
///     SilverScriptError: If compilation fails (syntax error, type error, or
///         incompatible pragma).
#[gen_stub_pyfunction(module = "kaspa.experimental.silverscript")]
#[pyfunction]
#[pyo3(name = "compile")]
#[pyo3(signature = (source, constructor_args=None, *, allow_entrypoint_return=false, record_debug_infos=false))]
pub fn py_compile(
    source: String,
    constructor_args: Option<Bound<'_, PyAny>>,
    allow_entrypoint_return: bool,
    record_debug_infos: bool,
) -> PyResult<PyCompiledContract> {
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
            .map(|entry| PyFunctionAbiEntry {
                name: entry.name.clone(),
                inputs: entry
                    .inputs
                    .iter()
                    .map(|input| PyFunctionInputAbi {
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

    Ok(PyCompiledContract {
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

/// The `kaspa.experimental.silverscript` extension module. Compiles SilverScript
/// to script bytes; see the package docstring for the experimental-API caveats.
#[pymodule]
fn silverscript(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_compile, m)?)?;
    m.add_class::<PyCompiledContract>()?;
    m.add_class::<PyFunctionAbiEntry>()?;
    m.add_class::<PyFunctionInputAbi>()?;
    m.add_class::<PySilverScriptError>()?;
    Ok(())
}

define_stub_info_gatherer!(stub_info);
