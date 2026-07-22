use crate::crypto::txscript::builder::build_engine_flags;
use crate::crypto::txscript::zk_sdk::result::PyFinalizedR0Script;
use crate::crypto::txscript::zk_sdk::utils::{
    PyZkError, decode_groth16_receipt, decode_hash_fn_id, decode_succinct_receipt, into_array_32,
    zk_err,
};
use crate::types::PyBinary;
use kaspa_txscript::script_builder::ScriptBuilder;
use kaspa_txscript_zk_sdk::{
    BoundedR0Groth16Script, BoundedR0SuccinctScript, UnboundedR0Script,
    ZkScriptBuilder as NativeZkScriptBuilder, append_r0_groth16_verifier,
    append_r0_groth16_verifier_with_fixed_journal, append_r0_succinct_verifier,
    append_r0_succinct_verifier_with_fixed_journal, push_r0_groth16_proof,
    push_r0_succinct_witness,
};
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use risc0_zkvm::Digest;
use workflow_core::hex::ToHex;

/// Runtime mirror of the native compile-time type-state. FFI cannot carry the
/// native crate's `PhantomData` type-state, so the `Unbounded` →
/// `BoundedGroth16` / `BoundedSuccinct` → finalized transitions are checked at
/// runtime and the wrong call raises `ZkError`. `Taken` is a transient sentinel
/// held while ownership of the inner native builder is moved across a state
/// transition.
enum InnerState {
    Unbounded(NativeZkScriptBuilder<UnboundedR0Script>),
    BoundedGroth16(NativeZkScriptBuilder<BoundedR0Groth16Script>),
    BoundedSuccinct(NativeZkScriptBuilder<BoundedR0SuccinctScript>),
    Taken,
}

impl InnerState {
    fn script(&self) -> &[u8] {
        match self {
            InnerState::Unbounded(b) => b.script(),
            InnerState::BoundedGroth16(b) => b.script(),
            InnerState::BoundedSuccinct(b) => b.script(),
            InnerState::Taken => &[],
        }
    }

    fn drain(&mut self) -> Vec<u8> {
        match std::mem::replace(self, InnerState::Taken) {
            InnerState::Unbounded(b) => b.drain(),
            InnerState::BoundedGroth16(b) => b.drain(),
            InnerState::BoundedSuccinct(b) => b.drain(),
            InnerState::Taken => Vec::new(),
        }
    }

    fn builder_mut(&mut self) -> PyResult<&mut ScriptBuilder> {
        match self {
            InnerState::Unbounded(b) => Ok(b.builder_mut()),
            InnerState::BoundedGroth16(b) => Ok(b.builder_mut()),
            InnerState::BoundedSuccinct(b) => Ok(b.builder_mut()),
            InnerState::Taken => Err(PyZkError::new_err("builder has been consumed")),
        }
    }
}

/// A staged builder for RISC Zero zk-to-script locking scripts.
///
/// Build flow:
///     1. `ZkScriptBuilder.new_r0(...)` — unbounded.
///     2. `commit_to_groth16(image_id)` *or*
///        `commit_to_succinct(image_id, control_id, hash_fn_id=None)` — bounded.
///     3. `finalize_with_groth16_proof(receipt, journal_hash)` *or*
///        `finalize_with_succinct_proof(receipt, journal)` — a FinalizedR0Script.
///
/// Calling a method in the wrong state raises `ZkError`. The lower-level
/// fragment methods (`add_data`, `push_*`, `append_*`) operate on the builder in
/// any state, for composing scripts by hand.
#[gen_stub_pyclass]
#[pyclass(name = "ZkScriptBuilder")]
pub struct PyZkScriptBuilder {
    inner: InnerState,
}

impl PyZkScriptBuilder {
    fn take(&mut self) -> InnerState {
        std::mem::replace(&mut self.inner, InnerState::Taken)
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyZkScriptBuilder {
    /// Construct a new `ZkScriptBuilder` for the RISC Zero proving flow.
    ///
    /// Exposed as a static factory (not a constructor) — matching the WASM SDK
    /// — so future non-R0 backends can add their own factories without a
    /// breaking change.
    ///
    /// Args:
    ///     covenants_enabled: Enable covenant opcodes and post-Toccata script
    ///         limits (default: False). Must be True to emit the zk precompile
    ///         opcode the verifier fragments rely on.
    ///     sigop_script_units: Script units charged per signature operation.
    ///         Defaults to the native engine default when omitted.
    ///
    /// Returns:
    ///     ZkScriptBuilder: A new unbounded builder.
    #[staticmethod]
    #[pyo3(signature = (covenants_enabled=false, sigop_script_units=None))]
    pub fn new_r0(covenants_enabled: bool, sigop_script_units: Option<u64>) -> Self {
        let flags = build_engine_flags(covenants_enabled, sigop_script_units);
        Self {
            inner: InnerState::Unbounded(NativeZkScriptBuilder::new_r0_with_flags(flags)),
        }
    }

    /// The current script bytes as a hex string.
    ///
    /// Returns:
    ///     str: The script built so far, hex-encoded.
    pub fn script(&self) -> String {
        self.inner.script().to_hex()
    }

    /// Drain (empty) the builder and return the script bytes as a hex string.
    ///
    /// Returns:
    ///     str: The script bytes, hex-encoded.
    pub fn drain(&mut self) -> String {
        self.inner.drain().to_hex()
    }

    /// Push raw data (canonical encoding) onto the builder.
    ///
    /// Use this to push the caller-owned journal / journal hash or a redeem
    /// script when composing a script from fragments.
    ///
    /// Args:
    ///     data: Data bytes as hex, bytes, or list of ints.
    ///
    /// Raises:
    ///     ZkError: If the data cannot be added (or the builder is consumed).
    pub fn add_data(&mut self, data: PyBinary) -> PyResult<()> {
        self.inner
            .builder_mut()?
            .add_data(data.as_ref())
            .map_err(zk_err)?;
        Ok(())
    }

    /// Commit the script to unlocking only on a valid Groth16 proof for the
    /// given 32-byte image id. Transitions an unbounded builder into the
    /// Groth16-bounded state.
    ///
    /// Args:
    ///     image_id: The 32-byte RISC Zero image id, as hex, bytes, or list.
    ///
    /// Raises:
    ///     ZkError: If the builder is not unbounded or `image_id` is malformed.
    pub fn commit_to_groth16(&mut self, image_id: PyBinary) -> PyResult<()> {
        let image_id = into_array_32(image_id.into(), "image_id")?;
        match self.take() {
            InnerState::Unbounded(b) => {
                let bounded = b.commit_to_groth16(image_id).map_err(zk_err)?;
                self.inner = InnerState::BoundedGroth16(bounded);
                Ok(())
            }
            other => {
                self.inner = other;
                Err(PyZkError::new_err(
                    "commit_to_groth16 requires an unbounded builder",
                ))
            }
        }
    }

    /// Commit the script to unlocking only on a valid succinct proof for the
    /// given image id and control id. Transitions an unbounded builder into the
    /// succinct-bounded state.
    ///
    /// Args:
    ///     image_id: The 32-byte RISC Zero image id, as hex, bytes, or list.
    ///     control_id: The 32-byte control id, as hex, bytes, or list.
    ///     hash_fn_id: The hash function id; currently only "poseidon2" (also
    ///         the default when omitted).
    ///
    /// Raises:
    ///     ZkError: If the builder is not unbounded, an id is malformed, or the
    ///         hash function id is unsupported.
    #[pyo3(signature = (image_id, control_id, hash_fn_id=None))]
    pub fn commit_to_succinct(
        &mut self,
        image_id: PyBinary,
        control_id: PyBinary,
        hash_fn_id: Option<String>,
    ) -> PyResult<()> {
        let image_id = into_array_32(image_id.into(), "image_id")?;
        let control_id = into_array_32(control_id.into(), "control_id")?;
        let hash_fn = decode_hash_fn_id(hash_fn_id)?;
        match self.take() {
            InnerState::Unbounded(b) => {
                let bounded = b
                    .commit_to_succinct(image_id, control_id, hash_fn)
                    .map_err(zk_err)?;
                self.inner = InnerState::BoundedSuccinct(bounded);
                Ok(())
            }
            other => {
                self.inner = other;
                Err(PyZkError::new_err(
                    "commit_to_succinct requires an unbounded builder",
                ))
            }
        }
    }

    /// Decode a Groth16 receipt and push the compressed proof bytes onto the
    /// builder.
    ///
    /// Typically called while building a signature script; the script that
    /// invokes `append_r0_groth16_verifier` is responsible for placing
    /// `journal_hash` under this proof so the verifier sees
    /// `[..., journal_hash, compressed_proof]`.
    ///
    /// Args:
    ///     receipt: A borsh-encoded `Groth16Receipt<ReceiptClaim>`, as hex,
    ///         bytes, or list.
    ///
    /// Raises:
    ///     ZkError: If the receipt cannot be decoded or the proof cannot be added.
    pub fn push_r0_groth16_proof(&mut self, receipt: PyBinary) -> PyResult<()> {
        let receipt = decode_groth16_receipt(&receipt)?;
        push_r0_groth16_proof(self.inner.builder_mut()?, receipt).map_err(zk_err)?;
        Ok(())
    }

    /// Append the r0-over-groth16 verifier fragment for the given 32-byte image
    /// id. Expects `[..., journal_hash, compressed_proof]` on the stack.
    ///
    /// Args:
    ///     image_id: The 32-byte RISC Zero image id, as hex, bytes, or list.
    ///
    /// Raises:
    ///     ZkError: If `image_id` is malformed or the fragment cannot be appended.
    pub fn append_r0_groth16_verifier(&mut self, image_id: PyBinary) -> PyResult<()> {
        let image_id = into_array_32(image_id.into(), "image_id")?;
        append_r0_groth16_verifier(self.inner.builder_mut()?, image_id).map_err(zk_err)?;
        Ok(())
    }

    /// Append a fixed-journal r0-over-groth16 verifier fragment, binding
    /// verification to `journal_hash` baked into the script. Expects only
    /// `[..., compressed_proof]` on the stack.
    ///
    /// Args:
    ///     image_id: The 32-byte RISC Zero image id, as hex, bytes, or list.
    ///     journal_hash: The 32-byte journal hash, as hex, bytes, or list.
    ///
    /// Raises:
    ///     ZkError: If an argument is malformed or the fragment cannot be appended.
    pub fn append_r0_groth16_verifier_with_fixed_journal(
        &mut self,
        image_id: PyBinary,
        journal_hash: PyBinary,
    ) -> PyResult<()> {
        let image_id = into_array_32(image_id.into(), "image_id")?;
        let journal_hash = into_array_32(journal_hash.into(), "journal_hash")?;
        append_r0_groth16_verifier_with_fixed_journal(
            self.inner.builder_mut()?,
            image_id,
            journal_hash,
        )
        .map_err(zk_err)?;
        Ok(())
    }

    /// Decode a succinct receipt and push its witness material (claim, control
    /// index, control digests, seal) onto the builder.
    ///
    /// The caller-owned `journal` is pushed afterwards (on top) to form the
    /// verifier's pre-stack.
    ///
    /// Args:
    ///     receipt: A borsh-encoded `SuccinctReceipt<ReceiptClaim>`, as hex,
    ///         bytes, or list.
    ///
    /// Raises:
    ///     ZkError: If the receipt cannot be decoded or the witness cannot be added.
    pub fn push_r0_succinct_witness(&mut self, receipt: PyBinary) -> PyResult<()> {
        let receipt = decode_succinct_receipt(&receipt)?;
        push_r0_succinct_witness(self.inner.builder_mut()?, receipt).map_err(zk_err)?;
        Ok(())
    }

    /// Append the r0 succinct verifier fragment for the given image id, control
    /// id and hash function. Expects
    /// `[..., claim, control_index, control_digests, seal, journal]`.
    ///
    /// Args:
    ///     image_id: The 32-byte RISC Zero image id, as hex, bytes, or list.
    ///     control_id: The 32-byte control id, as hex, bytes, or list.
    ///     hash_fn_id: The hash function id; currently only "poseidon2" (also
    ///         the default when omitted).
    ///
    /// Raises:
    ///     ZkError: If an argument is malformed or the fragment cannot be appended.
    #[pyo3(signature = (image_id, control_id, hash_fn_id=None))]
    pub fn append_r0_succinct_verifier(
        &mut self,
        image_id: PyBinary,
        control_id: PyBinary,
        hash_fn_id: Option<String>,
    ) -> PyResult<()> {
        let image_id = into_array_32(image_id.into(), "image_id")?;
        let control_id = into_array_32(control_id.into(), "control_id")?;
        let hash_fn_id = decode_hash_fn_id(hash_fn_id)?;
        append_r0_succinct_verifier(self.inner.builder_mut()?, image_id, control_id, hash_fn_id)
            .map_err(zk_err)?;
        Ok(())
    }

    /// Append a fixed-journal r0 succinct verifier fragment, binding
    /// verification to `journal` baked into the script. Expects only
    /// `[..., claim, control_index, control_digests, seal]`.
    ///
    /// Args:
    ///     image_id: The 32-byte RISC Zero image id, as hex, bytes, or list.
    ///     control_id: The 32-byte control id, as hex, bytes, or list.
    ///     hash_fn_id: The hash function id; currently only "poseidon2" (also
    ///         the default when omitted).
    ///     journal: The 32-byte journal, as hex, bytes, or list.
    ///
    /// Raises:
    ///     ZkError: If an argument is malformed or the fragment cannot be appended.
    #[pyo3(signature = (image_id, control_id, hash_fn_id=None, *, journal))]
    pub fn append_r0_succinct_verifier_with_fixed_journal(
        &mut self,
        image_id: PyBinary,
        control_id: PyBinary,
        hash_fn_id: Option<String>,
        journal: PyBinary,
    ) -> PyResult<()> {
        let image_id = into_array_32(image_id.into(), "image_id")?;
        let control_id = into_array_32(control_id.into(), "control_id")?;
        let hash_fn_id = decode_hash_fn_id(hash_fn_id)?;
        let journal = into_array_32(journal.into(), "journal")?;
        append_r0_succinct_verifier_with_fixed_journal(
            self.inner.builder_mut()?,
            image_id,
            control_id,
            hash_fn_id,
            journal,
        )
        .map_err(zk_err)?;
        Ok(())
    }

    /// Finalize a Groth16-bounded builder with a receipt and journal hash.
    ///
    /// Consumes the builder and returns the spending script and inner redeem
    /// script, ready to unlock a zk-locked UTXO.
    ///
    /// Args:
    ///     receipt: A borsh-encoded `Groth16Receipt<ReceiptClaim>`, as hex,
    ///         bytes, or list.
    ///     journal_hash: The 32-byte journal hash, as hex, bytes, or list.
    ///
    /// Returns:
    ///     FinalizedR0Script: The sig script and redeem script.
    ///
    /// Raises:
    ///     ZkError: If the builder is not Groth16-bounded, the receipt cannot be
    ///         decoded, or `journal_hash` is malformed.
    pub fn finalize_with_groth16_proof(
        &mut self,
        receipt: PyBinary,
        journal_hash: PyBinary,
    ) -> PyResult<PyFinalizedR0Script> {
        let receipt = decode_groth16_receipt(&receipt)?;
        let journal_hash = into_array_32(journal_hash.into(), "journal_hash")?;
        match self.take() {
            InnerState::BoundedGroth16(b) => {
                let finalized = b
                    .finalize_with_proof(receipt, journal_hash)
                    .map_err(zk_err)?;
                Ok(finalized.into())
            }
            other => {
                self.inner = other;
                Err(PyZkError::new_err(
                    "finalize_with_groth16_proof requires a Groth16-bounded builder",
                ))
            }
        }
    }

    /// Finalize a succinct-bounded builder with a receipt and journal digest.
    ///
    /// Consumes the builder and returns the spending script and inner redeem
    /// script, ready to unlock a zk-locked UTXO.
    ///
    /// Args:
    ///     receipt: A borsh-encoded `SuccinctReceipt<ReceiptClaim>`, as hex,
    ///         bytes, or list.
    ///     journal: The 32-byte journal digest, as hex, bytes, or list.
    ///
    /// Returns:
    ///     FinalizedR0Script: The sig script and redeem script.
    ///
    /// Raises:
    ///     ZkError: If the builder is not succinct-bounded, the receipt cannot
    ///         be decoded, or `journal` is malformed.
    pub fn finalize_with_succinct_proof(
        &mut self,
        receipt: PyBinary,
        journal: PyBinary,
    ) -> PyResult<PyFinalizedR0Script> {
        let receipt = decode_succinct_receipt(&receipt)?;
        let journal_bytes: Vec<u8> = journal.into();
        let journal_digest: Digest = journal_bytes
            .as_slice()
            .try_into()
            .map_err(|_| PyZkError::new_err("journal must be 32 bytes"))?;
        match self.take() {
            InnerState::BoundedSuccinct(b) => {
                let finalized = b
                    .finalize_with_proof(receipt, journal_digest)
                    .map_err(zk_err)?;
                Ok(finalized.into())
            }
            other => {
                self.inner = other;
                Err(PyZkError::new_err(
                    "finalize_with_succinct_proof requires a succinct-bounded builder",
                ))
            }
        }
    }

    /// The detailed string representation.
    ///
    /// Returns:
    ///     str: The ZkScriptBuilder as a repr string.
    fn __repr__(&self) -> String {
        let state = match self.inner {
            InnerState::Unbounded(_) => "unbounded",
            InnerState::BoundedGroth16(_) => "groth16",
            InnerState::BoundedSuccinct(_) => "succinct",
            InnerState::Taken => "consumed",
        };
        format!(
            "ZkScriptBuilder(state='{}', script={} bytes)",
            state,
            self.inner.script().len()
        )
    }
}
