use pyo3::prelude::*;
use pyo3_stub_gen::derive::gen_stub_pyfunction;

use crate::consensus::core::network::PyNetworkType;

/// Convert KAS to sompi (1 KAS = 100,000,000 sompi).
///
/// Args:
///     kaspa: The amount in KAS.
///
/// Returns:
///     int: The amount in sompi.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "kaspa_to_sompi")]
pub fn py_kaspa_to_sompi(kaspa: f64) -> u64 {
    kaspa_wallet_core::utils::kaspa_to_sompi(kaspa)
}

/// Convert sompi to KAS (1 KAS = 100,000,000 sompi).
///
/// Args:
///     sompi: The amount in sompi.
///
/// Returns:
///     float: The amount in KAS.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "sompi_to_kaspa")]
pub fn py_sompi_to_kaspa(sompi: u64) -> f64 {
    kaspa_wallet_core::utils::sompi_to_kaspa(sompi)
}

/// Convert sompi to a formatted KAS string with network suffix.
///
/// Args:
///     sompi: The amount in sompi.
///     network: The network type for the suffix.
///
/// Returns:
///     str: Formatted string like "1.5 KAS" or "1.5 TKAS".
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "sompi_to_kaspa_string_with_suffix")]
pub fn py_sompi_to_kaspa_string_with_suffix(
    sompi: u64,
    #[gen_stub(override_type(type_repr = "str | NetworkType"))] network: PyNetworkType,
) -> PyResult<String> {
    Ok(kaspa_wallet_core::utils::sompi_to_kaspa_string_with_suffix(
        sompi,
        &network.into(),
    ))
}
