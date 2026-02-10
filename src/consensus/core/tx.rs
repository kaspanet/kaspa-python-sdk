use crate::crypto::hashes::PyHash;
use kaspa_consensus_core::tx::CovenantBinding;
use pyo3::{prelude::*, types::PyDict};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

pub type TransactionId = PyHash;

/// Binds a transaction output to the covenant and input authorizing its creation.
#[gen_stub_pyclass]
#[pyclass(name = "CovenantBinding")]
#[derive(Clone)]
pub struct PyCovenantBinding(CovenantBinding);

#[gen_stub_pymethods]
#[pymethods]
impl PyCovenantBinding {
    #[new]
    pub fn new(authorizing_input: u16, covenant_id: PyHash) -> Self {
        let inner = CovenantBinding::new(authorizing_input, covenant_id.into());
        Self(inner)
    }
}

impl From<CovenantBinding> for PyCovenantBinding {
    fn from(value: CovenantBinding) -> Self {
        Self(value)
    }
}

impl From<PyCovenantBinding> for CovenantBinding {
    fn from(value: PyCovenantBinding) -> Self {
        value.0
    }
}

impl TryFrom<&Bound<'_, PyDict>> for PyCovenantBinding {
    type Error = PyErr;

    fn try_from(dict: &Bound<'_, PyDict>) -> Result<Self, Self::Error> {
        let authorizing_input = dict
            .as_any()
            .get_item("authorizing_input")?
            .extract::<u16>()?;

        let covenant_id = dict.as_any().get_item("covenant_id")?.extract::<PyHash>()?;

        let inner = CovenantBinding::new(authorizing_input, covenant_id.into());

        Ok(Self(inner))
    }
}
