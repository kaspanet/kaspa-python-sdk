use crate::{
    consensus::client::{outpoint::PyTransactionOutpoint, utxo::PyUtxoEntryReference},
    types::PyBinary,
};
use kaspa_consensus_client::{TransactionInput, UtxoEntryReference};
use pyo3::prelude::*;
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
    ///
    /// Returns:
    ///     TransactionOutpoint: The previous output reference.
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

    /// The unlocking script (signature) that proves ownership of the UTXO.
    ///
    /// Returns:
    ///     str | None: The signature script as a hex string, or None if not set.
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
    ///
    /// Returns:
    ///     int: The sequence number.
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
    ///
    /// Returns:
    ///     int: The signature operation count.
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

    /// The UTXO entry reference for transaction signing.
    ///
    /// Returns:
    ///     UtxoEntryReference | None: The UTXO reference, or None if not set.
    #[getter]
    pub fn get_utxo(&self) -> Option<PyUtxoEntryReference> {
        self.0.inner().utxo.clone().map(PyUtxoEntryReference::from)
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
