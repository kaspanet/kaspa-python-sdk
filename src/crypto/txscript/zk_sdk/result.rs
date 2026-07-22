use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use workflow_core::hex::ToHex;

/// The result of finalizing a `ZkScriptBuilder`.
///
/// `sig_script` is the spending script — set it on the transaction input.
/// `redeem_script` is the inner commit script — hash it with
/// `pay_to_script_hash_script` to derive the P2SH script-public-key.
#[gen_stub_pyclass]
#[pyclass(name = "FinalizedR0Script")]
pub struct PyFinalizedR0Script {
    sig_script: Vec<u8>,
    redeem_script: Vec<u8>,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyFinalizedR0Script {
    /// The spending (signature) script, hex-encoded.
    ///
    /// Returns:
    ///     str: The sig script bytes as a hex string.
    #[getter]
    pub fn sig_script(&self) -> String {
        self.sig_script.to_hex()
    }

    /// The inner redeem (commit) script, hex-encoded.
    ///
    /// Returns:
    ///     str: The redeem script bytes as a hex string.
    #[getter]
    pub fn redeem_script(&self) -> String {
        self.redeem_script.to_hex()
    }

    /// The detailed string representation.
    ///
    /// Returns:
    ///     str: The FinalizedR0Script as a repr string.
    fn __repr__(&self) -> String {
        format!(
            "FinalizedR0Script(sig_script={} bytes, redeem_script={} bytes)",
            self.sig_script.len(),
            self.redeem_script.len()
        )
    }
}

impl PyFinalizedR0Script {
    pub(crate) fn new(sig_script: Vec<u8>, redeem_script: Vec<u8>) -> Self {
        Self {
            sig_script,
            redeem_script,
        }
    }
}

/// The receipt-derived succinct witness items, hex-encoded, in the order the
/// verifier consumes them (the journal is caller-owned and excluded).
#[gen_stub_pyclass]
#[pyclass(name = "R0SuccinctWitnessParts")]
pub struct PyR0SuccinctWitnessParts {
    pub(crate) claim: Vec<u8>,
    pub(crate) control_index: Vec<u8>,
    pub(crate) control_digests: Vec<u8>,
    pub(crate) seal: Vec<u8>,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyR0SuccinctWitnessParts {
    /// The SHA-256 digest of the receipt claim, hex-encoded.
    #[getter]
    pub fn claim(&self) -> String {
        self.claim.to_hex()
    }

    /// The control inclusion-proof leaf index (u32 little-endian), hex-encoded.
    #[getter]
    pub fn control_index(&self) -> String {
        self.control_index.to_hex()
    }

    /// The flattened control inclusion-proof sibling digests, hex-encoded.
    #[getter]
    pub fn control_digests(&self) -> String {
        self.control_digests.to_hex()
    }

    /// The flattened STARK seal (u32 little-endian words), hex-encoded.
    #[getter]
    pub fn seal(&self) -> String {
        self.seal.to_hex()
    }

    /// The detailed string representation.
    ///
    /// Returns:
    ///     str: The R0SuccinctWitnessParts as a repr string.
    fn __repr__(&self) -> String {
        format!(
            "R0SuccinctWitnessParts(claim={} bytes, control_index={} bytes, control_digests={} bytes, seal={} bytes)",
            self.claim.len(),
            self.control_index.len(),
            self.control_digests.len(),
            self.seal.len()
        )
    }
}
