use kaspa_bip32::Language;
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::gen_stub_pyclass_enum;
use std::str::FromStr;

crate::wrap_unit_enum_for_py!(
    /// BIP-39 mnemonic word list language.
    ///
    /// Variants:
    ///     English: English word list (2048 words).
    ///
    /// Category: Wallet/Keys
    PyLanguage, "Language", Language, { English }
);

impl FromStr for PyLanguage {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "english" => Ok(PyLanguage::English),
            _ => Err(PyException::new_err(
                "Unsupported string value for Language",
            )),
        }
    }
}

impl<'py> FromPyObject<'_, 'py> for PyLanguage {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(s) = obj.extract::<String>() {
            PyLanguage::from_str(&s).map_err(|err| PyException::new_err(err.to_string()))
        } else if let Ok(t) = obj.cast::<PyLanguage>() {
            Ok(t.borrow().clone())
        } else {
            Err(PyException::new_err("Expected type `str` or `Language`"))
        }
    }
}
