use crate::wallet::keys::derivation::PyDerivationPath;
use crate::wallet::keys::{privatekey::PyPrivateKey, xpub::PyXPub};
use kaspa_bip32::Error;
use kaspa_bip32::{ChildNumber, ExtendedPrivateKey};
use kaspa_utils::hex::FromHex;
use kaspa_wallet_keys::prelude::PrivateKey;
use kaspa_wallet_keys::xpub::XPub;
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use secp256k1::SecretKey;
use std::str::FromStr;
use workflow_core::hex::ToHex;

/// An extended private key (BIP-32).
///
/// Allows hierarchical deterministic key derivation from a seed.
/// All keys in an HD wallet can be derived from a single XPrv.
///
/// Category: Wallet/Keys
#[gen_stub_pyclass]
#[pyclass(name = "XPrv")]
#[derive(Clone)]
pub struct PyXPrv(ExtendedPrivateKey<SecretKey>);

impl PyXPrv {
    pub(super) fn inner(&self) -> &ExtendedPrivateKey<SecretKey> {
        &self.0
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyXPrv {
    /// Create an XPrv from a seed hex string.
    ///
    /// Args:
    ///     seed: A hex-encoded seed (typically from Mnemonic.to_seed()).
    ///
    /// Returns:
    ///     XPrv: A new XPrv instance.
    ///
    /// Raises:
    ///     Exception: If the seed is invalid.
    #[new]
    fn try_new(seed: &str) -> PyResult<PyXPrv> {
        let seed_bytes = Vec::<u8>::from_hex(seed)
            .map_err(|e| PyErr::new::<PyException, _>(format!("{}", e)))?;

        let inner = ExtendedPrivateKey::<SecretKey>::new(seed_bytes)
            .map_err(|err: Error| PyException::new_err(err.to_string()))?;
        Ok(Self(inner))
    }

    /// Create an XPrv from a serialized xprv string.
    ///
    /// Args:
    ///     xprv: A Base58-encoded extended private key string.
    ///
    /// Returns:
    ///     XPrv: A new XPrv instance.
    ///
    /// Raises:
    ///     Exception: If the xprv string is invalid.
    #[staticmethod]
    #[pyo3(name = "from_xprv")]
    pub fn from_xprv_str(xprv: &str) -> PyResult<PyXPrv> {
        Ok(Self(
            ExtendedPrivateKey::<SecretKey>::from_str(xprv)
                .map_err(|err| PyException::new_err(err.to_string()))?,
        ))
    }

    /// Derive a child key at the given index.
    ///
    /// Args:
    ///     child_number: The child index.
    ///     hardened: Whether to use hardened derivation (default: False).
    ///
    /// Returns:
    ///     XPrv: The derived child XPrv.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    #[pyo3(signature = (child_number, hardened=None))]
    pub fn derive_child(&self, child_number: u32, hardened: Option<bool>) -> PyResult<PyXPrv> {
        let child_number = ChildNumber::new(child_number, hardened.unwrap_or(false))
            .map_err(|err: Error| PyException::new_err(err.to_string()))?;
        let inner = self
            .0
            .derive_child(child_number)
            .map_err(|err: Error| PyException::new_err(err.to_string()))?;
        Ok(Self(inner))
    }

    /// Derive a key at the given derivation path.
    ///
    /// Args:
    ///     path: A derivation path string (e.g., "m/44'/111111'/0'") or DerivationPath.
    ///
    /// Returns:
    ///     XPrv: The derived XPrv at that path.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    pub fn derive_path(&self, path: &Bound<PyAny>) -> PyResult<PyXPrv> {
        let path = if let Ok(path_str) = path.extract::<String>() {
            Ok(PyDerivationPath::new(path_str.as_str())?)
        } else if let Ok(path_obj) = path.extract::<PyDerivationPath>() {
            Ok(path_obj)
        } else {
            Err(PyException::new_err(
                "`path` must be of type `str` or `DerivationPath`",
            ))
        }?;

        let inner = self
            .0
            .clone()
            .derive_path(&(path).into())
            .map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(Self(inner))
    }

    /// Serialize to string with custom prefix.
    ///
    /// Args:
    ///     prefix: The key prefix (e.g., "kprv", "xprv").
    ///
    /// Returns:
    ///     str: The serialized extended private key.
    ///
    /// Raises:
    ///     Exception: If serialization fails.
    #[allow(clippy::wrong_self_convention)]
    pub fn into_string(&self, prefix: &str) -> PyResult<String> {
        let str = self
            .0
            .to_extended_key(
                prefix
                    .try_into()
                    .map_err(|err: Error| PyException::new_err(err.to_string()))?,
            )
            .to_string();
        Ok(str)
    }

    /// Serialize to string with default "kprv" prefix.
    ///
    /// Returns:
    ///     str: The serialized extended private key.
    ///
    /// Raises:
    ///     Exception: If serialization fails.
    pub fn to_string(&self) -> PyResult<String> {
        let str = self
            .0
            .to_extended_key(
                "kprv"
                    .try_into()
                    .map_err(|err: Error| PyException::new_err(err.to_string()))?,
            )
            .to_string();
        Ok(str)
    }

    /// Get the corresponding extended public key.
    ///
    /// Returns:
    ///     XPub: The derived extended public key.
    pub fn to_xpub(&self) -> PyResult<PyXPub> {
        let public_key = self.0.public_key();
        let inner = XPub::from(public_key);
        Ok(PyXPub::new(inner))
    }

    /// Get the private key at this derivation level.
    ///
    /// Returns:
    ///     PrivateKey: The private key.
    pub fn to_private_key(&self) -> PyResult<PyPrivateKey> {
        let private_key = self.0.private_key();
        let inner = PrivateKey::from(private_key);
        Ok(PyPrivateKey::new(inner))
    }

    /// The serialized extended private key string.
    ///
    /// Returns:
    ///     str: The xprv string.
    #[getter]
    pub fn get_xprv(&self) -> PyResult<String> {
        let str = self
            .0
            .to_extended_key(
                "kprv"
                    .try_into()
                    .map_err(|err: Error| PyException::new_err(err.to_string()))?,
            )
            .to_string();
        Ok(str)
    }

    /// The private key as a hex string.
    ///
    /// Returns:
    ///     str: The private key hex.
    #[getter]
    pub fn get_private_key(&self) -> String {
        use kaspa_bip32::PrivateKey;
        self.0.private_key().to_bytes().to_vec().to_hex()
    }

    /// The derivation depth (0 for master key).
    ///
    /// Returns:
    ///     int: The depth.
    #[getter]
    pub fn get_depth(&self) -> u8 {
        self.0.attrs().depth
    }

    /// The parent key's fingerprint as hex.
    ///
    /// Returns:
    ///     str: The parent fingerprint.
    #[getter]
    pub fn get_parent_fingerprint(&self) -> String {
        self.0.attrs().parent_fingerprint.to_vec().to_hex()
    }

    /// The child number used to derive this key.
    ///
    /// Returns:
    ///     int: The child number.
    #[getter]
    pub fn get_child_number(&self) -> u32 {
        self.0.attrs().child_number.into()
    }

    /// The chain code as hex.
    ///
    /// Returns:
    ///     str: The chain code.
    #[getter]
    #[pyo3(name = "chain_code")]
    pub fn get_chain_code(&self) -> String {
        self.0.attrs().chain_code.to_vec().to_hex()
    }
}
