use crate::wallet::core::{account::kind::PyAccountKind, utxo::balance::PyBalance};
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
    #[getter]
    pub fn get_kind(&self) -> PyAccountKind {
        PyAccountKind::from(self.0.kind)
    }

    #[getter]
    pub fn get_account_id(&self) -> String {
        self.0.account_id.to_hex()
    }

    #[getter]
    pub fn get_account_name(&self) -> Option<String> {
        self.0.account_name.clone()
    }

    #[getter]
    pub fn get_balance(&self) -> Option<PyBalance> {
        self.0.balance.clone().map(PyBalance::from)
    }

    #[getter]
    pub fn get_receive_address(&self) -> Option<String> {
        self.0.receive_address.as_ref().map(|a| a.to_string())
    }

    #[getter]
    pub fn get_change_address(&self) -> Option<String> {
        self.0.change_address.as_ref().map(|a| a.to_string())
    }
}

impl From<AccountDescriptor> for PyAccountDescriptor {
    fn from(value: AccountDescriptor) -> Self {
        Self(value)
    }
}
