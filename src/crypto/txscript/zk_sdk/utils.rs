use crate::crypto::txscript::zk_sdk::result::PyR0SuccinctWitnessParts;
use crate::types::PyBinary;
use kaspa_python_sdk_core::create_py_exception;
use kaspa_txscript::zk_precompiles::risc0::rcpt::HashFnId;
use kaspa_txscript_zk_sdk::{
    prepare_r0_groth16_proof as native_prepare_r0_groth16_proof,
    prepare_r0_succinct_witness as native_prepare_r0_succinct_witness,
};
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyfunction};
use risc0_zkvm::{Groth16Receipt, ReceiptClaim, SuccinctReceipt};
use workflow_core::hex::ToHex;

create_py_exception!(
    /// Raised when ZK script construction, RISC Zero receipt decoding, or a
    /// builder state-machine transition fails.
    PyZkError,
    "ZkError",
    "kaspa.exceptions"
);

// Maps any zk-sdk / txscript error to the `ZkError` Python exception.
pub(crate) fn zk_err(err: impl std::fmt::Display) -> PyErr {
    PyZkError::new_err(err.to_string())
}

pub(crate) fn into_array_32(bytes: Vec<u8>, name: &'static str) -> PyResult<[u8; 32]> {
    bytes
        .as_slice()
        .try_into()
        .map_err(|_| PyZkError::new_err(format!("{name} must be 32 bytes")))
}

// Only "poseidon2" is currently accepted (also the default when omitted); other
// hash functions are not yet supported by the R0 precompile.
fn parse_hash_fn_id(value: &str) -> PyResult<HashFnId> {
    match value {
        "poseidon2" => Ok(HashFnId::Poseidon2),
        _ => Err(PyZkError::new_err(format!(
            "invalid hash function id: {value}"
        ))),
    }
}

pub(crate) fn decode_hash_fn_id(hash_fn_id: Option<String>) -> PyResult<Option<HashFnId>> {
    hash_fn_id.as_deref().map(parse_hash_fn_id).transpose()
}

pub(crate) fn decode_groth16_receipt(receipt: &PyBinary) -> PyResult<Groth16Receipt<ReceiptClaim>> {
    borsh::from_slice(receipt.as_ref())
        .map_err(|e| PyZkError::new_err(format!("failed to decode Groth16 receipt: {e}")))
}

pub(crate) fn decode_succinct_receipt(
    receipt: &PyBinary,
) -> PyResult<SuccinctReceipt<ReceiptClaim>> {
    borsh::from_slice(receipt.as_ref())
        .map_err(|e| PyZkError::new_err(format!("failed to decode succinct receipt: {e}")))
}

/// Convert a borsh-encoded `Groth16Receipt<ReceiptClaim>` to the compressed
/// ark-groth16 proof bytes (hex), without a builder.
///
/// Args:
///     receipt: A borsh-encoded `Groth16Receipt<ReceiptClaim>`, as hex, bytes,
///         or list of ints.
///
/// Returns:
///     str: The compressed proof bytes, hex-encoded.
///
/// Raises:
///     ZkError: If the receipt cannot be decoded or the proof cannot be prepared.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "prepare_r0_groth16_proof")]
pub fn py_prepare_r0_groth16_proof(receipt: PyBinary) -> PyResult<String> {
    let receipt = decode_groth16_receipt(&receipt)?;
    let bytes = native_prepare_r0_groth16_proof(&receipt).map_err(zk_err)?;
    Ok(bytes.to_hex())
}

/// Convert a borsh-encoded `SuccinctReceipt<ReceiptClaim>` to its four on-stack
/// witness byte vectors (hex), without a builder.
///
/// Args:
///     receipt: A borsh-encoded `SuccinctReceipt<ReceiptClaim>`, as hex, bytes,
///         or list of ints.
///
/// Returns:
///     R0SuccinctWitnessParts: The claim, control index, control digests, and seal.
///
/// Raises:
///     ZkError: If the receipt cannot be decoded or the witness cannot be prepared.
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "prepare_r0_succinct_witness")]
pub fn py_prepare_r0_succinct_witness(receipt: PyBinary) -> PyResult<PyR0SuccinctWitnessParts> {
    let receipt = decode_succinct_receipt(&receipt)?;
    let w = native_prepare_r0_succinct_witness(&receipt).map_err(zk_err)?;
    Ok(PyR0SuccinctWitnessParts {
        claim: w.claim,
        control_index: w.control_index,
        control_digests: w.control_digests,
        seal: w.seal,
    })
}
