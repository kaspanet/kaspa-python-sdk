use crate::{
    consensus::core::script_public_key::PyScriptPublicKey, crypto::txscript::opcodes::PyOpcodes,
    types::PyBinary,
};
use kaspa_txscript::{script_builder as native, standard};
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
    /// Returns:
    ///     ScriptBuilder: A new empty ScriptBuilder instance.
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a script builder from an existing script.
    ///
    /// Args:
    ///     script: Existing script bytes as hex, bytes, or list.
    ///
    /// Returns:
    ///     ScriptBuilder: A new ScriptBuilder initialized with the script.
    #[staticmethod]
    pub fn from_script(script: PyBinary) -> PyResult<Self> {
        let builder = PyScriptBuilder::default();
        let script: Vec<u8> = script.into();
        builder.inner().script_mut().extend(&script);

        Ok(builder)
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
    pub fn add_op(&self, op: &Bound<PyAny>) -> PyResult<Self> {
        let op = extract_ops(op)?;
        let mut inner = self.inner();
        inner
            .add_op(op[0])
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
    pub fn add_ops(&self, opcodes: &Bound<PyAny>) -> PyResult<Self> {
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
        let generated_script =
            standard::pay_to_script_hash_signature_script(script.into(), signature.into())
                .map_err(|err| PyException::new_err(format!("{}", err)))?;

        Ok(generated_script.to_hex())
    }

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
}

// TODO change to PyOpcode struct and handle similar to PyBinary?
fn extract_ops(input: &Bound<PyAny>) -> PyResult<Vec<u8>> {
    if let Ok(opcode) = extract_op(input) {
        // Single u8 or Opcodes variant
        Ok(vec![opcode])
    } else if let Ok(list) = input.cast::<pyo3::types::PyList>() {
        // List of u8 or Opcodes variants
        list.iter()
            .map(|item| extract_op(&item))
            .collect::<PyResult<Vec<u8>>>()
    } else {
        Err(PyException::new_err(
            "Expected an Opcodes enum variant or an integer.",
        ))
    }
}

fn extract_op(item: &Bound<PyAny>) -> PyResult<u8> {
    if let Ok(op) = item.extract::<u8>() {
        Ok(op)
    } else if let Ok(op) = item.extract::<PyOpcodes>() {
        Ok(op.get_value())
    } else {
        Err(PyException::new_err("Expected Opcodes enum variant or u8"))
    }
}
