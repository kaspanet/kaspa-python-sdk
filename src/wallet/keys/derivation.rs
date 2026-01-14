use kaspa_bip32::ChildNumber;
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use std::str::FromStr;

/// A BIP-32 derivation path for hierarchical key derivation.
///
/// Represents a path like "m/44'/111111'/0'/0/0" for deriving keys.
///
/// Category: Wallet/Keys
#[gen_stub_pyclass]
#[pyclass(name = "DerivationPath", eq)]
#[derive(Clone, PartialEq)]
pub struct PyDerivationPath(kaspa_bip32::DerivationPath);

#[gen_stub_pymethods]
#[pymethods]
impl PyDerivationPath {
    /// Create a derivation path from a string.
    ///
    /// Args:
    ///     path: A path string (e.g., "m/44'/111111'/0'").
    ///
    /// Returns:
    ///     DerivationPath: A new DerivationPath instance.
    ///
    /// Raises:
    ///     Exception: If the path format is invalid.
    #[new]
    pub fn new(path: &str) -> PyResult<PyDerivationPath> {
        let inner = kaspa_bip32::DerivationPath::from_str(path)
            .map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(PyDerivationPath(inner))
    }

    /// Check if the path is empty (no components).
    ///
    /// Returns:
    ///     bool: True if empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of path components.
    ///
    /// Returns:
    ///     int: The path length.
    #[pyo3(name = "length")]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Get the parent path (one level up).
    ///
    /// Returns:
    ///     DerivationPath | None: The parent path, or None if at root.
    pub fn parent(&self) -> Option<PyDerivationPath> {
        self.0.parent().map(PyDerivationPath)
    }

    /// Append a child index to the path.
    ///
    /// Args:
    ///     child_number: The child index.
    ///     hardened: Whether to use hardened derivation (default: False).
    ///
    /// Raises:
    ///     Exception: If the child number is invalid.
    #[pyo3(signature = (child_number, hardened=None))]
    pub fn push(&mut self, child_number: u32, hardened: Option<bool>) -> PyResult<()> {
        let child = ChildNumber::new(child_number, hardened.unwrap_or(false))
            .map_err(|err| PyException::new_err(err.to_string()))?;
        self.0.push(child);
        Ok(())
    }

    /// Convert to string representation.
    ///
    /// Returns:
    ///     str: The path as a string (e.g., "m/44'/111111'/0'").
    #[pyo3(name = "to_string")]
    pub fn to_str(&self) -> String {
        self.0.to_string()
    }
}

impl From<PyDerivationPath> for kaspa_bip32::DerivationPath {
    fn from(value: PyDerivationPath) -> Self {
        value.0
    }
}
