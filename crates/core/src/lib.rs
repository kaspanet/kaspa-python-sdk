//! Shared building blocks for the Kaspa Python SDK crates: currently just
//! `create_py_exception!`, used by the core `kaspa` module and `kaspa.experimental.silverscript`.

/// Defines a `#[pyclass(extends = PyException)]` exception class. Unlike PyO3's
/// `create_exception!`, the result takes `#[gen_stub_pyclass]`, so pyo3-stub-gen
/// captures it in the `.pyi`.
///
/// `$module` is `:tt`, not `:literal`: stub-gen matches a raw `module = "..."`
/// token, and a `:literal` interpolates as a group it silently drops.
///
/// ```ignore
/// create_py_exception!(
///     /// Doc comment becomes the Python class docstring.
///     MyError, "MyError", "my_module"
/// );
/// ```
/// Requires `pyclass`, `PyException`, `PyErr`, and `gen_stub_pyclass` in scope.
#[macro_export]
macro_rules! create_py_exception {
    ($(#[$meta:meta])* $name:ident, $py_name:literal, $module:tt) => {
        $(#[$meta])*
        #[allow(dead_code)]
        #[gen_stub_pyclass]
        #[pyclass(name = $py_name, extends = PyException, module = $module)]
        pub struct $name {
            message: String,
        }

        // This is required, otherwise PyO3 cannot initialize the Exception on Python side
        #[pymethods]
        impl $name {
            #[new]
            pub fn new(message: String) -> Self {
                Self { message }
            }
        }

        impl $name {
            pub fn new_err(message: impl Into<String>) -> PyErr {
                PyErr::new::<Self, String>(message.into())
            }
        }
    };
}
