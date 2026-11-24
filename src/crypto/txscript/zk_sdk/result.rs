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
