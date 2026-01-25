use crate::address::PyAddress;
use crate::consensus::client::input::PyTransactionInput;
use crate::consensus::client::output::PyTransactionOutput;
use crate::consensus::core::network::PyNetworkType;
use crate::crypto::hashes::PyHash;
use crate::types::PyBinary;
use kaspa_consensus_client::{Transaction, TransactionInner, TransactionInput, TransactionOutput};
use kaspa_consensus_core::network::NetworkType;
use kaspa_consensus_core::subnets;
use kaspa_consensus_core::subnets::SubnetworkId;
use kaspa_consensus_core::tx as cctx;
use kaspa_txscript::extract_script_pub_key_address;
use pyo3::prelude::*;
use pyo3::types::PyType;
use pyo3::{exceptions::PyException, types::PyDict};
use pyo3_stub_gen::derive::*;
use workflow_core::hex::{FromHex, ToHex};

/// A Kaspa transaction.
///
/// Represents a complete transaction with inputs, outputs, and metadata.
/// Transactions are the fundamental unit of value transfer on the Kaspa network.
#[gen_stub_pyclass]
#[pyclass(name = "Transaction")]
#[derive(Clone)]
pub struct PyTransaction(Transaction);

impl PyTransaction {
    pub fn inner(&self) -> &Transaction {
        &self.0
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyTransaction {
    /// Check if this is a coinbase transaction.
    ///
    /// Returns:
    ///     bool: True if this is a coinbase (mining reward) transaction.
    #[pyo3(name = "is_coinbase")]
    pub fn is_coinbase(&self) -> bool {
        self.0.inner().subnetwork_id == subnets::SUBNETWORK_ID_COINBASE
    }

    /// Finalize the transaction and compute its ID.
    ///
    /// Returns:
    ///     Hash: The computed transaction ID.
    #[pyo3(name = "finalize")]
    pub fn finalize(&self) -> PyResult<PyHash> {
        let tx: cctx::Transaction = self.into();
        self.0.inner().id = tx.id();
        Ok(self.0.inner().id.into())
    }

    /// The transaction ID (hash).
    ///
    /// Returns:
    ///     str: The transaction ID as a hex string.
    #[getter]
    pub fn get_id(&self) -> String {
        self.0.inner().id.to_string()
    }

    /// Create a new transaction.
    ///
    /// Args:
    ///     version: Transaction version number.
    ///     inputs: List of transaction inputs.
    ///     outputs: List of transaction outputs.
    ///     lock_time: Lock time (block DAA score or timestamp).
    ///     subnetwork_id: Subnetwork identifier (hex string or bytes).
    ///     gas: Gas limit for smart contract execution.
    ///     payload: Optional transaction payload data.
    ///     mass: Transaction mass (for fee calculation).
    ///
    /// Returns:
    ///     Transaction: A new Transaction instance.
    ///
    /// Raises:
    ///     Exception: If the subnetwork_id is invalid or transaction creation fails.
    #[new]
    pub fn constructor(
        version: u16,
        inputs: Vec<PyTransactionInput>,
        outputs: Vec<PyTransactionOutput>,
        lock_time: u64,
        subnetwork_id: PyBinary,
        gas: u64,
        payload: PyBinary,
        mass: u64,
    ) -> PyResult<Self> {
        let subnetwork_id: SubnetworkId =
            subnetwork_id.data.as_slice().try_into().map_err(|err| {
                PyException::new_err(format!("subnetwork_id conversion error: {}", err))
            })?;

        let inner = Transaction::new(
            None,
            version,
            inputs.into_iter().map(TransactionInput::from).collect(),
            outputs.into_iter().map(TransactionOutput::from).collect(),
            lock_time,
            subnetwork_id,
            gas,
            payload.into(),
            mass,
        )
        .map_err(|err| PyException::new_err(err.to_string()))?;

        Ok(Self(inner))
    }

    /// The list of transaction inputs.
    ///
    /// Returns:
    ///     list[TransactionInput]: List of inputs spending previous outputs.
    #[getter]
    pub fn get_inputs(&self) -> PyResult<Vec<PyTransactionInput>> {
        Ok(self
            .0
            .inner()
            .inputs
            .clone()
            .into_iter()
            .map(PyTransactionInput::from)
            .collect())
    }

    /// Set the transaction inputs.
    ///
    /// Args:
    ///     value: List of TransactionInput objects.
    #[setter]
    pub fn set_inputs(&mut self, value: Vec<PyTransactionInput>) {
        self.0.inner().inputs = value.into_iter().map(TransactionInput::from).collect();
    }

    /// Extract unique addresses from transaction inputs.
    ///
    /// Args:
    ///     network_type: The network type to use for address encoding.
    ///
    /// Returns:
    ///     list[Address]: List of unique addresses referenced by inputs.
    pub fn addresses(
        &self,
        #[gen_stub(override_type(type_repr = "str | NetworkType"))] network_type: PyNetworkType,
    ) -> PyResult<Vec<PyAddress>> {
        let network_type: NetworkType = network_type.into();
        let mut list = std::collections::HashSet::new();
        for input in &self.0.inner().inputs {
            if let Some(utxo) = input.get_utxo() {
                if let Some(address) = &utxo.utxo.address {
                    list.insert(address.clone());
                } else if let Ok(address) = extract_script_pub_key_address(
                    &utxo.utxo.script_public_key,
                    network_type.into(),
                ) {
                    list.insert(address);
                }
            }
        }
        Ok(list.into_iter().map(PyAddress::from).collect())
    }

    /// The list of transaction outputs.
    ///
    /// Returns:
    ///     list[TransactionOutput]: List of outputs defining value destinations.
    #[getter]
    pub fn get_outputs(&self) -> PyResult<Vec<PyTransactionOutput>> {
        Ok(self
            .0
            .inner()
            .outputs
            .clone()
            .into_iter()
            .map(PyTransactionOutput::from)
            .collect())
    }

    /// Set the transaction outputs.
    ///
    /// Args:
    ///     value: List of TransactionOutput objects.
    #[setter]
    pub fn set_outputs(&mut self, value: Vec<PyTransactionOutput>) {
        self.0.inner().outputs = value.into_iter().map(TransactionOutput::from).collect();
    }

    /// The transaction version number.
    ///
    /// Returns:
    ///     int: The version number.
    #[getter]
    pub fn get_version(&self) -> u16 {
        self.0.inner().version
    }

    /// Set the transaction version number.
    ///
    /// Args:
    ///     value: The version number.
    #[setter]
    pub fn set_version(&mut self, value: u16) {
        self.0.inner().version = value;
    }

    /// The transaction lock time.
    /// Represents a DAA score or Unix timestamp before which the transaction cannot be included.
    ///
    /// Returns:
    ///     int: The lock time value.
    #[getter]
    pub fn get_lock_time(&self) -> u64 {
        self.0.inner().lock_time
    }

    /// Set the transaction lock time.
    ///
    /// Args:
    ///     value: The lock time (DAA score or Unix timestamp).
    #[setter]
    pub fn set_lock_time(&mut self, value: u64) {
        self.0.inner().lock_time = value;
    }

    /// The gas limit for smart contract execution.
    ///
    /// Returns:
    ///     int: The gas limit.
    #[getter]
    pub fn get_gas(&self) -> u64 {
        self.0.inner().gas
    }

    /// Set the gas limit for smart contract execution.
    ///
    /// Args:
    ///     value: The gas limit.
    #[setter]
    pub fn set_gas(&mut self, value: u64) {
        self.0.inner().gas = value;
    }

    /// The subnetwork identifier.
    ///
    /// Returns:
    ///     str: The subnetwork ID as a hex string.
    #[getter]
    pub fn get_subnetwork(&self) -> String {
        self.0.inner().subnetwork_id.to_string()
    }

    /// Set the subnetwork identifier.
    ///
    /// Args:
    ///     value: The subnetwork ID as a hex string.
    ///
    /// Raises:
    ///     Exception: If the hex string is invalid or has incorrect length.
    #[setter]
    pub fn set_subnetwork_id(&mut self, value: &str) -> PyResult<()> {
        let subnetwork_id = Vec::from_hex(value)
            .map_err(|err| PyException::new_err(err.to_string()))?
            .as_slice()
            .try_into()
            .map_err(|err| {
                PyException::new_err(format!("subnetwork_id conversion error: {}", err))
            })?;
        self.0.inner().subnetwork_id = subnetwork_id;
        Ok(())
    }

    /// The transaction payload data.
    ///
    /// Returns:
    ///     str: The payload as a hex string.
    #[getter]
    pub fn get_payload(&self) -> String {
        self.0.inner().payload.to_hex()
    }

    /// Set the transaction payload data.
    ///
    /// Args:
    ///     value: The payload as bytes or hex string.
    #[setter]
    pub fn set_payload(&mut self, value: PyBinary) {
        self.0.inner().payload = value.into();
    }

    /// The transaction mass used for fee calculation.
    ///
    /// Returns:
    ///     int: The transaction mass.
    #[getter]
    pub fn get_mass(&self) -> u64 {
        self.0.inner().mass
    }

    /// Set the transaction mass.
    ///
    /// Args:
    ///     value: The transaction mass value.
    #[setter]
    pub fn set_mass(&mut self, value: u64) {
        self.0.inner().mass = value;
    }

    /// Get a dictionary representation of the Transaction.
    /// Note that this creates a second separate object on the Python heap.
    ///
    /// Returns:
    ///     dict: the Transaction in dictionary form.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = serde_pyobject::to_pyobject(py, &self.inner().clone())?;
        Ok(dict.cast_into()?)
    }

    /// Create a Transaction from a dictionary.
    ///
    /// Args:
    ///     dict: Dictionary containing transaction fields with keys:
    ///         'id', 'version', `inputs`, `outputs`, `lockTime`,
    ///         `subnetworkId, `gas`, `payload`, `mass`.
    ///
    /// Returns:
    ///     Transaction: A new Transaction instance.
    ///
    /// Raises:
    ///     Exception: If required keys are missing or values are invalid.
    #[classmethod]
    fn from_dict(_cls: &Bound<'_, PyType>, dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        Self::try_from(dict)
    }

    // Cannot be derived via pyclass(eq) as wrapped Transaction type does not derive PartialEq/Eq
    fn __eq__(&self, other: &PyTransaction) -> bool {
        match (bincode::serialize(&self.0), bincode::serialize(&other.0)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}

impl From<Transaction> for PyTransaction {
    fn from(value: Transaction) -> Self {
        PyTransaction(value)
    }
}

impl From<PyTransaction> for Transaction {
    fn from(value: PyTransaction) -> Self {
        value.0
    }
}

impl From<&PyTransaction> for cctx::Transaction {
    fn from(value: &PyTransaction) -> Self {
        cctx::Transaction::from(&value.0)
    }
}

impl TryFrom<&Bound<'_, PyDict>> for PyTransaction {
    type Error = PyErr;
    fn try_from(dict: &Bound<PyDict>) -> PyResult<Self> {
        let inner: TransactionInner = serde_pyobject::from_pyobject(dict.clone())?;
        let tx = Transaction::new(
            Some(inner.id),
            inner.version,
            inner.inputs,
            inner.outputs,
            inner.lock_time,
            inner.subnetwork_id,
            inner.gas,
            inner.payload,
            inner.mass,
        )
        .map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(Self(tx))
    }
}
