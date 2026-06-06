"""
Forced-Recipient Covenant — Kaspa KIP-17 Example
=================================================
A UTXO whose redeem script enforces that it can *only* be spent to a single
predetermined address.

AI disclaimer - Credit to Claude for large parts of this example :)
We just provided the building blocks.
"""

import asyncio
import os

from kaspa import (
    Hash,
    Keypair,
    Opcodes,
    PrivateKey,
    RpcClient,
    ScriptBuilder,
    Transaction,
    TransactionInput,
    TransactionOutpoint,
    TransactionOutput,
    UtxoEntryReference,
    address_from_script_public_key,
    calculate_transaction_mass,
    create_input_signature,
    pay_to_address_script,
    pay_to_script_hash_signature_script,
    sign_transaction,
)

# =============================================================================
# Configuration
# =============================================================================

NETWORK_ID = "testnet-12"
NETWORK_TYPE = "testnet"
RPC_URL = os.environ.get("KASPA_RPC_URL")
SUBNETWORK_ID = bytes(20)


# =============================================================================
# Covenant Script Construction
# =============================================================================

def build_covenant_redeem_script(recipient_spk, owner_xonly_pubkey_hex: str) -> ScriptBuilder:
    """
    Build the P2SH redeem script that forces spending to a single recipient.

    Execution trace when spending (initial stack: [<sig>]):

        OpTxOutputCount       push number of outputs    → [sig, N]
        OpTrue                push 1                    → [sig, N, 1]
        OpEqualVerify         assert N == 1             → [sig]
        OpFalse               push 0 (output index)     → [sig, 0]
        OpTxOutputSpk         push output[0].spk        → [sig, spk0]
        <recipient_spk_bytes> push hardcoded SPK        → [sig, spk0, rec_spk]
        OpEqualVerify         assert spk0 == rec_spk    → [sig]
        <owner_pubkey>        push 32-byte x-only key   → [sig, pubkey]
        OpCheckSig            verify Schnorr sig        → [true]
    """
    # OpTxOutputSpk pushes version (2 bytes, big-endian) + script bytes.
    # bytes(spk) returns only the script; prepend the version to match.
    spk_bytes = recipient_spk.version.to_bytes(2, 'big') + bytes(recipient_spk)

    return (
        ScriptBuilder()
        .add_op(Opcodes.OpTxOutputCount)
        .add_op(Opcodes.OpTrue)
        .add_op(Opcodes.OpEqualVerify)
        .add_op(Opcodes.OpFalse)
        .add_op(Opcodes.OpTxOutputSpk)
        .add_data(spk_bytes)
        .add_op(Opcodes.OpEqualVerify)
        .add_data(bytes.fromhex(owner_xonly_pubkey_hex))
        .add_op(Opcodes.OpCheckSig)
    )


# =============================================================================
# Genesis — send funds into the covenant P2SH UTXO
# =============================================================================

async def genesis(client: RpcClient, owner_key: PrivateKey, funding_utxos: list):
    """
    Create and broadcast the genesis transaction.

    Moves funds from the owner's regular P2PK address into a covenant
    P2SH UTXO whose redeem script enforces a single spending destination.
    """
    keypair = Keypair.from_private_key(owner_key)
    owner_pubkey_hex = keypair.xonly_public_key

    # In this demo the recipient is the same keypair (self-contained).
    # In a real use-case this is likely an independent (pre-agreed) address.
    recipient_address = keypair.to_address(NETWORK_TYPE)
    recipient_spk = pay_to_address_script(recipient_address)

    redeem_script = build_covenant_redeem_script(recipient_spk, owner_pubkey_hex)
    covenant_spk = redeem_script.create_pay_to_script_hash_script()
    covenant_address = address_from_script_public_key(covenant_spk, NETWORK_TYPE)
    print(f"  Covenant P2SH address : {covenant_address.to_string()}")

    # Use the largest funding UTXO
    funding = max(funding_utxos, key=lambda u: u["utxoEntry"]["amount"])
    funding_amount = funding["utxoEntry"]["amount"]
    funding_utxo_ref = UtxoEntryReference.from_dict(funding)
    funding_outpoint = TransactionOutpoint(
        Hash(funding["outpoint"]["transactionId"]),
        funding["outpoint"]["index"],
    )

    # ── Mass / fee calculation ──────────────────────────────────────────────
    ph_input = TransactionInput(funding_outpoint, b"", 0, 1, funding_utxo_ref)
    ph_output = TransactionOutput(funding_amount, covenant_spk)
    ph_tx = Transaction(0, [ph_input], [ph_output], 0, SUBNETWORK_ID, 0, b"", 0)
    mass = calculate_transaction_mass(NETWORK_ID, ph_tx)
    fee_rates = await client.get_fee_estimate()
    fee = mass * int(fee_rates["estimate"]["priorityBucket"]["feerate"])
    covenant_amount = funding_amount - fee

    # ── Build & sign the genesis transaction ────────────────────────────────
    inp = TransactionInput(funding_outpoint, b"", 0, 1, funding_utxo_ref)
    out = TransactionOutput(covenant_amount, covenant_spk)
    tx = Transaction(0, [inp], [out], 0, SUBNETWORK_ID, 0, b"", mass)

    # sign_transaction handles standard P2PK inputs automatically
    signed_tx = sign_transaction(tx, [owner_key], True)

    print(f"  Fee: {fee} sompi")
    print(f" Covenant amount: {covenant_amount} sompi")

    result = await client.submit_transaction({
        "transaction": signed_tx,
        "allowOrphan": False,
    })
    txid = result["transactionId"]
    print(f"  Genesis TXID: {txid}")

    outpoint = {"transactionId": txid, "index": 0}
    return txid, outpoint, covenant_amount, redeem_script, covenant_spk


# =============================================================================
# Spend — demonstrate that the covenant enforces the recipient
# =============================================================================

async def spend(
    client: RpcClient,
    owner_key: PrivateKey,
    covenant_outpoint: dict,
    covenant_amount: int,
    redeem_script: ScriptBuilder,
    covenant_spk,
) -> str:
    """
    Spend the covenant UTXO.  The script forces all funds to the recipient
    encoded in the redeem script — any other destination fails script evaluation.

    Steps:
      1. Build the spending transaction with an empty signature script
      2. Sign with create_input_signature (P2SH Schnorr)
      3. Build the P2SH unlocking script: <sig_push> <redeem_script_push>
      4. Submit
    """
    keypair = Keypair.from_private_key(owner_key)
    recipient_address = keypair.to_address(NETWORK_TYPE)
    recipient_spk = pay_to_address_script(recipient_address)
    covenant_address = address_from_script_public_key(covenant_spk, NETWORK_TYPE)

    # Build a UTXO reference for the covenant input
    cov_utxo_ref = UtxoEntryReference.from_dict({
        "address": covenant_address.to_string(),
        "outpoint": {
            "transactionId": covenant_outpoint["transactionId"],
            "index": covenant_outpoint["index"],
        },
        "utxoEntry": {
            "amount": covenant_amount,
            "scriptPublicKey": {"version": 0, "script": covenant_spk.script},
            "blockDaaScore": 0,
            "isCoinbase": False,
            "covenantId": None,
        },
    })
    cov_outpoint = TransactionOutpoint(
        Hash(covenant_outpoint["transactionId"]),
        covenant_outpoint["index"],
    )

    # ── Mass / fee calculation ──────────────────────────────────────────────
    ph_input = TransactionInput(cov_outpoint, b"", 0, 1, cov_utxo_ref)
    ph_output = TransactionOutput(covenant_amount, recipient_spk)
    ph_tx = Transaction(0, [ph_input], [ph_output], 0, SUBNETWORK_ID, 0, b"", 0)
    mass = calculate_transaction_mass(NETWORK_ID, ph_tx)
    fee_rates = await client.get_fee_estimate()
    fee = mass * int(fee_rates["estimate"]["priorityBucket"]["feerate"])
    spend_amount = covenant_amount - fee

    # ── Build the unsigned transaction ─────────────────────────────────────
    # (empty sig script — the sighash does not commit to the sig script)
    inp = TransactionInput(cov_outpoint, b"", 0, 1, cov_utxo_ref)
    out = TransactionOutput(spend_amount, recipient_spk)
    tx_unsigned = Transaction(0, [inp], [out], 0, SUBNETWORK_ID, 0, b"", mass)

    # ── Create the P2SH signature ───────────────────────────────────────────
    # create_input_signature returns hex of [OP_DATA65, 64_sig_bytes, sighash_type]
    # pass the full bytes directly to pay_to_script_hash_signature_script
    sig_hex = create_input_signature(tx_unsigned, 0, owner_key)
    sig_bytes = bytes.fromhex(sig_hex)               # full 66 bytes including push opcode
    redeem_bytes = bytes.fromhex(redeem_script.to_string())
    unlock_script_hex = pay_to_script_hash_signature_script(redeem_bytes, sig_bytes)

    # ── Build and submit the final signed transaction ───────────────────────
    signed_inp = TransactionInput(
        cov_outpoint, bytes.fromhex(unlock_script_hex), 0, 1, cov_utxo_ref
    )
    tx = Transaction(0, [signed_inp], [out], 0, SUBNETWORK_ID, 0, b"", mass)

    print(f"  Fee: {fee} sompi")
    print(f"  Spend amount: {spend_amount} sompi")

    result = await client.submit_transaction({
        "transaction": tx,
        "allowOrphan": False,
    })
    txid = result["transactionId"]
    print(f"  Spend TXID: {txid}")
    return txid


# =============================================================================
# Helpers — UTXO subscription and confirmation waiting
# =============================================================================

async def wait_for_utxos(client: RpcClient, address) -> list:
    """Poll every 5 s until at least one UTXO exists at `address`."""
    result = await client.get_utxos_by_addresses({"addresses": [address]})
    entries = result.get("entries", [])
    if entries:
        return entries

    print(f"  Waiting for funds — send KAS to:\n  {address.to_string()}\n")
    while True:
        await asyncio.sleep(5)
        result = await client.get_utxos_by_addresses({"addresses": [address]})
        entries = result.get("entries", [])
        if entries:
            return entries


async def wait_for_confirmation(client: RpcClient, txid: str):
    """Poll the mempool every 1 s; once the tx leaves it has been accepted."""
    print(f"  Waiting for confirmation of {txid}")
    while True:
        await asyncio.sleep(1)
        try:
            await client.get_mempool_entry({"transactionId": txid, "includeOrphanPool": True})
        except Exception:
            print("  Confirmed!")
            return


# =============================================================================
# Main
# =============================================================================

async def main():
    keypair = Keypair.random()
    owner_key = PrivateKey(keypair.private_key)
    funding_address = keypair.to_address(NETWORK_TYPE)

    print("=" * 60)
    print("Forced-Recipient Covenant  —  KIP-17 Demo")
    print("=" * 60)
    print(f"\nFund this testnet-12 address (owner key):")
    print(f"  {funding_address.to_string()}")
    print("\nThe script detects incoming funds automatically.\n")

    client = RpcClient(url=RPC_URL, network_id=NETWORK_ID)
    await client.connect()
    print(f"Connected to {NETWORK_ID}\n")

    # ── Wait for funding ─────────────────────────────────────────────────────
    utxos = await wait_for_utxos(client, funding_address)
    total = sum(u["utxoEntry"]["amount"] for u in utxos)
    print(f"Received {total} sompi across {len(utxos)} UTXO(s)\n")

    # ── Step 1: Genesis (lock funds into covenant P2SH) ──────────────────────
    print("[Step 1/2] Broadcasting genesis transaction…")
    txid, outpoint, covenant_amount, redeem_script, covenant_spk = await genesis(
        client, owner_key, utxos
    )
    await wait_for_confirmation(client, txid)
    print()

    # ── Step 2: Spend to incorrect address ───────────────────────────────────
    # ── (recipient enforced by the covenant script, this will reject) ────────
    print("[Step 2/3] Broadcasting spend transaction to INCORRECT address…")
    print("  Will reject, destination address does not match script enforced address.")
    try:
        spend_txid = await spend(
            client,
            PrivateKey(Keypair.random().private_key),
            outpoint,
            covenant_amount,
            redeem_script,
            covenant_spk
        )
    except Exception as e:
        print(f"  Transaction properly rejected with error: {e}")
    print()

    # ── Step 3: Spend to correct address ─────────────────────────────────────
    # ── (recipient enforced by the covenant script, this will succeed) ───────
    print("[Step 3/3] Broadcasting spend transaction to CORRECT address…")
    print("  Will succeed, destination address matches script enforced address.")
    spend_txid = await spend(
        client, owner_key, outpoint, covenant_amount, redeem_script, covenant_spk
    )
    print()

    print("=" * 60)
    print("Demo complete!")
    print(f"  Genesis TXID : {txid}")
    print(f"  Spend TXID   : {spend_txid}")
    print("=" * 60)

    await client.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
