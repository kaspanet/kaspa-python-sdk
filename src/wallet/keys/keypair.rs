use super::privatekey::PyPrivateKey;
use crate::{address::PyAddress, consensus::core::network::PyNetworkType};
use kaspa_addresses::{Address, Version};
use kaspa_consensus_core::network::NetworkType;
use kaspa_wallet_keys::{privatekey::PrivateKey, publickey::PublicKey};
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use std::str::FromStr;
use zeroize::Zeroize;

/// A cryptographic keypair containing private and public keys.
///
/// Provides convenient access to all key forms needed for signing
/// and address generation.
///
/// Category: Wallet/Keys
#[gen_stub_pyclass]
#[pyclass(name = "Keypair")]
pub struct PyKeypair {
    secret_key: secp256k1::SecretKey,
    public_key: secp256k1::PublicKey,
    xonly_public_key: secp256k1::XOnlyPublicKey,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyKeypair {
    /// Create a keypair from hex string representations.
    ///
    /// Args:
    ///     secret_key: The secret key as hex.
    ///     public_key: The public key as hex.
    ///     xonly_public_key: The x-only public key as hex.
    ///
    /// Returns:
    ///     Keypair: A new Keypair instance.
    ///
    /// Raises:
    ///     Exception: If any key format is invalid.
    #[new]
    pub fn new(secret_key: &str, public_key: &str, xonly_public_key: &str) -> PyResult<Self> {
        let secret_key = secp256k1::SecretKey::from_str(secret_key)
            .map_err(|err| PyException::new_err(err.to_string()))?;
        let public_key = secp256k1::PublicKey::from_str(public_key)
            .map_err(|err| PyException::new_err(err.to_string()))?;
        let xonly_public_key = secp256k1::XOnlyPublicKey::from_str(xonly_public_key)
            .map_err(|err| PyException::new_err(err.to_string()))?;

        Ok(PyKeypair {
            secret_key,
            public_key,
            xonly_public_key,
        })
    }

    /// The x-only public key as hex.
    ///
    /// Returns:
    ///     str: The x-only public key.
    #[getter]
    pub fn get_xonly_public_key(&self) -> String {
        self.xonly_public_key.to_string()
    }

    /// The full public key as hex.
    ///
    /// Returns:
    ///     str: The public key.
    #[getter]
    pub fn get_public_key(&self) -> String {
        PublicKey::from(&self.public_key).to_string()
    }

    /// The private key as hex.
    ///
    /// Returns:
    ///     str: The private key.
    #[getter]
    pub fn get_private_key(&self) -> String {
        PrivateKey::from(&self.secret_key).to_hex()
    }

    /// Derive a Schnorr address from this keypair.
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
        let payload = &self.xonly_public_key.serialize();
        let address = Address::new(NetworkType::from(network).into(), Version::PubKey, payload);
        Ok(address.into())
    }

    /// Derive an ECDSA address from this keypair.
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
        let payload = &self.public_key.serialize();
        let address = Address::new(
            NetworkType::from(network).into(),
            Version::PubKeyECDSA,
            payload,
        );
        Ok(address.into())
    }

    /// Generate a random keypair.
    ///
    /// Returns:
    ///     Keypair: A new random Keypair.
    #[staticmethod]
    #[pyo3(name = "random")]
    pub fn random() -> PyResult<PyKeypair> {
        let secp = secp256k1::Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());
        let (xonly_public_key, _) = public_key.x_only_public_key();
        Ok(PyKeypair {
            secret_key,
            public_key,
            xonly_public_key,
        })
    }

    /// Create a keypair from a private key.
    ///
    /// Args:
    ///     private_key: The private key to derive from.
    ///
    /// Returns:
    ///     Keypair: A new Keypair with derived public keys.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    #[staticmethod]
    #[pyo3(name = "from_private_key")]
    pub fn from_private_key(private_key: &PyPrivateKey) -> PyResult<PyKeypair> {
        let secp = secp256k1::Secp256k1::new();
        let mut key_bytes = private_key.secret_bytes();
        let secret_key = secp256k1::SecretKey::from_slice(&key_bytes)
            .map_err(|e| PyException::new_err(format!("{e}")))?;
        key_bytes.zeroize();
        let public_key = secp256k1::PublicKey::from_secret_key(&secp, &secret_key);
        let (xonly_public_key, _) = public_key.x_only_public_key();
        Ok(PyKeypair {
            secret_key,
            public_key,
            xonly_public_key,
        })
    }
}
