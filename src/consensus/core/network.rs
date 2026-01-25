use kaspa_addresses::Prefix;
use kaspa_consensus_core::network::{NetworkId, NetworkType};
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyclass_enum, gen_stub_pymethods};
use std::str::FromStr;

crate::wrap_unit_enum_for_py!(
    /// Kaspa network type enumeration.
    PyNetworkType, "NetworkType", NetworkType, {
    Mainnet,
    Testnet,
    Devnet,
    Simnet,
});

impl FromStr for PyNetworkType {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner =
            NetworkType::from_str(s).map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(inner.into())
    }
}

impl<'py> FromPyObject<'_, 'py> for PyNetworkType {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(s) = obj.extract::<String>() {
            PyNetworkType::from_str(&s).map_err(|err| PyException::new_err(err.to_string()))
        } else if let Ok(t) = obj.cast::<PyNetworkType>() {
            Ok(t.borrow().clone())
        } else {
            Err(PyException::new_err("Expected type `str` or `NetworkType`"))
        }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyNetworkType {
    pub fn default_rpc_port(&self) -> u16 {
        NetworkType::from(self).default_rpc_port()
    }

    pub fn default_borsh_rpc_port(&self) -> u16 {
        NetworkType::from(self).default_borsh_rpc_port()
    }

    pub fn default_json_rpc_port(&self) -> u16 {
        NetworkType::from(self).default_json_rpc_port()
    }
}

impl From<&PyNetworkType> for NetworkType {
    fn from(value: &PyNetworkType) -> Self {
        value.clone().into()
    }
}

/// Network identifier with optional suffix.
///
/// Represents a specific Kaspa network, optionally with a numeric suffix
/// for testnets (e.g., "testnet-10", "testnet-11").
#[gen_stub_pyclass]
#[pyclass(name = "NetworkId", skip_from_py_object, eq)]
#[derive(Clone, PartialEq)]
pub struct PyNetworkId(NetworkId);

#[gen_stub_pymethods]
#[pymethods]
impl PyNetworkId {
    /// Create a new NetworkId.
    ///
    /// Args:
    ///     network_id: A network string ("mainnet", "testnet-10") or NetworkType.
    ///
    /// Returns:
    ///     NetworkId: A new NetworkId instance.
    ///
    /// Raises:
    ///     Exception: If the network_id format is invalid.
    #[new]
    pub fn new(network_id: Bound<PyAny>) -> PyResult<Self> {
        if let Ok(network_id) = network_id.extract::<String>() {
            PyNetworkId::from_str(&network_id)
        } else if let Ok(network_type) = network_id.extract::<PyNetworkType>() {
            let inner = NetworkId::new(network_type.into());
            Ok(Self(inner))
        } else {
            Err(PyException::new_err(
                "`network_id` must be of type NetworkType or String representation (mainnet, testnet-10, etc)",
            ))
        }
    }

    /// Create a NetworkId with a specific suffix.
    ///
    /// Args:
    ///     network_type: The base network type.
    ///     suffix: The numeric suffix (e.g., 10 for testnet-10).
    ///
    /// Returns:
    ///     NetworkId: A new NetworkId with the specified suffix.
    #[staticmethod]
    pub fn with_suffix(
        #[gen_stub(override_type(type_repr = "str | NetworkType"))] network_type: PyNetworkType,
        suffix: u32,
    ) -> Self {
        let inner = NetworkId::with_suffix(network_type.into(), suffix);
        Self(inner)
    }

    /// The base network type (Mainnet, Testnet, Devnet, Simnet).
    ///
    /// Returns:
    ///     NetworkType: The network type.
    #[getter]
    pub fn get_network_type(&self) -> PyNetworkType {
        self.0.network_type.into()
    }

    /// Check if this is the mainnet.
    ///
    /// Returns:
    ///     bool: True if this is mainnet, False otherwise.
    pub fn is_mainnet(&self) -> bool {
        self.0.is_mainnet()
    }

    /// The optional numeric suffix (e.g., 10 for testnet-10).
    ///
    /// Returns:
    ///     int | None: The suffix, or None if not set.
    #[getter]
    pub fn get_suffix(&self) -> Option<u32> {
        self.0.suffix()
    }

    /// The default P2P port for this network.
    ///
    /// Returns:
    ///     int: The default P2P port number.
    #[getter]
    pub fn get_default_p2p_port(&self) -> u16 {
        self.0.default_p2p_port()
    }

    /// Get the prefixed string representation (e.g., "kaspa-mainnet").
    ///
    /// Returns:
    ///     str: The prefixed network identifier.
    pub fn to_prefixed(&self) -> String {
        self.0.to_prefixed()
    }

    /// Get the string representation (e.g., "mainnet", "testnet-10").
    ///
    /// Returns:
    ///     str: The network identifier string.
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    /// Get the address prefix for this network.
    ///
    /// Returns:
    ///     str: The prefix string ("kaspa", "kaspatest", "kaspadev", or "kaspasim").
    pub fn address_prefix(&self) -> String {
        Prefix::from(self.0.network_type).to_string()
    }

    /// The string representation.
    ///
    /// Returns:
    ///     str: The NetworkId as a string
    pub fn __str__(&self) -> String {
        self.0.to_string()
    }
}

impl From<PyNetworkId> for NetworkId {
    fn from(value: PyNetworkId) -> Self {
        Self {
            network_type: value.0.network_type,
            suffix: value.0.suffix,
        }
    }
}

impl From<NetworkId> for PyNetworkId {
    fn from(value: NetworkId) -> Self {
        Self(value)
    }
}

impl FromStr for PyNetworkId {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = NetworkId::from_str(s).map_err(|err| PyException::new_err(err.to_string()))?;

        Ok(Self(inner))
    }
}

impl<'py> FromPyObject<'_, 'py> for PyNetworkId {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(s) = obj.extract::<String>() {
            PyNetworkId::from_str(&s)
        } else if let Ok(network_type) = obj.cast::<PyNetworkType>() {
            let inner = NetworkId::new(network_type.borrow().clone().into());
            Ok(Self(inner))
        } else if let Ok(network_id) = obj.cast::<Self>() {
            Ok(network_id.borrow().clone())
        } else {
            Err(PyException::new_err(
                "`network_id` must be a String or NetworkId instance",
            ))
        }
    }
}
