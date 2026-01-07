use crate::wallet::keys::{privatekey::PyPrivateKey, publickey::PyPublicKey};
// use kaspa_wallet_core::imports::*;
use kaspa_wallet_core::message::*;
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::gen_stub_pyfunction;
use zeroize::Zeroize;

/// Sign an arbitrary message with a private key.
///
/// Args:
///     message: The message string to sign.
///     private_key: The private key to sign with.
///     no_aux_rand: If True, use deterministic signing (default: False).
///
/// Returns:
///     str: The signature as a hex string.
///
/// Raises:
///     Exception: If signing fails.
///
/// Category: Wallet/Core
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "sign_message")]
#[pyo3(signature = (message, private_key, no_aux_rand=false))]
pub fn py_sign_message(
    message: &str,
    private_key: &PyPrivateKey,
    no_aux_rand: bool,
) -> PyResult<String> {
    let mut privkey_bytes = [0u8; 32];
    privkey_bytes.copy_from_slice(&private_key.secret_bytes());
    let pm = PersonalMessage(message);
    let sign_options = SignMessageOptions { no_aux_rand };
    let sig_vec = sign_message(&pm, &privkey_bytes, &sign_options)
        .map_err(|err| PyException::new_err(format!("{}", err)))?;
    privkey_bytes.zeroize();
    Ok(faster_hex::hex_string(sig_vec.as_slice()))
}

/// Verify a message signature.
///
/// Args:
///     message: The original message.
///     signature: The signature hex string.
///     public_key: The public key to verify against.
///
/// Returns:
///     bool: True if the signature is valid, False otherwise.
///
/// Raises:
///     Exception: If the signature format is invalid.
///
/// Category: Wallet/Core
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "verify_message")]
pub fn py_verify_message(
    message: String,
    signature: String,
    public_key: PyPublicKey,
) -> PyResult<bool> {
    let pm = PersonalMessage(&message);
    let mut signature_bytes = [0u8; 64];
    faster_hex::hex_decode(signature.as_bytes(), &mut signature_bytes)
        .map_err(|err| PyException::new_err(format!("{}", err)))?;

    Ok(verify_message(
        &pm,
        &signature_bytes.to_vec(),
        &public_key.0.xonly_public_key,
    )
    .is_ok())
}
