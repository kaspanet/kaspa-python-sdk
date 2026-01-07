use std::str::FromStr;

use kaspa_wallet_core::account::kind::AccountKind;
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

/// Account kind is a string signature that represents an account type.
/// Account kind is used to identify the account type during
/// serialization, deserialization and various API calls.
///
/// Note:
///     Supported values:
///
///     - `legacy`: Legacy account type
///     - `bip32`: BIP-32 HD wallet account
///     - `multisig`: Multi-signature account
///     - `keypair`: Simple keypair account
///     - `bip32watch`: Watch-only BIP-32 account
///
/// Category: Wallet/Core
#[gen_stub_pyclass]
#[pyclass(name = "AccountKind")]
#[derive(Clone)]
pub struct PyAccountKind(AccountKind);

#[gen_stub_pymethods]
#[pymethods]
impl PyAccountKind {
    /// Create a new AccountKind from a string.
    ///
    /// Args:
    ///     kind: The account kind string.
    ///
    /// Returns:
    ///     AccountKind: A new AccountKind instance.
    ///
    /// Raises:
    ///     Exception: If the kind string is invalid.
    #[new]
    pub fn ctor(kind: &str) -> PyResult<Self> {
        let inner =
            AccountKind::from_str(kind).map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(Self(inner))
    }

    /// Get the string representation.
    ///
    /// Returns:
    ///     str: The account kind as a string.
    pub fn __str__(&self) -> String {
        self.py_to_string()
    }

    /// Get the string representation.
    ///
    /// Returns:
    ///     str: The account kind as a string.
    #[pyo3(name = "to_string")]
    pub fn py_to_string(&self) -> String {
        self.0.as_str().to_string()
    }
}

impl From<AccountKind> for PyAccountKind {
    fn from(value: AccountKind) -> Self {
        Self(value)
    }
}

impl From<PyAccountKind> for AccountKind {
    fn from(value: PyAccountKind) -> Self {
        value.0
    }
}
