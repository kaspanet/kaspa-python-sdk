"""
Populate Genesis Covenant Outputs — Kaspa KIP-17 Example
========================================================
Demonstrates populating genesis covenant bindings on transaction outputs,
converting to dict and back, and verifying the covenant data is preserved.
"""

from kaspa import (
    GenesisCovenantGroup,
    Keypair,
    Opcodes,
    ScriptBuilder,
    Transaction,
    TransactionInput,
    TransactionOutpoint,
    TransactionOutput,
    Hash,
    pay_to_address_script,
)

NETWORK_TYPE = "testnet"
SUBNETWORK_ID = bytes(20)


def main():
    # Generate two keypairs: one for the funding input, one as a covenant recipient
    funder = Keypair.random()
    recipient = Keypair.random()

    funder_address = funder.to_address(NETWORK_TYPE)
    recipient_address = recipient.to_address(NETWORK_TYPE)

    recipient_spk = pay_to_address_script(recipient_address)

    # Build a simple covenant redeem script (just owner pubkey + checksig for demo)
    redeem_script = (
        ScriptBuilder()
        .add_data(bytes.fromhex(funder.xonly_public_key))
        .add_op(Opcodes.OpCheckSig)
    )
    covenant_spk = redeem_script.create_pay_to_script_hash_script()

    # --- Build a transaction with multiple outputs ---
    dummy_txid = Hash("0" * 64)
    outpoint = TransactionOutpoint(dummy_txid, 0)
    inp = TransactionInput(outpoint, b"", 0, 1)

    # Three outputs: two covenant outputs, one regular change output
    out0 = TransactionOutput(1_000, covenant_spk)
    out1 = TransactionOutput(2_000, covenant_spk)
    out2 = TransactionOutput(500, recipient_spk)  # change / non-covenant

    tx = Transaction(0, [inp], [out0, out1, out2], 0, SUBNETWORK_ID, 0, b"", 0)

    # --- Verify outputs start without covenant bindings ---
    outputs_before = tx.outputs
    for i, out in enumerate(outputs_before):
        d = out.to_dict()
        assert d["covenant"] is None, f"Output {i} should have no covenant before populate"
    print("Before populate: all outputs have covenant = None")

    # --- Populate genesis covenants ---
    # Group: input 0 authorizes covenant outputs at indices 0 and 1
    group = GenesisCovenantGroup(authorizing_input=0, outputs=[0, 1])
    tx.populate_genesis_covenants([group])
    print("Called populate_genesis_covenants with group(input=0, outputs=[0, 1])")

    # --- Verify covenant bindings are now set ---
    outputs_after = tx.outputs
    for i, out in enumerate(outputs_after):
        d = out.to_dict()
        if i < 2:
            assert d["covenant"] is not None, f"Output {i} should have a covenant binding"
            print(f"  Output {i}: covenant = {d['covenant']}")
        else:
            assert d["covenant"] is None, f"Output {i} (change) should remain None"
            print(f"  Output {i}: covenant = None (change output, as expected)")

    # --- Round-trip: Transaction -> dict -> Transaction ---
    tx_dict = tx.to_dict()
    print(f"\nTransaction dict keys: {list(tx_dict.keys())}")

    tx_restored = Transaction.from_dict(tx_dict)

    # Verify outputs survived the round-trip
    restored_outputs = tx_restored.outputs
    assert len(restored_outputs) == 3, "Should have 3 outputs after round-trip"

    for i, out in enumerate(restored_outputs):
        d = out.to_dict()
        if i < 2:
            assert d["covenant"] is not None, f"Restored output {i} lost its covenant binding"
            print(f"  Restored output {i}: covenant = {d['covenant']}")
        else:
            assert d["covenant"] is None, f"Restored output {i} should remain None"
            print(f"  Restored output {i}: covenant = None (as expected)")

    # Verify equality between original and restored transaction
    assert tx == tx_restored, "Restored transaction should equal the original"
    print("\nRound-trip passed: original == restored")

    print("\nAll assertions passed!")


if __name__ == "__main__":
    main()
