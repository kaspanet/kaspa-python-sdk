use kaspa_consensus_core::network::NetworkType;
use kaspa_wallet_core::{derivation::create_address, prelude::AccountKind};
use kaspa_wallet_keys::publickey::PublicKey;
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::gen_stub_pyfunction;

use crate::{
    address::PyAddress,
    consensus::core::network::PyNetworkType,
    wallet::{core::account::kind::PyAccountKind, keys::publickey::PyPublicKey},
};

/// Create a multisig address from multiple public keys.
///
/// Args:
///     minimum_signatures: The minimum number of signatures required to spend.
///     keys: List of public keys for the multisig.
///     network_type: The network type for address encoding.
///     ecdsa: Use ECDSA signatures instead of Schnorr (default: False).
///     account_kind: Optional account kind for derivation.
///
/// Returns:
///     Address: The multisig address.
///
/// Raises:
///     Exception: If address creation fails.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "create_multisig_address")]
#[pyo3(signature = (minimum_signatures, keys, network_type, ecdsa=Some(false), account_kind=None))]
pub fn py_create_multisig_address(
    minimum_signatures: usize,
    keys: Vec<PyPublicKey>,
    #[gen_stub(override_type(type_repr = "str | NetworkType"))] network_type: PyNetworkType,
    ecdsa: Option<bool>,
    account_kind: Option<PyAccountKind>,
) -> PyResult<PyAddress> {
    let keys = keys
        .into_iter()
        .map(|pk| PublicKey::from(pk).try_into())
        .collect::<Result<Vec<_>, kaspa_wallet_keys::error::Error>>()
        .map_err(|err| PyException::new_err(err.to_string()))?;
    Ok(create_address(
        minimum_signatures,
        keys,
        NetworkType::from(network_type).into(),
        ecdsa.unwrap_or(false),
        account_kind.map(AccountKind::from),
    )
    .map_err(|err| PyException::new_err(err.to_string()))?
    .into())
}
