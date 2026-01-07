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
