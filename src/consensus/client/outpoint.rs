use crate::crypto::hashes::PyHash;
use kaspa_consensus_client::{TransactionOutpoint, TransactionOutpointInner};
use kaspa_consensus_core::tx::TransactionIndexType;
use pyo3::{prelude::*, types::PyDict};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

/// Reference to a specific output in a previous transaction.
///
/// An outpoint uniquely identifies a UTXO by its transaction ID and output index.
#[gen_stub_pyclass]
#[pyclass(name = "TransactionOutpoint")]
#[derive(Clone)]
pub struct PyTransactionOutpoint(TransactionOutpoint);

#[gen_stub_pymethods]
#[pymethods]
impl PyTransactionOutpoint {
    /// Create a new transaction outpoint.
    ///
    /// Args:
    ///     transaction_id: The ID of the transaction containing the output.
    ///     index: The index of the output within the transaction.
    ///
    /// Returns:
    ///     TransactionOutpoint: A new TransactionOutpoint instance.
    #[new]
    pub fn ctor(transaction_id: PyHash, index: u32) -> Self {
        let inner = TransactionOutpoint::new(transaction_id.into(), index);
        Self(inner)
    }

    /// Get the unique identifier string for this outpoint.
    ///
    /// Returns:
    ///     str: A string in format "transaction_id-index".
    #[pyo3(name = "get_id")]
    pub fn id_string(&self) -> String {
        format!(
            "{}-{}",
            self.0.get_transaction_id_as_string(),
            self.get_index()
        )
    }

    /// The ID of the transaction containing the referenced output.
    ///
    /// Returns:
    ///     str: The transaction ID as a hex string.
    #[getter]
    pub fn get_transaction_id(&self) -> String {
        self.0.inner().transaction_id.to_string()
    }

    /// The index of the output within the transaction.
    ///
    /// Returns:
    ///     int: The output index.
    #[getter]
    pub fn get_index(&self) -> TransactionIndexType {
        self.0.inner().index
    }

    // Cannot be derived via pyclass(eq) as wrapped PyTransactionOutpoint does not derive PartialEq/Eq
    fn __eq__(&self, other: &PyTransactionOutpoint) -> bool {
        match (bincode::serialize(&self.0), bincode::serialize(&other.0)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}

impl From<PyTransactionOutpoint> for TransactionOutpoint {
    fn from(value: PyTransactionOutpoint) -> Self {
        value.0
    }
}

impl From<TransactionOutpoint> for PyTransactionOutpoint {
    fn from(value: TransactionOutpoint) -> Self {
        Self(value)
    }
}

impl TryFrom<&Bound<'_, PyDict>> for PyTransactionOutpoint {
    type Error = PyErr;
    fn try_from(dict: &Bound<PyDict>) -> PyResult<Self> {
        let inner: TransactionOutpointInner = serde_pyobject::from_pyobject(dict.clone())?;
        let outpoint = TransactionOutpoint::new(inner.transaction_id, inner.index);
        Ok(Self(outpoint))
    }
}
