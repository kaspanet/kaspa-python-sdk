use std::str::FromStr;

use kaspa_utils::hex::ToHex;
use kaspa_wallet_core::storage::{PrvKeyDataInfo, keydata::PrvKeyDataVariantKind};
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::*;

crate::wrap_unit_enum_for_py!(
    /// Private Key Data Variant Kind
    PyPrvKeyDataVariantKind, "PrvKeyDataVariantKind", PrvKeyDataVariantKind, {
        Mnemonic,
        Bip39Seed,
        ExtendedPrivateKey,
        SecretKey
    }
);

impl FromStr for PyPrvKeyDataVariantKind {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = match s.to_lowercase().as_str() {
            "mnemonic" => PyPrvKeyDataVariantKind::Mnemonic,
            "bip39seed" => PyPrvKeyDataVariantKind::Bip39Seed,
            "extendedprivatekey" => PyPrvKeyDataVariantKind::ExtendedPrivateKey,
            "secretkey" => PyPrvKeyDataVariantKind::SecretKey,
            _ => Err(PyException::new_err(
                "Unsupported string value for `PrvKeyDataVariantKind`",
            ))?,
        };

        Ok(v)
    }
}

impl<'py> FromPyObject<'_, 'py> for PyPrvKeyDataVariantKind {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(s) = obj.extract::<String>() {
            PyPrvKeyDataVariantKind::from_str(&s)
                .map_err(|err| PyException::new_err(err.to_string()))
        } else if let Ok(t) = obj.cast::<PyPrvKeyDataVariantKind>() {
            Ok(t.borrow().clone())
        } else {
            Err(PyException::new_err(
                "Expected type `str` or `PrvKeyDataVariantKind`",
            ))
        }
    }
}

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

    fn __repr__(&self) -> String {
        format!(
            "PrvKeyDataInfo(id='{}', name={}, is_encrypted={})",
            self.0.id.to_hex(),
            match &self.0.name {
                Some(name) => format!("'{}'", name),
                None => "None".to_string(),
            },
            self.0.is_encrypted
        )
    }
}

impl From<&PrvKeyDataInfo> for PyPrvKeyDataInfo {
    fn from(value: &PrvKeyDataInfo) -> Self {
        Self(value.clone())
    }
}

impl From<PrvKeyDataInfo> for PyPrvKeyDataInfo {
    fn from(value: PrvKeyDataInfo) -> Self {
        Self(value)
    }
}
