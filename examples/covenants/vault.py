"""
Smart Vault Covenant — Kaspa KIP-17 + KIP-20 Example
=====================================================
A vault UTXO with two spending paths:
  • Emergency sweep (any time): send all funds to a pre-committed recovery address.
  • Normal withdrawal (after locktime): owner may spend freely, but any remainder
    must continue the covenant (singleton pattern — exactly one vault UTXO at all times).

Uses KIP-20 covenant_id for stable vault lineage.  The singleton pattern means
the UTXO set always contains at most one UTXO with this covenant_id.
"""

import asyncio
import os

from kaspa import (
    CovenantBinding,
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
    covenant_id,
    create_input_signature,
    pay_to_address_script,
    sign_transaction,
)

# =============================================================================
# Configuration
# =============================================================================

NETWORK_ID = "testnet-12"
NETWORK_TYPE = "testnet"
RPC_URL = os.environ.get("KASPA_RPC_URL")
SUBNETWORK_ID = bytes(20)

# Locktime for the normal withdrawal path (DAA score).
# testnet-12 produces ~10 block/sec so 100 blocks = approx 10 seconds
LOCKTIME_BLOCKS = 100


# =============================================================================
# Covenant Script Construction
# =============================================================================

def build_vault_redeem_script(
    recovery_spk,
    recovery_xonly_pubkey_hex: str,
    owner_xonly_pubkey_hex: str,
    locktime: int,
) -> ScriptBuilder:
    """
    Build the vault P2SH redeem script.

    Branch selection: the integer on top of the stack when the redeem script
    begins execution selects the spending path:
      1 (OP_TRUE)  → Emergency sweep  (OpIf body)
      0 (OP_FALSE) → Normal withdrawal (OpElse body)

    ── Emergency branch ────────────────────────────────────────────────────────
    Initial stack coming into redeem script execution: [<recovery_sig>, 1]

        OpIf                               pop 1, enter emergency branch → [recovery_sig]
          OpFalse OpTxOutputSpk            push output[0].spk            → [sig, spk0]
          <recovery_spk_bytes>             push hardcoded recovery SPK   → [sig, spk0, rec_spk]
          OpEqualVerify                    assert spk0 == rec_spk        → [sig]
          OpTxOutputCount OpTrue           push N, push 1                → [sig, N, 1]
          OpEqualVerify                    assert N == 1                 → [sig]
          <recovery_pubkey> OpCheckSig     verify recovery sig           → [true]
        OpElse ... OpEndIf

    ── Normal withdrawal branch ────────────────────────────────────────────────
    Initial stack: [<owner_sig>, 0]

        OpIf ... OpElse
          <locktime> OpCheckLockTimeVerify OpDrop  enforce locktime      → [sig]
          OpTxInputIndex                   push current input index      → [sig, idx]
          OpAuthOutputCount                count authorized outputs      → [sig, count]
          OpTrue OpEqualVerify             assert count == 1             → [sig]
          <owner_pubkey> OpCheckSig        verify owner sig              → [true]
        OpEndIf

    KIP-17: OpTxOutputSpk, OpTxOutputCount, OpCheckLockTimeVerify
    KIP-20: OpAuthOutputCount
    """
    # OpTxOutputSpk pushes version (2 bytes, big-endian) + script bytes.
    # bytes(spk) returns only the script; prepend the version to match.
    recovery_spk_bytes = recovery_spk.version.to_bytes(2, 'big') + bytes(recovery_spk)

    return (
        ScriptBuilder()
        .add_op(Opcodes.OpIf)

        # ── Emergency path ───────────────────────────────────────────────
        .add_op(Opcodes.OpFalse)           # push 0 (output index 0)
        .add_op(Opcodes.OpTxOutputSpk)     # push output[0].spk
        .add_data(recovery_spk_bytes)      # push hardcoded recovery SPK
        .add_op(Opcodes.OpEqualVerify)     # assert: destination == recovery SPK
        .add_op(Opcodes.OpTxOutputCount)   # push total number of outputs
        .add_op(Opcodes.OpTrue)            # push 1
        .add_op(Opcodes.OpEqualVerify)     # assert: exactly 1 output
        .add_data(bytes.fromhex(recovery_xonly_pubkey_hex))
        .add_op(Opcodes.OpCheckSig)

        .add_op(Opcodes.OpElse)

        # ── Normal withdrawal path ────────────────────────────────────────
        .add_lock_time(locktime)
        .add_op(Opcodes.OpCheckLockTimeVerify)
        .add_op(Opcodes.OpDrop)
        .add_op(Opcodes.OpTxInputIndex)    # push current input index
        .add_op(Opcodes.OpAuthOutputCount) # count authorized continuation outputs
        .add_op(Opcodes.OpTrue)            # push 1
        .add_op(Opcodes.OpEqualVerify)     # assert: exactly 1 authorized output
        .add_data(bytes.fromhex(owner_xonly_pubkey_hex))
        .add_op(Opcodes.OpCheckSig)

        .add_op(Opcodes.OpEndIf)
    )


def _make_p2sh_unlock(sig_hex: str, branch_opcode: int, redeem_bytes: bytes) -> bytes:
    """
    Build a P2SH unlock script for a branch-selecting covenant.

    The unlock script format is:
        [sig_push][branch_opcode][redeem_script_push]

    sig_hex is the 66-byte hex from create_input_signature:
        [OP_DATA65=0x41][64_schnorr_sig][sighash_type]

    Because the first byte (0x41 = OP_DATA65) is already a valid script push
    opcode, the 66 bytes are appended as-is.  When the script engine executes
    them, OP_DATA65 pushes the following 65 bytes (sig64 + sighash) onto the
    stack — which is exactly the 65-byte value OpCheckSig expects.

    branch_opcode: 0x51 (OP_TRUE=1) → emergency, 0x00 (OP_FALSE=0) → normal
    """
    sig_bytes = bytes.fromhex(sig_hex)                     # pre-encoded: [0x41, sig64, sighash]
    branch_byte = bytes([branch_opcode])
    redeem_push = bytes.fromhex(
        ScriptBuilder().add_data(redeem_bytes).to_string()
    )
    return sig_bytes + branch_byte + redeem_push


# =============================================================================
# Genesis — create the vault UTXO with a KIP-20 covenant_id
# =============================================================================

async def genesis(client: RpcClient, owner_key: PrivateKey, funding_utxos: list):
    """
    Create and broadcast the genesis (vault creation) transaction.

    Computes the vault's covenant_id from the funding outpoint and the
    planned vault output, then creates the covenant-bound output.
    """
    keypair = Keypair.from_private_key(owner_key)
    owner_pubkey_hex = keypair.xonly_public_key

    # Recovery and owner are the same keypair in this demo.
    recovery_address = keypair.to_address(NETWORK_TYPE)
    recovery_spk = pay_to_address_script(recovery_address)
    recovery_pubkey_hex = owner_pubkey_hex

    redeem_script = build_vault_redeem_script(
        recovery_spk, recovery_pubkey_hex, owner_pubkey_hex, LOCKTIME_BLOCKS
    )
    vault_spk = redeem_script.create_pay_to_script_hash_script()
    vault_address = address_from_script_public_key(vault_spk, NETWORK_TYPE)
    print(f"  Vault P2SH address : {vault_address.to_string()}")

    funding = max(funding_utxos, key=lambda u: u["utxoEntry"]["amount"])
    funding_amount = funding["utxoEntry"]["amount"]
    funding_utxo_ref = UtxoEntryReference.from_dict(funding)
    funding_outpoint = TransactionOutpoint(
        Hash(funding["outpoint"]["transactionId"]),
        funding["outpoint"]["index"],
    )

    # ── Mass / fee (version 1 required for covenant outputs) ────────────────
    ph_out = TransactionOutput(funding_amount, vault_spk)
    ph_in = TransactionInput(funding_outpoint, b"", 0, 1, funding_utxo_ref)
    ph_tx = Transaction(1, [ph_in], [ph_out], 0, SUBNETWORK_ID, 0, b"", 0)
    mass = calculate_transaction_mass(NETWORK_ID, ph_tx)
    fee_rates = await client.get_fee_estimate()
    fee = mass * int(fee_rates["estimate"]["priorityBucket"]["feerate"])
    vault_amount = funding_amount - fee

    # ── Compute genesis covenant_id ─────────────────────────────────────────
    # covenant_id is hashed from (authorizing outpoint, auth_outputs)
    # auth_outputs do NOT include the covenant binding (to avoid self-reference)
    auth_output = TransactionOutput(vault_amount, vault_spk)
    cov_id = covenant_id(funding_outpoint, [auth_output])
    print(f"  Vault covenant_id  : {cov_id}")

    # ── Create the covenant-bound vault output ──────────────────────────────
    binding = CovenantBinding(0, cov_id)
    vault_output = TransactionOutput(vault_amount, vault_spk, covenant_id=binding)

    inp = TransactionInput(funding_outpoint, b"", 0, 1, funding_utxo_ref)
    tx = Transaction(1, [inp], [vault_output], 0, SUBNETWORK_ID, 0, b"", mass)
    signed_tx = sign_transaction(tx, [owner_key], True)

    print(f"  Fee: {fee} sompi  |  Vault: {vault_amount} sompi")

    result = await client.submit_transaction({"transaction": signed_tx, "allowOrphan": False})
    txid = result["transactionId"]
    print(f"  Genesis TXID: {txid}")

    return txid, {"transactionId": txid, "index": 0}, vault_amount, redeem_script, vault_spk, cov_id


# =============================================================================
# Emergency Sweep — spend via the fast exit path
# =============================================================================

async def emergency_sweep(
    client: RpcClient,
    owner_key: PrivateKey,
    vault_outpoint: dict,
    vault_amount: int,
    redeem_script: ScriptBuilder,
    vault_spk,
    cov_id,
) -> str:
    """
    Sweep the vault to the recovery address using the emergency branch.

    Unlock script: [sig_push] [OP_TRUE=1] [redeem_script_push]
    Stack at redeem script start: [sig, 1]
    OpIf sees 1 → emergency branch.
    """
    keypair = Keypair.from_private_key(owner_key)
    recovery_address = keypair.to_address(NETWORK_TYPE)
    recovery_spk = pay_to_address_script(recovery_address)
    vault_address = address_from_script_public_key(vault_spk, NETWORK_TYPE)

    vault_utxo_ref = UtxoEntryReference.from_dict({
        "address": vault_address.to_string(),
        "outpoint": {
            "transactionId": vault_outpoint["transactionId"],
            "index": vault_outpoint["index"],
        },
        "utxoEntry": {
            "amount": vault_amount,
            "scriptPublicKey": {"version": 0, "script": vault_spk.script},
            "blockDaaScore": 0,
            "isCoinbase": False,
            "covenantId": str(cov_id),
        },
    })
    cov_outpoint = TransactionOutpoint(
        Hash(vault_outpoint["transactionId"]), vault_outpoint["index"]
    )

    # ── Mass / fee ──────────────────────────────────────────────────────────
    ph_in = TransactionInput(cov_outpoint, b"", 0, 1, vault_utxo_ref)
    ph_out = TransactionOutput(vault_amount, recovery_spk)
    # Emergency sweep has no covenant outputs → version 0 acceptable
    ph_tx = Transaction(0, [ph_in], [ph_out], 0, SUBNETWORK_ID, 0, b"", 0)
    mass = calculate_transaction_mass(NETWORK_ID, ph_tx)
    fee_rates = await client.get_fee_estimate()
    fee = mass * int(fee_rates["estimate"]["priorityBucket"]["feerate"])
    sweep_amount = vault_amount - fee

    # ── Build unsigned transaction & sign ───────────────────────────────────
    inp = TransactionInput(cov_outpoint, b"", 0, 1, vault_utxo_ref)
    out = TransactionOutput(sweep_amount, recovery_spk)
    tx_unsigned = Transaction(0, [inp], [out], 0, SUBNETWORK_ID, 0, b"", mass)

    sig_hex = create_input_signature(tx_unsigned, 0, owner_key)
    redeem_bytes = bytes.fromhex(redeem_script.to_string())

    # Build unlock script manually:
    # [sig_push=pre-encoded 66 bytes] [0x51=OP_TRUE] [redeem_script_push]
    # 0x51 is OP_TRUE → pushes 1 → OpIf enters the emergency branch
    unlock_bytes = _make_p2sh_unlock(sig_hex, 0x51, redeem_bytes)

    signed_inp = TransactionInput(cov_outpoint, unlock_bytes, 0, 1, vault_utxo_ref)
    tx = Transaction(0, [signed_inp], [out], 0, SUBNETWORK_ID, 0, b"", mass)

    print(f"  Fee: {fee} sompi  |  Sweep: {sweep_amount} sompi")

    result = await client.submit_transaction({"transaction": tx, "allowOrphan": False})
    txid = result["transactionId"]
    print(f"  Emergency sweep TXID: {txid}")
    return txid


# =============================================================================
# Normal Withdrawal — continue the vault covenant after locktime
# =============================================================================

async def normal_withdrawal(
    client: RpcClient,
    owner_key: PrivateKey,
    vault_outpoint: dict,
    vault_amount: int,
    redeem_script: ScriptBuilder,
    vault_spk,
    cov_id,
    withdraw_amount: int,
    current_daa_score: int,
) -> str:
    """
    Withdraw `withdraw_amount` from the vault after the locktime passes.

    The remainder stays in a continuation vault UTXO (same covenant_id,
    same vault P2SH script).  The lock_time field on the transaction must
    be >= LOCKTIME_BLOCKS for OpCheckLockTimeVerify to pass.

    Unlock script: [sig_push] [OP_FALSE=0] [redeem_script_push]
    Stack at redeem script start: [sig, 0]
    OpIf sees 0 → else branch (normal withdrawal).
    """
    keypair = Keypair.from_private_key(owner_key)
    dest_address = keypair.to_address(NETWORK_TYPE)
    dest_spk = pay_to_address_script(dest_address)
    vault_address = address_from_script_public_key(vault_spk, NETWORK_TYPE)

    vault_utxo_ref = UtxoEntryReference.from_dict({
        "address": vault_address.to_string(),
        "outpoint": {
            "transactionId": vault_outpoint["transactionId"],
            "index": vault_outpoint["index"],
        },
        "utxoEntry": {
            "amount": vault_amount,
            "scriptPublicKey": {"version": 0, "script": vault_spk.script},
            "blockDaaScore": 0,
            "isCoinbase": False,
            "covenantId": str(cov_id),
        },
    })
    cov_outpoint = TransactionOutpoint(
        Hash(vault_outpoint["transactionId"]), vault_outpoint["index"]
    )

    remainder = vault_amount - withdraw_amount
    # Continuation output: same vault script, same covenant_id
    binding = CovenantBinding(0, cov_id)
    cont_output = TransactionOutput(remainder, vault_spk, covenant_id=binding)
    dest_output = TransactionOutput(withdraw_amount, dest_spk)

    # ── Mass / fee ──────────────────────────────────────────────────────────
    ph_in = TransactionInput(cov_outpoint, b"", 0, 1, vault_utxo_ref)
    # lock_time must satisfy CLTV: set it to current_daa_score (>= LOCKTIME_BLOCKS)
    ph_tx = Transaction(
        1, [ph_in], [cont_output, dest_output], current_daa_score, SUBNETWORK_ID, 0, b"", 0
    )
    mass = calculate_transaction_mass(NETWORK_ID, ph_tx)
    fee_rates = await client.get_fee_estimate()
    fee = mass * int(fee_rates["estimate"]["priorityBucket"]["feerate"])
    final_withdraw = withdraw_amount - fee
    dest_output = TransactionOutput(final_withdraw, dest_spk)

    # ── Build unsigned transaction & sign ───────────────────────────────────
    inp = TransactionInput(cov_outpoint, b"", 0, 1, vault_utxo_ref)
    tx_unsigned = Transaction(
        1, [inp], [cont_output, dest_output], current_daa_score, SUBNETWORK_ID, 0, b"", mass
    )

    sig_hex = create_input_signature(tx_unsigned, 0, owner_key)
    redeem_bytes = bytes.fromhex(redeem_script.to_string())

    # Build unlock script:
    # [sig_push] [0x00=OP_FALSE] [redeem_script_push]
    # 0x00 is OP_FALSE → pushes empty bytes (0) → OpIf goes to else branch
    unlock_bytes = _make_p2sh_unlock(sig_hex, 0x00, redeem_bytes)

    signed_inp = TransactionInput(cov_outpoint, unlock_bytes, 0, 1, vault_utxo_ref)
    tx = Transaction(
        1, [signed_inp], [cont_output, dest_output], current_daa_score, SUBNETWORK_ID, 0, b"", mass
    )

    print(f"  Fee: {fee} sompi  |  Withdrawal: {final_withdraw}  |  Remainder: {remainder}")

    result = await client.submit_transaction({"transaction": tx, "allowOrphan": False})
    txid = result["transactionId"]
    print(f"  Normal withdrawal TXID: {txid}")
    return txid


# =============================================================================
# Helpers
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
    print(f"  Waiting for confirmation of {txid[:16]}…")
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
    print("Smart Vault Covenant  —  KIP-17 + KIP-20 Demo")
    print("=" * 60)
    print(f"\nFund this testnet-12 address (owner key):")
    print(f"  {funding_address.to_string()}")
    print("\nThe script detects incoming funds automatically.\n")

    client = RpcClient(url=RPC_URL, network_id=NETWORK_ID)
    await client.connect()
    print(f"Connected to {NETWORK_ID}\n")

    utxos = await wait_for_utxos(client, funding_address)
    total = sum(u["utxoEntry"]["amount"] for u in utxos)
    print(f"Received {total} sompi across {len(utxos)} UTXO(s)\n")

    print("[Step 1/2] Broadcasting genesis (vault creation) transaction…")
    txid, outpoint, vault_amount, redeem_script, vault_spk, cov_id = await genesis(
        client, owner_key, utxos
    )
    await wait_for_confirmation(client, txid)
    print()

    print("[Step 2/2] Broadcasting emergency sweep transaction…")
    sweep_txid = await emergency_sweep(
        client, owner_key, outpoint, vault_amount, redeem_script, vault_spk, cov_id
    )
    print()

    # ── Commented out: Normal withdrawal demo ─────────────────────────────────
    # To test the normal path, re-fund and uncomment:
    #
    # from kaspa import RpcClient as _RpcClient
    # info = await client.get_block_dag_info()
    # daa = info["virtualDaaScore"]
    # print("[Step 3/3] Normal withdrawal (after locktime)…")
    # withdrawal_txid = await normal_withdrawal(
    #     client, owner_key, outpoint, vault_amount,
    #     redeem_script, vault_spk, cov_id,
    #     withdraw_amount=vault_amount // 2,
    #     current_daa_score=daa,
    # )

    print("=" * 60)
    print("Demo complete!")
    print(f"  Genesis TXID        : {txid}")
    print(f"  Emergency sweep TXID: {sweep_txid}")
    print("=" * 60)

    await client.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
