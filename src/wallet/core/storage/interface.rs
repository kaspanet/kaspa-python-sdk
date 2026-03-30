use kaspa_wallet_core::storage::WalletDescriptor;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass]
#[pyclass(name = "WalletDescriptor")]
pub struct PyWalletDescriptor(WalletDescriptor);

#[gen_stub_pymethods]
#[pymethods]
impl PyWalletDescriptor {
    #[getter]
    fn get_title(&self) -> Option<String> {
        self.0.title.clone()
    }

    #[getter]
    fn get_filename(&self) -> String {
        self.0.filename.clone()
    }

    fn __repr__(&self) -> String {
        match &self.0.title {
            Some(title) => format!(
                "WalletDescriptor(title='{}', filename='{}')",
                title, self.0.filename
            ),
            None => format!("WalletDescriptor(filename='{}')", self.0.filename),
        }
    }
}

impl From<WalletDescriptor> for PyWalletDescriptor {
    fn from(value: WalletDescriptor) -> Self {
        PyWalletDescriptor(value)
    }
}

impl From<&WalletDescriptor> for PyWalletDescriptor {
    fn from(value: &WalletDescriptor) -> Self {
        PyWalletDescriptor(value.clone())
    }
}

impl From<PyWalletDescriptor> for WalletDescriptor {
    fn from(value: PyWalletDescriptor) -> Self {
        value.0
    }
}
