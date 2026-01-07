use kaspa_bip32::{ChildNumber, ExtendedPrivateKey};
use kaspa_wallet_keys::{derivation::gen1::WalletDerivationManager, prelude::PrivateKey};
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use secp256k1::SecretKey;

use crate::wallet::keys::{privatekey::PyPrivateKey, xprv::PyXPrv};

/// Generator for deriving private keys from an extended private key.
///
/// Used for creating wallets that can sign transactions.
///
/// Category: Wallet/Keys
#[gen_stub_pyclass]
#[pyclass(name = "PrivateKeyGenerator")]
pub struct PyPrivateKeyGenerator {
    receive: ExtendedPrivateKey<SecretKey>,
    change: ExtendedPrivateKey<SecretKey>,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyPrivateKeyGenerator {
    /// Create a new private key generator.
    ///
    /// Args:
    ///     xprv: The master extended private key, as a string or XPrv instance.
    ///     is_multisig: Whether this is for a multisig wallet.
    ///     account_index: The account index to use.
    ///     cosigner_index: Optional cosigner index for multisig.
    ///
    /// Returns:
    ///     PrivateKeyGenerator: A new generator instance.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    #[new]
    #[pyo3(signature = (xprv, is_multisig, account_index, cosigner_index=None))]
    pub fn new(
        #[gen_stub(override_type(type_repr = "str | XPrv"))] xprv: Bound<'_, PyAny>,
        is_multisig: bool,
        account_index: u64,
        cosigner_index: Option<u32>,
    ) -> PyResult<PyPrivateKeyGenerator> {
        let xprv = if let Ok(s) = xprv.extract::<String>() {
            PyXPrv::from_xprv_str(&s)?
        } else if let Ok(py_xprv) = xprv.extract::<PyXPrv>() {
            py_xprv
        } else {
            Err(PyException::new_err("`xprv` must be type str or XPrv"))?
        };

        let xprv = xprv.inner();
        let receive = xprv
            .clone()
            .derive_path(
                &WalletDerivationManager::build_derivate_path(
                    is_multisig,
                    account_index,
                    cosigner_index,
                    Some(kaspa_bip32::AddressType::Receive),
                )
                .map_err(|err| PyException::new_err(err.to_string()))?,
            )
            .map_err(|err| PyException::new_err(err.to_string()))?;
        let change = xprv
            .clone()
            .derive_path(
                &WalletDerivationManager::build_derivate_path(
                    is_multisig,
                    account_index,
                    cosigner_index,
                    Some(kaspa_bip32::AddressType::Change),
                )
                .map_err(|err| PyException::new_err(err.to_string()))?,
            )
            .map_err(|err| PyException::new_err(err.to_string()))?;

        Ok(Self { receive, change })
    }

    /// Get a receive (external) private key at the given index.
    ///
    /// Args:
    ///     index: The address index.
    ///
    /// Returns:
    ///     PrivateKey: The private key at that index.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    pub fn receive_key(&self, index: u32) -> PyResult<PyPrivateKey> {
        let xkey = self
            .receive
            .derive_child(
                ChildNumber::new(index, false)
                    .map_err(|err| PyException::new_err(err.to_string()))?,
            )
            .map_err(|err| PyException::new_err(err.to_string()))?;
        let inner = PrivateKey::from(xkey.private_key());
        Ok(PyPrivateKey::new(inner))
    }

    /// Get a change (internal) private key at the given index.
    ///
    /// Args:
    ///     index: The address index.
    ///
    /// Returns:
    ///     PrivateKey: The private key at that index.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    pub fn change_key(&self, index: u32) -> PyResult<PyPrivateKey> {
        let xkey = self
            .change
            .derive_child(
                ChildNumber::new(index, false)
                    .map_err(|err| PyException::new_err(err.to_string()))?,
            )
            .map_err(|err| PyException::new_err(err.to_string()))?;
        let inner = PrivateKey::from(xkey.private_key());
        Ok(PyPrivateKey::new(inner))
    }
}
