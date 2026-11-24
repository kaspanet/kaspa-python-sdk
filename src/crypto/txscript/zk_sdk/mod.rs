//! Python bindings for the rusty-kaspa ZK SDK (`kaspa-txscript-zk-sdk`).
//!
//! Mirrors the WASM `ZkScriptBuilder` surface: a staged builder that turns
//! RISC Zero Groth16 zero-knowledge proofs into Kaspa transaction scripts. It
//! produces the P2SH `redeem_script` that locks a UTXO behind on-chain
//! verification of a specific proof (by `image_id`, and optionally a fixed
//! `journal`), and the `sig_script` that unlocks it.
//!
//! Succinct (STARK) receipts are no longer supported: rusty-kaspa removed that
//! path from the ZK SDK following RISC Zero advisory GHSA-g6qg-9j8p-7j87.
//!
//! Everything here is registered on the top-level `kaspa` module; the `ZkError`
//! exception lives in `kaspa.exceptions`.
//!
//! The on-chain `OpZkPrecompile` opcode emitted by these scripts arrived with
//! the Toccata hardfork alongside the covenant opcodes; both are now
//! unconditionally active on every network.

pub mod builder;
pub mod result;
pub mod utils;
