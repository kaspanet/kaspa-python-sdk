use crate::consensus::client::transaction::PyTransaction;
use crate::consensus::core::network::PyNetworkId;

use super::super::imports::*;
use super::generator::{
    PendingTransaction, PyGenerator, PyGeneratorSummary, PyOutputs, PyUtxoEntries,
};
use kaspa_consensus_client::*;
use kaspa_consensus_core::subnets::SUBNETWORK_ID_NATIVE;
use kaspa_wallet_core::result::Result;
use pyo3_stub_gen::derive::gen_stub_pyfunction;
// use pyo3::{exceptions::PyException, prelude::*};

/// Create a single transaction from UTXOs.
///
/// Args:
///     utxo_entry_source: List of UTXO entries to spend.
///     outputs: List of payment outputs.
///     priority_fee: Priority fee in sompi.
///     payload: Optional transaction payload data.
///     sig_op_count: Signature operations per input (default: 1).
///
/// Returns:
///     Transaction: The created transaction (unsigned).
///
/// Raises:
///     Exception: If transaction creation fails or fee exceeds input amount.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "create_transaction")]
#[pyo3(signature = (utxo_entry_source, outputs, priority_fee, payload=None, sig_op_count=None))]
pub fn py_create_transaction(
    utxo_entry_source: PyUtxoEntries,
    outputs: PyOutputs,
    priority_fee: u64,
    payload: Option<PyBinary>,
    sig_op_count: Option<u8>,
) -> PyResult<PyTransaction> {
    let payload: Vec<u8> = payload.map(Into::into).unwrap_or_default();
    let sig_op_count = sig_op_count.unwrap_or(1);

    let mut total_input_amount = 0;
    let mut entries = vec![];

    let inputs = utxo_entry_source
        .entries
        .into_iter()
        .enumerate()
        .map(|(sequence, reference)| {
            let UtxoEntryReference { utxo } = &reference;
            total_input_amount += utxo.amount();
            entries.push(reference.clone());
            TransactionInput::new(
                utxo.outpoint.clone(),
                None,
                sequence as u64,
                sig_op_count,
                Some(reference),
            )
        })
        .collect::<Vec<TransactionInput>>();

    if priority_fee > total_input_amount {
        return Err(PyException::new_err(format!(
            "priority fee({priority_fee}) > amount({total_input_amount})"
        )));
    }

    let outputs = outputs
        .outputs
        .into_iter()
        .map(|output| output.into())
        .collect::<Vec<TransactionOutput>>();

    let transaction = Transaction::new(
        None,
        0,
        inputs,
        outputs,
        0,
        SUBNETWORK_ID_NATIVE,
        0,
        payload,
        0,
    )
    .map_err(|err| PyException::new_err(err.to_string()))?;

    Ok(transaction.into())
}

/// Create one or more transactions with automatic UTXO selection and change handling.
///
/// Handles large transfers that may require multiple transactions due to mass limits.
///
/// Args:
///     network_id: The network to build transactions for (required for UTXO entries).
///     entries: UtxoContext or list of UTXO entries to spend from.
///     change_address: Address to send change to.
///     outputs: Optional list of payment outputs.
///     payload: Optional transaction payload data.
///     fee_rate: Optional fee rate multiplier.
///     priority_fee: Additional fee in sompi.
///     priority_entries: UTXOs to use first.
///     sig_op_count: Signature operations per input (default: 1).
///     minimum_signatures: For multisig fee estimation.
///
/// Returns:
///     dict: Dictionary with "transactions" (list) and "summary" keys.
///
/// Raises:
///     Exception: If transaction creation fails.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "create_transactions")]
#[pyo3(signature = (entries, change_address, network_id=None, outputs=None, payload=None, fee_rate=None, priority_fee=None, priority_entries=None, sig_op_count=None, minimum_signatures=None))]
pub fn py_create_transactions<'a>(
    py: Python<'a>,
    #[gen_stub(override_type(type_repr = "UtxoEntries | UtxoContext"))] entries: Bound<'_, PyAny>,
    change_address: PyAddress,
    network_id: Option<PyNetworkId>,
    outputs: Option<PyOutputs>,
    payload: Option<PyBinary>,
    fee_rate: Option<f64>,
    priority_fee: Option<u64>,
    priority_entries: Option<PyUtxoEntries>,
    sig_op_count: Option<u8>,
    minimum_signatures: Option<u16>,
) -> PyResult<Bound<'a, PyDict>> {
    let generator = PyGenerator::ctor(
        entries,
        change_address,
        network_id,
        outputs,
        payload,
        fee_rate,
        priority_fee,
        priority_entries,
        sig_op_count,
        minimum_signatures,
    )?;

    let transactions = generator
        .iter()
        .map(|r| r.map(PendingTransaction::from))
        .collect::<Result<Vec<_>>>()
        .map_err(|err| PyException::new_err(err.to_string()))?;
    let summary = generator.summary();
    let dict = PyDict::new(py);
    dict.set_item("transactions", transactions)?;
    dict.set_item("summary", summary)?;
    Ok(dict)
}

/// Estimate transaction fees and count without creating transactions.
///
/// Args:
///     network_id: The network to estimate for (required for UTXO entries).
///     entries: UtxoContext or list of UTXO entries to spend from.
///     change_address: Address to send change to.
///     outputs: Optional list of payment outputs.
///     payload: Optional transaction payload data.
///     fee_rate: Optional fee rate multiplier.
///     priority_fee: Additional fee in sompi.
///     priority_entries: UTXOs to use first.
///     sig_op_count: Signature operations per input (default: 1).
///     minimum_signatures: For multisig fee estimation.
///
/// Returns:
///     GeneratorSummary: Summary with fee, transaction count, and other details.
///
/// Raises:
///     Exception: If estimation fails.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "estimate_transactions")]
#[pyo3(signature = (entries, change_address, network_id=None, outputs=None, payload=None, fee_rate=None, priority_fee=None, priority_entries=None, sig_op_count=None, minimum_signatures=None))]
pub fn py_estimate_transactions(
    #[gen_stub(override_type(type_repr = "UtxoEntries | UtxoContext"))] entries: Bound<'_, PyAny>,
    change_address: PyAddress,
    network_id: Option<PyNetworkId>,
    outputs: Option<PyOutputs>,
    payload: Option<PyBinary>,
    fee_rate: Option<f64>,
    priority_fee: Option<u64>,
    priority_entries: Option<PyUtxoEntries>,
    sig_op_count: Option<u8>,
    minimum_signatures: Option<u16>,
) -> PyResult<PyGeneratorSummary> {
    let generator = PyGenerator::ctor(
        entries,
        change_address,
        network_id,
        outputs,
        payload,
        fee_rate,
        priority_fee,
        priority_entries,
        sig_op_count,
        minimum_signatures,
    )?;

    generator
        .iter()
        .collect::<Result<Vec<_>>>()
        .map_err(|err| PyException::new_err(err.to_string()))?;
    Ok(generator.summary())
}
