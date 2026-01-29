//! This module helps convert from rusty-kaspa native types to Python dictionaries.
//!
//! This is currently intended as one way only:
//! rusty-kaspa native type -> Python dict
//!
//! The inverse (from Python dict) is intentionally not implemented here.
//! As all class initiatlization is handled by the Py wrapper.
//!
//! For example:
//! PyTransactionOutput::from_dict creates a new PyTransactionOutput instance
//! PyTransactionOutput::to_dict calls try_to_pydict on the wrapped type

pub mod native;

use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Trait for converting Rust types to Python dictionaries.
///
/// This trait provides a standard way to convert native rusty-kaspa
/// types to Python dicts with a flat structure (no unnecessary nesting).
///
/// A custom trait is required as `py: Python` is required fn arg so
/// that dict can be created on the Python heap.
pub trait TryToPyDict {
    fn try_to_pydict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>>;
}
