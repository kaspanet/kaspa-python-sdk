use crate::crypto::hashes::PyHash;
use kaspa_consensus_client::{CovenantBinding, GenesisCovenantGroup};
use pyo3::{
    exceptions::PyKeyError,
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
    /// Create a new CovenantBinding.
    ///
    /// Args:
    ///     authorizing_input: The index of the transaction input authorizing the covenant.
    ///     covenant_id: The covenant id the output is bound to.
    ///
    /// Returns:
    ///     CovenantBinding: A new CovenantBinding instance.
    #[new]
    pub fn new(authorizing_input: u16, covenant_id: PyHash) -> Self {
        let inner = CovenantBinding::new(authorizing_input, covenant_id.into());
        Self(inner)
    }

    /// The index of the transaction input authorizing the covenant.
    #[getter]
    pub fn get_authorizing_input(&self) -> u16 {
        self.0.get_authorizing_input()
    }

    /// Set the authorizing input index.
    ///
    /// Args:
    ///     value: The index of the transaction input authorizing the covenant.
    #[setter]
    pub fn set_authorizing_input(&mut self, value: u16) {
        self.0.set_authorizing_input(value);
    }

    /// The covenant id the output is bound to.
    #[getter]
    pub fn get_covenant_id(&self) -> PyHash {
        self.0.get_covenant_id().into()
    }

    /// Set the covenant id.
    ///
    /// Args:
    ///     value: The covenant id the output is bound to.
    #[setter]
    pub fn set_covenant_id(&mut self, value: PyHash) {
        self.0.set_covenant_id(value.into());
    }

    /// The detailed string representation.
    ///
    /// Returns:
    ///     str: The CovenantBinding as a repr string.
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
        let authorizing_input: u16 = dict
            .get_item("authorizingInput")?
            .ok_or_else(|| PyKeyError::new_err("Key `authorizingInput` not present"))?
            .extract()?;

        let covenant_id: PyHash = dict
            .get_item("covenantId")?
            .ok_or_else(|| PyKeyError::new_err("Key `covenantId` not present"))?
            .extract()?;

        Ok(Self(CovenantBinding::new(
            authorizing_input,
            covenant_id.into(),
        )))
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

/// A genesis covenant group for bulk covenant binding population.
///
/// All listed outputs are bound to the same covenant id, derived from the
/// authorizing input outpoint and the exact ordered output list. Used with
/// `Transaction.populate_genesis_covenants`.
#[gen_stub_pyclass]
#[pyclass(name = "GenesisCovenantGroup", skip_from_py_object)]
#[derive(Clone)]
pub struct PyGenesisCovenantGroup(GenesisCovenantGroup);

#[gen_stub_pymethods]
#[pymethods]
impl PyGenesisCovenantGroup {
    /// Create a new GenesisCovenantGroup.
    ///
    /// Args:
    ///     authorizing_input: The index of the transaction input authorizing the covenant.
    ///     outputs: The indices of the transaction outputs to bind to the covenant.
    ///
    /// Returns:
    ///     GenesisCovenantGroup: A new GenesisCovenantGroup instance.
    #[new]
    pub fn constructor(authorizing_input: u16, outputs: Vec<u32>) -> Self {
        Self(GenesisCovenantGroup::new(authorizing_input, outputs))
    }

    /// The index of the transaction input authorizing the covenant.
    #[getter]
    pub fn get_authorizing_input(&self) -> u16 {
        self.0.authorizing_input()
    }

    /// Set the authorizing input index.
    ///
    /// Args:
    ///     value: The index of the transaction input authorizing the covenant.
    #[setter]
    pub fn set_authorizing_input(&mut self, value: u16) {
        self.0.set_authorizing_input(value);
    }

    /// The indices of the transaction outputs bound to the covenant.
    ///
    /// Returns:
    ///     list[int]: The output indices in this group.
    #[getter]
    pub fn get_outputs(&self) -> Vec<u32> {
        self.0.outputs()
    }

    /// Set the output indices.
    ///
    /// Args:
    ///     value: The indices of the transaction outputs to bind to the covenant.
    #[setter]
    pub fn set_outputs(&mut self, value: Vec<u32>) {
        self.0.set_outputs(value);
    }

    /// The detailed string representation.
    ///
    /// Returns:
    ///     str: The GenesisCovenantGroup as a repr string.
    pub fn __repr__(&self) -> String {
        format!(
            "GenesisCovenantGroup(authorizing_input={}, outputs={:?})",
            self.0.authorizing_input(),
            self.0.outputs(),
        )
    }
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
        let authorizing_input: u16 = dict
            .get_item("authorizingInput")?
            .ok_or_else(|| PyKeyError::new_err("Key `authorizingInput` not present"))?
            .extract()?;

        let outputs: Vec<u32> = dict
            .get_item("outputs")?
            .ok_or_else(|| PyKeyError::new_err("Key `outputs` not present"))?
            .extract()?;

        Ok(Self(GenesisCovenantGroup::new(authorizing_input, outputs)))
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for PyGenesisCovenantGroup {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        // Try native GenesisCovenantGroup instance first, then fall back to dict
        if let Ok(group) = ob.cast::<Self>() {
            return Ok(group.to_owned().borrow().clone());
        }
        let dict = ob.cast::<PyDict>()?.to_owned();
        Self::try_from(&dict)
    }
}
