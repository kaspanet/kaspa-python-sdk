use kaspa_wallet_core::api::{AccountsDiscoveryKind, NewAddressKind};
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;
use std::str::FromStr;

crate::wrap_unit_enum_for_py!(
    /// Account Discovery Kind
    PyAccountsDiscoveryKind, "AccountsDiscoveryKind", AccountsDiscoveryKind, {
        Bip44
    }
);

impl FromStr for PyAccountsDiscoveryKind {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = match s.to_lowercase().as_str() {
            "bip44" => PyAccountsDiscoveryKind::Bip44,
            _ => Err(PyException::new_err(
                "Unsupported string value for `AccountsDiscoveryKind`",
            ))?,
        };

        Ok(v)
    }
}

impl<'py> FromPyObject<'_, 'py> for PyAccountsDiscoveryKind {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(s) = obj.extract::<String>() {
            PyAccountsDiscoveryKind::from_str(&s)
                .map_err(|err| PyException::new_err(err.to_string()))
        } else if let Ok(t) = obj.cast::<PyAccountsDiscoveryKind>() {
            Ok(t.borrow().clone())
        } else {
            Err(PyException::new_err(
                "Expected type `str` or `AccountsDiscoveryKind`",
            ))
        }
    }
}

crate::wrap_unit_enum_for_py!(
    /// Account Discovery Kind
    PyNewAddressKind, "NewAddressKind", NewAddressKind, {
        Change,
        Receive
    }
);

impl FromStr for PyNewAddressKind {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = match s.to_lowercase().as_str() {
            "change" => PyNewAddressKind::Change,
            "receive" => PyNewAddressKind::Receive,
            _ => Err(PyException::new_err(
                "Unsupported string value for `NewAddressKind`",
            ))?,
        };

        Ok(v)
    }
}

impl<'py> FromPyObject<'_, 'py> for PyNewAddressKind {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(s) = obj.extract::<String>() {
            PyNewAddressKind::from_str(&s).map_err(|err| PyException::new_err(err.to_string()))
        } else if let Ok(t) = obj.cast::<PyNewAddressKind>() {
            Ok(t.borrow().clone())
        } else {
            Err(PyException::new_err(
                "Expected type `str` or `NewAddressKind`",
            ))
        }
    }
}
