use kaspa_wallet_core::tx::payment::PaymentOutput;
use pyo3::{
    exceptions::{PyException, PyKeyError},
    prelude::*,
    types::PyDict,
};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::address::PyAddress;

/// A payment destination with address and amount.
///
/// Represents a single output in a transaction, specifying where funds
/// should be sent and how much.
#[gen_stub_pyclass]
#[pyclass(name = "PaymentOutput", skip_from_py_object)]
#[derive(Clone)]
pub struct PyPaymentOutput(PaymentOutput);

#[gen_stub_pymethods]
#[pymethods]
impl PyPaymentOutput {
    /// Create a new Payment Output.
    ///
    /// Args:
    ///     address: The address to send this output to.
    ///     amount: The amount, in sompi, to send on this output.
    #[new]
    fn new(address: PyAddress, amount: u64) -> Self {
        Self(PaymentOutput::new(address.into(), amount))
    }

    /// Equality comparison.
    ///
    /// Args:
    ///     other: Another PaymentOutput to compare against.
    ///
    /// Returns:
    ///     bool: True if both outputs have identical address and amount.
    // Cannot be derived via pyclass(eq)
    fn __eq__(&self, other: &PyPaymentOutput) -> bool {
        match (bincode::serialize(&self.0), bincode::serialize(&other.0)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}

impl From<PyPaymentOutput> for PaymentOutput {
    fn from(value: PyPaymentOutput) -> Self {
        value.0
    }
}

impl TryFrom<&Bound<'_, PyDict>> for PyPaymentOutput {
    type Error = PyErr;
    fn try_from(value: &Bound<PyDict>) -> PyResult<Self> {
        let address_value = value
            .get_item("address")?
            .ok_or_else(|| PyKeyError::new_err("Key `address` not present"))?;

        let address = if let Ok(address) = address_value.extract::<PyAddress>() {
            address
        } else if let Ok(s) = address_value.extract::<String>() {
            PyAddress::try_from(s).map_err(|err| PyException::new_err(format!("{}", err)))?
        } else {
            return Err(PyException::new_err(
                "Addresses must be either an Address instance or a string",
            ));
        };

        let amount: u64 = value
            .get_item("amount")?
            .ok_or_else(|| PyKeyError::new_err("Key `amount` not present"))?
            .extract()?;

        let inner = PaymentOutput::new(address.into(), amount);

        Ok(Self(inner))
    }
}

impl<'py> FromPyObject<'_, 'py> for PyPaymentOutput {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(output) = obj.cast::<PyPaymentOutput>() {
            Ok(output.borrow().clone())
        } else if let Ok(dict) = obj.cast::<PyDict>() {
            Ok(PyPaymentOutput::try_from(&*dict)?)
        } else {
            Err(PyException::new_err(
                "PaymentOutput must be an instance of `PaymentOutput` or compatible `dict`",
            ))
        }
    }
}
