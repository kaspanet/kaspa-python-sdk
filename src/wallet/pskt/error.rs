use kaspa_wallet_pskt::wasm::error::Error;
use pyo3::{PyErrArguments, create_exception};
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::gen_stub_pyclass;

pub struct PyPsktError(pub Error);

#[gen_stub_pyclass]
#[pyclass(name = "PsktCustomError", extends = PyException)]
pub struct PyPsktCustomError;

impl PyPsktCustomError {
    pub fn new_err<A>(args: A) -> PyErr
    where
        A: PyErrArguments + Send + Sync + 'static,
    {
        PyErr::new::<Self, A>(args)
    }
}

// #[gen_stub_pyclass]
// create_exception!("kaspa.exceptions", PsktCustomError, PyException);
create_exception!("kaspa.exceptions", PsktStateError, PyException);
create_exception!("kaspa.exceptions", PsktExpectedStateError, PyException);
create_exception!("kaspa.exceptions", PsktCtorError, PyException);
create_exception!("kaspa.exceptions", PsktInvalidPayloadError, PyException);
create_exception!("kaspa.exceptions", PsktTxNotFinalizedError, PyException);
create_exception!("kaspa.exceptions", PsktCreateNotAllowedError, PyException);
create_exception!("kaspa.exceptions", PsktNotInitializedError, PyException);
create_exception!("kaspa.exceptions", PsktConsensusClientError, PyException);
create_exception!("kaspa.exceptions", PsktError, PyException);

impl From<PyPsktError> for PyErr {
    fn from(value: PyPsktError) -> Self {
        match value.0 {
            Error::Custom(msg) => PyPsktCustomError::new_err(msg),
            Error::State(msg) => PsktStateError::new_err(msg),
            Error::ExpectedState(msg) => PsktExpectedStateError::new_err(msg),
            Error::Ctor(msg) => PsktCtorError::new_err(msg),
            Error::InvalidPayload => PsktInvalidPayloadError::new_err("Invalid payload"),
            Error::TxNotFinalized(inner) => PsktTxNotFinalizedError::new_err(inner.to_string()),
            Error::CreateNotAllowed => PsktCreateNotAllowedError::new_err(
                "Create state is not allowed for PSKT initialized from transaction or a payload",
            ),
            Error::NotInitialized => PsktNotInitializedError::new_err(
                "PSKT must be initialized with a payload or CREATE role",
            ),
            Error::ConsensusClient(inner) => PsktConsensusClientError::new_err(inner.to_string()),
            Error::Pskt(inner) => PsktError::new_err(inner.to_string()),
            _ => PyException::new_err("Unhandled error type"),
        }
    }
}

impl From<Error> for PyPsktError {
    fn from(value: Error) -> Self {
        PyPsktError(value)
    }
}
