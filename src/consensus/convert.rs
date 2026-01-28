use kaspa_consensus_client::{TransactionOutpoint, UtxoEntryReference};
use kaspa_utils::hex::ToHex;
use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Trait for converting Rust types to Python dictionaries.
///
/// This trait provides a standardized way to convert wrapped SDK types
/// to Python dicts with a flat structure (no unnecessary nesting).
/// 
/// A custom trait is required as `py: Python` is required fn arg so 
/// that dict can be created on the Python heap.
pub trait ToPyDict {
    /// Convert this value to a Python dictionary.
    ///
    /// # Arguments
    /// * `py` - Python interpreter token
    ///
    /// # Returns
    /// A Python dictionary representation of the value.
    fn to_py_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>>;
}

// **********************************************
// Trait impls for rusty-kaspa native types
// **********************************************

impl ToPyDict for TransactionOutpoint {
    fn to_py_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = serde_pyobject::to_pyobject(py, self.inner())?;
        Ok(dict.cast_into::<PyDict>()?)
    }
}

impl ToPyDict for UtxoEntryReference {
    fn to_py_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);

        // Set `address` key
        if let Some(addr) = self.address() {
            dict.set_item("address", addr.to_string())?;
        } else {
            dict.set_item("address", py.None())?;
        }

        // Set `outpoint` key
        dict.set_item(
            "outpoint",
            serde_pyobject::to_pyobject(py, self.outpoint().inner())?,
        )?;

        // Set `amount` key
        dict.set_item("amount", self.amount())?;

        // Set `scriptPublicKey` key
        dict.set_item(
            "scriptPublicKey",
            format!(
                "{:02x}{}",
                self.script_public_key().version(),
                self.script_public_key().script().to_hex()
            ),
        )?;

        // Set `blockDaaScore` key
        dict.set_item("blockDaaScore", self.block_daa_score())?;

        // Set `isCoinbase` key
        dict.set_item("isCoinbase", self.is_coinbase())?;

        Ok(dict)
    }
}

// ToPyDict for Transaction
// ToPyDict for TransactionInput
// ToPyDict for TransactionOutput
// ToPyDict for UtxoEntry
// ToPyDict for UtxoEntries