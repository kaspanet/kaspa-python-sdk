use crate::{
    consensus::{client::transaction::PyTransaction, core::hashing::PySighashType},
    crypto::hashes::PyHash,
    wallet::keys::privatekey::PyPrivateKey,
};
use kaspa_consensus_client::{Transaction, sign_with_multiple_v3};
use kaspa_consensus_core::{
    hashing::{sighash_type::SIG_HASH_ALL, wasm::SighashType},
    sign::{sign_input, verify},
    tx::PopulatedTransaction,
};
use kaspa_hashes::Hash;
use kaspa_wallet_core::result::Result;
use pyo3::{exceptions::PyException, prelude::*, types::PyList};
use pyo3_stub_gen::derive::gen_stub_pyfunction;
use workflow_core::hex::ToHex;
use zeroize::Zeroize;

/// Sign a transaction with one or more private keys.
///
/// Args:
///     tx: The transaction to sign.
///     signer: List of PrivateKey objects for signing.
///     verify_sig: Whether to verify signatures after signing.
///
/// Returns:
///     Transaction: The signed transaction.
///
/// Raises:
///     Exception: If signing or verification fails.
///
/// Category: Wallet/Transactions
#[gen_stub_pyfunction]
#[pyfunction(name = "sign_transaction")]
pub fn py_sign_transaction<'py>(
    tx: PyTransaction,
    signer: Bound<'py, PyList>,
    verify_sig: bool,
) -> PyResult<PyTransaction> {
    let mut private_keys: Vec<[u8; 32]> = Vec::with_capacity(signer.len());
    for item in signer.iter() {
        let key: PyRef<'_, PyPrivateKey> = item.extract()?;
        private_keys.push(key.secret_bytes());
    }

    let transaction: Transaction = tx.into();
    let tx = sign_transaction(&transaction, &private_keys, verify_sig)
        .map_err(|err| PyException::new_err(format!("Unable to sign: {err:?}")))?;
    private_keys.zeroize();
    Ok(tx.clone().into())
}

/// Create a signature for a specific transaction input.
///
/// Args:
///     tx: The transaction containing the input to sign.
///     input_index: The index of the input to sign.
///     private_key: The private key for signing.
///     sighash_type: The signature hash type (default: All).
///
/// Returns:
///     str: The signature as a hex string.
///
/// Raises:
///     Exception: If signing fails.
///
/// Category: Wallet/Transactions
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "create_input_signature")]
#[pyo3(signature = (tx, input_index, private_key, sighash_type=None))]
pub fn py_create_input_signature(
    tx: &PyTransaction,
    input_index: u8,
    private_key: &PyPrivateKey,
    #[gen_stub(override_type(type_repr = "str | SighashType | None = SighashType.All"))]
    sighash_type: Option<PySighashType>,
) -> PyResult<String> {
    let (cctx, utxos) = tx
        .inner()
        .tx_and_utxos()
        .map_err(|err| PyException::new_err(err.to_string()))?;
    let populated_transaction = PopulatedTransaction::new(&cctx, utxos);

    let sighash_type: SighashType = sighash_type.unwrap_or(PySighashType::All).into();

    let mut key_bytes = private_key.secret_bytes();
    let signature = sign_input(
        &populated_transaction,
        input_index.into(),
        &key_bytes,
        sighash_type.into(),
    );
    key_bytes.zeroize();

    Ok(signature.to_hex())
}

/// Sign a script hash with a private key.
///
/// Args:
///     script_hash: The script hash to sign as a hex string.
///     privkey: The private key for signing.
///
/// Returns:
///     str: The signature as a hex string.
///
/// Raises:
///     Exception: If signing fails.
///
/// Category: Wallet/Transactions
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "sign_script_hash")]
pub fn py_sign_script_hash(script_hash: String, privkey: &PyPrivateKey) -> PyResult<String> {
    let script_hash = PyHash::try_from(script_hash)?;
    let mut key_bytes = privkey.secret_bytes();
    let result = sign_hash(script_hash.into(), &key_bytes)
        .map_err(|err| PyException::new_err(err.to_string()))?;
    key_bytes.zeroize();
    Ok(result.to_hex())
}

fn sign_transaction<'a>(
    tx: &'a Transaction,
    private_keys: &[[u8; 32]],
    verify_sig: bool,
) -> Result<&'a Transaction> {
    let tx = sign(tx, private_keys)?;
    if verify_sig {
        let (cctx, utxos) = tx.tx_and_utxos()?;
        let populated_transaction = PopulatedTransaction::new(&cctx, utxos);
        verify(&populated_transaction)?;
    }
    Ok(tx)
}

// Sign a transaction using schnorr, returns a new transaction with the signatures added.
// The resulting transaction may be partially signed if the supplied keys are not sufficient
// to sign all of its inputs.
fn sign<'a>(tx: &'a Transaction, privkeys: &[[u8; 32]]) -> Result<&'a Transaction> {
    Ok(sign_with_multiple_v3(tx, privkeys)?.unwrap())
}

fn sign_hash(sig_hash: Hash, privkey: &[u8; 32]) -> Result<Vec<u8>> {
    let msg = secp256k1::Message::from_digest_slice(sig_hash.as_bytes().as_slice())?;
    let schnorr_key = secp256k1::Keypair::from_seckey_slice(secp256k1::SECP256K1, privkey)?;
    let sig: [u8; 64] = *schnorr_key.sign_schnorr(msg).as_ref();
    let signature = std::iter::once(65u8)
        .chain(sig)
        .chain([SIG_HASH_ALL.to_u8()])
        .collect();
    Ok(signature)
}
