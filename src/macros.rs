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

// PyO3 provides create_exception! macro. However we cannot use it.
// Because we need to use proc macro #[gen_stub_pyclass] to include the defined
// exception in the stub file. When using create_exception!, we cannot apply
// #[gen_stub_pyclass].
// When PyO3 is able to generate stub files (currently experimental)
// this could likely be removed in favor of that approach.
#[macro_export]
macro_rules! create_py_exception {
    ($(#[$meta:meta])* $name:ident, $py_name:literal) => {
        $(#[$meta])*
        #[allow(dead_code)]
        #[gen_stub_pyclass]
        #[pyclass(name = $py_name, extends = PyException)]
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
                PyErr::new::<Self, _>(message.into())
            }
        }
    };
}
