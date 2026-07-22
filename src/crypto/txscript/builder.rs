use crate::{
    consensus::core::script_public_key::PyScriptPublicKey, crypto::txscript::opcodes::PyOpcodes,
    types::PyBinary,
};
use kaspa_consensus_core::mass::ScriptUnits;
use kaspa_txscript::{EngineFlags, script_builder as native, standard};
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use std::sync::{Arc, Mutex, MutexGuard};
use workflow_core::hex::ToHex;

/// Builder for constructing transaction scripts.
///
/// Provides a fluent interface for building custom scripts with opcodes and data.
/// Used for creating complex spending conditions like multi-signature or time-locked
/// transactions.
#[gen_stub_pyclass]
#[pyclass(name = "ScriptBuilder")]
#[derive(Clone)]
pub struct PyScriptBuilder(Arc<Mutex<native::ScriptBuilder>>);

impl PyScriptBuilder {
    #[inline]
    pub fn inner(&self) -> MutexGuard<'_, native::ScriptBuilder> {
        self.0.lock().unwrap()
    }
}

impl Default for PyScriptBuilder {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(native::ScriptBuilder::new())))
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyScriptBuilder {
    /// Create a new empty script builder.
    ///
    /// Args:
    ///     covenants_enabled: Enable covenant opcodes and post-Toccata script
    ///         limits (default: False).
    ///     sigop_script_units: Script units charged per signature operation.
    ///         Defaults to the native engine default when omitted.
    ///
    /// Returns:
    ///     ScriptBuilder: A new empty ScriptBuilder instance.
    #[new]
    #[pyo3(signature = (covenants_enabled=false, sigop_script_units=None))]
    pub fn new(covenants_enabled: bool, sigop_script_units: Option<u64>) -> Self {
        let flags = build_engine_flags(covenants_enabled, sigop_script_units);
        Self(Arc::new(Mutex::new(native::ScriptBuilder::with_flags(
            flags,
        ))))
    }

    /// Create a script builder from an existing script.
    ///
    /// Args:
    ///     script: Existing script bytes as hex, bytes, or list.
    ///     covenants_enabled: Enable covenant opcodes and post-Toccata script
    ///         limits (default: False).
    ///     sigop_script_units: Script units charged per signature operation.
    ///         Defaults to the native engine default when omitted.
    ///
    /// Returns:
    ///     ScriptBuilder: A new ScriptBuilder initialized with the script.
    #[staticmethod]
    #[pyo3(signature = (script, covenants_enabled=false, sigop_script_units=None))]
    pub fn from_script(
        script: PyBinary,
        covenants_enabled: bool,
        sigop_script_units: Option<u64>,
    ) -> PyResult<Self> {
        let flags = build_engine_flags(covenants_enabled, sigop_script_units);
        let builder = Self(Arc::new(Mutex::new(native::ScriptBuilder::with_flags(
            flags,
        ))));
        let script: Vec<u8> = script.into();
        builder.inner().script_mut().extend(&script);

        Ok(builder)
    }

    /// Whether covenant opcodes and post-Toccata script limits are enabled.
    ///
    /// Returns:
    ///     bool: True if covenants are enabled for this builder.
    #[getter]
    pub fn get_covenants_enabled(&self) -> bool {
        self.inner().flags().covenants_enabled
    }

    /// Script units charged for each signature operation.
    ///
    /// Returns:
    ///     int: The configured sigop script units.
    #[getter]
    pub fn get_sigop_script_units(&self) -> u64 {
        self.inner().flags().sigop_script_units.0
    }

    /// Add a single opcode to the script.
    ///
    /// Args:
    ///     op: An Opcodes enum value or integer.
    ///
    /// Returns:
    ///     ScriptBuilder: Self for method chaining.
    ///
    /// Raises:
    ///     Exception: If the opcode is invalid.
    pub fn add_op(
        &self,
        #[gen_stub(override_type(type_repr = "int | Opcodes"))] op: &Bound<PyAny>,
    ) -> PyResult<Self> {
        let op = extract_op(op)?;
        let mut inner = self.inner();
        inner
            .add_op(op)
            .map_err(|err| PyException::new_err(format!("{}", err)))?;

        Ok(self.clone())
    }

    /// Add multiple opcodes to the script.
    ///
    /// Args:
    ///     opcodes: List of Opcodes enum values or integers.
    ///
    /// Returns:
    ///     ScriptBuilder: Self for method chaining.
    ///
    /// Raises:
    ///     Exception: If any opcode is invalid.
    pub fn add_ops(
        &self,
        #[gen_stub(override_type(type_repr = "builtins.list[int | Opcodes]"))] opcodes: &Bound<
            PyAny,
        >,
    ) -> PyResult<Self> {
        let ops = extract_ops(opcodes)?;
        self.inner()
            .add_ops(ops.as_slice())
            .map_err(|err| PyException::new_err(format!("{}", err)))?;

        Ok(self.clone())
    }

    /// Add data to the script with appropriate push opcodes.
    ///
    /// Args:
    ///     data: Data bytes as hex, bytes, or list.
    ///
    /// Returns:
    ///     ScriptBuilder: Self for method chaining.
    ///
    /// Raises:
    ///     Exception: If the data cannot be added.
    pub fn add_data(&self, data: PyBinary) -> PyResult<Self> {
        let mut inner = self.inner();
        inner
            .add_data(data.as_ref())
            .map_err(|err| PyException::new_err(format!("{}", err)))?;

        Ok(self.clone())
    }

    /// Add an integer value to the script.
    ///
    /// Args:
    ///     value: The integer to add.
    ///
    /// Returns:
    ///     ScriptBuilder: Self for method chaining.
    ///
    /// Raises:
    ///     Exception: If the value cannot be added.
    pub fn add_i64(&self, value: i64) -> PyResult<Self> {
        let mut inner = self.inner();
        inner
            .add_i64(value)
            .map_err(|err| PyException::new_err(format!("{}", err)))?;

        Ok(self.clone())
    }

    /// Add a lock time value for CLTV (CheckLockTimeVerify).
    ///
    /// Args:
    ///     lock_time: DAA score or timestamp for time lock.
    ///
    /// Returns:
    ///     ScriptBuilder: Self for method chaining.
    ///
    /// Raises:
    ///     Exception: If the lock time cannot be added.
    pub fn add_lock_time(&self, lock_time: u64) -> PyResult<Self> {
        let mut inner = self.inner();
        inner
            .add_lock_time(lock_time)
            .map_err(|err| PyException::new_err(format!("{}", err)))?;

        Ok(self.clone())
    }

    /// Add a sequence value for CSV (CheckSequenceVerify).
    ///
    /// Args:
    ///     sequence: Relative time lock value.
    ///
    /// Returns:
    ///     ScriptBuilder: Self for method chaining.
    ///
    /// Raises:
    ///     Exception: If the sequence cannot be added.
    pub fn add_sequence(&self, sequence: u64) -> PyResult<Self> {
        let mut inner = self.inner();
        inner
            .add_sequence(sequence)
            .map_err(|err| PyException::new_err(format!("{}", err)))?;

        Ok(self.clone())
    }

    /// Calculate the canonical size for data in a script.
    ///
    /// Args:
    ///     data: Data bytes.
    ///
    /// Returns:
    ///     int: The size in bytes including push opcodes.
    #[staticmethod]
    pub fn canonical_data_size(data: PyBinary) -> PyResult<u32> {
        let size = native::ScriptBuilder::canonical_data_size(data.as_ref()) as u32;

        Ok(size)
    }

    /// Get the script as a hex string.
    ///
    /// Returns:
    ///     str: The script bytes as a hex string.
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        let inner = self.inner();

        inner
            .script()
            .to_vec()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }

    /// Drain and return the script, clearing the builder.
    ///
    /// Returns:
    ///     str: The script as a string.
    pub fn drain(&self) -> String {
        let mut inner = self.inner();

        String::from_utf8(inner.drain()).unwrap()
    }

    /// Create a P2SH (pay-to-script-hash) locking script.
    ///
    /// Returns:
    ///     ScriptPublicKey: The locking script for a P2SH address.
    #[pyo3(name = "create_pay_to_script_hash_script")]
    pub fn pay_to_script_hash_script(&self) -> PyScriptPublicKey {
        let inner = self.inner();
        let script = inner.script();

        standard::pay_to_script_hash_script(script).into()
    }

    /// Encode a P2SH signature script for spending.
    ///
    /// Args:
    ///     signature: The signature bytes.
    ///
    /// Returns:
    ///     str: The encoded signature script as hex.
    ///
    /// Raises:
    ///     Exception: If encoding fails.
    #[pyo3(name = "encode_pay_to_script_hash_signature_script")]
    pub fn pay_to_script_hash_signature_script(&self, signature: PyBinary) -> PyResult<String> {
        let inner = self.inner();
        let script = inner.script();
        let flags = inner.flags();
        let generated_script = standard::pay_to_script_hash_signature_script_with_flags(
            script.into(),
            signature.into(),
            flags,
        )
        .map_err(|err| PyException::new_err(format!("{}", err)))?;

        Ok(generated_script.to_hex())
    }

    /// Equality comparison.
    ///
    /// Args:
    ///     other: Another ScriptBuilder to compare against.
    ///
    /// Returns:
    ///     bool: True if both builders have produced identical scripts.
    // Cannot be derived via pyclass(eq)
    fn __eq__(&self, other: &PyScriptBuilder) -> bool {
        match (
            bincode::serialize(&self.inner().script()),
            bincode::serialize(&other.inner().script()),
        ) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }

    /// The detailed string representation.
    ///
    /// Returns:
    ///     str: The ScriptBuilder as a repr string.
    fn __repr__(&self) -> String {
        let inner = self.inner();
        format!("ScriptBuilder(script='{}')", inner.script().to_hex())
    }
}

// Builds script engine flags from the Python-facing kwargs, mirroring the WASM
// SDK's `ScriptBuilderOptions { flags: { covenantsEnabled, sigopScriptUnits } }`.
// `sigop_script_units` falls back to the native engine default when omitted.
// Shared with the zk-sdk builder (`PyZkScriptBuilder::new_r0`).
pub(crate) fn build_engine_flags(
    covenants_enabled: bool,
    sigop_script_units: Option<u64>,
) -> EngineFlags {
    let mut flags = EngineFlags {
        covenants_enabled,
        ..Default::default()
    };
    if let Some(units) = sigop_script_units {
        flags.sigop_script_units = ScriptUnits(units);
    }
    flags
}

// TODO change to PyOpcode struct and handle similar to PyBinary?
// Extracts multiple opcodes from a Python list[int | Opcodes]
fn extract_ops(input: &Bound<PyAny>) -> PyResult<Vec<u8>> {
    if let Ok(list) = input.cast::<pyo3::types::PyList>() {
        list.iter()
            .map(|item| extract_op(&item))
            .collect::<PyResult<Vec<u8>>>()
    } else {
        Err(PyException::new_err(
            "Expected a list containing ints and/or Opcodes enum variants.",
        ))
    }
}

// Extracts a single opcode from a Python int | Opcodes variant
fn extract_op(item: &Bound<PyAny>) -> PyResult<u8> {
    if let Ok(op) = item.extract::<u8>() {
        Ok(op)
    } else if let Ok(op) = item.extract::<PyOpcodes>() {
        Ok(op.get_value())
    } else {
        Err(PyException::new_err(
            "Expected int (u8) or Opcodes enum variant.",
        ))
    }
}
