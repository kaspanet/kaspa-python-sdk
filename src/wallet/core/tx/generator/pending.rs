use super::super::super::imports::*;
use crate::{
    consensus::{
        client::{transaction::PyTransaction, utxo::PyUtxoEntryReference},
        core::hashing::PySighashType,
    },
    rpc::wrpc::client::PyRpcClient,
    wallet::keys::privatekey::PyPrivateKey,
};
use kaspa_consensus_client::Transaction;
use kaspa_consensus_core::hashing::wasm::SighashType;
use kaspa_wallet_core::tx::generator as native;
use pyo3::types::PyList;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use workflow_core::hex::ToHex;
use zeroize::Zeroize;

/// A transaction ready for signing and submission.
///
/// Created by iterating over a Generator. Contains the transaction
/// along with metadata about fees, amounts, and UTXOs being spent.
#[gen_stub_pyclass]
#[pyclass]
pub struct PendingTransaction(native::PendingTransaction);

#[gen_stub_pymethods]
#[pymethods]
impl PendingTransaction {
    /// The transaction ID (hash) as a hex string.
    #[getter]
    fn get_id(&self) -> String {
        self.0.id().to_string()
    }

    /// The total payment amount in sompi (excluding change and fees), or None for sweep transactions.
    #[getter]
    fn get_payment_amount(&self) -> Option<u64> {
        self.0.payment_value()
    }

    /// The change amount returned to the sender in sompi.
    #[getter]
    fn get_change_amount(&self) -> u64 {
        self.0.change_value()
    }

    /// The transaction fee in sompi.
    #[getter]
    fn get_fee_amount(&self) -> u64 {
        self.0.fees()
    }

    /// The transaction mass (used for fee calculation).
    #[getter]
    fn get_mass(&self) -> u64 {
        self.0.mass()
    }

    /// The minimum number of signatures required.
    #[getter]
    fn get_minimum_signatures(&self) -> u16 {
        self.0.minimum_signatures()
    }

    /// The total value of all inputs in sompi.
    #[getter]
    fn get_aggregate_input_amount(&self) -> u64 {
        self.0.aggregate_input_value()
    }

    /// The total value of all outputs in sompi.
    #[getter]
    fn get_aggregate_output_amount(&self) -> u64 {
        self.0.aggregate_output_value()
    }

    /// The transaction type: "batch" for intermediate or "final" for last.
    #[getter]
    fn get_transaction_type(&self) -> String {
        if self.0.is_batch() {
            "batch".to_string()
        } else {
            "final".to_string()
        }
    }

    /// Get the unique addresses referenced by this transaction's inputs.
    ///
    /// Returns:
    ///     list[Address]: List of addresses.
    fn addresses(&self) -> Vec<PyAddress> {
        self.0
            .addresses()
            .clone()
            .into_iter()
            .map(PyAddress::from)
            .collect()
    }

    /// Get the UTXO entries being spent by this transaction.
    ///
    /// Returns:
    ///     list[UtxoEntryReference]: List of UTXO entries.
    fn get_utxo_entries(&self) -> Vec<PyUtxoEntryReference> {
        self.0
            .utxo_entries()
            .values()
            .map(|utxo_entry| PyUtxoEntryReference::from(utxo_entry.clone()))
            .collect()
    }

    /// Create a signature for a specific input.
    ///
    /// Args:
    ///     input_index: The index of the input to sign.
    ///     private_key: The private key for signing.
    ///     sighash_type: The signature hash type (default: All).
    ///
    /// Returns:
    ///     str: The signature as a hex string.
    ///
    /// Raises:
    ///     Exception: If signing fails.
    #[pyo3(signature = (input_index, private_key, sighash_type=None))]
    fn create_input_signature(
        &self,
        input_index: u8,
        private_key: &PyPrivateKey,
        #[gen_stub(override_type(type_repr = "str | SighashType | None = SighashType.All"))]
        sighash_type: Option<PySighashType>,
    ) -> PyResult<String> {
        let sighash_type: SighashType = sighash_type.unwrap_or(PySighashType::All).into();

        let mut key_bytes = private_key.secret_bytes();
        let signature = self
            .0
            .create_input_signature(input_index.into(), &key_bytes, sighash_type.into())
            .map_err(|err| PyException::new_err(format!("{}", err)))?;
        key_bytes.zeroize();

        Ok(signature.to_hex())
    }

    /// Fill an input's signature script with a pre-computed signature.
    ///
    /// Args:
    ///     input_index: The index of the input to fill.
    ///     signature_script: The signature script bytes.
    ///
    /// Raises:
    ///     Exception: If filling fails.
    fn fill_input(&self, input_index: u8, signature_script: PyBinary) -> PyResult<()> {
        self.0
            .fill_input(input_index.into(), signature_script.into())
            .map_err(|err| PyException::new_err(err.to_string()))?;

        Ok(())
    }

    /// Sign a specific input with a private key.
    ///
    /// Args:
    ///     input_index: The index of the input to sign.
    ///     private_key: The private key for signing.
    ///     sighash_type: The signature hash type (default: All).
    ///
    /// Raises:
    ///     Exception: If signing fails.
    fn sign_input(
        &self,
        input_index: u8,
        private_key: &PyPrivateKey,
        #[gen_stub(override_type(type_repr = "str | SighashType | None = SighashType.All"))]
        sighash_type: Option<PySighashType>,
    ) -> PyResult<()> {
        let sighash_type: SighashType = sighash_type.unwrap_or(PySighashType::All).into();

        let mut key_bytes = private_key.secret_bytes();
        self.0
            .sign_input(input_index.into(), &key_bytes, sighash_type.into())
            .map_err(|err| PyException::new_err(format!("{}", err)))?;
        key_bytes.zeroize();

        Ok(())
    }

    /// Sign all inputs with the provided private keys.
    ///
    /// Args:
    ///     private_keys: List of PrivateKey objects for signing.
    ///     check_fully_signed: Verify all inputs are signed (default: None).
    ///
    /// Raises:
    ///     Exception: If signing fails or transaction is not fully signed.
    #[pyo3(signature = (private_keys, check_fully_signed=None))]
    fn sign<'py>(
        &self,
        private_keys: Bound<'py, PyList>,
        check_fully_signed: Option<bool>,
    ) -> PyResult<()> {
        let mut keys: Vec<[u8; 32]> = Vec::with_capacity(private_keys.len());
        for item in private_keys.iter() {
            let key: PyRef<'_, PyPrivateKey> = item.extract()?;
            keys.push(key.secret_bytes());
        }
        self.0
            .try_sign_with_keys(&keys, check_fully_signed)
            .map_err(|err| PyException::new_err(format!("{}", err)))?;
        keys.zeroize();
        Ok(())
    }

    /// Submit the signed transaction to the network.
    ///
    /// Args:
    ///     rpc_client: The RPC client for submission.
    ///
    /// Returns:
    ///     str: The transaction ID on success (async).
    ///
    /// Raises:
    ///     Exception: If submission fails.
    fn submit<'py>(
        &self,
        py: Python<'py>,
        rpc_client: &PyRpcClient,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.0.clone();
        let rpc: Arc<DynRpcApi> = rpc_client.client().clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let txid = inner
                .try_submit(&rpc)
                .await
                .map_err(|err| PyException::new_err(format!("{}", err)))?;
            Ok(txid.to_string())
        })
    }

    /// The underlying transaction object for manual inspection or modification.
    #[getter]
    fn get_transaction(&self) -> PyResult<PyTransaction> {
        Ok(Transaction::from_cctx_transaction(&self.0.transaction(), self.0.utxo_entries()).into())
    }
}

impl From<native::PendingTransaction> for PendingTransaction {
    fn from(pending_transaction: native::PendingTransaction) -> Self {
        Self(pending_transaction)
    }
}
