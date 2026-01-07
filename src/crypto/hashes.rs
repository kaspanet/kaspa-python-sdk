use kaspa_hashes::Hash;
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::*;
use std::str::FromStr;

/// A 32-byte hash value.
///
/// Used for transaction IDs, block hashes, and other cryptographic purposes.
///
/// Category: Core/Types
#[gen_stub_pyclass]
#[pyclass(name = "Hash")]
#[derive(Clone)]
pub struct PyHash(Hash);

#[gen_stub_pymethods]
#[pymethods]
impl PyHash {
    /// Create a new Hash from a hex string.
    ///
    /// Args:
    ///     hex_str: A 64-character hex string representing the hash.
    ///
    /// Returns:
    ///     Hash: A new Hash instance.
    ///
    /// Raises:
    ///     Exception: If the hex string is invalid.
    #[new]
    pub fn constructor(hex_str: &str) -> PyResult<Self> {
        let inner =
            Hash::from_str(hex_str).map_err(|err| PyException::new_err(format!("{}", err)))?;
        Ok(Self(inner))
    }

    /// Convert the hash to a hex string.
    ///
    /// Returns:
    ///     str: A 64-character hex string.
    #[pyo3(name = "to_string")]
    pub fn py_to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<PyHash> for Hash {
    fn from(value: PyHash) -> Self {
        value.0
    }
}

impl From<Hash> for PyHash {
    fn from(value: Hash) -> Self {
        PyHash(value)
    }
}

impl TryFrom<String> for PyHash {
    type Error = PyErr;

    fn try_from(value: String) -> PyResult<Self> {
        let inner = Hash::from_str(&value).map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(Self(inner))
    }
}

// impl PyStubType for PyHash {
//     fn type_output() -> TypeInfo {
//         TypeInfo::locally_defined("Hash", "kaspa".into())
//     }
// }
