use kaspa_consensus_core::hashing::wasm::SighashType;
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::gen_stub_pyclass_enum;
use std::str::FromStr;

crate::wrap_unit_enum_for_py!(
    /// Kaspa signature hash types for transaction signing.
    PySighashType, "SighashType", SighashType, {
    All,
    None,
    Single,
    AllAnyOneCanPay,
    NoneAnyOneCanPay,
    SingleAnyOneCanPay,
});

impl FromStr for PySighashType {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all" => Ok(PySighashType::All),
            "none" => Ok(PySighashType::None),
            "single" => Ok(PySighashType::Single),
            "allanyonecanpay" => Ok(PySighashType::AllAnyOneCanPay),
            "noneanyonecanpay" => Ok(PySighashType::NoneAnyOneCanPay),
            "singleanyonecanpay" => Ok(PySighashType::SingleAnyOneCanPay),
            _ => Err(PyException::new_err(
                "Unsupported string value for SighashType",
            )),
        }
    }
}

impl<'py> FromPyObject<'_, 'py> for PySighashType {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(s) = obj.extract::<String>() {
            PySighashType::from_str(&s).map_err(|err| PyException::new_err(err.to_string()))
        } else if let Ok(t) = obj.cast::<PySighashType>() {
            Ok(t.borrow().clone())
        } else {
            Err(PyException::new_err("Expected type `str` or `SighashType`"))
        }
    }
}
