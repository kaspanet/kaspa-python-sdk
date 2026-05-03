use crate::crypto::hashes::PyHash;
use kaspa_consensus_client::{CovenantBinding, GenesisCovenantGroup};
use kaspa_consensus_core::tx::GenesisCovenantGroup as CoreGenesisCovenantGroup;
use pyo3::{
    prelude::*,
    types::{PyAny, PyDict},
};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

/// Binds a transaction output to the covenant and input authorizing its creation.
#[gen_stub_pyclass]
#[pyclass(name = "CovenantBinding", skip_from_py_object)]
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

    #[getter]
    pub fn get_authorizing_input(&self) -> u16 {
        self.0.get_authorizing_input()
    }

    #[setter]
    pub fn set_authorizing_input(&mut self, value: u16) {
        self.0.set_authorizing_input(value);
    }

    #[getter]
    pub fn get_covenant_id(&self) -> PyHash {
        self.0.get_covenant_id().into()
    }

    #[setter]
    pub fn set_covenant_id(&mut self, value: PyHash) {
        self.0.set_covenant_id(value.into());
    }

    pub fn __repr__(&self) -> String {
        format!(
            "CovenantBinding(authorizing_input={}, covenant_id={})",
            self.0.get_authorizing_input(),
            self.get_covenant_id().__repr__(),
        )
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
        let inner = serde_pyobject::from_pyobject(dict.clone())?;

        Ok(Self(inner))
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for PyCovenantBinding {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        // Try native CovenantBinding instance first, then fall back to dict
        if let Ok(cb) = ob.cast::<Self>() {
            return Ok(cb.to_owned().borrow().clone());
        }
        let dict = ob.cast::<PyDict>()?.to_owned();
        Self::try_from(&dict)
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "GenesisCovenantGroup")]
#[derive(Clone)]
pub struct PyGenesisCovenantGroup(GenesisCovenantGroup);

#[gen_stub_pymethods]
#[pymethods]
impl PyGenesisCovenantGroup {
    #[new]
    pub fn constructor(authorizing_input: u16, outputs: Vec<u32>) -> Self {
        // TODO this tmp construction process is temporary
        //  until  client::GenesisCovenantGroup exposes new fn that takes rust native types
        let tmp = CoreGenesisCovenantGroup::new(authorizing_input, outputs);
        let inner = GenesisCovenantGroup::from(tmp);
        Self(inner)
    }

    #[getter]
    pub fn get_authorizing_input(&self) -> u16 {
        self.0.authorizing_input()
    }

    #[setter]
    pub fn set_authorizing_input(&mut self, value: u16) {
        self.0.set_authorizing_input(value);
    }

    // TODO blocked until GenesisCovenantGroup exposes non-WASM `outputs` fns
    // `GenesisCovenantGroup::outputs -> NumberArray` instead of Vec<u32> return type
    // NumberArray is WASM type, we should not convert from that here.
    // Better solution is that native GenesisCovenantGroup exposes native Rust getter/setter
    // #[getter]
    // pub fn get_outputs(&self) -> Vec<u32> {
    //     self.0.outputs.clone()
    // }

    // #[setter]
    // pub fn set_outputs(&mut self, value: Vec<u32>) {
    //     self.0.outputs = value
    // }
}

impl From<GenesisCovenantGroup> for PyGenesisCovenantGroup {
    fn from(value: GenesisCovenantGroup) -> Self {
        Self(value)
    }
}

impl From<PyGenesisCovenantGroup> for GenesisCovenantGroup {
    fn from(value: PyGenesisCovenantGroup) -> Self {
        value.0.clone()
    }
}

impl TryFrom<&Bound<'_, PyDict>> for PyGenesisCovenantGroup {
    type Error = PyErr;

    fn try_from(dict: &Bound<'_, PyDict>) -> Result<Self, Self::Error> {
        let inner: GenesisCovenantGroup = serde_pyobject::from_pyobject(dict.clone())?;

        Ok(Self(inner))
    }
}
