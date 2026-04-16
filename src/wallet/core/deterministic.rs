use kaspa_utils::hex::{FromHex, ToHex};
use kaspa_wallet_core::prelude::AccountId;
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

/// A Kaspa account identifier.
///
/// Wraps a hex-encoded account id. Can be constructed from a hex string
/// or obtained from an AccountDescriptor.
#[gen_stub_pyclass]
#[pyclass(name = "AccountId", skip_from_py_object, eq)]
#[derive(Clone, PartialEq)]
pub struct PyAccountId(AccountId);

#[gen_stub_pymethods]
#[pymethods]
impl PyAccountId {
    /// Create a new AccountId from a hex string.
    ///
    /// Args:
    ///     id: Hex-encoded account id.
    ///
    /// Returns:
    ///     AccountId: A new AccountId instance.
    ///
    /// Raises:
    ///     Exception: If the hex string is invalid.
    #[new]
    pub fn ctor(id: &str) -> PyResult<Self> {
        let inner = AccountId::from_hex(id).map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(Self(inner))
    }

    /// The hex string representation.
    ///
    /// Returns:
    ///     str: The account id as a hex string.
    pub fn __str__(&self) -> String {
        self.0.to_hex()
    }

    /// The detailed string representation.
    ///
    /// Returns:
    ///     str: The account id as a repr string.
    pub fn __repr__(&self) -> String {
        format!("AccountId('{}')", self.0.to_hex())
    }

    /// Get the hex string representation.
    ///
    /// Returns:
    ///     str: The account id as a hex string.
    #[pyo3(name = "to_hex")]
    pub fn py_to_hex(&self) -> String {
        self.0.to_hex()
    }
}

impl From<AccountId> for PyAccountId {
    fn from(value: AccountId) -> Self {
        Self(value)
    }
}

impl From<PyAccountId> for AccountId {
    fn from(value: PyAccountId) -> Self {
        value.0
    }
}

impl<'py> FromPyObject<'_, 'py> for PyAccountId {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(s) = obj.extract::<String>() {
            let inner =
                AccountId::from_hex(&s).map_err(|err| PyException::new_err(err.to_string()))?;
            Ok(Self(inner))
        } else if let Ok(id) = obj.cast::<Self>() {
            Ok(id.borrow().clone())
        } else {
            Err(PyException::new_err("Expected type `str` or `AccountId`"))
        }
    }
}
