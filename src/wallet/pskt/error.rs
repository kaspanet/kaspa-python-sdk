use kaspa_wallet_pskt::wasm::error::Error;
use pyo3::PyErrArguments;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::gen_stub_pyclass;

pub struct PsktError(pub Error);

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

impl From<PsktError> for PyErr {
    fn from(value: PsktError) -> Self {
        match value.0 {
            Error::Custom(msg) => PyPsktCustomError::new_err(msg),
            Error::State(msg) => PyPsktStateError::new_err(msg),
            Error::ExpectedState(msg) => PyPsktExpectedStateError::new_err(msg),
            Error::Ctor(msg) => PyPsktCtorError::new_err(msg),
            Error::InvalidPayload => PyPsktInvalidPayloadError::new_err("Invalid payload"),
            Error::TxNotFinalized(inner) => PyPsktTxNotFinalizedError::new_err(inner.to_string()),
            Error::CreateNotAllowed => PyPsktCreateNotAllowedError::new_err(
                "Create state is not allowed for PSKT initialized from transaction or a payload",
            ),
            Error::NotInitialized => PyPsktNotInitializedError::new_err(
                "PSKT must be initialized with a payload or CREATE role",
            ),
            Error::ConsensusClient(inner) => PyPsktConsensusClientError::new_err(inner.to_string()),
            Error::Pskt(inner) => PyPsktError::new_err(inner.to_string()),
            _ => PyException::new_err("Unhandled error type"),
        }
    }
}

impl From<Error> for PsktError {
    fn from(value: Error) -> Self {
        PsktError(value)
    }
}
