use kaspa_consensus_client::TransactionOutput;
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyDict, PyType},
};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::{
    consensus::core::script_public_key::PyScriptPublicKey, traits::TryToPyDict, types::PyBinary,
};

/// A transaction output defining a payment destination.
///
/// Outputs specify an amount and a locking script (script_public_key)
/// that defines the conditions for spending.
#[gen_stub_pyclass]
#[pyclass(name = "TransactionOutput")]
#[derive(Clone)]
pub struct PyTransactionOutput(TransactionOutput);

#[gen_stub_pymethods]
#[pymethods]
impl PyTransactionOutput {
    /// Create a new transaction output.
    ///
    /// Args:
    ///     value: Amount in sompi (1 KAS = 100,000,000 sompi).
    ///     script_public_key: The locking script.
    ///
    /// Returns:
    ///     TransactionOutput: A new TransactionOutput instance.
    #[new]
    pub fn ctor(value: u64, script_public_key: PyScriptPublicKey) -> Self {
        let inner = TransactionOutput::new(value, script_public_key.into());
        Self(inner)
    }

    /// The output value in sompi (1 KAS = 100,000,000 sompi).
    #[getter]
    pub fn get_value(&self) -> u64 {
        self.0.inner().value
    }

    /// Set the output value.
    ///
    /// Args:
    ///     value: The amount in sompi.
    #[setter]
    pub fn set_value(&mut self, value: u64) {
        self.0.inner().value = value;
    }

    /// The locking script that defines spending conditions.
    #[getter]
    pub fn get_script_public_key(&self) -> PyScriptPublicKey {
        self.0.inner().script_public_key.clone().into()
    }

    /// Set the locking script.
    ///
    /// Args:
    ///     value: The script public key.
    #[setter]
    pub fn set_script_public_key(&mut self, value: PyScriptPublicKey) {
        self.0.inner().script_public_key = value.clone().into();
    }

    /// Get a dictionary representation of the TransactionOutput.
    /// Note that this creates a second separate object on the Python heap.
    ///
    /// Returns:
    ///     dict: the TransactionOutput in dictionary form.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        self.0.try_to_pydict(py)
    }

    /// Create a TransactionOutput from a dictionary.
    ///
    /// Args:
    ///     dict: Dictionary containing transaction output fields with keys:
    ///         - 'value' (int): The output value in sompi
    ///         - 'scriptPublicKey' (dict): Dict with 'version' (int) and 'script' (str) keys
    ///
    /// Returns:
    ///     TransactionOutput: A new TransactionOutput instance.
    ///
    /// Raises:
    ///     KeyError: If required keys are missing.
    ///     ValueError: If values are invalid.
    #[classmethod]
    fn from_dict(_cls: &Bound<'_, PyType>, dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        Self::try_from(dict)
    }

    // Cannot be derived via pyclass(eq) as wrapped PyTransactionOutput type does not derive PartialEq/Eq
    fn __eq__(&self, other: &PyTransactionOutput) -> bool {
        match (bincode::serialize(&self.0), bincode::serialize(&other.0)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}

impl From<TransactionOutput> for PyTransactionOutput {
    fn from(value: TransactionOutput) -> Self {
        Self(value)
    }
}

impl From<PyTransactionOutput> for TransactionOutput {
    fn from(value: PyTransactionOutput) -> Self {
        value.0
    }
}

impl TryFrom<&Bound<'_, PyDict>> for PyTransactionOutput {
    type Error = PyErr;
    fn try_from(dict: &Bound<PyDict>) -> PyResult<Self> {
        let value = dict.as_any().get_item("value")?.extract::<u64>()?;

        let spk_obj = dict.as_any().get_item("scriptPublicKey")?;
        let spk = if let Ok(spk) = spk_obj.extract::<PyScriptPublicKey>() {
            spk
        } else if let Ok(dict) = spk_obj.cast::<PyDict>() {
            PyScriptPublicKey::constructor(
                dict.as_any().get_item("version")?.extract::<u16>()?,
                dict.as_any().get_item("script")?.extract::<PyBinary>()?,
            )?
        } else {
            return Err(PyValueError::new_err(
                "Value for `scriptPublicKey` must be type ScriptPublicKey or dict",
            ));
        };

        Ok(Self::ctor(value, spk))
    }
}
