use super::outpoint::PyTransactionOutpoint;
use crate::{address::PyAddress, consensus::core::script_public_key::PyScriptPublicKey};
use kaspa_consensus_client::{UtxoEntry, UtxoEntryReference};
use kaspa_utils::hex::FromHex;
use pyo3::{
    exceptions::{PyException, PyKeyError},
    prelude::*,
    types::{PyDict, PyType},
};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use std::sync::Arc;

/// An unspent transaction output (UTXO).
///
/// Represents a spendable output from a previous transaction.
/// Contains information about the amount, locking script, and block position.
#[gen_stub_pyclass]
#[pyclass(name = "UtxoEntry")]
#[derive(Clone)]
pub struct PyUtxoEntry(UtxoEntry);

#[gen_stub_pymethods]
#[pymethods]
impl PyUtxoEntry {
    /// The address associated with this UTXO.
    ///
    /// Returns:
    ///     Address | None: The address, or None if not available.
    #[getter]
    pub fn get_address(&self) -> Option<PyAddress> {
        self.0.address.clone().map(PyAddress::from)
    }

    /// The outpoint identifying this UTXO.
    ///
    /// Returns:
    ///     TransactionOutpoint: The transaction outpoint reference.
    #[getter]
    pub fn get_outpoint(&self) -> PyTransactionOutpoint {
        self.0.outpoint.clone().into()
    }

    /// The amount in sompi (1 KAS = 100,000,000 sompi).
    ///
    /// Returns:
    ///     int: The UTXO value in sompi.
    #[getter]
    pub fn get_amount(&self) -> u64 {
        self.0.amount
    }

    /// The locking script for this UTXO.
    ///
    /// Returns:
    ///     ScriptPublicKey: The script public key.
    #[getter]
    pub fn get_script_public_key(&self) -> PyScriptPublicKey {
        self.0.script_public_key.clone().into()
    }

    /// The DAA score of the block containing this UTXO.
    ///
    /// Returns:
    ///     int: The block DAA score.
    #[getter]
    pub fn get_block_daa_score(&self) -> u64 {
        self.0.block_daa_score
    }

    /// Whether this UTXO is from a coinbase transaction.
    ///
    /// Returns:
    ///     bool: True if this is a coinbase UTXO.
    #[getter]
    pub fn get_is_coinbase(&self) -> bool {
        self.0.is_coinbase
    }

    /// Get a dictionary representation of the UtxoEntry.
    /// Note that this creates a second separate object on the Python heap.
    ///
    /// Returns:
    ///     dict: the UtxoEntry in dictionary form.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = serde_pyobject::to_pyobject(py, &self.0.clone())?;
        Ok(dict.cast_into()?)
    }

    /// Create a UtxoEntry from a dictionary.
    ///
    /// Args:
    ///     dict: Dictionary containing utxo entry fields with keys:
    ///         'address', 'outpoint', `amount`, `scriptPublicKey`,
    ///         `blockDaaScore`, `isCoinbase`.
    ///
    /// Returns:
    ///     UtxoEntry: A new UtxoEntry instance.
    ///
    /// Raises:
    ///     Exception: If required keys are missing or values are invalid.
    #[classmethod]
    fn from_dict(_cls: &Bound<'_, PyType>, dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        Self::try_from(dict)
    }

    // Cannot be derived via pyclass(eq) as wrapped PyUtxoEntry type does not derive PartialEq/Eq
    fn __eq__(&self, other: &PyUtxoEntry) -> bool {
        match (bincode::serialize(&self.0), bincode::serialize(&other.0)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}

impl From<PyUtxoEntry> for UtxoEntry {
    fn from(value: PyUtxoEntry) -> Self {
        value.0
    }
}

impl From<UtxoEntry> for PyUtxoEntry {
    fn from(value: UtxoEntry) -> Self {
        Self(value)
    }
}

impl TryFrom<&Bound<'_, PyDict>> for PyUtxoEntry {
    type Error = PyErr;
    fn try_from(dict: &Bound<PyDict>) -> PyResult<Self> {
        let utxo_entry: UtxoEntry = serde_pyobject::from_pyobject(dict.clone())?;
        Ok(Self(utxo_entry))
    }
}

/// A collection of UTXO entry references.
///
/// Provides methods for managing and querying multiple UTXOs.
#[gen_stub_pyclass]
#[pyclass(name = "UtxoEntries")]
#[derive(Clone)]
pub struct PyUtxoEntries(Arc<Vec<UtxoEntryReference>>);

#[gen_stub_pymethods]
#[pymethods]
impl PyUtxoEntries {
    /// The list of UTXO entry references.
    ///
    /// Returns:
    ///     list[UtxoEntryReference]: List of UTXO references.
    #[getter]
    pub fn get_items(&self) -> Vec<PyUtxoEntryReference> {
        self.0
            .as_ref()
            .clone()
            .into_iter()
            .map(PyUtxoEntryReference::from)
            .collect()
    }

    /// Set the list of UTXO entry references.
    ///
    /// Args:
    ///     value: List of UtxoEntryReference objects.
    #[setter]
    pub fn set_items(&mut self, value: Vec<PyUtxoEntryReference>) {
        self.0 = Arc::new(value.iter().map(UtxoEntryReference::from).collect());
    }

    /// Sort the UTXO entries by amount in ascending order.
    pub fn sort(&mut self) {
        let mut items = (*self.0).clone();
        items.sort_by_key(|e| e.amount());
        self.0 = Arc::new(items);
    }

    /// Calculate the total amount of all UTXOs.
    ///
    /// Returns:
    ///     int: The sum of all UTXO values in sompi.
    #[pyo3(name = "amount")]
    pub fn amount(&self) -> u64 {
        self.0.iter().map(|e| e.amount()).sum()
    }

    /// Get a dictionary representation of the UtxoEntries.
    /// Note that this creates a second separate object on the Python heap.
    ///
    /// Returns:
    ///     dict: the UtxoEntries in dictionary form.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = serde_pyobject::to_pyobject(py, &self.0.clone())?;
        Ok(dict.cast_into()?)
    }

    // Cannot be derived via pyclass(eq) as wrapped PyUtxoEntries type does not derive PartialEq/Eq
    fn __eq__(&self, other: &PyUtxoEntries) -> bool {
        match (bincode::serialize(&self.0), bincode::serialize(&other.0)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}

/// A reference to a UTXO entry.
///
/// Provides access to UTXO data for transaction building and signing.
#[gen_stub_pyclass]
#[pyclass(name = "UtxoEntryReference", eq)]
#[derive(Clone, PartialEq)]
pub struct PyUtxoEntryReference(UtxoEntryReference);

#[gen_stub_pymethods]
#[pymethods]
impl PyUtxoEntryReference {
    /// The underlying UTXO entry.
    ///
    /// Returns:
    ///     UtxoEntry: The UTXO entry data.
    #[getter]
    pub fn get_entry(&self) -> PyUtxoEntry {
        self.0.as_ref().clone().into()
    }

    /// The outpoint identifying this UTXO.
    ///
    /// Returns:
    ///     TransactionOutpoint: The transaction outpoint reference.
    #[getter]
    pub fn get_outpoint(&self) -> PyTransactionOutpoint {
        self.0.utxo.outpoint.clone().into()
    }

    /// The address associated with this UTXO.
    ///
    /// Returns:
    ///     Address | None: The address, or None if not available.
    #[getter]
    pub fn get_address(&self) -> Option<PyAddress> {
        self.0.utxo.address.clone().map(PyAddress::from)
    }

    /// The amount in sompi (1 KAS = 100,000,000 sompi).
    ///
    /// Returns:
    ///     int: The UTXO value in sompi.
    #[getter]
    pub fn get_amount(&self) -> u64 {
        self.0.utxo.amount
    }

    /// Whether this UTXO is from a coinbase transaction.
    ///
    /// Returns:
    ///     bool: True if this is a coinbase UTXO.
    #[getter]
    pub fn get_is_coinbase(&self) -> bool {
        self.0.utxo.is_coinbase
    }

    /// The DAA score of the block containing this UTXO.
    ///
    /// Returns:
    ///     int: The block DAA score.
    #[getter]
    pub fn get_block_daa_score(&self) -> u64 {
        self.0.utxo.block_daa_score
    }

    /// The locking script for this UTXO.
    ///
    /// Returns:
    ///     ScriptPublicKey: The script public key.
    #[getter]
    pub fn get_script_public_key(&self) -> PyScriptPublicKey {
        self.0.utxo.script_public_key.clone().into()
    }

    /// Get a dictionary representation of the Transaction.
    /// Note that this creates a second separate object on the Python heap.
    ///
    /// Returns:
    ///     dict: the Transaction in dictionary form.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = serde_pyobject::to_pyobject(py, &self.0.clone())?;
        Ok(dict.cast_into()?)
    }
}

impl From<PyUtxoEntryReference> for UtxoEntryReference {
    fn from(value: PyUtxoEntryReference) -> Self {
        value.0
    }
}

impl From<&PyUtxoEntryReference> for UtxoEntryReference {
    fn from(value: &PyUtxoEntryReference) -> Self {
        value.0.clone()
    }
}

impl From<UtxoEntryReference> for PyUtxoEntryReference {
    fn from(value: UtxoEntryReference) -> Self {
        Self(value)
    }
}

impl TryFrom<&Bound<'_, PyDict>> for PyUtxoEntryReference {
    type Error = PyErr;
    fn try_from(dict: &Bound<PyDict>) -> PyResult<Self> {
        let address = PyAddress::try_from(
            dict.get_item("address")?
                .ok_or_else(|| PyKeyError::new_err("Key `address` not present"))?
                .extract::<String>()?,
        )?;

        let outpoint = PyTransactionOutpoint::try_from(
            dict.get_item("outpoint")?
                .ok_or_else(|| PyKeyError::new_err("Key `outpoint` not present"))?
                .cast::<PyDict>()?,
        )?;

        let utxo_entry_value = dict
            .get_item("utxoEntry")?
            .ok_or_else(|| PyKeyError::new_err("Key `utxoEntry` not present"))?;
        let utxo_entry = utxo_entry_value.cast::<PyDict>()?;

        let amount: u64 = utxo_entry
            .get_item("amount")?
            .ok_or_else(|| PyKeyError::new_err("Key `amount` not present"))?
            .extract()?;

        let script_public_key = PyScriptPublicKey::from_hex(
            utxo_entry
                .get_item("scriptPublicKey")?
                .ok_or_else(|| PyKeyError::new_err("Key `scriptPublicKey` not present"))?
                .extract::<&str>()?,
        )
        .map_err(|err| PyException::new_err(format!("{}", err)))?;

        let block_daa_score: u64 = utxo_entry
            .get_item("blockDaaScore")?
            .ok_or_else(|| PyKeyError::new_err("Key `blockDaaScore` not present"))?
            .extract()?;

        let is_coinbase: bool = utxo_entry
            .get_item("isCoinbase")?
            .ok_or_else(|| PyKeyError::new_err("Key `is_coinbase` not present"))?
            .extract()?;

        let utxo = UtxoEntry {
            address: Some(address.into()),
            outpoint: outpoint.into(),
            amount,
            script_public_key: script_public_key.into(),
            block_daa_score,
            is_coinbase,
        };

        let inner = UtxoEntryReference {
            utxo: Arc::new(utxo),
        };

        Ok(Self(inner))
    }
}
