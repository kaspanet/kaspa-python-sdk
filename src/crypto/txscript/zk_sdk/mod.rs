//! Python bindings for the rusty-kaspa ZK SDK (`kaspa-txscript-zk-sdk`).
//!
//! Mirrors the WASM `ZkScriptBuilder` surface: a staged builder that turns
//! RISC Zero zero-knowledge proofs (Groth16 and Succinct/STARK receipts) into
//! Kaspa transaction scripts. It produces the P2SH `redeem_script` that locks a
//! UTXO behind on-chain verification of a specific proof (by `image_id`, and
//! optionally a fixed `journal`), and the `sig_script` that unlocks it.
//!
//! Everything here is registered on the top-level `kaspa` module; the `ZkError`
//! exception lives in `kaspa.exceptions`.
//!
//! The on-chain `OpZkPrecompile` opcode emitted by these scripts is gated by the
//! same Toccata activation as the covenant opcodes, so build with
//! `covenants_enabled=True` and spend on a network where Toccata is active.

pub mod builder;
pub mod result;
pub mod utils;
