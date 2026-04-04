use kaspa_wallet_core::tx::Fees;
use kaspa_wallet_core::wasm::FeeSource;
use pyo3::exceptions::{PyException, PyKeyError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3_stub_gen::derive::*;
use std::str::FromStr;

crate::wrap_unit_enum_for_py!(
    /// Fee source
    PyFeeSource, "FeeSource", FeeSource, {
        SenderPays,
        ReceiverPays
    }
);

impl FromStr for PyFeeSource {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = match s.to_lowercase().as_str() {
            "senderpays" => PyFeeSource::SenderPays,
            "receiverpays" => PyFeeSource::ReceiverPays,
            _ => Err(PyException::new_err(
                "Unsupported string value for `FeeSource`",
            ))?,
        };

        Ok(v)
    }
}

impl<'py> FromPyObject<'_, 'py> for PyFeeSource {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(s) = obj.extract::<String>() {
            PyFeeSource::from_str(&s).map_err(|err| PyException::new_err(err.to_string()))
        } else if let Ok(t) = obj.cast::<PyFeeSource>() {
            Ok(t.borrow().clone())
        } else {
            Err(PyException::new_err("Expected type `str` or `FeeSource`"))
        }
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "Fees")]
pub struct PyFees {
    pub amount: u64,
    pub source: Option<PyFeeSource>,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyFees {
    #[new]
    pub fn new(amount: u64, source: Option<PyFeeSource>) -> Self {
        Self { amount, source }
    }
}

impl<'py> FromPyObject<'_, 'py> for PyFees {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(fees) = obj.extract::<PyFees>() {
            Ok(fees)
        } else if let Ok(dict) = obj.cast::<PyDict>() {
            let amount: u64 = dict
                .get_item("amount")?
                .ok_or_else(|| PyKeyError::new_err("Key `amount` not present"))?
                .extract()?;

            let source: Option<PyFeeSource> = dict
                .get_item("source")?
                .ok_or_else(|| PyKeyError::new_err("Key `source` not present"))?
                .extract()?;

            Ok(PyFees { amount, source })
        } else {
            Err(PyException::new_err(
                "Expected type `dict` or `Fees` instance",
            ))
        }
    }
}

impl TryFrom<&Bound<'_, PyDict>> for PyFees {
    type Error = PyErr;

    fn try_from(value: &Bound<PyDict>) -> PyResult<Self> {
        let amount: u64 = value.as_any().get_item("amount")?.extract()?;

        let source: Option<PyFeeSource> = value.as_any().get_item("source")?.extract()?;

        Ok(PyFees { amount, source })
    }
}

impl From<PyFees> for Fees {
    fn from(value: PyFees) -> Self {
        match value.source {
            Some(PyFeeSource::SenderPays) => Fees::SenderPays(value.amount),
            Some(PyFeeSource::ReceiverPays) => Fees::ReceiverPays(value.amount),
            None => Fees::None,
        }
    }
}
