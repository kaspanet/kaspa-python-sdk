use crate::{address::PyAddress, consensus::core::network::PyNetworkType};
use kaspa_addresses::{Address, Version};
use kaspa_consensus_core::network::NetworkType;
use kaspa_wallet_keys::{prelude::XOnlyPublicKey, publickey::PublicKey};
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

/// A public key for verifying signatures and deriving addresses.
///
/// Can be created from a private key or parsed from a hex string.
#[gen_stub_pyclass]
#[pyclass(name = "PublicKey")]
#[derive(Clone)]
pub struct PyPublicKey(pub PublicKey);

#[gen_stub_pymethods]
#[pymethods]
impl PyPublicKey {
    /// Create a public key from a hex string.
    ///
    /// Args:
    ///     key: A hex-encoded public key string.
    ///
    /// Returns:
    ///     PublicKey: A new PublicKey instance.
    ///
    /// Raises:
    ///     Exception: If the hex string is invalid.
    #[new]
    pub fn try_new(key: &str) -> PyResult<PyPublicKey> {
        let public_key =
            PublicKey::try_new(key).map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(PyPublicKey(public_key))
    }

    /// Convert to hex string representation.
    ///
    /// Returns:
    ///     str: The public key as a hex string.
    #[pyo3(name = "to_string")]
    pub fn to_string_impl(&self) -> String {
        self.0
            .public_key
            .as_ref()
            .map(|pk| pk.to_string())
            .unwrap_or_else(|| self.0.xonly_public_key.to_string())
    }

    /// Derive a Schnorr address from this public key.
    ///
    /// Args:
    ///     network: The network type for address encoding.
    ///
    /// Returns:
    ///     Address: The derived Schnorr address.
    ///
    /// Raises:
    ///     Exception: If address derivation fails.
    pub fn to_address(
        &self,
        #[gen_stub(override_type(type_repr = "str | NetworkType"))] network: PyNetworkType,
    ) -> PyResult<PyAddress> {
        let address = self
            .0
            .to_address(NetworkType::from(network))
            .map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(PyAddress(address))
    }

    /// Derive an ECDSA address from this public key.
    ///
    /// Args:
    ///     network: The network type for address encoding.
    ///
    /// Returns:
    ///     Address: The derived ECDSA address.
    ///
    /// Raises:
    ///     Exception: If address derivation fails.
    pub fn to_address_ecdsa(
        &self,
        #[gen_stub(override_type(type_repr = "str | NetworkType"))] network: PyNetworkType,
    ) -> PyResult<PyAddress> {
        let address = self
            .0
            .to_address_ecdsa(NetworkType::from(network))
            .map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(PyAddress(address))
    }

    /// Get the x-only public key (32 bytes, no parity byte).
    ///
    /// Returns:
    ///     XOnlyPublicKey: The x-only representation.
    #[pyo3(name = "to_x_only_public_key")]
    pub fn to_x_only_public_key(&self) -> PyXOnlyPublicKey {
        PyXOnlyPublicKey(self.0.xonly_public_key.into())
    }

    /// Get the key fingerprint (first 4 bytes of hash).
    ///
    /// Returns:
    ///     str | None: The fingerprint as hex, or None if unavailable.
    #[pyo3(name = "fingerprint")]
    pub fn fingerprint(&self) -> Option<String> {
        // if let Some(public_key) = self.0.public_key.as_ref() {
        //     let digest = Ripemd160::digest(Sha256::digest(public_key.serialize().as_slice()));
        //     Some(digest[..4].as_ref().to_hex().into())
        // } else {
        //     None
        // }
        self.0.fingerprint().map(|v| String::try_from(v).unwrap())
    }
}

impl From<PublicKey> for PyPublicKey {
    fn from(value: PublicKey) -> Self {
        PyPublicKey(value)
    }
}

impl From<PyPublicKey> for PublicKey {
    fn from(value: PyPublicKey) -> Self {
        value.0
    }
}

/// An x-only public key (32 bytes, Schnorr compatible).
///
/// Used for Schnorr signatures and Taproot-style addresses.
#[gen_stub_pyclass]
#[pyclass(name = "XOnlyPublicKey")]
pub struct PyXOnlyPublicKey(XOnlyPublicKey);

#[gen_stub_pymethods]
#[pymethods]
impl PyXOnlyPublicKey {
    /// Create an x-only public key from a hex string.
    ///
    /// Args:
    ///     key: A 64-character hex string.
    ///
    /// Returns:
    ///     XOnlyPublicKey: A new XOnlyPublicKey instance.
    ///
    /// Raises:
    ///     Exception: If the hex string is invalid.
    #[new]
    pub fn try_new(key: &str) -> PyResult<PyXOnlyPublicKey> {
        let xonly_public_key =
            XOnlyPublicKey::try_new(key).map_err(|err| PyException::new_err(err.to_string()))?;
        // let xonly_public_key = secp256k1::XOnlyPublicKey::from_str(key).map_err(|err| PyException::new_err(format!("{}", err)))?;
        Ok(PyXOnlyPublicKey(xonly_public_key))
    }

    /// Convert to hex string representation.
    ///
    /// Returns:
    ///     str: The x-only public key as a hex string.
    #[pyo3(name = "to_string")]
    pub fn to_string_impl(&self) -> String {
        self.0.inner.to_string()
    }

    /// Derive a Schnorr address from this x-only public key.
    ///
    /// Args:
    ///     network: The network type for address encoding.
    ///
    /// Returns:
    ///     Address: The derived Schnorr address.
    pub fn to_address(
        &self,
        #[gen_stub(override_type(type_repr = "str | NetworkType"))] network: PyNetworkType,
    ) -> PyResult<PyAddress> {
        let payload = &self.0.inner.serialize();
        let address = Address::new(NetworkType::from(network).into(), Version::PubKey, payload);
        Ok(PyAddress(address))
    }

    /// Derive an ECDSA address from this x-only public key.
    ///
    /// Args:
    ///     network: The network type for address encoding.
    ///
    /// Returns:
    ///     Address: The derived ECDSA address.
    pub fn to_address_ecdsa(
        &self,
        #[gen_stub(override_type(type_repr = "str | NetworkType"))] network: PyNetworkType,
    ) -> PyResult<PyAddress> {
        let payload = &self.0.inner.serialize();
        let address = Address::new(
            NetworkType::from(network).into(),
            Version::PubKeyECDSA,
            payload,
        );
        Ok(PyAddress(address))
    }

    /// Extract an x-only public key from an address.
    ///
    /// Args:
    ///     address: A Kaspa address.
    ///
    /// Returns:
    ///     XOnlyPublicKey: The extracted public key.
    ///
    /// Raises:
    ///     Exception: If extraction fails.
    #[pyo3(name = "from_address")]
    #[staticmethod]
    pub fn from_address(address: PyAddress) -> PyResult<PyXOnlyPublicKey> {
        let xonly_public_key = XOnlyPublicKey::from_address(&address.into())
            .map_err(|err| PyException::new_err(err.to_string()))?;
        // let xonly_public_key = secp256k1::XOnlyPublicKey::from_slice(&address.payload)
        //     .map_err(|err| PyException::new_err(format!("{}", err)))?;
        Ok(xonly_public_key.into())
    }
}

impl From<XOnlyPublicKey> for PyXOnlyPublicKey {
    fn from(value: XOnlyPublicKey) -> Self {
        PyXOnlyPublicKey(value)
    }
}
