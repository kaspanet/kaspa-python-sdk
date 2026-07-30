//! Source-level contract call debugging (`debug_call`).
//!
//! Simulates a full transaction spend through SilverScript's `DebugSession` —
//! the engine behind the upstream CLI debugger — and reports the outcome as a
//! plain result object: pass/fail, a source-level failure report with decoded
//! variables per frame, and captured `console.log` output. Unlike the upstream
//! interactive debugger there is no stepping or breakpoint surface: a Python
//! script is not an IDE, so the session always runs to completion.
//!
//! The transaction harness is a port of the upstream CLI debugger's scenario
//! builder (`debugger/cli/src/main.rs`), adapted to native Python values in
//! place of raw CLI argument strings.

use std::collections::HashMap;

use debugger_session::FailureReport;
use debugger_session::args::parse_hex_bytes;
use debugger_session::covenant::{
    CovenantBinding as DebugCovenantBinding, ResolvedCovenantCallTarget,
    resolve_covenant_call_target,
};
use debugger_session::session::{DebugEngine, DebugSession, DebugValue, ShadowTxContext, Variable};
use debugger_session::{format_failure_report, format_value};
use kaspa_consensus_core::Hash;
use kaspa_consensus_core::hashing::sighash::SigHashReusedValuesUnsync;
use kaspa_consensus_core::mass::units::SigopCount;
use kaspa_consensus_core::tx::{
    CovenantBinding, PopulatedTransaction, ScriptPublicKey, Transaction, TransactionId,
    TransactionInput, TransactionOutpoint, TransactionOutput, UtxoEntry, VerifiableTransaction,
};
use kaspa_txscript::caches::Cache;
use kaspa_txscript::covenants::CovenantsContext;
use kaspa_txscript::script_builder::ScriptBuilder;
use kaspa_txscript::{EngineCtx, EngineFlags, pay_to_script_hash_script};
use pyo3::prelude::*;
use pyo3::types::{PyByteArray, PyBytes, PyDict, PyList, PyString};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyfunction, gen_stub_pymethods};
use silverscript_lang::ast::{
    ArrayDim, ContractAst, Expr, ExprKind, StateFieldExpr, TypeBase, TypeRef, parse_contract_ast,
};
use silverscript_lang::compiler::{
    CompileOptions, CompiledContract, compile_contract, compile_contract_ast,
};

use crate::{PySilverScriptError, Value, collect_args, map_err, py_to_value, value_to_expr};

const DEBUG_OPTS: CompileOptions = CompileOptions {
    allow_entrypoint_return: false,
    record_debug_infos: true,
};

fn err(message: impl Into<String>) -> PyErr {
    PySilverScriptError::new_err(message.into())
}

// ---------------------------------------------------------------------------
// Result classes
// ---------------------------------------------------------------------------

/// A variable decoded from the script engine's stacks at a failure site,
/// presented in source-level terms.
#[gen_stub_pyclass]
#[pyclass(
    name = "DebugVariable",
    module = "kaspa.experimental.silverscript",
    frozen
)]
#[derive(Clone)]
pub struct PyDebugVariable {
    name: String,
    type_name: String,
    origin: String,
    display: String,
    value: DebugValue,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyDebugVariable {
    /// The variable name from the SilverScript source.
    #[getter]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The SilverScript type name (e.g. `int`, `byte[4]`, `State`).
    #[getter]
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    /// Where the variable comes from: `local`, `arg`, `state` (contract
    /// field), `ctor` (constructor argument), or `const`.
    #[getter]
    pub fn origin(&self) -> &str {
        &self.origin
    }

    /// The value formatted for display, following the type (e.g. bytes are
    /// hex-encoded, `State` values render as objects).
    #[getter]
    pub fn display(&self) -> &str {
        &self.display
    }

    /// The decoded value as a native Python object (`int`, `bool`, `bytes`,
    /// `str`, `list`, or `dict`). None if the value could not be decoded.
    #[getter]
    pub fn value<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        debug_value_to_py(py, &self.value)
    }

    pub fn __repr__(&self) -> String {
        format!(
            "DebugVariable(name={:?}, type_name={:?}, value={})",
            self.name, self.type_name, self.display
        )
    }
}

/// One frame of a failure report: a function on the (inlined) call stack at
/// the point the script failed, with the variables in scope there.
#[gen_stub_pyclass]
#[pyclass(
    name = "FailureFrame",
    module = "kaspa.experimental.silverscript",
    frozen
)]
#[derive(Clone)]
pub struct PyFailureFrame {
    function_name: String,
    line: Option<u32>,
    variables: Vec<PyDebugVariable>,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyFailureFrame {
    /// The function executing in this frame.
    #[getter]
    pub fn function_name(&self) -> &str {
        &self.function_name
    }

    /// 1-based source line: the failure site for the innermost frame, the
    /// call site for caller frames. None when no source mapping exists.
    #[getter]
    pub fn line(&self) -> Option<u32> {
        self.line
    }

    /// The variables in scope in this frame (locals, arguments, contract
    /// state, and constructor arguments; constants are omitted).
    #[getter]
    pub fn variables(&self) -> Vec<PyDebugVariable> {
        self.variables.clone()
    }

    pub fn __repr__(&self) -> String {
        format!(
            "FailureFrame(function_name={:?}, line={:?}, {} variable(s))",
            self.function_name,
            self.line,
            self.variables.len()
        )
    }
}

/// A source-level report of why a debugged contract call failed: the failing
/// statement, the inlined call stack, and the variables in scope per frame.
///
/// `str()` (or `render()`) formats the report the way the SilverScript CLI
/// debugger prints it, including source context lines.
#[gen_stub_pyclass]
#[pyclass(
    name = "FailureReport",
    module = "kaspa.experimental.silverscript",
    frozen
)]
#[derive(Clone)]
pub struct PyFailureReport {
    message: String,
    frames: Vec<PyFailureFrame>,
    rendered: String,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyFailureReport {
    /// Human-readable failure description (the script engine error).
    #[getter]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The call stack at the failure, innermost frame first.
    #[getter]
    pub fn frames(&self) -> Vec<PyFailureFrame> {
        self.frames.clone()
    }

    /// Format the report for terminal display: failing source lines with
    /// context, plus each frame's variables.
    pub fn render(&self) -> &str {
        &self.rendered
    }

    pub fn __str__(&self) -> &str {
        &self.rendered
    }

    pub fn __repr__(&self) -> String {
        format!(
            "FailureReport(message={:?}, {} frame(s))",
            self.message,
            self.frames.len()
        )
    }
}

/// The outcome of a `debug_call` simulation.
#[gen_stub_pyclass]
#[pyclass(
    name = "DebugCallResult",
    module = "kaspa.experimental.silverscript",
    frozen
)]
pub struct PyDebugCallResult {
    success: bool,
    error: Option<String>,
    failure: Option<PyFailureReport>,
    console: Vec<String>,
    function_name: String,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyDebugCallResult {
    /// True if the simulated spend passed script validation.
    #[getter]
    pub fn success(&self) -> bool {
        self.success
    }

    /// The script engine error message on failure; None on success.
    #[getter]
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// The source-level failure report (failing statement, call stack, and
    /// per-frame variables); None on success or when execution failed before
    /// the debug session started (e.g. in the signature script).
    #[getter]
    pub fn failure(&self) -> Option<PyFailureReport> {
        self.failure.clone()
    }

    /// Output captured from the contract's `console.log` calls, in execution
    /// order.
    #[getter]
    pub fn console(&self) -> Vec<String> {
        self.console.clone()
    }

    /// The entrypoint that was called (resolved to the first ABI entry when
    /// `function_name` was omitted).
    #[getter]
    pub fn function_name(&self) -> &str {
        &self.function_name
    }

    pub fn __repr__(&self) -> String {
        format!(
            "DebugCallResult(function_name={:?}, success={}, error={})",
            self.function_name,
            if self.success { "True" } else { "False" },
            match &self.error {
                Some(error) => format!("{error:?}"),
                None => "None".to_string(),
            }
        )
    }
}

fn failed_result(function_name: String, error: String) -> PyDebugCallResult {
    PyDebugCallResult {
        success: false,
        error: Some(error),
        failure: None,
        console: Vec::new(),
        function_name,
    }
}

// ---------------------------------------------------------------------------
// DebugValue conversions
// ---------------------------------------------------------------------------

fn debug_value_to_py<'py>(py: Python<'py>, value: &DebugValue) -> PyResult<Bound<'py, PyAny>> {
    Ok(match value {
        DebugValue::Int(value) => value.into_pyobject(py)?.into_any(),
        DebugValue::Bool(value) => value.into_pyobject(py)?.to_owned().into_any(),
        DebugValue::Bytes(bytes) => PyBytes::new(py, bytes).into_any(),
        DebugValue::String(value) => PyString::new(py, value).into_any(),
        DebugValue::Array(items) => {
            let list = PyList::empty(py);
            for item in items {
                list.append(debug_value_to_py(py, item)?)?;
            }
            list.into_any()
        }
        DebugValue::Object(fields) => {
            let dict = PyDict::new(py);
            for (name, item) in fields {
                dict.set_item(name, debug_value_to_py(py, item)?)?;
            }
            dict.into_any()
        }
        DebugValue::Unknown(_) => py.None().into_bound(py),
    })
}

fn value_to_debug_value(value: &Value) -> DebugValue {
    match value {
        Value::Int(value) => DebugValue::Int(*value),
        Value::Bool(value) => DebugValue::Bool(*value),
        Value::Str(value) => DebugValue::String(value.clone()),
        Value::Bytes(bytes) => DebugValue::Bytes(bytes.clone()),
        Value::List(items) => DebugValue::Array(items.iter().map(value_to_debug_value).collect()),
        Value::Struct(fields) => DebugValue::Object(
            fields
                .iter()
                .map(|(name, value)| (name.clone(), value_to_debug_value(value)))
                .collect(),
        ),
    }
}

fn expr_to_debug_value(expr: &Expr<'_>) -> PyResult<DebugValue> {
    match &expr.kind {
        ExprKind::Int(value) => Ok(DebugValue::Int(*value)),
        ExprKind::Bool(value) => Ok(DebugValue::Bool(*value)),
        ExprKind::Byte(value) => Ok(DebugValue::Bytes(vec![*value])),
        ExprKind::String(value) => Ok(DebugValue::String(value.clone())),
        ExprKind::Array(values) => {
            if values
                .iter()
                .all(|value| matches!(value.kind, ExprKind::Byte(_)))
            {
                return Ok(DebugValue::Bytes(
                    values
                        .iter()
                        .map(|value| match value.kind {
                            ExprKind::Byte(byte) => byte,
                            _ => unreachable!("checked"),
                        })
                        .collect(),
                ));
            }
            Ok(DebugValue::Array(
                values
                    .iter()
                    .map(expr_to_debug_value)
                    .collect::<PyResult<Vec<_>>>()?,
            ))
        }
        ExprKind::StructLiteral(fields) => Ok(DebugValue::Object(
            fields
                .iter()
                .map(|field| Ok((field.name.clone(), expr_to_debug_value(&field.expr)?)))
                .collect::<PyResult<Vec<_>>>()?,
        )),
        other => Err(err(format!(
            "unsupported resolved state expression in debugger: {other:?}"
        ))),
    }
}

fn debug_value_to_expr(value: &DebugValue) -> Option<Expr<'static>> {
    Some(match value {
        DebugValue::Int(value) => Expr::int(*value),
        DebugValue::Bool(value) => Expr::new(ExprKind::Bool(*value), Default::default()),
        DebugValue::Bytes(bytes) => Expr::new(
            ExprKind::Array(
                bytes
                    .iter()
                    .map(|byte| Expr::new(ExprKind::Byte(*byte), Default::default()))
                    .collect(),
            ),
            Default::default(),
        ),
        DebugValue::String(value) => Expr::new(ExprKind::String(value.clone()), Default::default()),
        DebugValue::Array(values) => Expr::new(
            ExprKind::Array(
                values
                    .iter()
                    .map(debug_value_to_expr)
                    .collect::<Option<Vec<_>>>()?,
            ),
            Default::default(),
        ),
        DebugValue::Object(fields) => Expr::new(
            ExprKind::StructLiteral(
                fields
                    .iter()
                    .map(|(name, value)| {
                        Some(StateFieldExpr {
                            name: name.clone(),
                            expr: debug_value_to_expr(value)?,
                            span: Default::default(),
                            name_span: Default::default(),
                        })
                    })
                    .collect::<Option<Vec<_>>>()?,
            ),
            Default::default(),
        ),
        DebugValue::Unknown(_) => return None,
    })
}

// ---------------------------------------------------------------------------
// Transaction scenario spec (parsed from the Python `tx` dict)
// ---------------------------------------------------------------------------

struct InputSpec {
    prev_txid: Option<TransactionId>,
    prev_index: u32,
    sequence: u64,
    sig_op_count: u8,
    utxo_value: u64,
    covenant_id: Option<Hash>,
    constructor_args: Option<Vec<Value>>,
    state: Option<Value>,
    signature_script: Option<Vec<u8>>,
    utxo_script: Option<Vec<u8>>,
}

struct OutputSpec {
    value: u64,
    covenant_id: Option<Hash>,
    authorizing_input: Option<u16>,
    constructor_args: Option<Vec<Value>>,
    state: Option<Value>,
    script: Option<Vec<u8>>,
    p2pk_pubkey: Option<Vec<u8>>,
}

struct TxSpec {
    version: u16,
    lock_time: u64,
    active_input_index: usize,
    inputs: Vec<InputSpec>,
    outputs: Vec<OutputSpec>,
}

impl TxSpec {
    /// The upstream CLI debugger's default scenario: one contract input, one
    /// plain output, no covenant bindings.
    fn default_single_spend() -> Self {
        TxSpec {
            version: 1,
            lock_time: 0,
            active_input_index: 0,
            inputs: vec![InputSpec {
                prev_txid: None,
                prev_index: 0,
                sequence: 0,
                sig_op_count: 100,
                utxo_value: 5000,
                covenant_id: None,
                constructor_args: None,
                state: None,
                signature_script: None,
                utxo_script: None,
            }],
            outputs: vec![OutputSpec {
                value: 5000,
                covenant_id: None,
                authorizing_input: None,
                constructor_args: None,
                state: None,
                script: None,
                p2pk_pubkey: None,
            }],
        }
    }
}

fn check_keys(dict: &Bound<'_, PyDict>, allowed: &[&str], what: &str) -> PyResult<()> {
    for key in dict.keys() {
        let key: String = key
            .extract()
            .map_err(|_| err(format!("{what} keys must be strings")))?;
        if !allowed.contains(&key.as_str()) {
            return Err(err(format!(
                "unknown {what} key '{key}' (expected one of: {})",
                allowed.join(", ")
            )));
        }
    }
    Ok(())
}

/// A present, non-None dict entry.
fn get_item<'py>(dict: &Bound<'py, PyDict>, key: &str) -> PyResult<Option<Bound<'py, PyAny>>> {
    Ok(dict.get_item(key)?.filter(|value| !value.is_none()))
}

fn get_extract<'py, T: for<'a> FromPyObject<'a, 'py>>(
    dict: &Bound<'py, PyDict>,
    key: &str,
    what: &str,
) -> PyResult<Option<T>> {
    get_item(dict, key)?
        .map(|value| {
            value
                .extract::<T>()
                .map_err(|_| err(format!("invalid '{key}' in {what}")))
        })
        .transpose()
}

fn extract_bytes_like(obj: &Bound<'_, PyAny>, what: &str) -> PyResult<Vec<u8>> {
    if let Ok(bytes) = obj.cast::<PyBytes>() {
        return Ok(bytes.as_bytes().to_vec());
    }
    if let Ok(bytes) = obj.cast::<PyByteArray>() {
        return Ok(bytes.to_vec());
    }
    if let Ok(string) = obj.cast::<PyString>() {
        return parse_hex_bytes(string.to_str()?).map_err(|e| err(format!("invalid {what}: {e}")));
    }
    Err(err(format!("{what} must be bytes or a hex string")))
}

fn get_bytes(dict: &Bound<'_, PyDict>, key: &str, what: &str) -> PyResult<Option<Vec<u8>>> {
    get_item(dict, key)?
        .map(|value| extract_bytes_like(&value, &format!("'{key}' in {what}")))
        .transpose()
}

fn get_hash32(dict: &Bound<'_, PyDict>, key: &str, what: &str) -> PyResult<Option<Hash>> {
    get_bytes(dict, key, what)?
        .map(|bytes| {
            let bytes: [u8; 32] = bytes
                .try_into()
                .map_err(|_| err(format!("'{key}' in {what} must be 32 bytes")))?;
            Ok(Hash::from_bytes(bytes))
        })
        .transpose()
}

fn get_args(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<Option<Vec<Value>>> {
    get_item(dict, key)?
        .map(|value| collect_args(Some(&value)))
        .transpose()
}

fn get_state(dict: &Bound<'_, PyDict>, key: &str, what: &str) -> PyResult<Option<Value>> {
    get_item(dict, key)?
        .map(|value| {
            let value = py_to_value(&value)?;
            if !matches!(value, Value::Struct(_)) {
                return Err(err(format!(
                    "'{key}' in {what} must be a dict of state fields"
                )));
            }
            Ok(value)
        })
        .transpose()
}

fn parse_input_spec(obj: &Bound<'_, PyAny>, index: usize) -> PyResult<InputSpec> {
    let what = format!("tx input {index}");
    let dict = obj
        .cast::<PyDict>()
        .map_err(|_| err(format!("{what} must be a dict")))?;
    check_keys(
        dict,
        &[
            "prev_txid",
            "prev_index",
            "sequence",
            "sig_op_count",
            "utxo_value",
            "covenant_id",
            "constructor_args",
            "state",
            "signature_script",
            "utxo_script",
        ],
        &what,
    )?;
    Ok(InputSpec {
        prev_txid: get_bytes(dict, "prev_txid", &what)?
            .map(|bytes| {
                let bytes: [u8; 32] = bytes
                    .try_into()
                    .map_err(|_| err(format!("'prev_txid' in {what} must be 32 bytes")))?;
                Ok::<_, PyErr>(TransactionId::from_bytes(bytes))
            })
            .transpose()?,
        prev_index: get_extract(dict, "prev_index", &what)?.unwrap_or(0),
        sequence: get_extract(dict, "sequence", &what)?.unwrap_or(0),
        sig_op_count: get_extract(dict, "sig_op_count", &what)?.unwrap_or(100),
        utxo_value: get_extract(dict, "utxo_value", &what)?
            .ok_or_else(|| err(format!("{what} requires 'utxo_value'")))?,
        covenant_id: get_hash32(dict, "covenant_id", &what)?,
        constructor_args: get_args(dict, "constructor_args")?,
        state: get_state(dict, "state", &what)?,
        signature_script: get_bytes(dict, "signature_script", &what)?,
        utxo_script: get_bytes(dict, "utxo_script", &what)?,
    })
}

fn parse_output_spec(obj: &Bound<'_, PyAny>, index: usize) -> PyResult<OutputSpec> {
    let what = format!("tx output {index}");
    let dict = obj
        .cast::<PyDict>()
        .map_err(|_| err(format!("{what} must be a dict")))?;
    check_keys(
        dict,
        &[
            "value",
            "covenant_id",
            "authorizing_input",
            "constructor_args",
            "state",
            "script",
            "p2pk_pubkey",
        ],
        &what,
    )?;
    Ok(OutputSpec {
        value: get_extract(dict, "value", &what)?
            .ok_or_else(|| err(format!("{what} requires 'value'")))?,
        covenant_id: get_hash32(dict, "covenant_id", &what)?,
        authorizing_input: get_extract(dict, "authorizing_input", &what)?,
        constructor_args: get_args(dict, "constructor_args")?,
        state: get_state(dict, "state", &what)?,
        script: get_bytes(dict, "script", &what)?,
        p2pk_pubkey: get_bytes(dict, "p2pk_pubkey", &what)?,
    })
}

fn parse_tx_spec(obj: Option<&Bound<'_, PyAny>>) -> PyResult<TxSpec> {
    let Some(obj) = obj else {
        return Ok(TxSpec::default_single_spend());
    };
    let dict = obj.cast::<PyDict>().map_err(|_| err("tx must be a dict"))?;
    check_keys(
        dict,
        &[
            "version",
            "lock_time",
            "active_input_index",
            "inputs",
            "outputs",
        ],
        "tx",
    )?;

    let inputs = get_item(dict, "inputs")?
        .ok_or_else(|| err("tx requires 'inputs'"))?
        .try_iter()
        .map_err(|_| err("tx 'inputs' must be a list of dicts"))?
        .enumerate()
        .map(|(index, item)| parse_input_spec(&item?, index))
        .collect::<PyResult<Vec<_>>>()?;
    let outputs = match get_item(dict, "outputs")? {
        Some(value) => value
            .try_iter()
            .map_err(|_| err("tx 'outputs' must be a list of dicts"))?
            .enumerate()
            .map(|(index, item)| parse_output_spec(&item?, index))
            .collect::<PyResult<Vec<_>>>()?,
        None => Vec::new(),
    };

    let spec = TxSpec {
        version: get_extract(dict, "version", "tx")?.unwrap_or(1),
        lock_time: get_extract(dict, "lock_time", "tx")?.unwrap_or(0),
        active_input_index: get_extract(dict, "active_input_index", "tx")?.unwrap_or(0),
        inputs,
        outputs,
    };
    if spec.inputs.is_empty() {
        return Err(err("tx 'inputs' must contain at least one input"));
    }
    if spec.active_input_index >= spec.inputs.len() {
        return Err(err(format!(
            "tx 'active_input_index' {} out of range for {} input(s)",
            spec.active_input_index,
            spec.inputs.len()
        )));
    }
    Ok(spec)
}

// ---------------------------------------------------------------------------
// Harness helpers (ported from the upstream CLI debugger)
// ---------------------------------------------------------------------------

fn ctor_exprs(ctor: &[Value]) -> Vec<Expr<'static>> {
    ctor.iter().map(value_to_expr).collect()
}

fn compile_script_for_ctor_args(
    source: &str,
    ctor: &[Value],
    cache: &mut HashMap<Vec<Value>, Vec<u8>>,
) -> PyResult<Vec<u8>> {
    if let Some(script) = cache.get(ctor) {
        return Ok(script.clone());
    }
    let script = compile_contract(source, &ctor_exprs(ctor), DEBUG_OPTS)
        .map_err(map_err)?
        .script;
    cache.insert(ctor.to_vec(), script.clone());
    Ok(script)
}

fn resolve_state_for_ctor_args(
    contract: &ContractAst<'_>,
    ctor: &[Value],
    cache: &mut HashMap<Vec<Value>, DebugValue>,
) -> PyResult<DebugValue> {
    if let Some(state) = cache.get(ctor) {
        return Ok(state.clone());
    }
    let fields = contract
        .resolve_contract_state_values(&ctor_exprs(ctor))
        .map_err(map_err)?;
    let state = DebugValue::Object(
        fields
            .iter()
            .map(|field| Ok((field.name.clone(), expr_to_debug_value(&field.value)?)))
            .collect::<PyResult<Vec<_>>>()?,
    );
    cache.insert(ctor.to_vec(), state.clone());
    Ok(state)
}

fn contract_with_explicit_state<'i>(
    contract: &ContractAst<'i>,
    state: &Expr<'i>,
) -> PyResult<ContractAst<'i>> {
    let ExprKind::StructLiteral(entries) = &state.kind else {
        return Err(err("state value must be a dict of state fields"));
    };

    // Unknown keys first, in source order: a misspelled field would otherwise
    // surface as "missing" the field it failed to name.
    if let Some(extra) = entries
        .iter()
        .find(|entry| !contract.fields.iter().any(|field| field.name == entry.name))
    {
        return Err(err(format!("unknown state field '{}'", extra.name)));
    }

    let mut provided = entries
        .iter()
        .map(|entry| (entry.name.as_str(), entry.expr.clone()))
        .collect::<std::collections::HashMap<_, _>>();
    let mut materialized = contract.clone();
    for field in &mut materialized.fields {
        field.expr = provided
            .remove(field.name.as_str())
            .ok_or_else(|| err(format!("missing state field '{}'", field.name)))?;
    }
    Ok(materialized)
}

/// Splice an explicit state into the compiled script's state region, verifying
/// the surrounding template bytes are unchanged.
fn materialize_script_for_explicit_state(
    source: &str,
    contract: &ContractAst<'_>,
    ctor: &[Value],
    state: &Value,
) -> PyResult<Vec<u8>> {
    let instance_args = ctor_exprs(ctor);
    let state_expr = value_to_expr(state);
    let base_compiled = compile_contract(source, &instance_args, DEBUG_OPTS).map_err(map_err)?;
    let materialized_contract = contract_with_explicit_state(contract, &state_expr)?;
    let materialized = compile_contract_ast(&materialized_contract, &instance_args, DEBUG_OPTS)
        .map_err(map_err)?;

    let base_start = base_compiled.state_layout.start;
    let base_end = base_start + base_compiled.state_layout.len;
    let materialized_start = materialized.state_layout.start;
    let materialized_end = materialized_start + materialized.state_layout.len;
    if base_compiled.state_layout.len != materialized.state_layout.len {
        return Err(err(
            "explicit state changes encoded script size; provide a raw 'utxo_script'/'script' instead",
        ));
    }
    if base_compiled.script.len() < base_end || materialized.script.len() < materialized_end {
        return Err(err("state layout exceeds compiled script length"));
    }
    if base_compiled.script[..base_start] != materialized.script[..materialized_start]
        || base_compiled.script[base_end..] != materialized.script[materialized_end..]
    {
        return Err(err(
            "explicit state changed non-state bytecode; provide a raw 'utxo_script'/'script' instead",
        ));
    }

    let mut script = base_compiled.script;
    script[base_start..base_end]
        .copy_from_slice(&materialized.script[materialized_start..materialized_end]);
    Ok(script)
}

fn is_state_type(type_ref: &TypeRef) -> bool {
    type_ref.is_custom() && matches!(&type_ref.base, TypeBase::Custom(name) if name == "State")
}

fn is_state_array_type(type_ref: &TypeRef) -> bool {
    matches!(&type_ref.base, TypeBase::Custom(name) if name == "State")
        && matches!(type_ref.array_dims.as_slice(), [ArrayDim::Dynamic])
}

/// Covenant entrypoints take the verified output state(s) as a hidden leading
/// parameter; synthesize it from the scenario's output states when the caller
/// passed only the source-level arguments.
fn synthesized_covenant_prefix_args(
    compiled: &CompiledContract<'_>,
    entrypoint_name: &str,
    target: &ResolvedCovenantCallTarget,
    output_states: Option<&[DebugValue]>,
) -> PyResult<Vec<Expr<'static>>> {
    if target.binding == DebugCovenantBinding::Cov && entrypoint_name.starts_with("__delegate_") {
        return Ok(Vec::new());
    }

    let function = compiled
        .ast
        .functions
        .iter()
        .find(|function| function.name == entrypoint_name)
        .ok_or_else(|| err("generated covenant entrypoint not found"))?;
    let Some(first_param) = function.params.first() else {
        return Ok(Vec::new());
    };

    let states = output_states.ok_or_else(|| {
        err("missing output states needed to synthesize covenant verification arguments")
    })?;
    if is_state_type(&first_param.type_ref) {
        if states.len() != 1 {
            return Err(err(format!(
                "expected exactly 1 output State for '{entrypoint_name}', got {}",
                states.len()
            )));
        }
        return Ok(vec![debug_value_to_expr(&states[0]).ok_or_else(|| {
            err("failed to materialize synthesized output State")
        })?]);
    }
    if is_state_array_type(&first_param.type_ref) {
        return Ok(vec![Expr::new(
            ExprKind::Array(
                states
                    .iter()
                    .map(debug_value_to_expr)
                    .collect::<Option<Vec<_>>>()
                    .ok_or_else(|| err("failed to materialize synthesized output State[]"))?,
            ),
            Default::default(),
        )]);
    }

    Ok(Vec::new())
}

fn build_covenant_input_sigscript(
    compiled: &CompiledContract<'_>,
    target: &ResolvedCovenantCallTarget,
    is_leader: bool,
    call_args: &[Value],
    output_states: Option<&[DebugValue]>,
) -> PyResult<Vec<u8>> {
    let entrypoint_name = target.generated_entrypoint_name_for(is_leader);
    let typed_args = if target.binding == DebugCovenantBinding::Cov && !is_leader {
        Vec::new()
    } else {
        let function = compiled
            .ast
            .functions
            .iter()
            .find(|function| function.name == entrypoint_name)
            .ok_or_else(|| err("generated covenant entrypoint not found"))?;
        let user_exprs = ctor_exprs(call_args);
        if call_args.len() == function.params.len() {
            user_exprs
        } else {
            let mut all_args = synthesized_covenant_prefix_args(
                compiled,
                &entrypoint_name,
                target,
                output_states,
            )?;
            all_args.extend(user_exprs);
            all_args
        }
    };
    compiled
        .build_sig_script(&entrypoint_name, typed_args)
        .map_err(map_err)
}

fn covenant_flags() -> EngineFlags {
    EngineFlags {
        covenants_enabled: true,
        ..Default::default()
    }
}

fn build_p2pk_script(pubkey: &[u8]) -> PyResult<Vec<u8>> {
    let mut builder = ScriptBuilder::with_flags(covenant_flags());
    builder.add_data(pubkey).map_err(|e| err(e.to_string()))?;
    builder
        .add_op(kaspa_txscript::opcodes::codes::OpCheckSig)
        .map_err(|e| err(e.to_string()))?;
    Ok(builder.drain())
}

fn sigscript_push_script(script: &[u8]) -> PyResult<Vec<u8>> {
    let mut builder = ScriptBuilder::with_flags(covenant_flags());
    builder.add_data(script).map_err(|e| err(e.to_string()))?;
    Ok(builder.drain())
}

fn combine_action_and_redeem(action: &[u8], redeem_script: &[u8]) -> PyResult<Vec<u8>> {
    let mut builder = ScriptBuilder::with_flags(covenant_flags());
    builder.add_ops(action).map_err(|e| err(e.to_string()))?;
    builder
        .add_data(redeem_script)
        .map_err(|e| err(e.to_string()))?;
    Ok(builder.drain())
}

// ---------------------------------------------------------------------------
// Failure report conversion
// ---------------------------------------------------------------------------

fn convert_variable(variable: &Variable) -> PyDebugVariable {
    PyDebugVariable {
        display: format_value(&variable.type_name, &variable.value),
        name: variable.name.clone(),
        type_name: variable.type_name.clone(),
        origin: variable.origin.label().to_string(),
        value: variable.value.clone(),
    }
}

fn convert_report(report: &FailureReport) -> PyFailureReport {
    PyFailureReport {
        message: report.message.clone(),
        frames: report
            .frames
            .iter()
            .map(|frame| PyFailureFrame {
                function_name: frame.function_name.clone(),
                line: frame.span.map(|span| span.line),
                variables: frame.variables.iter().map(convert_variable).collect(),
            })
            .collect(),
        rendered: format_failure_report(report, &format_value),
    }
}

// ---------------------------------------------------------------------------
// The harness
// ---------------------------------------------------------------------------

fn run_harness(
    source: &str,
    function_name: Option<&str>,
    call_args: &[Value],
    ctor_args: &[Value],
    tx: &TxSpec,
) -> PyResult<PyDebugCallResult> {
    let parsed_contract = parse_contract_ast(source).map_err(map_err)?;

    // CLI parity: fall back to the active input's constructor args when the
    // caller gave none.
    let root_ctor: Vec<Value> = if ctor_args.is_empty() {
        tx.inputs[tx.active_input_index]
            .constructor_args
            .clone()
            .unwrap_or_default()
    } else {
        ctor_args.to_vec()
    };

    let root_ctor_exprs = ctor_exprs(&root_ctor);
    let compiled = compile_contract(source, &root_ctor_exprs, DEBUG_OPTS).map_err(map_err)?;
    let debug_info = compiled.debug_info.clone();

    let selected_name = match function_name {
        Some(name) => name.to_string(),
        None => compiled
            .abi
            .first()
            .map(|entry| entry.name.clone())
            .ok_or_else(|| err("contract has no entrypoints"))?,
    };

    let covenant_target = resolve_covenant_call_target(&parsed_contract, &compiled, &selected_name);

    // Memoize per-constructor-args work (upstream CLI parity): scenarios reuse
    // the same ctor args across inputs and outputs, so compile/resolve once.
    let mut ctor_script_cache: HashMap<Vec<Value>, Vec<u8>> = HashMap::new();
    let mut ctor_state_cache: HashMap<Vec<Value>, DebugValue> = HashMap::new();
    ctor_script_cache.insert(root_ctor.clone(), compiled.script.clone());

    // --- Resolve inputs: redeem scripts, UTXO script public keys, covenant state ---
    let mut input_prev_outpoints = Vec::with_capacity(tx.inputs.len());
    let mut input_sequences = Vec::with_capacity(tx.inputs.len());
    let mut input_sig_op_counts = Vec::with_capacity(tx.inputs.len());
    let mut explicit_input_sigs = Vec::with_capacity(tx.inputs.len());
    let mut utxo_specs = Vec::with_capacity(tx.inputs.len());
    let mut input_covenant_ids = Vec::with_capacity(tx.inputs.len());
    let mut input_covenant_states = Vec::with_capacity(tx.inputs.len());
    let mut input_redeem_scripts = Vec::with_capacity(tx.inputs.len());
    for (input_idx, input) in tx.inputs.iter().enumerate() {
        let prev_txid = input
            .prev_txid
            .unwrap_or_else(|| TransactionId::from_bytes([input_idx as u8; 32]));

        let input_ctor = input
            .constructor_args
            .clone()
            .unwrap_or_else(|| root_ctor.clone());
        let input_covenant_state = if let Some(state) = &input.state {
            Some(value_to_debug_value(state))
        } else if input.utxo_script.is_none() || input.constructor_args.is_some() {
            Some(resolve_state_for_ctor_args(
                &parsed_contract,
                &input_ctor,
                &mut ctor_state_cache,
            )?)
        } else {
            None
        };
        let redeem_script = if input.utxo_script.is_none() {
            if let Some(state) = &input.state {
                Some(materialize_script_for_explicit_state(
                    source,
                    &parsed_contract,
                    &input_ctor,
                    state,
                )?)
            } else {
                Some(compile_script_for_ctor_args(
                    source,
                    &input_ctor,
                    &mut ctor_script_cache,
                )?)
            }
        } else {
            None
        };

        let utxo_spk = if let Some(raw_script) = &input.utxo_script {
            ScriptPublicKey::new(0, raw_script.clone().into())
        } else {
            let redeem = redeem_script
                .as_ref()
                .expect("redeem script exists when no explicit utxo script");
            pay_to_script_hash_script(redeem)
        };

        input_prev_outpoints.push(TransactionOutpoint {
            transaction_id: prev_txid,
            index: input.prev_index,
        });
        input_sequences.push(input.sequence);
        input_sig_op_counts.push(input.sig_op_count);
        explicit_input_sigs.push(input.signature_script.clone());
        utxo_specs.push((input.utxo_value, utxo_spk, input.covenant_id));
        input_covenant_ids.push(input.covenant_id);
        input_covenant_states.push(input_covenant_state);
        input_redeem_scripts.push(redeem_script);
    }

    // --- Resolve outputs: script public keys, covenant bindings and states ---
    let mut tx_outputs = Vec::with_capacity(tx.outputs.len());
    let mut output_covenant_ids = Vec::with_capacity(tx.outputs.len());
    let mut output_covenant_states = Vec::with_capacity(tx.outputs.len());
    for output in tx.outputs.iter() {
        let output_ctor = output
            .constructor_args
            .clone()
            .unwrap_or_else(|| root_ctor.clone());
        let output_state = if let Some(state) = &output.state {
            Some(value_to_debug_value(state))
        } else if output.script.is_none() || output.constructor_args.is_some() {
            Some(resolve_state_for_ctor_args(
                &parsed_contract,
                &output_ctor,
                &mut ctor_state_cache,
            )?)
        } else {
            None
        };
        let script_public_key = if let Some(raw_script) = &output.script {
            ScriptPublicKey::new(0, raw_script.clone().into())
        } else if let Some(raw_pubkey) = &output.p2pk_pubkey {
            ScriptPublicKey::new(0, build_p2pk_script(raw_pubkey)?.into())
        } else {
            let output_script = if let Some(state) = &output.state {
                materialize_script_for_explicit_state(
                    source,
                    &parsed_contract,
                    &output_ctor,
                    state,
                )?
            } else {
                compile_script_for_ctor_args(source, &output_ctor, &mut ctor_script_cache)?
            };
            pay_to_script_hash_script(&output_script)
        };

        let covenant = output.covenant_id.map(|covenant_id| CovenantBinding {
            authorizing_input: output
                .authorizing_input
                .unwrap_or(tx.active_input_index as u16),
            covenant_id,
        });

        let output_covenant_id = covenant.as_ref().map(|binding| binding.covenant_id);
        tx_outputs.push(TransactionOutput {
            value: output.value,
            script_public_key,
            covenant,
        });
        output_covenant_ids.push(output_covenant_id);
        output_covenant_states.push(output_state);
    }

    // --- Covenant group bookkeeping (leader election, verified output states) ---
    let active_covenant_id = input_covenant_ids
        .get(tx.active_input_index)
        .copied()
        .flatten();
    let companion_leader_index = if covenant_target
        .as_ref()
        .is_some_and(|target| target.binding == DebugCovenantBinding::Cov)
    {
        active_covenant_id.and_then(|covenant_id| {
            input_covenant_ids
                .iter()
                .enumerate()
                .filter_map(|(index, input_covenant_id)| {
                    (*input_covenant_id == Some(covenant_id)).then_some(index)
                })
                .min()
        })
    } else {
        None
    };
    let active_authorized_output_states = tx
        .outputs
        .iter()
        .zip(output_covenant_states.iter())
        .filter_map(|(output, output_state)| {
            (output
                .authorizing_input
                .unwrap_or(tx.active_input_index as u16)
                == tx.active_input_index as u16)
                .then_some(output_state.clone())
        })
        .collect::<Option<Vec<_>>>();
    let covenant_group_output_states = active_covenant_id.and_then(|covenant_id| {
        output_covenant_ids
            .iter()
            .zip(output_covenant_states.iter())
            .filter_map(|(output_covenant_id, output_state)| {
                (*output_covenant_id == Some(covenant_id)).then_some(output_state.clone())
            })
            .collect::<Option<Vec<_>>>()
    });

    // --- The active input's signature script (the call being debugged) ---
    let active_ctor = tx.inputs[tx.active_input_index]
        .constructor_args
        .clone()
        .unwrap_or_else(|| root_ctor.clone());
    // Without a per-input ctor override the active input is the root compile.
    let active_compiled_storage;
    let active_compiled = if active_ctor == root_ctor {
        &compiled
    } else {
        active_compiled_storage =
            compile_contract(source, &ctor_exprs(&active_ctor), DEBUG_OPTS).map_err(map_err)?;
        &active_compiled_storage
    };
    let active_is_cov_leader = companion_leader_index
        .map(|index| index == tx.active_input_index)
        .unwrap_or(true);
    let active_sigscript = if let Some(target) = covenant_target.as_ref() {
        match target.binding {
            DebugCovenantBinding::Auth => build_covenant_input_sigscript(
                active_compiled,
                target,
                true,
                call_args,
                active_authorized_output_states.as_deref(),
            )?,
            DebugCovenantBinding::Cov => build_covenant_input_sigscript(
                active_compiled,
                target,
                active_is_cov_leader,
                call_args,
                covenant_group_output_states.as_deref(),
            )?,
        }
    } else {
        active_compiled
            .build_sig_script(&selected_name, ctor_exprs(call_args))
            .map_err(map_err)?
    };

    // --- Assemble the transaction ---
    let mut tx_inputs = Vec::with_capacity(tx.inputs.len());
    for input_idx in 0..tx.inputs.len() {
        let signature_script =
            if let Some(signature_script) = explicit_input_sigs[input_idx].clone() {
                signature_script
            } else if input_idx == tx.active_input_index {
                if let Some(redeem) = input_redeem_scripts[input_idx].as_ref() {
                    combine_action_and_redeem(&active_sigscript, redeem)?
                } else {
                    active_sigscript.clone()
                }
            } else if let Some(target) = covenant_target.as_ref()
                && target.binding == DebugCovenantBinding::Cov
                && input_covenant_ids[input_idx] == active_covenant_id
                && input_redeem_scripts[input_idx].is_some()
            {
                // Companion input of the same covenant group: auto-build its
                // leader/delegate call so the group verifies as a whole.
                let is_leader = Some(input_idx) == companion_leader_index;
                let input_ctor = tx.inputs[input_idx]
                    .constructor_args
                    .clone()
                    .unwrap_or_else(|| root_ctor.clone());
                let input_compiled_storage;
                let input_compiled = if input_ctor == root_ctor {
                    &compiled
                } else {
                    input_compiled_storage =
                        compile_contract(source, &ctor_exprs(&input_ctor), DEBUG_OPTS)
                            .map_err(map_err)?;
                    &input_compiled_storage
                };
                let auto_action = build_covenant_input_sigscript(
                    input_compiled,
                    target,
                    is_leader,
                    call_args,
                    covenant_group_output_states.as_deref(),
                )?;
                combine_action_and_redeem(
                    &auto_action,
                    input_redeem_scripts[input_idx]
                        .as_ref()
                        .expect("checked is_some above"),
                )?
            } else if let Some(redeem) = input_redeem_scripts[input_idx].as_ref() {
                sigscript_push_script(redeem)?
            } else {
                vec![]
            };

        tx_inputs.push(TransactionInput {
            previous_outpoint: input_prev_outpoints[input_idx],
            signature_script,
            sequence: input_sequences[input_idx],
            compute_commit: SigopCount(input_sig_op_counts[input_idx]).into(),
        });
    }

    let kas_tx = Transaction::new(
        tx.version,
        tx_inputs,
        tx_outputs,
        tx.lock_time,
        Default::default(),
        0,
        vec![],
    );

    // --- Engine + session ---
    let sig_cache = Cache::new(10_000);
    let reused_values = SigHashReusedValuesUnsync::new();

    let utxos = utxo_specs
        .into_iter()
        .map(|(value, spk, covenant_id)| {
            UtxoEntry::new(value, spk, 0, kas_tx.is_coinbase(), covenant_id)
        })
        .collect::<Vec<_>>();
    let populated_tx = PopulatedTransaction::new(&kas_tx, utxos);
    let cov_ctx = match CovenantsContext::from_tx(&populated_tx) {
        Ok(cov_ctx) => cov_ctx,
        Err(e) => {
            return Ok(failed_result(
                selected_name,
                format!("Covenant structure error: {e}"),
            ));
        }
    };
    let ctx = EngineCtx::new(&sig_cache)
        .with_reused(&reused_values)
        .with_covenants_ctx(&cov_ctx);
    let active_input = &kas_tx.inputs[tx.active_input_index];
    let active_utxo = populated_tx
        .utxo(tx.active_input_index)
        .ok_or_else(|| err("missing utxo entry for active input"))?;
    let active_lockscript = input_redeem_scripts[tx.active_input_index]
        .clone()
        .unwrap_or_else(|| compiled.script.clone());
    let covenant_input_states = active_utxo.covenant_id.and_then(|covenant_id| {
        let mut values = Vec::new();
        for (input_covenant_id, covenant_input_state) in
            input_covenant_ids.iter().zip(input_covenant_states.iter())
        {
            if *input_covenant_id != Some(covenant_id) {
                continue;
            }
            values.push(covenant_input_state.clone()?);
        }
        Some(values)
    });
    let covenant_param_value = match covenant_target.as_ref().map(|target| target.binding) {
        Some(DebugCovenantBinding::Auth) => input_covenant_states[tx.active_input_index].clone(),
        Some(DebugCovenantBinding::Cov) => covenant_input_states.map(DebugValue::Array),
        None => None,
    };

    let engine = DebugEngine::from_transaction_input(
        &populated_tx,
        active_input,
        tx.active_input_index,
        active_utxo,
        ctx,
        covenant_flags(),
    );
    let shadow_tx_context = ShadowTxContext {
        tx: &populated_tx,
        input: active_input,
        input_index: tx.active_input_index,
        utxo_entry: active_utxo,
        covenants_ctx: &cov_ctx,
    };

    let mut session = match DebugSession::full(
        &active_sigscript,
        &active_lockscript,
        source,
        debug_info,
        engine,
    ) {
        Ok(session) => session,
        // Failed before lockscript debugging began (e.g. in the signature
        // script) — no session to build a source-level report from.
        Err(e) => return Ok(failed_result(selected_name, e.to_string())),
    };
    session = session
        .with_shadow_tx_context(shadow_tx_context)
        .with_covenant_mode(covenant_param_value, covenant_target);

    match session.run_to_completion() {
        Ok(()) => Ok(PyDebugCallResult {
            success: true,
            error: None,
            failure: None,
            console: session.take_console_output(),
            function_name: selected_name,
        }),
        Err(e) => {
            let report = session.build_failure_report(&e);
            Ok(PyDebugCallResult {
                success: false,
                error: Some(e.to_string()),
                failure: Some(convert_report(&report)),
                console: session.take_console_output(),
                function_name: selected_name,
            })
        }
    }
}

/// Debug a SilverScript contract call by simulating the full spend locally.
///
/// **Experimental:** SilverScript and these bindings are under active
/// development; the API may change in breaking ways between releases.
///
/// Compiles `source` with debug info, builds a synthetic transaction that
/// spends the contract's P2SH UTXO with a call to the chosen entrypoint, and
/// executes it through SilverScript's source-level debug engine. The result
/// reports pass/fail; on failure it includes a source-level report with the
/// failing statement, the call stack, and the decoded variables (locals,
/// arguments, contract state) in each frame. `console.log` output is captured
/// either way.
///
/// This simulates script validation on a synthetic transaction — it does not
/// touch the network and says nothing about fees, mass, or maturity.
///
/// Args:
///     source: The SilverScript contract source.
///     function_name: The entrypoint to call. Defaults to the contract's
///         first entrypoint. For covenant functions, use the source-level
///         name (e.g. `"add"`).
///     args: Native Python values matching the entrypoint's parameters. For
///         covenant transition functions the leading `State` parameter is
///         synthesized from the scenario's output states — pass only the
///         source-level arguments after it.
///     constructor_args: Native Python values for the contract's constructor
///         parameters.
///     tx: Optional transaction scenario dict. Defaults to a single-input,
///         single-output spend of the contract. Keys: `version` (default 1),
///         `lock_time` (default 0), `active_input_index` (default 0; the
///         input being debugged), `inputs`, and `outputs`. Each input dict
///         accepts `utxo_value` (required), `covenant_id` (32 bytes or hex),
///         `state` (dict of contract state fields carried by the spent UTXO),
///         `constructor_args`, `prev_txid`, `prev_index`, `sequence`,
///         `sig_op_count`, `signature_script`, and `utxo_script` (raw bytes
///         overrides). Each output dict accepts `value` (required),
///         `covenant_id`, `authorizing_input`, `state` (the post-transition
///         contract state to verify), `constructor_args`, `script`, and
///         `p2pk_pubkey`.
///
/// Returns:
///     DebugCallResult: The simulation outcome. Script failures are reported
///     in the result, not raised.
///
/// Raises:
///     SilverScriptError: If compilation fails, the entrypoint or an argument
///     is invalid, or the `tx` scenario is malformed.
#[gen_stub_pyfunction(module = "kaspa.experimental.silverscript")]
#[pyfunction]
#[pyo3(name = "debug_call")]
#[pyo3(signature = (source, function_name=None, args=None, constructor_args=None, tx=None))]
pub fn py_debug_call(
    source: String,
    function_name: Option<String>,
    args: Option<Bound<'_, PyAny>>,
    constructor_args: Option<Bound<'_, PyAny>>,
    tx: Option<Bound<'_, PyAny>>,
) -> PyResult<PyDebugCallResult> {
    let call_args = collect_args(args.as_ref())?;
    let ctor_args = collect_args(constructor_args.as_ref())?;
    let tx_spec = parse_tx_spec(tx.as_ref())?;
    run_harness(
        &source,
        function_name.as_deref(),
        &call_args,
        &ctor_args,
        &tx_spec,
    )
}
