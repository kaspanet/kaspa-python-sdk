use kaspa_wallet_core::utxo::Balance;
use kaspa_wallet_core::utxo::balance::BalanceStrings;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

/// UTXO context balance summary.
#[gen_stub_pyclass]
#[pyclass(name = "Balance")]
#[derive(Clone)]
pub struct PyBalance(Balance);

#[gen_stub_pymethods]
#[pymethods]
impl PyBalance {
    /// Total mature balance in sompi.
    #[getter]
    pub fn get_mature(&self) -> u64 {
        self.0.mature
    }

    /// Total pending balance in sompi.
    #[getter]
    pub fn get_pending(&self) -> u64 {
        self.0.pending
    }

    /// Total outgoing balance in sompi.
    #[getter]
    pub fn get_outgoing(&self) -> u64 {
        self.0.outgoing
    }

    /// Number of mature UTXOs.
    #[getter]
    pub fn get_mature_utxo_count(&self) -> usize {
        self.0.mature_utxo_count
    }

    /// Number of pending UTXOs.
    #[getter]
    pub fn get_pending_utxo_count(&self) -> usize {
        self.0.pending_utxo_count
    }

    /// Number of stasis (coinbase) UTXOs.
    #[getter]
    pub fn get_stasis_utxo_count(&self) -> usize {
        self.0.stasis_utxo_count
    }
}

impl From<Balance> for PyBalance {
    fn from(value: Balance) -> Self {
        Self(value)
    }
}

impl From<PyBalance> for Balance {
    fn from(value: PyBalance) -> Self {
        value.0
    }
}

/// String-formatted balance values with network suffix.
#[gen_stub_pyclass]
#[pyclass(name = "BalanceStrings")]
pub struct PyBalanceStrings(BalanceStrings);

#[gen_stub_pymethods]
#[pymethods]
impl PyBalanceStrings {
    /// Mature balance formatted as a string (e.g., "1.23 KAS").
    #[getter]
    pub fn get_mature(&self) -> String {
        self.0.mature.clone()
    }

    /// Pending balance formatted as a string (if any).
    #[getter]
    pub fn get_pending(&self) -> Option<String> {
        self.0.pending.clone()
    }
}

impl From<BalanceStrings> for PyBalanceStrings {
    fn from(value: BalanceStrings) -> Self {
        Self(value)
    }
}
