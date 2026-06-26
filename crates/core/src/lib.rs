//! Shared building blocks for the Kaspa Python SDK crates: `create_py_exception!`
//! and `strip_py_prefix`, used by the core `kaspa` module and
//! `kaspa.experimental.silverscript`.

/// Strips the `Py` prefix from class names whose definition line contains
/// `marker`, along with every reference to those names elsewhere in `content`.
///
/// pyo3-stub-gen honors `#[pyclass(name = "...")]` for regular classes, but for
/// `extends = PyException` and enum classes it emits the **Rust ident** instead
/// (e.g. `PySilverScriptError`, `PyNetworkType`), leaking a `Py`-prefixed name
/// into the `.pyi`. Both crates' `stub_gen` binaries call this to repair it:
/// pass `"(builtins.Exception)"` for exceptions, `"(enum.Enum)"` for enums.
pub fn strip_py_prefix(content: String, marker: &str) -> String {
    let mut names: Vec<String> = Vec::new();

    for line in content.lines() {
        if let Some(start) = line.find("class Py")
            && line.contains(marker)
        {
            let after_class = &line[start + 6..];
            if let Some(paren_pos) = after_class.find('(') {
                let class_name = &after_class[..paren_pos];
                if class_name.starts_with("Py") {
                    names.push(class_name.to_string());
                }
            }
        }
    }

    let mut result = content;
    for py_name in &names {
        if let Some(stripped) = py_name.strip_prefix("Py") {
            result = result.replace(py_name, stripped);
        }
    }

    result
}

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
