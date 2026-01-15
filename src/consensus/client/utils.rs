use crate::{
    address::PyAddress,
    consensus::core::{network::PyNetworkType, script_public_key::PyScriptPublicKey},
    types::PyBinary,
};
use kaspa_consensus_core::network::NetworkType;
use kaspa_txscript::{script_class::ScriptClass, standard};
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::gen_stub_pyfunction;
use workflow_core::hex::ToHex;

/// Create a pay-to-address locking script.
///
/// Args:
///     address: The destination address.
///
/// Returns:
///     ScriptPublicKey: The locking script for the address.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "pay_to_address_script")]
pub fn py_pay_to_address_script(address: PyAddress) -> PyResult<PyScriptPublicKey> {
    Ok(standard::pay_to_address_script(&address.into()).into())
}

/// Create a pay-to-script-hash (P2SH) locking script.
///
/// Args:
///     redeem_script: The redeem script to hash.
///
/// Returns:
///     ScriptPublicKey: The P2SH locking script.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "pay_to_script_hash_script")]
pub fn py_pay_to_script_hash_script(redeem_script: PyBinary) -> PyResult<PyScriptPublicKey> {
    Ok(standard::pay_to_script_hash_script(redeem_script.data.as_slice()).into())
}

/// Create a signature script for spending a P2SH output.
///
/// Args:
///     redeem_script: The original redeem script.
///     signature: The signature proving authorization.
///
/// Returns:
///     str: The signature script as a hex string.
///
/// Raises:
///     Exception: If script creation fails.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "pay_to_script_hash_signature_script")]
pub fn py_pay_to_script_hash_signature_script(
    redeem_script: PyBinary,
    signature: PyBinary,
) -> PyResult<String> {
    let script = standard::pay_to_script_hash_signature_script(redeem_script.data, signature.data)
        .map_err(|err| PyException::new_err(err.to_string()))?;
    Ok(script.to_hex())
}

/// Extract the address from a script public key.
///
/// Args:
///     script_public_key: The script to extract the address from.
///     network: The network type for address encoding.
///
/// Returns:
///     Address: The extracted address.
///
/// Raises:
///     Exception: If address extraction fails.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "address_from_script_public_key")]
pub fn py_address_from_script_public_key(
    script_public_key: PyScriptPublicKey,
    #[gen_stub(override_type(type_repr = "str | NetworkType"))] network: PyNetworkType,
) -> PyResult<PyAddress> {
    match standard::extract_script_pub_key_address(
        &script_public_key.into(),
        NetworkType::from(network).into(),
    ) {
        Ok(address) => Ok(address.into()),
        Err(err) => Err(pyo3::exceptions::PyException::new_err(format!("{}", err))),
    }
}

/// Check if a script is a pay-to-pubkey (P2PK) script.
///
/// Args:
///     script: The script bytes to check.
///
/// Returns:
///     bool: True if the script is a P2PK script.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "is_script_pay_to_pubkey")]
pub fn py_is_script_pay_to_pubkey(script: PyBinary) -> PyResult<bool> {
    Ok(ScriptClass::is_pay_to_pubkey(script.data.as_slice()))
}

/// Check if a script is a pay-to-pubkey-ECDSA script.
///
/// Args:
///     script: The script bytes to check.
///
/// Returns:
///     bool: True if the script is a P2PK-ECDSA script.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "is_script_pay_to_pubkey_ecdsa")]
pub fn py_is_script_pay_to_pubkey_ecdsa(script: PyBinary) -> PyResult<bool> {
    Ok(ScriptClass::is_pay_to_pubkey_ecdsa(script.data.as_slice()))
}

/// Check if a script is a pay-to-script-hash (P2SH) script.
///
/// Args:
///     script: The script bytes to check.
///
/// Returns:
///     bool: True if the script is a P2SH script.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "is_script_pay_to_script_hash")]
pub fn py_is_script_pay_to_script_hash(script: PyBinary) -> PyResult<bool> {
    Ok(ScriptClass::is_pay_to_script_hash(script.data.as_slice()))
}
