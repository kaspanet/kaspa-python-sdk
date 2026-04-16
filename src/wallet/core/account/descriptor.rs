use crate::wallet::core::{
    account::kind::PyAccountKind, deterministic::PyAccountId, utxo::balance::PyBalance,
};
use kaspa_utils::hex::ToHex;
use kaspa_wallet_core::account::descriptor::AccountDescriptor;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

#[gen_stub_pyclass]
#[pyclass(name = "AccountDescriptor")]
#[derive(Clone)]
pub struct PyAccountDescriptor(AccountDescriptor);

#[gen_stub_pymethods]
#[pymethods]
impl PyAccountDescriptor {
    /// The account kind (e.g. Bip32, Keypair, MultiSig).
    #[getter]
    pub fn get_kind(&self) -> PyAccountKind {
        PyAccountKind::from(self.0.kind)
    }

    /// The account id.
    #[getter]
    pub fn get_account_id(&self) -> PyAccountId {
        PyAccountId::from(self.0.account_id)
    }

    /// The user-assigned account name, or None if unset.
    #[getter]
    pub fn get_account_name(&self) -> Option<String> {
        self.0.account_name.clone()
    }

    /// The current balance of the account, or None if not yet known.
    #[getter]
    pub fn get_balance(&self) -> Option<PyBalance> {
        self.0.balance.clone().map(PyBalance::from)
    }

    /// The current receive address as a string, or None if not available.
    #[getter]
    pub fn get_receive_address(&self) -> Option<String> {
        self.0.receive_address.as_ref().map(|a| a.to_string())
    }

    /// The current change address as a string, or None if not available.
    #[getter]
    pub fn get_change_address(&self) -> Option<String> {
        self.0.change_address.as_ref().map(|a| a.to_string())
    }

    /// The string representation.
    ///
    /// Returns:
    ///     str: The AccountDescriptor as a string.
    fn __repr__(&self) -> String {
        let name = match &self.0.account_name {
            Some(name) => format!("'{}'", name),
            None => "None".to_string(),
        };
        format!(
            "AccountDescriptor(kind={}, account_id='{}', account_name={})",
            self.0.kind,
            self.0.account_id.to_hex(),
            name
        )
    }
}

impl From<AccountDescriptor> for PyAccountDescriptor {
    fn from(value: AccountDescriptor) -> Self {
        Self(value)
    }
}
