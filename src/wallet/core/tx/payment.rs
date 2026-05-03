use kaspa_consensus_client::CovenantBinding;
use kaspa_wallet_core::tx::payment::PaymentOutput;
use pyo3::{
    exceptions::{PyException, PyKeyError},
    prelude::*,
    types::PyDict,
};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::{address::PyAddress, consensus::client::covenant::PyCovenantBinding};

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

    #[staticmethod]
    fn with_covenant(address: PyAddress, amount: u64, covenant: PyCovenantBinding) -> Self {
        Self(PaymentOutput::with_covenant(
            address.into(),
            amount,
            covenant.into(),
        ))
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

    /// The detailed string representation.
    ///
    /// Returns:
    ///     str: The PaymentOutput as a repr string.
    fn __repr__(&self) -> String {
        format!(
            "PaymentOutput(address='{}', amount={}, covenant={})",
            self.0.address.address_to_string(),
            self.0.amount,
            match &self.0.covenant {
                Some(covenant) => PyCovenantBinding::from(*covenant).__repr__(),
                None => "None".to_string(),
            }
        )
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

        let covenant_id = value
            .as_any()
            .get_item("covenant")?
            .extract::<Option<PyCovenantBinding>>()?;

        let inner = PaymentOutput {
            address: address.into(),
            amount,
            covenant: covenant_id.map(CovenantBinding::from),
        };

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
