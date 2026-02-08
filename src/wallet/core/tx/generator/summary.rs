// use crate::imports::*;
use kaspa_wallet_core::tx::generator as core;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

/// A class containing a summary produced by transaction Generator.
///
/// This class contains the number of transactions, the aggregated fees,
/// the aggregated UTXOs and the final transaction amount that includes
/// both network and QoS (priority) fees.
///
/// See Also:
///     create_transactions, Generator
#[gen_stub_pyclass]
#[pyclass(name = "GeneratorSummary")]
pub struct PyGeneratorSummary(core::GeneratorSummary);

#[gen_stub_pymethods]
#[pymethods]
impl PyGeneratorSummary {
    /// The network type used for generation.
    #[getter]
    pub fn get_network_type(&self) -> String {
        self.0.network_type().to_string()
    }

    /// The total number of UTXOs consumed.
    #[getter]
    pub fn get_utxos(&self) -> usize {
        self.0.aggregated_utxos()
    }

    /// The total fees across all generated transactions in sompi.
    #[getter]
    pub fn get_fees(&self) -> u64 {
        self.0.aggregate_fees()
    }

    /// The number of transactions generated.
    #[getter]
    pub fn get_transactions(&self) -> usize {
        self.0.number_of_generated_transactions()
    }

    /// The final transaction amount in sompi, or None if not applicable.
    #[getter]
    pub fn get_final_amount(&self) -> Option<u64> {
        self.0.final_transaction_amount()
    }

    /// The ID of the final transaction, or None if not yet generated.
    #[getter]
    pub fn get_final_transaction_id(&self) -> Option<String> {
        self.0.final_transaction_id().map(|id| id.to_string())
    }

    // Cannot be derived via pyclass(eq)
    fn __eq__(&self, other: &PyGeneratorSummary) -> bool {
        match (bincode::serialize(&self.0), bincode::serialize(&other.0)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}

impl From<core::GeneratorSummary> for PyGeneratorSummary {
    fn from(inner: core::GeneratorSummary) -> Self {
        Self(inner)
    }
}
