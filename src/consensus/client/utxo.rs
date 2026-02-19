use super::outpoint::PyTransactionOutpoint;
use crate::{
    address::PyAddress, consensus::core::script_public_key::PyScriptPublicKey,
    crypto::hashes::PyHash, traits::TryToPyDict, types::PyBinary,
};
use kaspa_consensus_client::{UtxoEntry, UtxoEntryReference};
use kaspa_utils::hex::FromHex;
use pyo3::{
    exceptions::{PyKeyError, PyValueError},
    prelude::*,
    types::{PyDict, PyList, PyType},
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
    /// The address associated with this UTXO, or None if not available.
    #[getter]
    pub fn get_address(&self) -> Option<PyAddress> {
        self.0.address.clone().map(PyAddress::from)
    }

    /// The outpoint identifying this UTXO.
    #[getter]
    pub fn get_outpoint(&self) -> PyTransactionOutpoint {
        self.0.outpoint.clone().into()
    }

    /// The amount in sompi (1 KAS = 100,000,000 sompi).
    #[getter]
    pub fn get_amount(&self) -> u64 {
        self.0.amount
    }

    /// The locking script for this UTXO.
    #[getter]
    pub fn get_script_public_key(&self) -> PyScriptPublicKey {
        self.0.script_public_key.clone().into()
    }

    /// The DAA score of the block containing this UTXO.
    #[getter]
    pub fn get_block_daa_score(&self) -> u64 {
        self.0.block_daa_score
    }

    /// Whether this UTXO is from a coinbase transaction.
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
        self.0.try_to_pydict(py)
    }

    /// Create a UtxoEntry from a dictionary.
    ///
    /// Args:
    ///     dict: Dictionary containing utxo entry fields with keys:
    ///         - 'address' (str | None): The address string
    ///         - 'outpoint' (dict): Transaction outpoint with 'transactionId' and 'index'
    ///         - 'amount' (int): The UTXO value in sompi
    ///         - 'scriptPublicKey' (dict): Dict with 'version' (int) and 'script' (str) keys
    ///         - 'blockDaaScore' (int): Block DAA score
    ///         - 'isCoinbase' (bool): Whether from coinbase transaction
    ///         - 'covenantId' (str | None): The optional covenant ID of the UTXO.
    ///
    /// Returns:
    ///     UtxoEntry: A new UtxoEntry instance.
    ///
    /// Raises:
    ///     KeyError: If required keys are missing.
    ///     ValueError: If values are invalid.
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
        let address = if let Some(addr_item) = dict.get_item("address")? {
            if addr_item.is_none() {
                None
            } else {
                Some(PyAddress::try_from(addr_item.extract::<String>()?)?)
            }
        } else {
            None
        };

        let outpoint = PyTransactionOutpoint::try_from(
            dict.get_item("outpoint")?
                .ok_or_else(|| PyKeyError::new_err("Key `outpoint` not present"))?
                .cast::<PyDict>()?,
        )?;

        let amount: u64 = dict
            .get_item("amount")?
            .ok_or_else(|| PyKeyError::new_err("Key `amount` not present"))?
            .extract()?;

        let spk_obj = dict
            .get_item("scriptPublicKey")?
            .ok_or_else(|| PyKeyError::new_err("Key `scriptPublicKey` not present"))?;
        let script_public_key = if let Ok(spk) = spk_obj.extract::<PyScriptPublicKey>() {
            spk
        } else if let Ok(spk_dict) = spk_obj.cast::<PyDict>() {
            PyScriptPublicKey::constructor(
                spk_dict.as_any().get_item("version")?.extract::<u16>()?,
                spk_dict
                    .as_any()
                    .get_item("script")?
                    .extract::<PyBinary>()?,
            )?
        } else {
            return Err(PyValueError::new_err(
                "Value for `scriptPublicKey` must be type ScriptPublicKey or dict",
            ));
        };

        let block_daa_score: u64 = dict
            .get_item("blockDaaScore")?
            .ok_or_else(|| PyKeyError::new_err("Key `blockDaaScore` not present"))?
            .extract()?;

        let is_coinbase: bool = dict
            .get_item("isCoinbase")?
            .ok_or_else(|| PyKeyError::new_err("Key `isCoinbase` not present"))?
            .extract()?;

        let covenant_id: Option<PyHash> = dict
            .get_item("covenantId")?
            .ok_or_else(|| PyKeyError::new_err("Key `covenantId` not present"))?
            .extract()?;

        let utxo = UtxoEntry {
            address: address.map(|a| a.into()),
            outpoint: outpoint.into(),
            amount,
            script_public_key: script_public_key.into(),
            block_daa_score,
            is_coinbase,
            covenant_id: covenant_id.map(PyHash::into),
        };

        Ok(Self(utxo))
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
        let utxos = self
            .0
            .clone()
            .iter()
            .map(|utxo_ref| utxo_ref.try_to_pydict(py))
            .collect::<PyResult<Vec<Bound<'_, PyDict>>>>()?;

        let dict = PyDict::new(py);
        dict.set_item("utxos", PyList::new(py, utxos)?)?;

        Ok(dict)
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
    #[getter]
    pub fn get_entry(&self) -> PyUtxoEntry {
        self.0.as_ref().clone().into()
    }

    /// The outpoint identifying this UTXO.
    #[getter]
    pub fn get_outpoint(&self) -> PyTransactionOutpoint {
        self.0.utxo.outpoint.clone().into()
    }

    /// The address associated with this UTXO, or None if not available.
    #[getter]
    pub fn get_address(&self) -> Option<PyAddress> {
        self.0.utxo.address.clone().map(PyAddress::from)
    }

    /// The amount in sompi (1 KAS = 100,000,000 sompi).
    #[getter]
    pub fn get_amount(&self) -> u64 {
        self.0.utxo.amount
    }

    /// Whether this UTXO is from a coinbase transaction.
    #[getter]
    pub fn get_is_coinbase(&self) -> bool {
        self.0.utxo.is_coinbase
    }

    /// The DAA score of the block containing this UTXO.
    #[getter]
    pub fn get_block_daa_score(&self) -> u64 {
        self.0.utxo.block_daa_score
    }

    /// The locking script for this UTXO.
    #[getter]
    pub fn get_script_public_key(&self) -> PyScriptPublicKey {
        self.0.utxo.script_public_key.clone().into()
    }

    /// Get a dictionary representation of the UtxoEntryReference.
    /// Note that this creates a second separate object on the Python heap.
    ///
    /// Returns:
    ///     dict: the UtxoEntryReference in dictionary form.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        self.0.try_to_pydict(py)
    }

    /// Create a UtxoEntryReference from a dictionary.
    ///
    /// Supports two formats:
    ///
    /// Flat format:
    ///     - 'address' (str | None): The address string
    ///     - 'outpoint' (dict): Transaction outpoint with 'transactionId' and 'index'
    ///     - 'amount' (int): The UTXO value in sompi
    ///     - 'scriptPublicKey' (dict | str): Dict with 'version' and 'script', or hex string
    ///     - 'blockDaaScore' (int): Block DAA score
    ///     - 'isCoinbase' (bool): Whether from coinbase transaction
    ///     - 'covenantId' (str | None): The optional covenant ID of the UTXO.
    ///
    /// Nested format:
    ///     - 'address' (str | None): The address string
    ///     - 'outpoint' (dict): Transaction outpoint with 'transactionId' and 'index'
    ///     - 'utxoEntry' (dict): Nested dict containing amount, scriptPublicKey, blockDaaScore, isCoinbase, covenantId
    ///
    /// Returns:
    ///     UtxoEntryReference: A new UtxoEntryReference instance.
    ///
    /// Raises:
    ///     KeyError: If required keys are missing.
    ///     ValueError: If values are invalid.
    #[classmethod]
    fn from_dict(_cls: &Bound<'_, PyType>, dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        Self::try_from(dict)
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
        // Parse address (optional in flat format, required in nested)
        let address = if let Some(addr_item) = dict.get_item("address")? {
            if addr_item.is_none() {
                None
            } else {
                Some(PyAddress::try_from(addr_item.extract::<String>()?)?)
            }
        } else {
            None
        };

        let outpoint = PyTransactionOutpoint::try_from(
            dict.get_item("outpoint")?
                .ok_or_else(|| PyKeyError::new_err("Key `outpoint` not present"))?
                .cast::<PyDict>()?,
        )?;

        // Determine if using nested format (utxoEntry key) or flat format
        let source_dict = if let Some(utxo_entry_any) = dict.get_item("utxoEntry")? {
            // Nested format: read from utxoEntry dict
            utxo_entry_any.cast::<PyDict>()?.clone()
        } else {
            // Flat format: read directly from dict
            dict.clone()
        };

        let amount: u64 = source_dict
            .get_item("amount")?
            .ok_or_else(|| PyKeyError::new_err("Key `amount` not present"))?
            .extract()?;

        let spk_obj = source_dict
            .get_item("scriptPublicKey")?
            .ok_or_else(|| PyKeyError::new_err("Key `scriptPublicKey` not present"))?;
        let script_public_key = if let Ok(spk) = spk_obj.extract::<PyScriptPublicKey>() {
            spk
        } else if let Ok(spk_str) = spk_obj.extract::<String>() {
            PyScriptPublicKey::from_hex(&spk_str)?
        } else if let Ok(spk_dict) = spk_obj.cast::<PyDict>() {
            PyScriptPublicKey::constructor(
                spk_dict.as_any().get_item("version")?.extract::<u16>()?,
                spk_dict
                    .as_any()
                    .get_item("script")?
                    .extract::<PyBinary>()?,
            )?
        } else {
            return Err(PyValueError::new_err(
                "Value for `scriptPublicKey` must be type ScriptPublicKey, str, or dict",
            ));
        };

        let block_daa_score: u64 = source_dict
            .get_item("blockDaaScore")?
            .ok_or_else(|| PyKeyError::new_err("Key `blockDaaScore` not present"))?
            .extract()?;

        let is_coinbase: bool = source_dict
            .get_item("isCoinbase")?
            .ok_or_else(|| PyKeyError::new_err("Key `isCoinbase` not present"))?
            .extract()?;

        let covenant_id: Option<PyHash> = source_dict
            .get_item("covenantId")?
            .ok_or_else(|| PyKeyError::new_err("Key `covenantId` not present"))?
            .extract()?;

        let utxo = UtxoEntry {
            address: address.map(|a| a.into()),
            outpoint: outpoint.into(),
            amount,
            script_public_key: script_public_key.into(),
            block_daa_score,
            is_coinbase,
            covenant_id: covenant_id.map(PyHash::into),
        };

        let inner = UtxoEntryReference {
            utxo: Arc::new(utxo),
        };

        Ok(Self(inner))
    }
}
