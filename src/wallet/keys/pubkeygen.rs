use kaspa_addresses::Address;
use kaspa_consensus_core::network::NetworkType;
use kaspa_wallet_core::derivation::WalletDerivationManagerTrait;
use kaspa_wallet_keys::publickey::PublicKey;
use kaspa_wallet_keys::result::Result;
use kaspa_wallet_keys::{derivation::gen1::WalletDerivationManager, xpub::XPub};
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::consensus::core::network::PyNetworkType;
use crate::wallet::keys::xprv::PyXPrv;
use crate::{address::PyAddress, wallet::keys::publickey::PyPublicKey};

/// Generator for deriving public keys and addresses from an extended public key.
///
/// Useful for creating watch-only wallets that can generate addresses
/// without access to private keys.
///
/// Category: Wallet/Keys
#[gen_stub_pyclass]
#[pyclass(name = "PublicKeyGenerator")]
#[derive(Clone)]
pub struct PyPublicKeyGenerator {
    hd_wallet: WalletDerivationManager,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyPublicKeyGenerator {
    /// Create a generator from an extended public key string.
    ///
    /// Args:
    ///     kpub: The extended public key (xpub/kpub format).
    ///     cosigner_index: Optional cosigner index for multisig.
    ///
    /// Returns:
    ///     PublicKeyGenerator: A new generator instance.
    ///
    /// Raises:
    ///     Exception: If parsing fails.
    #[staticmethod]
    #[pyo3(name = "from_xpub")]
    #[pyo3(signature = (kpub, cosigner_index=None))]
    fn from_xpub(kpub: &str, cosigner_index: Option<u32>) -> PyResult<PyPublicKeyGenerator> {
        let kpub = XPub::try_new(kpub).map_err(|err| PyException::new_err(err.to_string()))?;
        let xpub = kpub.inner();
        let hd_wallet =
            WalletDerivationManager::from_extended_public_key(xpub.clone(), cosigner_index)
                .map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(Self { hd_wallet })
    }

    /// Create a generator from a master extended private key.
    ///
    /// Derives the account-level public key and creates a generator.
    ///
    /// Args:
    ///     xprv: The master extended private key, as a string or XPrv instance.
    ///     is_multisig: Whether this is for a multisig wallet.
    ///     account_index: The account index to derive.
    ///     cosigner_index: Optional cosigner index for multisig.
    ///
    /// Returns:
    ///     PublicKeyGenerator: A new generator instance.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    #[staticmethod]
    #[pyo3(signature = (xprv, is_multisig, account_index, cosigner_index=None))]
    fn from_master_xprv(
        #[gen_stub(override_type(type_repr = "str | XPrv"))] xprv: Bound<'_, PyAny>,
        is_multisig: bool,
        account_index: u64,
        cosigner_index: Option<u32>,
    ) -> PyResult<PyPublicKeyGenerator> {
        let xprv = if let Ok(s) = xprv.extract::<String>() {
            PyXPrv::from_xprv_str(&s)?
        } else if let Ok(py_xprv) = xprv.extract::<PyXPrv>() {
            py_xprv
        } else {
            Err(PyException::new_err("`xprv` must be type str or XPrv"))?
        };

        let path =
            WalletDerivationManager::build_derivate_path(is_multisig, account_index, None, None)
                .map_err(|err| PyException::new_err(err.to_string()))?;
        let xprv = xprv
            .inner()
            .clone()
            .derive_path(&path)
            .map_err(|err| PyException::new_err(err.to_string()))?;
        let xpub = xprv.public_key();
        let hd_wallet = WalletDerivationManager::from_extended_public_key(xpub, cosigner_index)
            .map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(Self { hd_wallet })
    }

    /// Derive a range of receive (external) public keys.
    ///
    /// Args:
    ///     start: Start index (inclusive).
    ///     end: End index (exclusive).
    ///
    /// Returns:
    ///     list[PublicKey]: The derived public keys.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    #[pyo3(name = "receive_pubkeys")]
    fn receive_pubkeys(&self, mut start: u32, mut end: u32) -> PyResult<Vec<PyPublicKey>> {
        if start > end {
            (start, end) = (end, start)
        }
        let pubkeys = self
            .hd_wallet
            .receive_pubkey_manager()
            .derive_pubkey_range(start..end)
            .map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(pubkeys
            .into_iter()
            .map(|pk| PyPublicKey(PublicKey::from(pk)))
            .collect())
    }

    /// Derive a receive (external) public key at the given index.
    ///
    /// Args:
    ///     index: The address index.
    ///
    /// Returns:
    ///     PublicKey: The derived public key.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    #[pyo3(name = "receive_pubkey")]
    pub fn receive_pubkey(&self, index: u32) -> PyResult<PyPublicKey> {
        let inner = self
            .hd_wallet
            .receive_pubkey_manager()
            .derive_pubkey(index)
            .map_err(|err| PyException::new_err(err.to_string()))?
            .into();

        Ok(PyPublicKey(inner))
    }

    /// Derive a range of receive public keys as hex strings.
    ///
    /// Args:
    ///     start: Start index (inclusive).
    ///     end: End index (exclusive).
    ///
    /// Returns:
    ///     list[str]: The derived public keys as hex.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    #[pyo3(name = "receive_pubkeys_as_strings")]
    fn receive_pubkeys_as_strings(&self, mut start: u32, mut end: u32) -> PyResult<Vec<String>> {
        if start > end {
            (start, end) = (end, start);
        }
        let pubkeys = self
            .hd_wallet
            .receive_pubkey_manager()
            .derive_pubkey_range(start..end)
            .map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(pubkeys
            .into_iter()
            .map(|pk| PublicKey::from(pk).to_string())
            .collect())
    }

    /// Derive a receive public key as hex string.
    ///
    /// Args:
    ///     index: The address index.
    ///
    /// Returns:
    ///     str: The public key as hex.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    #[pyo3(name = "receive_pubkey_as_string")]
    pub fn receive_pubkey_as_string(&self, index: u32) -> PyResult<String> {
        Ok(self
            .hd_wallet
            .receive_pubkey_manager()
            .derive_pubkey(index)
            .map_err(|err| PyException::new_err(err.to_string()))?
            .to_string())
    }

    /// Derive a range of receive addresses.
    ///
    /// Args:
    ///     network_type: The network type for address encoding.
    ///     start: Start index (inclusive).
    ///     end: End index (exclusive).
    ///
    /// Returns:
    ///     list[Address]: The derived addresses.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    fn receive_addresses(
        &self,
        #[gen_stub(override_type(type_repr = "str | NetworkType"))] network_type: PyNetworkType,
        mut start: u32,
        mut end: u32,
    ) -> PyResult<Vec<PyAddress>> {
        if start > end {
            (start, end) = (end, start);
        }
        let network_type: NetworkType = network_type.into();
        let pubkeys = self
            .hd_wallet
            .receive_pubkey_manager()
            .derive_pubkey_range(start..end)
            .map_err(|err| PyException::new_err(err.to_string()))?;
        let addresses = pubkeys
            .into_iter()
            .map(|pk| PublicKey::from(pk).to_address(network_type))
            .collect::<Result<Vec<Address>>>()
            .map_err(|err| PyException::new_err(err.to_string()))?;
        let addresses = addresses.into_iter().map(PyAddress::from).collect();
        Ok(addresses)
    }

    /// Derive a receive address at the given index.
    ///
    /// Args:
    ///     network_type: The network type for address encoding.
    ///     index: The address index.
    ///
    /// Returns:
    ///     Address: The derived address.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    fn receive_address(
        &self,
        #[gen_stub(override_type(type_repr = "str | NetworkType"))] network_type: PyNetworkType,
        index: u32,
    ) -> PyResult<PyAddress> {
        let inner = PublicKey::from(
            self.hd_wallet
                .receive_pubkey_manager()
                .derive_pubkey(index)
                .map_err(|err| PyException::new_err(err.to_string()))?,
        )
        .to_address(NetworkType::from(network_type))
        .map_err(|err| PyException::new_err(err.to_string()))?;

        Ok(PyAddress(inner))
    }

    /// Derive a range of receive addresses as strings.
    ///
    /// Args:
    ///     network_type: The network type for address encoding.
    ///     start: Start index (inclusive).
    ///     end: End index (exclusive).
    ///
    /// Returns:
    ///     list[str]: The derived addresses as strings.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    fn receive_addresses_as_strings(
        &self,
        #[gen_stub(override_type(type_repr = "str | NetworkType"))] network_type: PyNetworkType,
        mut start: u32,
        mut end: u32,
    ) -> PyResult<Vec<String>> {
        if start > end {
            (start, end) = (end, start);
        }
        let network_type: NetworkType = network_type.into();
        let pubkeys = self
            .hd_wallet
            .receive_pubkey_manager()
            .derive_pubkey_range(start..end)
            .map_err(|err| PyException::new_err(err.to_string()))?;
        let addresses = pubkeys
            .into_iter()
            .map(|pk| PublicKey::from(pk).to_address(network_type))
            .collect::<Result<Vec<Address>>>()
            .map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(addresses
            .into_iter()
            .map(|a| a.address_to_string())
            .collect())
    }

    /// Derive a receive address as string.
    ///
    /// Args:
    ///     network_type: The network type for address encoding.
    ///     index: The address index.
    ///
    /// Returns:
    ///     str: The derived address string.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    fn receive_address_as_string(
        &self,
        #[gen_stub(override_type(type_repr = "str | NetworkType"))] network_type: PyNetworkType,
        index: u32,
    ) -> PyResult<String> {
        Ok(PublicKey::from(
            self.hd_wallet
                .receive_pubkey_manager()
                .derive_pubkey(index)
                .map_err(|err| PyException::new_err(err.to_string()))?,
        )
        .to_address(NetworkType::from(network_type))
        .map_err(|err| PyException::new_err(err.to_string()))?
        .to_string())
    }

    /// Derive a range of change (internal) public keys.
    ///
    /// Args:
    ///     start: Start index (inclusive).
    ///     end: End index (exclusive).
    ///
    /// Returns:
    ///     list[PublicKey]: The derived public keys.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    #[pyo3(name = "change_pubkeys")]
    pub fn change_pubkeys(&self, mut start: u32, mut end: u32) -> PyResult<Vec<PyPublicKey>> {
        if start > end {
            (start, end) = (end, start);
        }
        let pubkeys = self
            .hd_wallet
            .change_pubkey_manager()
            .derive_pubkey_range(start..end)
            .map_err(|err| PyException::new_err(err.to_string()))?;
        let pubkeys = pubkeys
            .into_iter()
            .map(PublicKey::from)
            .collect::<Vec<PublicKey>>();
        let pubkeys = pubkeys.into_iter().map(PyPublicKey::from).collect();
        Ok(pubkeys)
    }

    /// Derive a change (internal) public key at the given index.
    ///
    /// Args:
    ///     index: The address index.
    ///
    /// Returns:
    ///     PublicKey: The derived public key.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    #[pyo3(name = "change_pubkey")]
    pub fn change_pubkey(&self, index: u32) -> PyResult<PyPublicKey> {
        let inner: PublicKey = self
            .hd_wallet
            .change_pubkey_manager()
            .derive_pubkey(index)
            .map_err(|err| PyException::new_err(err.to_string()))?
            .into();

        Ok(PyPublicKey(inner))
    }

    /// Derive a range of change public keys as hex strings.
    ///
    /// Args:
    ///     start: Start index (inclusive).
    ///     end: End index (exclusive).
    ///
    /// Returns:
    ///     list[str]: The derived public keys as hex.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    #[pyo3(name = "change_pubkeys_as_strings")]
    pub fn change_pubkeys_as_strings(&self, mut start: u32, mut end: u32) -> PyResult<Vec<String>> {
        if start > end {
            (start, end) = (end, start);
        }
        let pubkeys = self
            .hd_wallet
            .change_pubkey_manager()
            .derive_pubkey_range(start..end)
            .map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(pubkeys
            .into_iter()
            .map(|pk| PublicKey::from(pk).to_string())
            .collect())
    }

    /// Derive a change public key as hex string.
    ///
    /// Args:
    ///     index: The address index.
    ///
    /// Returns:
    ///     str: The public key as hex.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    #[pyo3(name = "change_pubkey_as_string")]
    pub fn change_pubkey_as_string(&self, index: u32) -> PyResult<String> {
        Ok(self
            .hd_wallet
            .change_pubkey_manager()
            .derive_pubkey(index)
            .map_err(|err| PyException::new_err(err.to_string()))?
            .to_string())
    }

    /// Derive a range of change addresses.
    ///
    /// Args:
    ///     network_type: The network type for address encoding.
    ///     start: Start index (inclusive).
    ///     end: End index (exclusive).
    ///
    /// Returns:
    ///     list[Address]: The derived addresses.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    pub fn change_addresses(
        &self,
        #[gen_stub(override_type(type_repr = "str | NetworkType"))] network_type: PyNetworkType,
        mut start: u32,
        mut end: u32,
    ) -> PyResult<Vec<PyAddress>> {
        if start > end {
            (start, end) = (end, start);
        }
        let network_type: NetworkType = network_type.into();
        let pubkeys = self
            .hd_wallet
            .receive_pubkey_manager()
            .derive_pubkey_range(start..end)
            .map_err(|err| PyException::new_err(err.to_string()))?;
        let addresses = pubkeys
            .into_iter()
            .map(|pk| PublicKey::from(pk).to_address(network_type))
            .collect::<Result<Vec<Address>>>()
            .map_err(|err| PyException::new_err(err.to_string()))?;
        let addresses = addresses.into_iter().map(PyAddress::from).collect();
        Ok(addresses)
    }

    /// Derive a change address at the given index.
    ///
    /// Args:
    ///     network_type: The network type for address encoding.
    ///     index: The address index.
    ///
    /// Returns:
    ///     Address: The derived address.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    pub fn change_address(
        &self,
        #[gen_stub(override_type(type_repr = "str | NetworkType"))] network_type: PyNetworkType,
        index: u32,
    ) -> PyResult<PyAddress> {
        let inner = PublicKey::from(
            self.hd_wallet
                .change_pubkey_manager()
                .derive_pubkey(index)
                .map_err(|err| PyException::new_err(err.to_string()))?,
        )
        .to_address(NetworkType::from(network_type))
        .map_err(|err| PyException::new_err(err.to_string()))?;

        Ok(PyAddress(inner))
    }

    /// Derive a range of change addresses as strings.
    ///
    /// Args:
    ///     network_type: The network type for address encoding.
    ///     start: Start index (inclusive).
    ///     end: End index (exclusive).
    ///
    /// Returns:
    ///     list[str]: The derived addresses as strings.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    pub fn change_addresses_as_strings(
        &self,
        #[gen_stub(override_type(type_repr = "str | NetworkType"))] network_type: PyNetworkType,
        mut start: u32,
        mut end: u32,
    ) -> PyResult<Vec<String>> {
        if start > end {
            (start, end) = (end, start);
        }
        let network_type: NetworkType = network_type.into();
        let pubkeys = self
            .hd_wallet
            .change_pubkey_manager()
            .derive_pubkey_range(start..end)
            .map_err(|err| PyException::new_err(err.to_string()))?;
        let addresses = pubkeys
            .into_iter()
            .map(|pk| PublicKey::from(pk).to_address(network_type))
            .collect::<Result<Vec<Address>>>()
            .map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(addresses
            .into_iter()
            .map(|a| a.address_to_string())
            .collect())
    }

    /// Derive a change address as string.
    ///
    /// Args:
    ///     network_type: The network type for address encoding.
    ///     index: The address index.
    ///
    /// Returns:
    ///     str: The derived address string.
    ///
    /// Raises:
    ///     Exception: If derivation fails.
    pub fn change_address_as_string(
        &self,
        #[gen_stub(override_type(type_repr = "str | NetworkType"))] network_type: PyNetworkType,
        index: u32,
    ) -> PyResult<String> {
        Ok(PublicKey::from(
            self.hd_wallet
                .receive_pubkey_manager()
                .derive_pubkey(index)
                .map_err(|err| PyException::new_err(err.to_string()))?,
        )
        .to_address(NetworkType::from(network_type))
        .map_err(|err| PyException::new_err(err.to_string()))?
        .to_string())
    }

    /// Get the string representation of this generator.
    ///
    /// Returns:
    ///     str: The generator info string.
    #[pyo3(name = "to_string")]
    pub fn to_string(&self) -> PyResult<String> {
        Ok(self.hd_wallet.to_string(None).to_string())
    }
}
