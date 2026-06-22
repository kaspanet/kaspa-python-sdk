//! Shared building blocks for the Kaspa Python SDK crates.
//!
//! Currently this is just `create_py_exception!`, used by both the core `kaspa`
//! module (for its wallet exceptions) and `kaspa.silverscript`. The macro is
//! `macro_rules!`, so its expansion resolves `pyo3` / `pyo3-stub-gen` names at
//! the call site — this crate needs no dependencies of its own.

/// Defines a Python exception class backed by a `#[pyclass]`, the idiomatic way
/// to expose a custom exception that pyo3-stub-gen can capture.
///
/// We can't use PyO3's `create_exception!` here: that macro produces a type we
/// can't decorate with `#[gen_stub_pyclass]`, so the exception would be missing
/// from the generated `.pyi`. Defining it as `#[pyclass(extends = PyException)]`
/// instead lets stub generation pick it up automatically.
///
/// `$module` is matched as `:tt` (not `:literal`) on purpose: pyo3-stub-gen's
/// attribute parser pattern-matches a raw `Literal` token for `module = "..."`.
/// A `:literal` metavariable interpolates as an invisible None-delimited group,
/// which the parser silently drops — routing the exception into the wrong stub
/// file. `:tt` passes the string literal through as a raw token.
///
/// Usage:
///
/// ```ignore
/// create_py_exception!(
///     /// Doc comment becomes the Python class docstring.
///     MyError, "MyError", "my_module"
/// );
/// ```
///
/// The call site must have `pyclass`, `PyException`, `PyErr`, and
/// `gen_stub_pyclass` in scope (e.g. `use pyo3::prelude::*;
/// use pyo3::exceptions::PyException; use pyo3_stub_gen::derive::*;`).
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
