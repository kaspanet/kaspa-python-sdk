use std::str::FromStr;

use kaspa_utils::hex::{FromHex, ToHex};
use kaspa_wallet_core::storage::{PrvKeyDataId, PrvKeyDataInfo, keydata::PrvKeyDataVariantKind};
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

/// A private key data identifier.
///
/// Wraps a hex-encoded private key data id. Can be constructed from a hex
/// string or obtained from a PrvKeyDataInfo.
#[gen_stub_pyclass]
#[pyclass(name = "PrvKeyDataId", skip_from_py_object, eq)]
#[derive(Clone, PartialEq)]
pub struct PyPrvKeyDataId(PrvKeyDataId);

#[gen_stub_pymethods]
#[pymethods]
impl PyPrvKeyDataId {
    /// Create a new PrvKeyDataId from a hex string.
    ///
    /// Args:
    ///     id: Hex-encoded private key data id.
    ///
    /// Returns:
    ///     PrvKeyDataId: A new PrvKeyDataId instance.
    ///
    /// Raises:
    ///     Exception: If the hex string is invalid.
    #[new]
    pub fn ctor(id: &str) -> PyResult<Self> {
        let inner =
            PrvKeyDataId::from_hex(id).map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(Self(inner))
    }

    /// The hex string representation.
    ///
    /// Returns:
    ///     str: The private key data id as a hex string.
    pub fn __str__(&self) -> String {
        self.0.to_hex()
    }

    /// The detailed string representation.
    ///
    /// Returns:
    ///     str: The private key data id as a repr string.
    pub fn __repr__(&self) -> String {
        format!("PrvKeyDataId('{}')", self.0.to_hex())
    }

    /// Get the hex string representation.
    ///
    /// Returns:
    ///     str: The private key data id as a hex string.
    #[pyo3(name = "to_hex")]
    pub fn py_to_hex(&self) -> String {
        self.0.to_hex()
    }
}

impl From<PrvKeyDataId> for PyPrvKeyDataId {
    fn from(value: PrvKeyDataId) -> Self {
        Self(value)
    }
}

impl From<&PrvKeyDataId> for PyPrvKeyDataId {
    fn from(value: &PrvKeyDataId) -> Self {
        Self(*value)
    }
}

impl From<PyPrvKeyDataId> for PrvKeyDataId {
    fn from(value: PyPrvKeyDataId) -> Self {
        value.0
    }
}

impl<'py> FromPyObject<'_, 'py> for PyPrvKeyDataId {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(s) = obj.extract::<String>() {
            let inner =
                PrvKeyDataId::from_hex(&s).map_err(|err| PyException::new_err(err.to_string()))?;
            Ok(Self(inner))
        } else if let Ok(id) = obj.cast::<Self>() {
            Ok(id.borrow().clone())
        } else {
            Err(PyException::new_err(
                "Expected type `str` or `PrvKeyDataId`",
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
    /// The private key data id.
    #[getter]
    pub fn get_id(&self) -> PyPrvKeyDataId {
        PyPrvKeyDataId::from(self.0.id)
    }

    /// The user-assigned name of this private key data, or None if unset.
    #[getter]
    pub fn get_name(&self) -> Option<String> {
        self.0.name.clone()
    }

    /// Whether the private key data is encrypted at rest.
    #[getter]
    pub fn get_is_encrypted(&self) -> bool {
        self.0.is_encrypted
    }

    /// The string representation.
    ///
    /// Returns:
    ///     str: The PrvKeyDataInfo as a string.
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
