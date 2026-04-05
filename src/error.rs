use pyo3::prelude::PyResult;

/// A trait to help reduce `map_err()` calls
/// Frequently we need to return rusty-kaspa native errors to Python as custom Python exceptions
/// This provides a slightly more ergnomic approach than calling
/// ``map_err(CustomPythonErrT::from)` on rusty-kaspa native errors
pub trait IntoPyResult<T> {
    fn into_py_result(self) -> PyResult<T>;
}
