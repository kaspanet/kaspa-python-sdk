use crate::crypto::txscript::builder::{build_engine_flags, warn_if_covenants_enabled_passed};
use crate::crypto::txscript::zk_sdk::result::PyFinalizedR0Script;
use crate::crypto::txscript::zk_sdk::utils::{
    PyZkError, decode_groth16_receipt, into_array_32, zk_err,
};
use crate::types::PyBinary;
use kaspa_txscript::EngineFlags;
use kaspa_txscript::script_builder::ScriptBuilder;
use kaspa_txscript_zk_sdk::{
    append_r0_groth16_verifier, append_r0_groth16_verifier_dynamic_image_id,
    append_r0_groth16_verifier_with_fixed_journal, push_r0_groth16_proof,
};
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use workflow_core::hex::ToHex;

/// Runtime mirror of the native compile-time type-state. FFI cannot carry the
/// native crate's `PhantomData` type-state, so the `Unbounded` →
/// `BoundedGroth16` → finalized transitions are checked at runtime and the
/// wrong call raises `ZkError`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ZkState {
    Unbounded,
    BoundedGroth16,
    Consumed,
}

/// A staged builder for RISC Zero zk-to-script locking scripts.
///
/// Build flow:
///     1. `ZkScriptBuilder.new_r0(...)` — unbounded.
///     2. `commit_to_groth16(image_id)` — bounded.
///     3. `finalize_with_groth16_proof(receipt, journal_hash)` — a FinalizedR0Script.
///
/// Only RISC Zero Groth16 receipts are supported. The succinct (STARK) path was
/// removed from rusty-kaspa's ZK SDK following RISC Zero advisory
/// GHSA-g6qg-9j8p-7j87.
///
/// Calling a method in the wrong state raises `ZkError`. The lower-level
/// fragment methods (`add_data`, `push_*`, `append_*`) operate on the builder in
/// any state, for composing scripts by hand.
#[gen_stub_pyclass]
#[pyclass(name = "ZkScriptBuilder")]
pub struct PyZkScriptBuilder {
    state: ZkState,
    // `ScriptBuilder` has no flags getter, and finalize needs a scratch
    // builder with the same flags — so keep our own copy.
    flags: EngineFlags,
    builder: ScriptBuilder,
}

// The state transitions and finalize recipes below mirror the native
// type-state `ZkScriptBuilder`'s methods, but call the same public fragment
// functions on the inner `ScriptBuilder` in place, flipping `state` only on
// success. The native methods consume the builder and errors do not return it,
// so going through them would destroy the builder on any failure; this way a
// failed call leaves the state and script untouched.
impl PyZkScriptBuilder {
    fn expect_state(&self, expected: ZkState, message: &str) -> PyResult<()> {
        if self.state == expected {
            Ok(())
        } else {
            Err(PyZkError::new_err(message.to_string()))
        }
    }

    fn builder_mut(&mut self) -> PyResult<&mut ScriptBuilder> {
        match self.state {
            ZkState::Consumed => Err(PyZkError::new_err("builder has been consumed")),
            _ => Ok(&mut self.builder),
        }
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
    ///     covenants_enabled: Deprecated and ignored. Covenant opcodes and
    ///         their script limits are always enabled. Passing a value emits
    ///         a DeprecationWarning; the parameter will be removed in a
    ///         future major version.
    ///     sigop_script_units: Script units charged per signature operation.
    ///         Defaults to the native engine default when omitted.
    ///
    /// Returns:
    ///     ZkScriptBuilder: A new unbounded builder.
    #[staticmethod]
    #[pyo3(signature = (covenants_enabled=None, sigop_script_units=None))]
    pub fn new_r0(
        py: Python<'_>,
        covenants_enabled: Option<bool>,
        sigop_script_units: Option<u64>,
    ) -> PyResult<Self> {
        warn_if_covenants_enabled_passed(py, covenants_enabled)?;
        let flags = build_engine_flags(sigop_script_units);
        Ok(Self {
            state: ZkState::Unbounded,
            flags,
            builder: ScriptBuilder::with_flags(flags),
        })
    }

    /// The current script bytes as a hex string.
    ///
    /// Returns:
    ///     str: The script built so far, hex-encoded.
    pub fn script(&self) -> String {
        self.builder.script().to_hex()
    }

    /// Drain the builder: return the accumulated script bytes and consume the
    /// builder. Matching the WASM SDK (and the native builder's move
    /// semantics) — and unlike `ScriptBuilder.drain` — the builder is not
    /// reusable afterwards: subsequent mutating calls raise `ZkError`.
    ///
    /// Returns:
    ///     str: The script bytes, hex-encoded.
    pub fn drain(&mut self) -> String {
        self.state = ZkState::Consumed;
        self.builder.drain().to_hex()
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
        self.builder_mut()?
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
        self.expect_state(
            ZkState::Unbounded,
            "commit_to_groth16 requires an unbounded builder",
        )?;
        let snapshot = self.builder.script().to_vec();
        if let Err(e) = append_r0_groth16_verifier(&mut self.builder, image_id) {
            *self.builder.script_mut() = snapshot;
            return Err(zk_err(e));
        }
        self.state = ZkState::BoundedGroth16;
        Ok(())
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
        push_r0_groth16_proof(self.builder_mut()?, receipt).map_err(zk_err)?;
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
        append_r0_groth16_verifier(self.builder_mut()?, image_id).map_err(zk_err)?;
        Ok(())
    }

    /// Append the r0-over-groth16 verifier fragment that takes the image id
    /// from the stack instead of baking it into the script. Expects
    /// `[..., journal_hash, compressed_proof, image_id]` on the stack.
    ///
    /// `append_r0_groth16_verifier(image_id)` is equivalent to pushing
    /// `image_id` with `add_data` and then calling this.
    ///
    /// Raises:
    ///     ZkError: If the fragment cannot be appended (or the builder is consumed).
    pub fn append_r0_groth16_verifier_dynamic_image_id(&mut self) -> PyResult<()> {
        append_r0_groth16_verifier_dynamic_image_id(self.builder_mut()?).map_err(zk_err)?;
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
        append_r0_groth16_verifier_with_fixed_journal(self.builder_mut()?, image_id, journal_hash)
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
        self.expect_state(
            ZkState::BoundedGroth16,
            "finalize_with_groth16_proof requires a Groth16-bounded builder",
        )?;
        // Same recipe as the native `finalize_with_proof`: journal hash, then
        // the compressed proof, then the redeem script — built in a scratch
        // builder so a failure leaves this builder intact.
        let redeem_script = self.builder.script().to_vec();
        let mut sig = ScriptBuilder::with_flags(self.flags);
        sig.add_data(&journal_hash).map_err(zk_err)?;
        push_r0_groth16_proof(&mut sig, receipt).map_err(zk_err)?;
        sig.add_data(&redeem_script).map_err(zk_err)?;
        self.state = ZkState::Consumed;
        self.builder.drain();
        Ok(PyFinalizedR0Script::new(sig.drain(), redeem_script))
    }

    /// The detailed string representation.
    ///
    /// Returns:
    ///     str: The ZkScriptBuilder as a repr string.
    fn __repr__(&self) -> String {
        let state = match self.state {
            ZkState::Unbounded => "unbounded",
            ZkState::BoundedGroth16 => "groth16",
            ZkState::Consumed => "consumed",
        };
        format!(
            "ZkScriptBuilder(state='{}', script={} bytes)",
            state,
            self.builder.script().len()
        )
    }
}
