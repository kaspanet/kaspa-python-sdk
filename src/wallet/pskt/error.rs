use kaspa_wallet_pskt::wasm::error::Error as NativeError;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::gen_stub_pyclass;

// Custom Python Exceptions

crate::create_py_exception!(
    /// Custom PSKT Error
    PyPsktCustomError, "PsktCustomError"
);

crate::create_py_exception!(
    /// PSKT State Error
    PyPsktStateError, "PsktStateError"
);

crate::create_py_exception!(
    /// PSKT Expected State Error
    PyPsktExpectedStateError, "PsktExpectedStateError"
);

crate::create_py_exception!(
    /// PSKT Constructor Error
    PyPsktCtorError, "PsktCtorError"
);

crate::create_py_exception!(
    /// PSKT Invalid Payload Error
    PyPsktInvalidPayloadError, "PsktInvalidPayloadError"
);

crate::create_py_exception!(
    /// PSKT Tx Not Finalized Error
    PyPsktTxNotFinalizedError, "PsktTxNotFinalizedError"
);

crate::create_py_exception!(
    /// PSKT Creation Not Allowed Error
    PyPsktCreateNotAllowedError, "PsktCreateNotAllowedError"
);

crate::create_py_exception!(
    /// PSKT Not Initialized Error
    PyPsktNotInitializedError, "PsktNotInitializedError"
);

crate::create_py_exception!(
    /// PSKT Consensus Client Error
    PyPsktConsensusClientError, "PsktConsensusClientError"
);

crate::create_py_exception!(
    /// PSKT Error
    PyPsktError, "PsktError"
);

// Internal error type
// Wraps natively defined WASM Error
// Returns corresponding custom Python exception to python
pub struct Error(NativeError);

impl From<Error> for PyErr {
    fn from(value: Error) -> Self {
        match value.0 {
            NativeError::Custom(msg) => PyPsktCustomError::new_err(msg),
            NativeError::State(msg) => PyPsktStateError::new_err(msg),
            NativeError::ExpectedState(msg) => PyPsktExpectedStateError::new_err(msg),
            NativeError::Ctor(msg) => PyPsktCtorError::new_err(msg),
            NativeError::InvalidPayload => {
                PyPsktInvalidPayloadError::new_err(NativeError::InvalidPayload.to_string())
            }
            NativeError::TxNotFinalized(inner) => {
                PyPsktTxNotFinalizedError::new_err(inner.to_string())
            }
            NativeError::CreateNotAllowed => {
                PyPsktCreateNotAllowedError::new_err(NativeError::CreateNotAllowed.to_string())
            }
            NativeError::NotInitialized => {
                PyPsktNotInitializedError::new_err(NativeError::NotInitialized.to_string())
            }
            NativeError::ConsensusClient(inner) => {
                PyPsktConsensusClientError::new_err(inner.to_string())
            }
            NativeError::Pskt(inner) => PyPsktError::new_err(inner.to_string()),
            _ => PyException::new_err("Unhandled error type"),
        }
    }
}

impl From<NativeError> for Error {
    fn from(value: NativeError) -> Self {
        Error(value)
    }
}
