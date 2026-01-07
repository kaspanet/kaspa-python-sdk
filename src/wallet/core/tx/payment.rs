use kaspa_wallet_core::tx::payment::PaymentOutput;
use pyo3::{
    exceptions::{PyException, PyKeyError},
    prelude::*,
    types::PyDict,
};
use pyo3_stub_gen::derive::gen_stub_pyclass;

use crate::address::PyAddress;

/// A payment destination with address and amount.
///
/// Represents a single output in a transaction, specifying where funds
/// should be sent and how much. Used with Generator and create_transactions.
///
/// Category: Wallet/Transactions
#[gen_stub_pyclass]
#[pyclass(name = "PaymentOutput")]
#[derive(Clone)]
pub struct PyPaymentOutput(PaymentOutput);

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
