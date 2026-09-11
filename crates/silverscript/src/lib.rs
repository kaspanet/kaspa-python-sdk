//! Python bindings for the SilverScript compiler (`kaspa.experimental.silverscript`).
//!
//! A separate extension module from the core `kaspa`, since SilverScript pins a different rusty-kaspa dep commit.

use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyByteArray, PyBytes, PyDict, PyInt, PyList, PyString, PyTuple};
use pyo3_stub_gen::define_stub_info_gatherer;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyfunction, gen_stub_pymethods};
use self_cell::self_cell;

use kaspa_python_sdk_core::create_py_exception;
use silverscript_abi::{
    ArtifactValue, CodecError, SilAbiArtifact, TypeArtifact,
    encode_contract_covenant_decl_sig_script, encode_contract_entry_sig_script,
};
use silverscript_lang::ast::{ContractAst, Expr, parse_contract_ast};
use silverscript_lang::compiler::{
    CompileOptions, CompiledContract, artifact_value_to_expr, compile_contract,
    sil_abi_artifact_from_compiled,
};
use silverscript_lang::errors::CompilerError;

pub mod debug;

create_py_exception!(
    /// Raised when SilverScript compilation or signature-script construction fails.
    PySilverScriptError,
    "SilverScriptError",
    "kaspa.experimental.silverscript"
);

pub(crate) fn map_err(err: CompilerError) -> PyErr {
    match err.span() {
        Some(span) => {
            PySilverScriptError::new_err(format!("{err} (at bytes {}..{})", span.start, span.end))
        }
        None => PySilverScriptError::new_err(err.to_string()),
    }
}

pub(crate) fn map_codec_err(err: CodecError) -> PyErr {
    PySilverScriptError::new_err(err.to_string())
}

/// Owned, `'static` form of a Python argument. Converted once, then lowered on
/// demand into an `ArtifactValue` (for sig-script encoding) or a typed `Expr`
/// (for constructor args) — sidesteps `CompiledContract<'i>` borrowing the
/// source. `Eq`/`Hash` let the debug harness memoize per-constructor-args work.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) enum Value {
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

pub(crate) fn py_to_value(obj: &Bound<'_, PyAny>) -> PyResult<Value> {
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

/// Build a portable ABI value from a `Value`.
pub(crate) fn value_to_artifact(value: &Value) -> ArtifactValue {
    match value {
        Value::Int(i) => ArtifactValue::Int(*i),
        Value::Bool(b) => ArtifactValue::Bool(*b),
        Value::Str(s) => ArtifactValue::Text(s.clone()),
        Value::Bytes(b) => ArtifactValue::Bytes(b.clone()),
        Value::List(items) => ArtifactValue::Array(items.iter().map(value_to_artifact).collect()),
        Value::Struct(fields) => ArtifactValue::Object(
            fields
                .iter()
                .map(|(name, value)| (name.clone(), value_to_artifact(value)))
                .collect(),
        ),
    }
}

/// Lower constructor arguments against the contract's declared parameter types.
///
/// Mirrors upstream's private `artifact_values_to_constructor_args`, which is
/// not exported — the per-argument `artifact_value_to_expr` is.
pub(crate) fn ctor_exprs_for<'i>(
    values: &[Value],
    contract: &ContractAst<'i>,
) -> PyResult<Vec<Expr<'i>>> {
    if values.len() != contract.params.len() {
        return Err(PySilverScriptError::new_err(format!(
            "constructor argument count mismatch: expected {}, got {}",
            contract.params.len(),
            values.len()
        )));
    }
    values
        .iter()
        .zip(&contract.params)
        .map(|(value, param)| {
            artifact_value_to_expr(&value_to_artifact(value), &param.type_ref, contract)
                .map_err(map_err)
        })
        .collect()
}

/// Render a portable ABI type as the SilverScript type name.
///
/// Upstream's equivalent helper is private, and the spelling is Python-visible
/// through `FunctionInputAbi.type_name`, so it is reproduced here.
pub(crate) fn artifact_type_name(ty: &TypeArtifact) -> String {
    match ty {
        TypeArtifact::Int => "int".to_string(),
        TypeArtifact::Temporal => "temporal".to_string(),
        TypeArtifact::Bool => "bool".to_string(),
        TypeArtifact::Byte => "byte".to_string(),
        TypeArtifact::Bytes => "bytes".to_string(),
        TypeArtifact::Text => "string".to_string(),
        TypeArtifact::Pubkey => "pubkey".to_string(),
        TypeArtifact::Sig => "sig".to_string(),
        TypeArtifact::Datasig => "datasig".to_string(),
        TypeArtifact::FixedBytes { len } => format!("byte[{len}]"),
        TypeArtifact::FixedArray { item, len } => format!("{}[{len}]", artifact_type_name(item)),
        TypeArtifact::DynamicArray { item } => format!("{}[]", artifact_type_name(item)),
        TypeArtifact::Struct { name } => name.clone(),
    }
}

/// Convert an optional Python `list`/`tuple` of argument values into `Value`s.
pub(crate) fn collect_args(obj: Option<&Bound<'_, PyAny>>) -> PyResult<Vec<Value>> {
    let Some(obj) = obj else {
        return Ok(Vec::new());
    };
    if let Ok(list) = obj.cast::<PyList>() {
        list.iter().map(|item| py_to_value(&item)).collect()
    } else if let Ok(tuple) = obj.cast::<PyTuple>() {
        tuple.iter().map(|item| py_to_value(&item)).collect()
    } else {
        Err(PySilverScriptError::new_err(
            "arguments must be a list or tuple",
        ))
    }
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

/// The native compile products that borrow the contract source.
///
/// The portable ABI artifact is built here, next to the contract, because it is
/// what every `build_sig_script*` call encodes against.
pub(crate) struct CompiledParts<'i> {
    pub(crate) contract: CompiledContract<'i>,
    pub(crate) artifact: SilAbiArtifact,
}

// Holds the contract source alongside the native compile products that borrow
// it. `CompiledContract<'i>` borrows the source and has no owned form, so this
// self-referential cell lets us compile once in `py_compile` and reuse the
// result for every `build_sig_script*` call instead of recompiling per call.
self_cell!(
    struct CompiledCell {
        owner: String,
        #[covariant]
        dependent: CompiledParts,
    }
);

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
    script: Vec<u8>,
    abi: Vec<PyFunctionAbiEntry>,
    state_layout: (usize, usize),
    // The native `CompiledContract` compiled once at construction and reused.
    compiled: CompiledCell,
}

impl PyCompiledContract {
    fn sig_script(
        &self,
        function_name: &str,
        args: Vec<Value>,
        covenant: Option<bool>,
    ) -> PyResult<Vec<u8>> {
        let call_args: Vec<ArtifactValue> = args.iter().map(value_to_artifact).collect();
        let artifact = &self.compiled.borrow_dependent().artifact;
        match covenant {
            None => encode_contract_entry_sig_script(
                artifact,
                &self.contract_name,
                function_name,
                &call_args,
            ),
            Some(is_leader) => encode_contract_covenant_decl_sig_script(
                artifact,
                &self.contract_name,
                function_name,
                is_leader,
                &call_args,
            ),
        }
        .map_err(map_codec_err)
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

    /// The canonical length-bound template hash: a 32-byte digest over the
    /// script's template parts (the prefix before and suffix after the state
    /// region). Matches the SilverScript `templateHash(prefix, suffix)` builtin,
    /// so contracts can commit to this value and later reconstruct it on-chain.
    #[getter]
    pub fn template_hash<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(
            py,
            &self.compiled.borrow_dependent().contract.template_hash(),
        )
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

/// Build the Python-facing ABI for `contract_name` from a portable ABI artifact.
///
/// The artifact keys entries in a `BTreeMap` (alphabetical), but the ABI is
/// ordered — `debug_call` picks `abi[0]` as the default entrypoint, and callers
/// read it as the contract's entrypoints in the order they were written. So
/// entries are emitted in source order, with any generated entrypoint that has
/// no source function (covenant lowering adds these) appended afterwards.
fn abi_entries(
    artifact: &SilAbiArtifact,
    contract_name: &str,
    ast: &ContractAst<'_>,
) -> Vec<PyFunctionAbiEntry> {
    let Some(contract) = artifact.contract(contract_name) else {
        return Vec::new();
    };
    let build = |name: &str| {
        contract.entries.get(name).map(|entry| PyFunctionAbiEntry {
            name: name.to_string(),
            inputs: entry
                .params
                .iter()
                .map(|param| PyFunctionInputAbi {
                    name: param.name.clone(),
                    type_name: artifact_type_name(&param.ty),
                })
                .collect(),
        })
    };

    let mut out: Vec<PyFunctionAbiEntry> = Vec::with_capacity(contract.entries.len());
    for function in &ast.functions {
        if let Some(entry) = build(&function.name) {
            out.push(entry);
        }
    }
    for name in contract.entries.keys() {
        if !out.iter().any(|entry| &entry.name == name)
            && let Some(entry) = build(name)
        {
            out.push(entry);
        }
    }
    out
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

    // Compile once and keep the native artifact (alongside the source it borrows)
    // so `build_sig_script*` can reuse it rather than recompiling the whole
    // contract on every call.
    let compiled = CompiledCell::try_new(source, |source| -> PyResult<CompiledParts<'_>> {
        let ast = parse_contract_ast(source).map_err(map_err)?;
        let ctor = ctor_exprs_for(&constructor_args, &ast)?;
        let contract = compile_contract(source, &ctor, options).map_err(map_err)?;
        let artifact = sil_abi_artifact_from_compiled(&contract, &ctor).map_err(map_err)?;
        Ok(CompiledParts { contract, artifact })
    })?;

    let (contract_name, compiler_version, script, abi, state_layout) = {
        let parts = compiled.borrow_dependent();
        let contract = &parts.contract;
        let contract_name = contract.contract_name.clone();
        (
            contract_name.clone(),
            contract.compiler_version.clone(),
            contract.bytecode.clone(),
            abi_entries(&parts.artifact, &contract_name, &contract.ast),
            (contract.state_layout.start, contract.state_layout.len),
        )
    };

    Ok(PyCompiledContract {
        contract_name,
        compiler_version,
        script,
        abi,
        state_layout,
        compiled,
    })
}

/// The `kaspa.experimental.silverscript` extension module.
#[pymodule]
fn silverscript(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_compile, m)?)?;
    m.add_class::<PyCompiledContract>()?;
    m.add_class::<PyFunctionAbiEntry>()?;
    m.add_class::<PyFunctionInputAbi>()?;
    m.add_function(wrap_pyfunction!(debug::py_debug_call, m)?)?;
    m.add_class::<debug::PyDebugCallResult>()?;
    m.add_class::<debug::PyFailureReport>()?;
    m.add_class::<debug::PyFailureFrame>()?;
    m.add_class::<debug::PyDebugVariable>()?;
    m.add_class::<debug::PyTraceStep>()?;
    m.add_class::<PySilverScriptError>()?;
    Ok(())
}

define_stub_info_gatherer!(stub_info);
