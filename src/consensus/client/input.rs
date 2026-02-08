use crate::{
    consensus::client::{outpoint::PyTransactionOutpoint, utxo::PyUtxoEntryReference},
    consensus::convert::TryToPyDict,
    types::PyBinary,
};
use kaspa_consensus_client::{TransactionInput, UtxoEntryReference};
use pyo3::{
    exceptions::PyKeyError,
    prelude::*,
    types::{PyDict, PyType},
};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use workflow_core::hex::ToHex;

/// A transaction input referencing a previous output.
///
/// Inputs reference UTXOs (unspent transaction outputs) that are being spent.
#[gen_stub_pyclass]
#[pyclass(name = "TransactionInput")]
#[derive(Clone)]
pub struct PyTransactionInput(TransactionInput);

#[gen_stub_pymethods]
#[pymethods]
impl PyTransactionInput {
    /// Create a new transaction input.
    ///
    /// Args:
    ///     previous_outpoint: Reference to the UTXO being spent.
    ///     signature_script: The unlocking script (signature).
    ///     sequence: Sequence number for relative time locks.
    ///     sig_op_count: Number of signature operations.
    ///     utxo: Optional UTXO entry reference for signing.
    ///
    /// Returns:
    ///     TransactionInput: A new TransactionInput instance.
    #[new]
    #[pyo3(signature = (previous_outpoint, signature_script, sequence, sig_op_count, utxo=None))]
    pub fn constructor(
        previous_outpoint: PyTransactionOutpoint,
        signature_script: PyBinary,
        sequence: u64,
        sig_op_count: u8,
        utxo: Option<PyUtxoEntryReference>,
    ) -> PyResult<Self> {
        let inner = TransactionInput::new(
            previous_outpoint.into(),
            Some(signature_script.into()),
            sequence,
            sig_op_count,
            utxo.map(UtxoEntryReference::from),
        );
        Ok(Self(inner))
    }

    /// The outpoint referencing the UTXO being spent.
    #[getter]
    pub fn get_previous_outpoint(&self) -> PyTransactionOutpoint {
        self.0.inner().previous_outpoint.clone().into()
    }

    /// Set the outpoint referencing the UTXO being spent.
    ///
    /// Args:
    ///     value: The previous output reference.
    #[setter]
    pub fn set_previous_outpoint(&mut self, value: PyTransactionOutpoint) -> PyResult<()> {
        self.0.inner().previous_outpoint = value.into();
        Ok(())
    }

    /// The unlocking script (signature) as a hex string, or None if not set.
    #[getter]
    pub fn get_signature_script_as_hex(&self) -> Option<String> {
        self.0
            .inner()
            .signature_script
            .as_ref()
            .map(|script| script.to_hex())
    }

    /// Set the unlocking script (signature).
    ///
    /// Args:
    ///     value: The signature script as bytes or hex string.
    #[setter]
    pub fn set_signature_script(&mut self, value: PyBinary) -> PyResult<()> {
        self.0.set_signature_script(value.into());
        Ok(())
    }

    /// The sequence number used for relative time locks.
    #[getter]
    pub fn get_sequence(&self) -> u64 {
        self.0.inner().sequence
    }

    /// Set the sequence number.
    ///
    /// Args:
    ///     value: The sequence number for relative time locks.
    #[setter]
    pub fn set_sequence(&mut self, value: u64) {
        self.0.inner().sequence = value;
    }

    /// The number of signature operations in this input.
    #[getter]
    pub fn get_sig_op_count(&self) -> u8 {
        self.0.inner().sig_op_count
    }

    /// Set the signature operation count.
    ///
    /// Args:
    ///     value: The number of signature operations.
    #[setter]
    pub fn set_sig_op_count(&mut self, value: u8) {
        self.0.inner().sig_op_count = value;
    }

    /// The UTXO entry reference for transaction signing, or None if not set.
    #[getter]
    pub fn get_utxo(&self) -> Option<PyUtxoEntryReference> {
        self.0.inner().utxo.clone().map(PyUtxoEntryReference::from)
    }

    /// Get a dictionary representation of the TransactionInput.
    /// Note that this creates a second separate object on the Python heap.
    ///
    /// Returns:
    ///     dict: the TransactionInput in dictionary form.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        self.0.try_to_pydict(py)
    }

    /// Create a TransactionInput from a dictionary.
    ///
    /// Args:
    ///     dict: Dictionary containing transaction input fields with keys:
    ///         - 'previousOutpoint' (dict): Dict with 'transactionId' (str) and 'index' (int) keys
    ///         - 'signatureScript' (str | None): The signature script as hex string
    ///         - 'sequence' (int): Sequence number
    ///         - 'sigOpCount' (int): Signature operation count
    ///         - 'utxo' (dict | None): Optional UTXO entry reference dict
    ///
    /// Returns:
    ///     TransactionInput: A new TransactionInput instance.
    ///
    /// Raises:
    ///     KeyError: If required keys are missing.
    ///     ValueError: If values are invalid.
    #[classmethod]
    fn from_dict(_cls: &Bound<'_, PyType>, dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        Self::try_from(dict)
    }

    // Cannot be derived via pyclass(eq) as wrapped PyTransactionInput type does not derive PartialEq/Eq
    fn __eq__(&self, other: &PyTransactionInput) -> bool {
        match (bincode::serialize(&self.0), bincode::serialize(&other.0)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}

impl From<TransactionInput> for PyTransactionInput {
    fn from(value: TransactionInput) -> Self {
        Self(value)
    }
}

impl From<PyTransactionInput> for TransactionInput {
    fn from(value: PyTransactionInput) -> Self {
        value.0
    }
}

impl TryFrom<&Bound<'_, PyDict>> for PyTransactionInput {
    type Error = PyErr;
    fn try_from(dict: &Bound<PyDict>) -> PyResult<Self> {
        // Parse previousOutpoint
        let previous_outpoint = PyTransactionOutpoint::try_from(
            dict.get_item("previousOutpoint")?
                .ok_or_else(|| PyKeyError::new_err("Key `previousOutpoint` not present"))?
                .cast::<PyDict>()?,
        )?;

        // Parse signatureScript (optional, can be None or empty string)
        let signature_script: Option<Vec<u8>> =
            if let Some(sig_item) = dict.get_item("signatureScript")? {
                if sig_item.is_none() {
                    None
                } else {
                    let sig_hex: String = sig_item.extract()?;
                    if sig_hex.is_empty() {
                        Some(Vec::new())
                    } else {
                        let mut data = vec![0u8; sig_hex.len() / 2];
                        faster_hex::hex_decode(sig_hex.as_bytes(), &mut data)
                            .map_err(|e| PyKeyError::new_err(format!("Invalid hex: {}", e)))?;
                        Some(data)
                    }
                }
            } else {
                None
            };

        // Parse sequence
        let sequence: u64 = dict
            .get_item("sequence")?
            .ok_or_else(|| PyKeyError::new_err("Key `sequence` not present"))?
            .extract()?;

        // Parse sigOpCount
        let sig_op_count: u8 = dict
            .get_item("sigOpCount")?
            .ok_or_else(|| PyKeyError::new_err("Key `sigOpCount` not present"))?
            .extract()?;

        // Parse utxo (optional)
        let utxo: Option<UtxoEntryReference> = if let Some(utxo_item) = dict.get_item("utxo")? {
            if utxo_item.is_none() {
                None
            } else {
                let utxo_dict = utxo_item.cast::<PyDict>()?;
                Some(PyUtxoEntryReference::try_from(utxo_dict)?.into())
            }
        } else {
            None
        };

        let input = TransactionInput::new(
            previous_outpoint.into(),
            signature_script,
            sequence,
            sig_op_count,
            utxo,
        );
        Ok(Self(input))
    }
}
