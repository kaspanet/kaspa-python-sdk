use std::str::FromStr;

use kaspa_addresses::{Address, AddressError, Prefix, Version};
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::*;

crate::wrap_unit_enum_for_py!(
    /// Kaspa Address version (`PubKey`, `PubKeyECDSA`, `ScriptHash`)
    ///-  PubKey addresses always have the version byte set to 0
    /// - PubKey ECDSA addresses always have the version byte set to 1
    /// - ScriptHash addresses always have the version byte set to 8
    PyAddressVersion, "AddressVersion", Version, {
        PubKey,
        PubKeyECDSA,
        ScriptHash
    }
);

impl FromStr for PyAddressVersion {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = match s.to_lowercase().as_str() {
            "pubkey" => PyAddressVersion::PubKey,
            "pubkeyecdsa" => PyAddressVersion::PubKeyECDSA,
            "scripthash" => PyAddressVersion::ScriptHash,
            _ => Err(PyException::new_err(
                "Unsupported string value for `AddressVersion`",
            ))?,
        };

        Ok(v)
    }
}

impl<'py> FromPyObject<'_, 'py> for PyAddressVersion {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(s) = obj.extract::<String>() {
            PyAddressVersion::from_str(&s).map_err(|err| PyException::new_err(err.to_string()))
        } else if let Ok(t) = obj.cast::<PyAddressVersion>() {
            Ok(t.borrow().clone())
        } else {
            Err(PyException::new_err(
                "Expected type `str` or `AddressVersion`",
            ))
        }
    }
}

/// A Kaspa blockchain address.
///
/// In string form, the Kaspa addresses are represented by a `bech32`-encoded
/// address string combined with a network type prefix. The `bech32` string encoding is
/// comprised of a public key, the public key version and the resulting checksum.
#[gen_stub_pyclass]
#[pyclass(name = "Address", eq)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyAddress(pub Address);

#[gen_stub_pymethods]
#[pymethods]
impl PyAddress {
    /// Create a new Address from a string.
    ///
    /// Args:
    ///     address: A valid Kaspa address string.
    ///
    /// Returns:
    ///     Address: A new Address instance.
    ///
    /// Raises:
    ///     Exception: If the address string is invalid.
    #[new]
    pub fn constructor(address: &str) -> PyResult<PyAddress> {
        Ok(PyAddress(address.try_into().map_err(
            |err: AddressError| PyException::new_err(err.to_string()),
        )?))
    }

    /// Check if an address string is valid.
    ///
    /// Args:
    ///     address: A Kaspa address string to validate.
    ///
    /// Returns:
    ///     bool: True if the address is valid, False otherwise.
    #[staticmethod]
    #[pyo3(name = "validate")]
    pub fn validate(address: &str) -> bool {
        Address::try_from(address).is_ok()
    }

    /// The string representation of the Address.
    ///
    /// Returns:
    ///     str: A bech32 encoded Kaspa address string.
    #[pyo3(name = "to_string")]
    pub fn address_to_string(&self) -> String {
        self.0.address_to_string()
    }

    /// The string representation of the address version.
    /// Versions are `PubKey`, `PubKeyECDSA`, or `ScriptHash`.
    ///
    /// Returns:
    ///     str: The address version.
    #[getter]
    pub fn get_version(&self) -> String {
        self.0.version.to_string()
    }

    /// The network prefix of the address. Prefix is based on the network type (mainnet, testnet, etc..)
    ///
    /// Returns:
    ///     str: The network prefix string.
    ///
    /// Note:
    ///     - Mainnet prefix is `kaspa`
    ///     - Testnet prefix is `kaspatest`
    ///     - Simnet prefix is `kaspasim`
    ///     - Devnet prefix is `kaspadev`
    #[getter]
    pub fn get_prefix(&self) -> String {
        self.0.prefix.to_string()
    }

    /// Set the network prefix of the address.
    ///
    /// Args:
    ///     value: The network prefix string (e.g., `kaspa`, `kaspatest`, `kaspadev`).
    ///
    /// Raises:
    ///     Exception: If the prefix string is invalid.
    #[setter]
    pub fn set_prefix(&mut self, value: &str) -> PyResult<()> {
        self.0.prefix =
            Prefix::try_from(value).map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(())
    }

    /// The bech32 encoded payload of the address.
    ///
    /// Returns:
    ///     str: The payload portion of the address.
    #[getter]
    pub fn get_payload(&self) -> String {
        self.0.payload_to_string()
    }

    /// Get a shortened representation of the address.
    ///
    /// Args:
    ///     n: The number of characters to show at the start and end of the payload.
    ///
    /// Returns:
    ///     str: A shortened address string in the format `prefix:start....end`.
    pub fn short(&self, n: usize) -> String {
        self.0.short(n)
    }

    /// The string representation.
    ///
    /// Returns:
    ///     str: The address as a string
    pub fn __str__(&self) -> String {
        self.0.address_to_string()
    }
}

impl From<Address> for PyAddress {
    fn from(value: Address) -> Self {
        PyAddress(value)
    }
}

impl From<PyAddress> for Address {
    fn from(value: PyAddress) -> Address {
        value.0
    }
}

impl TryFrom<String> for PyAddress {
    type Error = PyErr;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let inner =
            Address::try_from(value).map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(PyAddress(inner))
    }
}
