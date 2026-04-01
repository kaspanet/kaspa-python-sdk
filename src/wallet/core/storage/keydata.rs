use kaspa_utils::hex::ToHex;
use kaspa_wallet_core::storage::PrvKeyDataInfo;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

#[gen_stub_pyclass]
#[pyclass(name = "PrvKeyDataInfo")]
pub struct PyPrvKeyDataInfo(PrvKeyDataInfo);

#[gen_stub_pymethods]
#[pymethods]
impl PyPrvKeyDataInfo {
    #[getter]
    pub fn get_id(&self) -> String {
        self.0.id.to_hex()
    }

    #[getter]
    pub fn get_name(&self) -> Option<String> {
        self.0.name.clone()
    }

    #[getter]
    pub fn get_is_encrypted(&self) -> bool {
        self.0.is_encrypted
    }
}

impl From<&PrvKeyDataInfo> for PyPrvKeyDataInfo {
    fn from(value: &PrvKeyDataInfo) -> Self {
        Self(value.clone())
    }
}
