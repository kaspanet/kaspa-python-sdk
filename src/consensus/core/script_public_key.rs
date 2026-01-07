use crate::types::PyBinary;
use kaspa_consensus_core::tx::ScriptPublicKey;
use kaspa_utils::hex::FromHex;
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use std::str::FromStr;

/// A script public key.
///
/// Represents the locking conditions for an output. This script defines
/// the conditions that must be met to spend the associated funds.
///
/// Category: Core/Types
#[gen_stub_pyclass]
#[pyclass(name = "ScriptPublicKey")]
#[derive(Clone)]
pub struct PyScriptPublicKey(ScriptPublicKey);

#[gen_stub_pymethods]
#[pymethods]
impl PyScriptPublicKey {
    /// Create a new script public key.
    ///
    /// Args:
    ///     version: The script version number.
    ///     script: The script bytes.
    ///
    /// Returns:
    ///     ScriptPublicKey: A new ScriptPublicKey instance.
    #[new]
    pub fn constructor(version: u16, script: PyBinary) -> PyResult<Self> {
        let inner = ScriptPublicKey::new(version, script.data.into());
        Ok(Self(inner))
    }

    /// The script bytes as a hex string.
    ///
    /// Returns:
    ///     str: The script data encoded as hexadecimal.
    #[getter]
    pub fn get_script(&self) -> String {
        // self.0.script.to_hex()
        self.0.script_as_hex()
    }
}

impl From<PyScriptPublicKey> for ScriptPublicKey {
    fn from(value: PyScriptPublicKey) -> Self {
        value.0
    }
}

impl From<ScriptPublicKey> for PyScriptPublicKey {
    fn from(value: ScriptPublicKey) -> Self {
        Self(value)
    }
}

impl FromHex for PyScriptPublicKey {
    type Error = PyErr;

    fn from_hex(hex_str: &str) -> Result<Self, Self::Error> {
        let inner = ScriptPublicKey::from_str(hex_str)
            .map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(Self(inner))
    }
}
