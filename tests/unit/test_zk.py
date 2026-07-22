"""Unit tests for the RISC Zero zk-to-script bindings in the `kaspa` module.

These are offline tests: they exercise the builder state machine, receipt
decoding, and script assembly, and assert that non-empty scripts come out. They
do not run the txscript engine — on-chain verification of a produced script is
covered by rusty-kaspa's own engine tests and by the fully on-chain example
`examples/zk/groth16_onchain.py`.

Fixtures (`tests/data/zk/`) are the borsh-encoded receipts vendored from
rusty-kaspa's `crypto/txscript/zk-sdk/tests/data/zk_builder_tests/`. The Groth16
image id / journal hash below are the matching public inputs for `groth.rcpt.hex`.
"""

import pathlib

import pytest

from kaspa import (
    FinalizedR0Script,
    R0SuccinctWitnessParts,
    ZkScriptBuilder,
    prepare_r0_groth16_proof,
    prepare_r0_succinct_witness,
)
from kaspa.exceptions import ZkError

DATA = pathlib.Path(__file__).resolve().parent.parent / "data" / "zk"

# Public inputs matching groth.rcpt.hex (from rusty-kaspa's zk-sdk fixtures).
GROTH16_IMAGE_ID = "75641a540ee2ad9ee5902bcdcdb8b55c0bef4a28287309b858f97b1356c6c2e0"
GROTH16_JOURNAL_HASH = "5df6e0e2761359d30a8275058e299fcc0381534545f55cf43e41983f5d4c9456"

# Public inputs matching succinct.rcpt.hex.
SUCCINCT_JOURNAL = "5df6e0e2761359d30a8275058e299fcc0381534545f55cf43e41983f5d4c9456"


def groth_receipt() -> str:
    return (DATA / "groth.rcpt.hex").read_text().strip()


def succinct_receipt() -> str:
    return (DATA / "succinct.rcpt.hex").read_text().strip()


def is_hex(s: str) -> bool:
    """Non-empty, even-length, all-hex-digit string."""
    if not s or len(s) % 2 != 0:
        return False
    try:
        bytes.fromhex(s)
    except ValueError:
        return False
    return True


# -----------------------------------------------------------------------------
# Groth16 high-level flow
# -----------------------------------------------------------------------------

def test_new_r0_starts_unbounded_and_empty():
    b = ZkScriptBuilder.new_r0(covenants_enabled=True)
    assert b.script() == ""
    assert "unbounded" in repr(b)


def test_groth16_full_flow():
    b = ZkScriptBuilder.new_r0(covenants_enabled=True)
    b.commit_to_groth16(GROTH16_IMAGE_ID)

    # After commit, script() is the redeem (commit) script.
    redeem = b.script()
    assert is_hex(redeem)
    assert "groth16" in repr(b)

    finalized = b.finalize_with_groth16_proof(groth_receipt(), GROTH16_JOURNAL_HASH)
    assert isinstance(finalized, FinalizedR0Script)
    assert is_hex(finalized.sig_script)
    assert is_hex(finalized.redeem_script)

    # The redeem script from finalize matches the one observed after commit, and
    # the sig script embeds both the redeem script and the compressed proof.
    assert finalized.redeem_script == redeem
    assert redeem in finalized.sig_script


def test_finalize_consumes_builder():
    b = ZkScriptBuilder.new_r0(covenants_enabled=True)
    b.commit_to_groth16(GROTH16_IMAGE_ID)
    b.finalize_with_groth16_proof(groth_receipt(), GROTH16_JOURNAL_HASH)

    assert "consumed" in repr(b)
    assert b.script() == ""
    # A consumed builder can't be finalized again.
    with pytest.raises(ZkError):
        b.finalize_with_groth16_proof(groth_receipt(), GROTH16_JOURNAL_HASH)


def test_prepare_r0_groth16_proof_round_trip():
    proof = prepare_r0_groth16_proof(groth_receipt())
    assert is_hex(proof)

    # The standalone prepare output is exactly what finalize pushes into the
    # sig script.
    b = ZkScriptBuilder.new_r0(covenants_enabled=True)
    b.commit_to_groth16(GROTH16_IMAGE_ID)
    finalized = b.finalize_with_groth16_proof(groth_receipt(), GROTH16_JOURNAL_HASH)
    assert proof in finalized.sig_script


# -----------------------------------------------------------------------------
# Fragment / fixed-journal building
# -----------------------------------------------------------------------------

def test_fixed_journal_fragment_builds():
    # Compose a redeem script by hand using the fixed-journal verifier fragment.
    b = ZkScriptBuilder.new_r0(covenants_enabled=True)
    b.append_r0_groth16_verifier_with_fixed_journal(GROTH16_IMAGE_ID, GROTH16_JOURNAL_HASH)
    assert is_hex(b.script())


def test_add_data_appends():
    b = ZkScriptBuilder.new_r0(covenants_enabled=True)
    b.add_data("deadbeef")
    script = b.script()
    assert is_hex(script)
    assert "deadbeef" in script


# -----------------------------------------------------------------------------
# Succinct receipt decoding
# -----------------------------------------------------------------------------

def test_prepare_r0_succinct_witness():
    parts = prepare_r0_succinct_witness(succinct_receipt())
    assert isinstance(parts, R0SuccinctWitnessParts)
    assert len(parts.claim) == 64          # 32-byte SHA-256 digest
    assert len(parts.control_index) == 8   # u32 little-endian
    assert is_hex(parts.control_digests)
    assert is_hex(parts.seal)


# -----------------------------------------------------------------------------
# Input flexibility (PyBinary: hex str, bytes, list[int])
# -----------------------------------------------------------------------------

def test_accepts_bytes_and_list_inputs():
    for image_id in (
        GROTH16_IMAGE_ID,
        bytes.fromhex(GROTH16_IMAGE_ID),
        list(bytes.fromhex(GROTH16_IMAGE_ID)),
    ):
        b = ZkScriptBuilder.new_r0(covenants_enabled=True)
        b.commit_to_groth16(image_id)
        assert is_hex(b.script())


# -----------------------------------------------------------------------------
# Error / wrong-state handling
# -----------------------------------------------------------------------------

def test_finalize_before_commit_raises():
    b = ZkScriptBuilder.new_r0(covenants_enabled=True)
    with pytest.raises(ZkError):
        b.finalize_with_groth16_proof(groth_receipt(), GROTH16_JOURNAL_HASH)


def test_double_commit_raises():
    b = ZkScriptBuilder.new_r0(covenants_enabled=True)
    b.commit_to_groth16(GROTH16_IMAGE_ID)
    with pytest.raises(ZkError):
        b.commit_to_groth16(GROTH16_IMAGE_ID)


def test_succinct_finalize_on_groth16_bounded_raises():
    b = ZkScriptBuilder.new_r0(covenants_enabled=True)
    b.commit_to_groth16(GROTH16_IMAGE_ID)
    with pytest.raises(ZkError):
        b.finalize_with_succinct_proof(succinct_receipt(), SUCCINCT_JOURNAL)


def test_bad_image_id_length_raises():
    b = ZkScriptBuilder.new_r0(covenants_enabled=True)
    with pytest.raises(ZkError):
        b.commit_to_groth16("00" * 31)  # 31 bytes, must be 32


def test_bad_journal_hash_length_raises():
    b = ZkScriptBuilder.new_r0(covenants_enabled=True)
    b.commit_to_groth16(GROTH16_IMAGE_ID)
    with pytest.raises(ZkError):
        b.finalize_with_groth16_proof(groth_receipt(), "00" * 16)  # 16 bytes


def test_bad_receipt_raises():
    b = ZkScriptBuilder.new_r0(covenants_enabled=True)
    b.commit_to_groth16(GROTH16_IMAGE_ID)
    with pytest.raises(ZkError):
        b.finalize_with_groth16_proof("deadbeef", GROTH16_JOURNAL_HASH)


def test_failed_finalize_preserves_builder():
    # Pre-Toccata limits reject the oversized proof push, so finalize fails —
    # the builder must survive with its state and committed script intact
    # instead of being consumed.
    b = ZkScriptBuilder.new_r0(covenants_enabled=False)
    b.commit_to_groth16(GROTH16_IMAGE_ID)
    script_before = b.script()
    with pytest.raises(ZkError):
        b.finalize_with_groth16_proof(groth_receipt(), GROTH16_JOURNAL_HASH)
    assert b.script() == script_before
    assert "state='groth16'" in repr(b)


def test_drain_returns_script_and_consumes_builder():
    # Matching the WASM SDK (and unlike ScriptBuilder.drain), drain returns
    # the script bytes and consumes the builder for good.
    b = ZkScriptBuilder.new_r0(covenants_enabled=True)
    b.commit_to_groth16(GROTH16_IMAGE_ID)
    committed = b.script()

    assert b.drain() == committed
    assert b.script() == ""
    assert "consumed" in repr(b)
    with pytest.raises(ZkError):
        b.add_data("deadbeef")


def test_bad_hash_fn_id_raises():
    b = ZkScriptBuilder.new_r0(covenants_enabled=True)
    with pytest.raises(ZkError):
        b.commit_to_succinct(GROTH16_IMAGE_ID, GROTH16_IMAGE_ID, "blake2b")
