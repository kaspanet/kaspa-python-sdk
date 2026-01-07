use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::gen_stub_pyclass_enum;
use std::str::FromStr;
use workflow_rpc::encoding::Encoding;

crate::wrap_unit_enum_for_py!(
    /// wRPC protocol encoding
    ///
    /// Variants:
    ///     - Borsh
    ///     - JSON
    ///
    /// Category: RPC/Core
    PyEncoding, "Encoding", Encoding, {
        Borsh,
        SerdeJson
});

impl FromStr for PyEncoding {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "borsh" => Ok(PyEncoding::Borsh),
            "json" => Ok(PyEncoding::SerdeJson),
            _ => Err(PyException::new_err(
                "Unsupported string value for Encoding",
            )),
        }
    }
}

impl<'py> FromPyObject<'_, 'py> for PyEncoding {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(s) = obj.extract::<String>() {
            PyEncoding::from_str(&s).map_err(|err| PyException::new_err(err.to_string()))
        } else if let Ok(t) = obj.cast::<PyEncoding>() {
            Ok(t.borrow().clone())
        } else {
            Err(PyException::new_err("Expected type `str` or `Encoding`"))
        }
    }
}
