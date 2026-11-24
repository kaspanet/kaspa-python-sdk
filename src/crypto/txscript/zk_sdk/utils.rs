use crate::types::PyBinary;
use kaspa_python_sdk_core::create_py_exception;
use kaspa_txscript_zk_sdk::prepare_r0_groth16_proof as native_prepare_r0_groth16_proof;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyfunction};
use risc0_zkvm::{Groth16Receipt, ReceiptClaim};
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

fn decode_receipt<T: borsh::BorshDeserialize>(receipt: &PyBinary, what: &str) -> PyResult<T> {
    borsh::from_slice(receipt.as_ref())
        .map_err(|e| PyZkError::new_err(format!("failed to decode {what} receipt: {e}")))
}

pub(crate) fn decode_groth16_receipt(receipt: &PyBinary) -> PyResult<Groth16Receipt<ReceiptClaim>> {
    decode_receipt(receipt, "Groth16")
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
