//! Python bindings for the SilverScript compiler (`kaspa.experimental.silverscript`).
//!
//! A separate extension module from the core `kaspa`, since SilverScript pins a different rusty-kaspa dep commit.

use std::collections::BTreeMap;

use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyByteArray, PyBytes, PyDict, PyInt, PyList, PyString, PyTuple};
use pyo3_stub_gen::define_stub_info_gatherer;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyfunction, gen_stub_pymethods};
use self_cell::self_cell;

use kaspa_python_sdk_core::create_py_exception;
use silverscript_abi::{
    ArtifactValue, CodecError, SilAbiArtifact, SilContractArtifact, SilEntryArtifact, TypeArtifact,
    encode_contract_covenant_decl_sig_script, encode_contract_entry_sig_script, encode_hex,
    to_pretty_json,
};
use silverscript_lang::ast::{
    ContractAst, Expr, STATE_TYPE_NAME, TypeBase, TypeRef, parse_contract_ast,
};
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

/// Build portable ABI values with no declared types to narrow against.
pub(crate) fn untyped_artifacts(values: &[Value]) -> Vec<ArtifactValue> {
    values.iter().map(value_to_artifact).collect()
}

/// Narrow a value to a portable ABI `byte`: an `int` in `0..=255`, or the
/// one-byte `bytes` a `byte` decodes back out of the debugger as.
fn byte_artifact(value: &Value) -> PyResult<ArtifactValue> {
    match value {
        Value::Int(int) => u8::try_from(*int).map(ArtifactValue::Byte).map_err(|_| {
            PySilverScriptError::new_err(format!("byte expects value in 0..=255, got {int}"))
        }),
        Value::Bytes(bytes) if bytes.len() == 1 => Ok(ArtifactValue::Byte(bytes[0])),
        other => Ok(value_to_artifact(other)),
    }
}

/// The declared types a call's arguments are narrowed against.
///
/// The codec is variant-strict — a `byte` parameter takes only
/// `ArtifactValue::Byte`, which no Python value maps to on its own — so the
/// narrowing is directed by the declared type, never by the value.
pub(crate) struct ArgTypes<'a> {
    artifact: &'a SilAbiArtifact,
    contract: &'a SilContractArtifact,
    contract_name: &'a str,
}

impl<'a> ArgTypes<'a> {
    /// Resolve `contract_name`'s declared types within an artifact.
    pub(crate) fn new(artifact: &'a SilAbiArtifact, contract_name: &'a str) -> Option<Self> {
        artifact.contract(contract_name).map(|contract| Self {
            artifact,
            contract,
            contract_name,
        })
    }

    pub(crate) fn contract(&self) -> &'a SilContractArtifact {
        self.contract
    }

    /// Resolve a covenant declaration to the entry the codec encodes it as.
    ///
    /// On the cov-bound follower path the codec returns `delegate_entry_abi`
    /// without consulting `cov_decl_to_abi` (`silverscript-abi/src/lib.rs:470`),
    /// so an unknown name would encode a well-formed delegate script instead of
    /// failing. Check it here, with the error the leader path already raises.
    pub(crate) fn covenant_decl_entry(
        &self,
        name: &str,
        is_leader: bool,
    ) -> PyResult<Option<&'a SilEntryArtifact>> {
        if !self.contract.cov_decl_to_abi.contains_key(name) {
            return Err(map_codec_err(CodecError::UnknownEntry {
                contract: self.contract_name.to_string(),
                entry: name.to_string(),
            }));
        }
        Ok(self.contract.covenant_decl_entry(name, is_leader))
    }

    /// Narrow a call's arguments against the entry the codec encodes them
    /// with. An unresolved entry or wrong argument count is left untyped for
    /// the codec to report.
    pub(crate) fn lower_call(
        &self,
        values: &[Value],
        entry: Option<&SilEntryArtifact>,
    ) -> PyResult<Vec<ArtifactValue>> {
        match entry {
            Some(entry) if entry.params.len() == values.len() => values
                .iter()
                .zip(&entry.params)
                .map(|(value, param)| self.lower(value, &param.ty))
                .collect(),
            _ => Ok(untyped_artifacts(values)),
        }
    }

    /// Narrow one value against its declared type. Anything the untyped
    /// conversion already gets right is left for the codec to check.
    pub(crate) fn lower(&self, value: &Value, ty: &TypeArtifact) -> PyResult<ArtifactValue> {
        match ty {
            TypeArtifact::Byte => byte_artifact(value),
            TypeArtifact::FixedArray { item, .. } | TypeArtifact::DynamicArray { item } => {
                let Value::List(items) = value else {
                    return Ok(value_to_artifact(value));
                };
                items
                    .iter()
                    .map(|item_value| self.lower(item_value, item))
                    .collect::<PyResult<Vec<_>>>()
                    .map(ArtifactValue::Array)
            }
            TypeArtifact::Struct { name } => {
                let (Value::Struct(entries), Some(fields)) = (value, self.struct_fields(name))
                else {
                    return Ok(value_to_artifact(value));
                };
                entries
                    .iter()
                    .map(|(field_name, field_value)| {
                        // Unknown fields are left for the codec to name.
                        let value = match fields.iter().find(|(name, _)| name == field_name) {
                            Some((_, ty)) => self.lower(field_value, ty)?,
                            None => value_to_artifact(field_value),
                        };
                        Ok((field_name.clone(), value))
                    })
                    .collect::<PyResult<BTreeMap<_, _>>>()
                    .map(ArtifactValue::Object)
            }
            _ => Ok(value_to_artifact(value)),
        }
    }

    /// A struct type's declared fields, resolved as the codec resolves them:
    /// `State` is the runtime state, anything else a declared struct.
    fn struct_fields(&self, name: &str) -> Option<Vec<(&'a str, &'a TypeArtifact)>> {
        if name == STATE_TYPE_NAME {
            return Some(
                self.contract
                    .runtime_state
                    .fields
                    .iter()
                    .map(|field| (field.name.as_str(), &field.ty))
                    .collect(),
            );
        }
        self.artifact.structs.get(name).map(|declared| {
            declared
                .fields
                .iter()
                .map(|field| (field.name.as_str(), &field.ty))
                .collect()
        })
    }
}

/// Narrow a constructor argument against its declared source-level type.
///
/// Mirrors the type walk in `artifact_value_to_expr`, which lowers these
/// against the declared `TypeRef` rather than the portable ABI's `TypeArtifact`.
fn ctor_artifact_for(
    value: &Value,
    type_ref: &TypeRef,
    contract: &ContractAst<'_>,
) -> PyResult<ArtifactValue> {
    if type_ref.is_array() {
        // A one-dimensional `byte[]`/`byte[N]` is `bytes`, not an array.
        if matches!(type_ref.base, TypeBase::Byte) && type_ref.array_dims.len() == 1 {
            return Ok(value_to_artifact(value));
        }
        let (Value::List(items), Some(element_type)) = (value, type_ref.array_element_type())
        else {
            return Ok(value_to_artifact(value));
        };
        return items
            .iter()
            .map(|item| ctor_artifact_for(item, &element_type, contract))
            .collect::<PyResult<Vec<_>>>()
            .map(ArtifactValue::Array);
    }
    match (&type_ref.base, value) {
        (TypeBase::Byte, _) => byte_artifact(value),
        (TypeBase::Custom(name), Value::Struct(entries)) => {
            let Some(declared) = contract.structs.iter().find(|item| item.name == *name) else {
                return Ok(value_to_artifact(value));
            };
            entries
                .iter()
                .map(|(field_name, field_value)| {
                    let declared_field = declared
                        .fields
                        .iter()
                        .find(|field| field.name == *field_name);
                    let value = match declared_field {
                        Some(field) => ctor_artifact_for(field_value, &field.type_ref, contract)?,
                        None => value_to_artifact(field_value),
                    };
                    Ok((field_name.clone(), value))
                })
                .collect::<PyResult<BTreeMap<_, _>>>()
                .map(ArtifactValue::Object)
        }
        _ => Ok(value_to_artifact(value)),
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
            let value = ctor_artifact_for(value, &param.type_ref, contract)?;
            artifact_value_to_expr(&value, &param.type_ref, contract).map_err(map_err)
        })
        .collect()
}

/// Render a portable ABI type as the SilverScript type name.
pub(crate) fn artifact_type_name(ty: &TypeArtifact) -> String {
    match ty {
        TypeArtifact::Int => "int".to_string(),
        TypeArtifact::Temporal => "temporal".to_string(),
        TypeArtifact::Bool => "bool".to_string(),
        TypeArtifact::Byte => "byte".to_string(),
        TypeArtifact::Bytes => "byte[]".to_string(),
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

/// A single parameter of a contract entrypoint.
#[gen_stub_pyclass]
#[pyclass(name = "ParamAbi", module = "kaspa.experimental.silverscript", frozen)]
#[derive(Clone)]
pub struct PyParamAbi {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    type_name: String,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyParamAbi {
    pub fn __repr__(&self) -> String {
        format!(
            "ParamAbi(name={:?}, type_name={:?})",
            self.name, self.type_name
        )
    }
}

/// A single callable entrypoint in a compiled contract's ABI.
#[gen_stub_pyclass]
#[pyclass(name = "EntryAbi", module = "kaspa.experimental.silverscript", frozen)]
#[derive(Clone)]
pub struct PyEntryAbi {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    params: Vec<PyParamAbi>,
    // Not `#[pyo3(get)]`: a `[u8; 4]` getter would surface as a tuple of ints.
    dispatch_tag: [u8; 4],
}

#[gen_stub_pymethods]
#[pymethods]
impl PyEntryAbi {
    /// The entrypoint's four-byte dispatch tag.
    ///
    /// `blake3("name(type,type)")[:4]` — content-addressed, so it depends only
    /// on the entrypoint's name and parameter types, never on constructor
    /// arguments. Every signature script built for this entrypoint ends with
    /// this value as its final data push.
    #[getter]
    pub fn dispatch_tag<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.dispatch_tag)
    }

    pub fn __repr__(&self) -> String {
        format!(
            "EntryAbi(name={:?}, params={}, dispatch_tag={:?})",
            self.name,
            self.params.len(),
            encode_hex(&self.dispatch_tag)
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
    abi: Vec<PyEntryAbi>,
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
        let artifact = &self.compiled.borrow_dependent().artifact;
        let call_args = match ArgTypes::new(artifact, &self.contract_name) {
            Some(types) => {
                let entry = match covenant {
                    None => types.contract().entry(function_name),
                    Some(is_leader) => types.covenant_decl_entry(function_name, is_leader)?,
                };
                types.lower_call(&args, entry)?
            }
            None => untyped_artifacts(&args),
        };
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
    pub fn bytecode<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.compiled.borrow_dependent().contract.bytecode)
    }

    /// The contract ABI: one entry per callable entrypoint, in source order.
    #[getter]
    pub fn abi(&self) -> Vec<PyEntryAbi> {
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

    /// Serialize the contract's portable ABI artifact as JSON.
    ///
    /// The artifact is built once by `compile` and is what `build_sig_script`
    /// encodes against. For the same source and constructor arguments this is
    /// identical to what the upstream `silverc` compiler writes, so artifacts
    /// are interchangeable between the two.
    ///
    /// Returns:
    ///     str: The portable artifact as pretty-printed JSON. Pass it to
    ///         `json.loads` for a dict.
    ///
    /// Raises:
    ///     SilverScriptError: If the artifact cannot be serialized.
    pub fn artifact_json(&self) -> PyResult<String> {
        to_pretty_json(&self.compiled.borrow_dependent().artifact)
            .map_err(|err| PySilverScriptError::new_err(err.to_string()))
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
            "CompiledContract(name={:?}, bytecode={} bytes, entries={})",
            self.contract_name,
            self.compiled.borrow_dependent().contract.bytecode.len(),
            self.abi.len()
        )
    }
}

/// Build the Python-facing ABI for `contract_name` from a portable ABI artifact.
fn abi_entries(
    artifact: &SilAbiArtifact,
    contract_name: &str,
    ast: &ContractAst<'_>,
) -> Vec<PyEntryAbi> {
    let Some(contract) = artifact.contract(contract_name) else {
        return Vec::new();
    };
    let build = |name: &str| {
        contract.entries.get(name).map(|entry| PyEntryAbi {
            name: name.to_string(),
            params: entry
                .params
                .iter()
                .map(|param| PyParamAbi {
                    name: param.name.clone(),
                    type_name: artifact_type_name(&param.ty),
                })
                .collect(),
            dispatch_tag: entry.dispatch_tag.into_bytes(),
        })
    };

    let mut out: Vec<PyEntryAbi> = Vec::with_capacity(contract.entries.len());
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

    let (contract_name, compiler_version, abi, state_layout) = {
        let parts = compiled.borrow_dependent();
        let contract = &parts.contract;
        let contract_name = contract.contract_name.clone();
        (
            contract_name.clone(),
            contract.compiler_version.clone(),
            abi_entries(&parts.artifact, &contract_name, &contract.ast),
            (contract.state_layout.start, contract.state_layout.len),
        )
    };

    Ok(PyCompiledContract {
        contract_name,
        compiler_version,
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
    m.add_class::<PyEntryAbi>()?;
    m.add_class::<PyParamAbi>()?;
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
