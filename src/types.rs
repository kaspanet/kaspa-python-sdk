use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList};
use pyo3_stub_gen::derive::gen_stub_pyclass;

/// Binary data type for flexible input handling.
///
/// This type is not intended to be instantiated directly from Python.
/// It serves as a helper type that allows Rust functions to accept binary
/// data in multiple convenient forms from Python.
///
/// Accepts:
///     - str: A hexadecimal string (e.g., "deadbeef").
///     - bytes: Python bytes object.
///     - list[int]: A list of byte values (0-255).
///
/// Category: Core/Types
#[gen_stub_pyclass]
#[pyclass(name = "Binary")]
pub struct PyBinary {
    pub data: Vec<u8>,
}

impl<'py> FromPyObject<'_, 'py> for PyBinary {
    type Error = PyErr;

    fn extract(value: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(str) = value.extract::<String>() {
            // Python `str` (of valid hex)
            let mut data = vec![0u8; str.len() / 2];
            match faster_hex::hex_decode(str.as_bytes(), &mut data) {
                Ok(()) => Ok(PyBinary { data }),
                Err(_) => Err(PyException::new_err("Invalid hex string")),
            }
        } else if let Ok(py_bytes) = value.cast::<PyBytes>() {
            // Python `bytes` type
            Ok(PyBinary {
                data: py_bytes.as_bytes().to_vec(),
            })
        } else if let Ok(op_list) = value.cast::<PyList>() {
            // Python `[int]` (list of bytes)
            let data = op_list
                .iter()
                .map(|item| item.extract::<u8>())
                .collect::<PyResult<Vec<u8>>>()?;
            Ok(PyBinary { data })
        } else {
            Err(PyException::new_err(
                "Expected `str` (of valid hex), `bytes`, or `[int]`",
            ))
        }
    }
}

impl TryFrom<&Bound<'_, PyAny>> for PyBinary {
    type Error = PyErr;
    fn try_from(value: &Bound<PyAny>) -> Result<Self, Self::Error> {
        if let Ok(str) = value.extract::<String>() {
            // Python `str` (of valid hex)
            let mut data = vec![0u8; str.len() / 2];
            match faster_hex::hex_decode(str.as_bytes(), &mut data) {
                Ok(()) => Ok(PyBinary { data }), // Hex string
                Err(_) => Err(PyException::new_err("Invalid hex string")),
            }
        } else if let Ok(py_bytes) = value.cast::<PyBytes>() {
            // Python `bytes` type
            Ok(PyBinary {
                data: py_bytes.as_bytes().to_vec(),
            })
        } else if let Ok(op_list) = value.cast::<PyList>() {
            // Python `[int]` (list of bytes)
            let data = op_list
                .iter()
                .map(|item| item.extract::<u8>().unwrap())
                .collect();
            Ok(PyBinary { data })
        } else {
            Err(PyException::new_err(
                "Expected `str` (of valid hex), `bytes`, or `[int]`",
            ))
        }
    }
}

impl From<PyBinary> for Vec<u8> {
    fn from(value: PyBinary) -> Vec<u8> {
        value.data
    }
}

impl AsRef<[u8]> for PyBinary {
    fn as_ref(&self) -> &[u8] {
        self.data.as_slice()
    }
}
