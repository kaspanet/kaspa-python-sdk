#[macro_export]
macro_rules! wrap_c_enum_for_py {
    ($(#[$meta:meta])* $name:ident, $py_name:literal, $source:ty, { $($variant:ident = $val:expr),* $(,)? }) => {
        $(#[$meta])*
        #[gen_stub_pyclass_enum]
        #[pyclass(name = $py_name, eq, eq_int)]
        #[derive(Clone, PartialEq)]
        pub enum $name { $($variant = $val),* }

        impl From<$source> for $name {
            fn from(value: $source) -> Self {
                match value {
                    $(<$source>::$variant => Self::$variant),*
                }
            }
        }

        impl From<$name> for $source {
            fn from(value: $name) -> Self {
                match value {
                    $(<$name>::$variant => Self::$variant),*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! wrap_unit_enum_for_py {
    ($(#[$meta:meta])* $name:ident, $py_name:literal, $source:ty, { $($variant:ident),* $(,)? }) => {
        $(#[$meta])*
        #[gen_stub_pyclass_enum]
        #[pyclass(name = $py_name, skip_from_py_object, eq, eq_int)]
        #[derive(Clone, PartialEq)]
        pub enum $name { $($variant),* }

        impl From<$source> for $name {
            fn from(value: $source) -> Self {
                match value {
                    $(<$source>::$variant => Self::$variant),*
                }
            }
        }

        impl From<$name> for $source {
            fn from(value: $name) -> Self {
                match value {
                    $(<$name>::$variant => Self::$variant),*
                }
            }
        }

        // impl PyStubType for $name {
        //     fn type_output() -> TypeInfo {
        //         TypeInfo::locally_defined($py_name, "kaspa".into())
        //     }

        //     fn type_input() -> TypeInfo {
        //         TypeInfo::locally_defined($py_name, "kaspa".into())
        //     }
        // }
    };
}

// Generates a Python exception class per native rusty-kaspa error variant,
// the `From<Error> for PyErr` impl that maps each variant to its dedicated
// Python exception, and a `register_exceptions` fn that adds every generated
// class to a given `PyModule`. The `match` produced has no `_` arm, so:
//
//   * Coverage — adding a variant upstream causes `non-exhaustive patterns` build
//     failure until it's listed here.
//   * Bijection — duplicate entries cause `unreachable_patterns` (variant) or
//     duplicate type definition (Rust ident).
//
// Each entry provides a full pattern, the Rust struct ident, and the Python
// class name literal:
//
//   <Pattern> => <PyRustIdent>, "<PythonName>";
//
// e.g.
//
//   NativeError::Custom(_) => PyWalletCustomError, "WalletCustomError";
//   NativeError::NotConnected  => PyWalletNotConnectedError, "WalletNotConnectedError";
//   NativeError::InsufficientFunds { .. } => PyWalletInsufficientFundsError, "WalletInsufficientFundsError";
//
// The macro is payload-agnostic: it relies on the native enum's `Display` impl
// (provided by `thiserror` `#[error(...)]` attributes) to format the message
// string, so all variants — unit, tuple, or struct — share one uniform
// per-entry shape.
#[macro_export]
macro_rules! py_error_map {
    (
        $(
            $pat:pat_param => $py:ident, $py_lit:literal
        );+ $(;)?
    ) => {
        $( ::kaspa_python_sdk_core::create_py_exception!($py, $py_lit, "kaspa.exceptions"); )+

        impl From<Error> for ::pyo3::PyErr {
            #[deny(unreachable_patterns)]
            fn from(err: Error) -> Self {
                match err.0 {
                    $(
                        e @ $pat => $py::new_err(e.to_string())
                    ),+
                }
            }
        }

        /// Registers every generated wallet exception class on `module`.
        /// Emitted by `py_error_map!` so the list stays single-sourced.
        pub fn register_exceptions(
            module: &::pyo3::Bound<'_, ::pyo3::types::PyModule>,
        ) -> ::pyo3::PyResult<()> {
            $( module.add_class::<$py>()?; )+
            Ok(())
        }
    };
}
