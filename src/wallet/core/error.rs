use crate::error::IntoPyResult;
use kaspa_wallet_core::error::Error as NativeError;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

crate::create_py_exception!(
    /// General catch-all wallet error
    PyWalletError, "WalletError"
);

crate::create_py_exception!(
    /// Custom wallet error
    PyWalletCustomError, "WalletCustomError"
);

pub struct Error(NativeError);

impl From<Error> for PyErr {
    fn from(err: Error) -> Self {
        match err.0 {
            NativeError::Custom(msg) => PyWalletCustomError::new_err(msg),
            other => PyWalletError::new_err(other.to_string()),
        }
    }
}

impl From<NativeError> for Error {
    fn from(err: NativeError) -> Self {
        Error(err)
    }
}

impl<T> IntoPyResult<T> for std::result::Result<T, NativeError> {
    fn into_py_result(self) -> PyResult<T> {
        self.map_err(|e| PyErr::from(Error(e)))
    }
}
