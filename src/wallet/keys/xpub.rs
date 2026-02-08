use crate::wallet::keys::publickey::PyPublicKey;
use kaspa_bip32::Error as Bip32Error;
use kaspa_bip32::{ChildNumber, ExtendedPublicKey};
use kaspa_wallet_keys::prelude::DerivationPath;
use kaspa_wallet_keys::{prelude::PublicKey, xpub::XPub};
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use std::str::FromStr;
use workflow_core::hex::ToHex;

/// An extended public key (BIP-32).
///
/// Allows hierarchical deterministic address generation without
/// access to private keys. Useful for watch-only wallets.
#[gen_stub_pyclass]
#[pyclass(name = "XPub")]
#[derive(Clone)]
pub struct PyXPub(XPub);

impl PyXPub {
    pub fn new(xpub: XPub) -> Self {
        Self(xpub)
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyXPub {
    /// Create an XPub from a serialized xpub string.
    ///
    /// Args:
    ///     xpub: A Base58-encoded extended public key string.
    ///
    /// Returns:
    ///     XPub: A new XPub instance.
    ///
    /// Raises:
    ///     Exception: If the xpub string is invalid.
    #[new]
    pub fn try_new(xpub: &str) -> PyResult<PyXPub> {
        let inner = XPub::from(
            ExtendedPublicKey::<secp256k1::PublicKey>::from_str(xpub)
                .map_err(|err| PyException::new_err(err.to_string()))?,
        );
        Ok(PyXPub(inner))
    }

    /// Derive a child key at the given index.
    ///
    /// Note: Extended public keys can only derive non-hardened children.
    ///
    /// Args:
    ///     child_number: The child index.
    ///     hardened: Whether to use hardened derivation (default: False).
    ///
    /// Returns:
    ///     XPub: The derived child XPub.
    ///
    /// Raises:
    ///     Exception: If derivation fails (e.g., hardened from xpub).
    #[pyo3(signature = (child_number, hardened = None))]
    pub fn derive_child(&self, child_number: u32, hardened: Option<bool>) -> PyResult<PyXPub> {
        let child_number = ChildNumber::new(child_number, hardened.unwrap_or(false))
            .map_err(|err| PyException::new_err(err.to_string()))?;
        let inner = XPub::from(
            self.0
                .inner()
                .derive_child(child_number)
                .map_err(|err| PyException::new_err(err.to_string()))?,
        );
        Ok(PyXPub(inner))
    }

    /// Derive a key at the given derivation path.
    ///
    /// Args:
    ///     path: A derivation path string (non-hardened only).
    ///
    /// Returns:
    ///     XPub: The derived XPub at that path.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    pub fn derive_path(&self, path: &str) -> PyResult<PyXPub> {
        let path =
            DerivationPath::new(path).map_err(|err| PyException::new_err(err.to_string()))?;
        // let inner = self.0.inner().clone().derive_path((&path).into())
        //     .map_err(|err| PyException::new_err(err.to_string()))?;
        let inner = XPub::from(
            self.0
                .inner()
                .clone()
                .derive_path((&path).into())
                .map_err(|err| PyException::new_err(err.to_string()))?,
        );
        Ok(PyXPub(inner))
    }

    /// Serialize to string with custom prefix.
    ///
    /// Args:
    ///     prefix: The key prefix (e.g., "kpub", "xpub").
    ///
    /// Returns:
    ///     str: The serialized extended public key.
    ///
    /// Raises:
    ///     Exception: If serialization fails.
    #[pyo3(name = "into_string")]
    pub fn to_str(&self, prefix: &str) -> PyResult<String> {
        Ok(self.0.inner().to_string(Some(
            prefix
                .try_into()
                .map_err(|err: Bip32Error| PyException::new_err(err.to_string()))?,
        )))
    }

    /// Get the public key at this derivation level.
    ///
    /// Returns:
    ///     PublicKey: The public key.
    #[pyo3(name = "to_public_key")]
    pub fn public_key(&self) -> PyPublicKey {
        let inner: PublicKey = self.0.inner().public_key().into();
        PyPublicKey(inner)
    }

    /// The serialized extended public key string.
    #[getter]
    pub fn get_xpub(&self) -> PyResult<String> {
        let str = self
            .0
            .inner()
            .to_extended_key("kpub".try_into().unwrap())
            .to_string();
        Ok(str)
    }

    /// The derivation depth (0 for master key).
    #[getter]
    pub fn get_depth(&self) -> u8 {
        self.0.inner().attrs().depth
    }

    /// The parent key's fingerprint as hex.
    #[getter]
    pub fn get_parent_fingerprint(&self) -> String {
        self.0.inner().attrs().parent_fingerprint.to_vec().to_hex()
    }

    /// The child number used to derive this key.
    #[getter]
    pub fn get_child_number(&self) -> u32 {
        self.0.inner().attrs().child_number.into()
    }

    /// The chain code as hex.
    #[getter]
    pub fn get_chain_code(&self) -> String {
        self.0.inner().attrs().chain_code.to_vec().to_hex()
    }
}
