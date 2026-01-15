use super::publickey::PyPublicKey;
use crate::{
    address::PyAddress, consensus::core::network::PyNetworkType, wallet::keys::keypair::PyKeypair,
};
use kaspa_addresses::{Address, Version};
use kaspa_consensus_core::network::NetworkType;
use kaspa_wallet_keys::privatekey::PrivateKey;
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

/// A private key for signing transactions and messages.
///
/// Private keys should be kept secret and never shared.
#[gen_stub_pyclass]
#[pyclass(name = "PrivateKey")]
pub struct PyPrivateKey(PrivateKey);

impl PyPrivateKey {
    pub fn new(key: PrivateKey) -> Self {
        Self(key)
    }

    pub fn inner(&self) -> &PrivateKey {
        &self.0
    }

    pub fn secret_bytes(&self) -> [u8; 32] {
        self.0.secret_bytes()
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyPrivateKey {
    /// Create a private key from a hex string.
    ///
    /// Args:
    ///     key: A 64-character hex string representing the secret key.
    ///
    /// Returns:
    ///     PrivateKey: A new PrivateKey instance.
    ///
    /// Raises:
    ///     Exception: If the hex string is invalid.
    #[new]
    pub fn try_new(key: &str) -> PyResult<PyPrivateKey> {
        let private_key =
            PrivateKey::try_new(key).map_err(|err| PyException::new_err(format!("{}", err)))?;
        Ok(PyPrivateKey(private_key))
    }

    /// Convert to hex string representation.
    ///
    /// Returns:
    ///     str: The private key as a hex string.
    #[pyo3(name = "to_string")]
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }

    /// Derive the corresponding public key.
    ///
    /// Returns:
    ///     PublicKey: The derived public key.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    pub fn to_public_key(&self) -> PyResult<PyPublicKey> {
        let public_key = self
            .0
            .to_public_key()
            .map_err(|_| PyException::new_err("Failed to derive public key"))?;

        Ok(public_key.into())
    }

    /// Derive a Schnorr address from this private key.
    ///
    /// Args:
    ///     network: The network type for address encoding.
    ///
    /// Returns:
    ///     Address: The derived Schnorr address.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    pub fn to_address(
        &self,
        #[gen_stub(override_type(type_repr = "str | NetworkType"))] network: PyNetworkType,
    ) -> PyResult<PyAddress> {
        let public_key = self
            .0
            .to_public_key()
            .map_err(|_| PyException::new_err("Failed to derive public key"))?;
        let (x_only_public_key, _) = public_key.public_key.unwrap().x_only_public_key();
        let payload = x_only_public_key.serialize();
        let address = Address::new(NetworkType::from(network).into(), Version::PubKey, &payload);
        Ok(address.into())
    }

    /// Derive an ECDSA address from this private key.
    ///
    /// Args:
    ///     network: The network type for address encoding.
    ///
    /// Returns:
    ///     Address: The derived ECDSA address.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    pub fn to_address_ecdsa(
        &self,
        #[gen_stub(override_type(type_repr = "str | NetworkType"))] network: PyNetworkType,
    ) -> PyResult<PyAddress> {
        let public_key = self
            .0
            .to_public_key()
            .map_err(|_| PyException::new_err("Failed to derive public key"))?;
        let payload = public_key.public_key.unwrap().serialize();
        let address = Address::new(
            NetworkType::from(network).into(),
            Version::PubKeyECDSA,
            &payload,
        );
        Ok(address.into())
    }

    /// Create a Keypair from this private key.
    ///
    /// Returns:
    ///     Keypair: A keypair containing this private key and derived public keys.
    ///
    /// Raises:
    ///     Exception: If keypair creation fails.
    pub fn to_keypair(&self) -> PyResult<PyKeypair> {
        PyKeypair::from_private_key(self).map_err(|err| PyException::new_err(err.to_string()))
    }
}

impl From<PyPrivateKey> for PrivateKey {
    fn from(value: PyPrivateKey) -> Self {
        value.0
    }
}
