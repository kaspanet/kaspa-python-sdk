use crate::consensus::client::transaction::PyTransaction;
use crate::consensus::core::network::PyNetworkId;

use super::super::imports::*;
use kaspa_consensus_core::config::params::Params;
use kaspa_consensus_core::mass::{UtxoCell, calc_storage_mass};
use kaspa_wallet_core::tx::{MAXIMUM_STANDARD_TRANSACTION_MASS, mass};
use pyo3_stub_gen::derive::gen_stub_pyfunction;
// use pyo3::prelude::*;

/// Get the maximum allowed mass for a standard transaction.
///
/// Returns:
///     int: The maximum standard transaction mass.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "maximum_standard_transaction_mass")]
pub fn py_maximum_standard_transaction_mass() -> u64 {
    MAXIMUM_STANDARD_TRANSACTION_MASS
}

/// Calculate the mass of an unsigned transaction.
///
/// Args:
///     network_id: The network identifier.
///     tx: The transaction to calculate mass for.
///     minimum_signatures: Minimum signatures per input (default: 1).
///
/// Returns:
///     int: The calculated transaction mass.
///
/// Raises:
///     Exception: If mass calculation fails.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "calculate_transaction_mass")]
#[pyo3(signature = (network_id, tx, minimum_signatures=None))]
pub fn py_calculate_unsigned_transaction_mass(
    network_id: PyNetworkId,
    tx: PyTransaction,
    minimum_signatures: Option<u16>,
) -> PyResult<u64> {
    let network_id: NetworkId = network_id.into();
    let consensus_params = Params::from(network_id);
    let mc = mass::MassCalculator::new(&consensus_params);
    mc.calc_overall_mass_for_unsigned_client_transaction(
        &tx.into(),
        minimum_signatures.unwrap_or(1),
    )
    .map_err(|err| PyException::new_err(err.to_string()))
}

/// Calculate and update the mass field of an unsigned transaction.
///
/// Args:
///     network_id: The network identifier.
///     tx: The transaction to update.
///     minimum_signatures: Minimum signatures per input (default: 1).
///
/// Returns:
///     bool: True if mass is within limits and was updated, False if too large.
///
/// Raises:
///     Exception: If mass calculation fails.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "update_transaction_mass")]
#[pyo3(signature = (network_id, tx, minimum_signatures=None))]
pub fn py_update_unsigned_transaction_mass(
    network_id: PyNetworkId,
    tx: PyTransaction,
    minimum_signatures: Option<u16>,
) -> PyResult<bool> {
    let network_id: NetworkId = network_id.into();
    let consensus_params = Params::from(network_id);
    let mc = mass::MassCalculator::new(&consensus_params);
    let tx: kaspa_consensus_client::Transaction = tx.into();
    let mass = mc
        .calc_overall_mass_for_unsigned_client_transaction(&tx, minimum_signatures.unwrap_or(1))
        .map_err(|err| PyException::new_err(err.to_string()))?;
    if mass > MAXIMUM_STANDARD_TRANSACTION_MASS {
        Ok(false)
    } else {
        tx.set_mass(mass);
        Ok(true)
    }
}

/// Calculate the fee for an unsigned transaction based on its mass.
///
/// Args:
///     network_id: The network identifier.
///     tx: The transaction to calculate fee for.
///     minimum_signatures: Minimum signatures per input (default: 1).
///
/// Returns:
///     int | None: The fee in sompi, or None if mass exceeds limits.
///
/// Raises:
///     Exception: If mass calculation fails.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "calculate_transaction_fee")]
#[pyo3(signature = (network_id, tx, minimum_signatures=None))]
pub fn py_calculate_unsigned_transaction_fee(
    network_id: PyNetworkId,
    tx: PyTransaction,
    minimum_signatures: Option<u16>,
) -> PyResult<Option<u64>> {
    let network_id: NetworkId = network_id.into();
    let consensus_params = Params::from(network_id);
    let mc = mass::MassCalculator::new(&consensus_params);
    let mass = mc
        .calc_overall_mass_for_unsigned_client_transaction(
            &tx.into(),
            minimum_signatures.unwrap_or(1),
        )
        .map_err(|err| PyException::new_err(err.to_string()))?;
    if mass > MAXIMUM_STANDARD_TRANSACTION_MASS {
        Ok(None)
    } else {
        Ok(Some(mc.calc_fee_for_mass(mass)))
    }
}

/// Calculate the storage mass for a transaction.
///
/// Storage mass penalizes transactions that increase the UTXO set size
/// or create many small outputs.
///
/// Args:
///     network_id: The network identifier.
///     input_values: List of input values in sompi.
///     output_values: List of output values in sompi.
///
/// Returns:
///     int | None: The storage mass, or None if not applicable.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "calculate_storage_mass")]
pub fn py_calculate_storage_mass(
    network_id: PyNetworkId,
    input_values: Vec<u64>,
    output_values: Vec<u64>,
) -> PyResult<Option<u64>> {
    let network_id: NetworkId = network_id.into();
    let consensus_params = Params::from(network_id);

    let input_values = input_values
        .iter()
        .map(|v| UtxoCell::new(1, *v))
        .collect::<Vec<UtxoCell>>();
    let output_values = output_values
        .iter()
        .map(|v| UtxoCell::new(1, *v))
        .collect::<Vec<UtxoCell>>();

    let storage_mass = calc_storage_mass(
        false,
        input_values.into_iter(),
        output_values.into_iter(),
        consensus_params.storage_mass_parameter,
    );

    Ok(storage_mass)
}
