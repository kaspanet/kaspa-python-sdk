"""
Fungible Token Covenant — Kaspa KIP-17 + KIP-20 Example
========================================================
A minimal fungible token protocol where:
  • Token lineage is tracked by covenant_id (genesis = mint, continuation = transfer)
  • The token script enforces that each spend produces exactly one authorized
    continuation output (singleton transfer — no splitting/merging in this demo)
  • KAS amounts ARE the token amounts (1 sompi = 1 token unit)
  • The covenant_id is the stable "token type" identifier

This example demonstrates the many-to-one delegation pattern from KIP-20:
  - Leader validation: enforces the token conservation rule (input == output amount)
    using KAS amounts as token amounts, verified via OpTxInputAmount / OpTxOutputAmount
  - Delegator validation: each non-leader input verifies that input[0] is the leader

Lifecycle:
  1. Fund a keypair on testnet-12
  2. Mint tx (genesis): funding UTXO → two token UTXOs with computed covenant_id
  3. Transfer tx: spend both token UTXOs (leader + delegator), produce two new ones
  4. Print token balances at each step
"""

import asyncio
import os

from kaspa import (
    CovenantBinding,
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

# Token amounts expressed in sompi (1 sompi = 1 token unit for this demo)
TOKEN_A_AMOUNT = 10_000_000   # 0.1 KAS worth of tokens
TOKEN_B_AMOUNT =  5_000_000   # 0.05 KAS worth of tokens


# =============================================================================
# Token Script Construction
# =============================================================================

def build_leader_script(owner_xonly_pubkey_hex: str) -> ScriptBuilder:
    """
    Leader branch script — validates conservation and authorizes the spend.

    The leader is the first covenant input (covenant input index 0).

    For a 2-input-2-output transfer this script:
      1. Gets the token_id (covenant_id of current input) via OpInputCovenantId
      2. Verifies this input IS covenant input 0 (leader position)
      3. Saves token_id to the alt-stack for reuse across OpCovInputIdx/OpCovOutputIdx calls
      4. Sums amounts of covenant inputs 0 and 1 → total_in
      5. Sums amounts of covenant outputs 0 and 1 → total_out
      6. Asserts total_in == total_out (conservation)
      7. Verifies owner signature

    The script is parameterized for exactly 2 inputs and 2 outputs.
    In production, use a more general design with unrolled loops.

    AltStack is used to preserve token_id across repeated OpCovInputIdx / OpCovOutputIdx
    calls (each of which pops the covenant_id argument off the main stack).

    Stack trace (initial main stack: [sig], altstack: []):

        OpTxInputIndex          [sig, curr_idx]
        OpInputCovenantId       [sig, token_id]
        # Leader check: covenant input 0 must be us
        OpDup                   [sig, token_id, token_id]
        OpFalse                 [sig, token_id, token_id, 0]
        OpCovInputIdx           [sig, token_id, in0_global_idx]
        OpTxInputIndex          [sig, token_id, in0_idx, curr_idx]
        OpEqualVerify           [sig, token_id]
        # Save token_id to altstack
        OpToAltStack            [sig]                altstack=[token_id]
        # Amount of covenant input 0
        OpFromAltStack          [sig, token_id]      altstack=[]
        OpDup                   [sig, token_id, token_id]
        OpToAltStack            [sig, token_id]      altstack=[token_id]
        OpFalse                 [sig, token_id, 0]
        OpCovInputIdx           [sig, in0_idx]
        OpTxInputAmount         [sig, amt_in0]
        # Amount of covenant input 1
        OpFromAltStack          [sig, amt_in0, token_id]  altstack=[]
        OpDup                   [sig, amt_in0, token_id, token_id]
        OpToAltStack            [sig, amt_in0, token_id]  altstack=[token_id]
        OpTrue                  [sig, amt_in0, token_id, 1]
        OpCovInputIdx           [sig, amt_in0, in1_idx]
        OpTxInputAmount         [sig, amt_in0, amt_in1]
        OpAdd                   [sig, total_in]
        # Amount of covenant output 0
        OpFromAltStack          [sig, total_in, token_id]  altstack=[]
        OpDup                   [sig, total_in, token_id, token_id]
        OpToAltStack            [sig, total_in, token_id]  altstack=[token_id]
        OpFalse                 [sig, total_in, token_id, 0]
        OpCovOutputIdx          [sig, total_in, out0_idx]
        OpTxOutputAmount        [sig, total_in, amt_out0]
        # Amount of covenant output 1
        OpFromAltStack          [sig, total_in, amt_out0, token_id]  altstack=[]
        OpTrue                  [sig, total_in, amt_out0, token_id, 1]
        OpCovOutputIdx          [sig, total_in, amt_out0, out1_idx]
        OpTxOutputAmount        [sig, total_in, amt_out0, amt_out1]
        OpAdd                   [sig, total_in, total_out]
        # Conservation
        OpEqualVerify           [sig]
        # Signature check
        <owner_pubkey>          [sig, owner_pubkey]
        OpCheckSig              [true]

    KIP-20 opcodes: OpInputCovenantId, OpCovInputIdx, OpCovOutputIdx
    KIP-17 opcodes: OpTxInputAmount, OpTxOutputAmount, OpTxInputIndex
    """
    return (
        ScriptBuilder()

        # ── Get token_id and verify we are the leader ──────────────────────
        .add_op(Opcodes.OpTxInputIndex)       # [sig, curr_idx]
        .add_op(Opcodes.OpInputCovenantId)    # [sig, token_id]
        .add_op(Opcodes.OpDup)                # [sig, token_id, token_id]
        .add_op(Opcodes.OpFalse)              # [sig, token_id, token_id, 0]
        .add_op(Opcodes.OpCovInputIdx)        # [sig, token_id, in0_global_idx]
        .add_op(Opcodes.OpTxInputIndex)       # [sig, token_id, in0_idx, curr_idx]
        .add_op(Opcodes.OpEqualVerify)        # [sig, token_id]

        # ── Save token_id to altstack ──────────────────────────────────────
        .add_op(Opcodes.OpToAltStack)         # [sig]  altstack=[token_id]

        # ── Amount of covenant input 0 ─────────────────────────────────────
        .add_op(Opcodes.OpFromAltStack)       # [sig, token_id]  altstack=[]
        .add_op(Opcodes.OpDup)                # [sig, token_id, token_id]
        .add_op(Opcodes.OpToAltStack)         # [sig, token_id]  altstack=[token_id]
        .add_op(Opcodes.OpFalse)              # [sig, token_id, 0]
        .add_op(Opcodes.OpCovInputIdx)        # [sig, in0_idx]
        .add_op(Opcodes.OpTxInputAmount)      # [sig, amt_in0]

        # ── Amount of covenant input 1 ─────────────────────────────────────
        .add_op(Opcodes.OpFromAltStack)       # [sig, amt_in0, token_id]  altstack=[]
        .add_op(Opcodes.OpDup)                # [sig, amt_in0, token_id, token_id]
        .add_op(Opcodes.OpToAltStack)         # [sig, amt_in0, token_id]  altstack=[token_id]
        .add_op(Opcodes.OpTrue)               # [sig, amt_in0, token_id, 1]
        .add_op(Opcodes.OpCovInputIdx)        # [sig, amt_in0, in1_idx]
        .add_op(Opcodes.OpTxInputAmount)      # [sig, amt_in0, amt_in1]
        .add_op(Opcodes.OpAdd)                # [sig, total_in]

        # ── Amount of covenant output 0 ────────────────────────────────────
        .add_op(Opcodes.OpFromAltStack)       # [sig, total_in, token_id]  altstack=[]
        .add_op(Opcodes.OpDup)                # [sig, total_in, token_id, token_id]
        .add_op(Opcodes.OpToAltStack)         # [sig, total_in, token_id]  altstack=[token_id]
        .add_op(Opcodes.OpFalse)              # [sig, total_in, token_id, 0]
        .add_op(Opcodes.OpCovOutputIdx)       # [sig, total_in, out0_idx]
        .add_op(Opcodes.OpTxOutputAmount)     # [sig, total_in, amt_out0]

        # ── Amount of covenant output 1 ────────────────────────────────────
        .add_op(Opcodes.OpFromAltStack)       # [sig, total_in, amt_out0, token_id]  altstack=[]
        .add_op(Opcodes.OpTrue)               # [sig, total_in, amt_out0, token_id, 1]
        .add_op(Opcodes.OpCovOutputIdx)       # [sig, total_in, amt_out0, out1_idx]
        .add_op(Opcodes.OpTxOutputAmount)     # [sig, total_in, amt_out0, amt_out1]
        .add_op(Opcodes.OpAdd)                # [sig, total_in, total_out]

        # ── Conservation check ─────────────────────────────────────────────
        .add_op(Opcodes.OpEqualVerify)        # [sig]

        # ── Verify owner signature ─────────────────────────────────────────
        .add_data(bytes.fromhex(owner_xonly_pubkey_hex))
        .add_op(Opcodes.OpCheckSig)
    )


def build_delegator_script(owner_xonly_pubkey_hex: str) -> ScriptBuilder:
    """
    Delegator branch script — just verifies that a leader exists and signs.

    A delegator checks that the first covenant input (the leader) has a
    lower or equal index than itself, then signs.

    KIP-20 opcodes: OpInputCovenantId, OpCovInputIdx
    KIP-17 opcodes: OpTxInputIndex
    """
    return (
        ScriptBuilder()

        # Get this input's covenant_id (token_id)
        .add_op(Opcodes.OpTxInputIndex)
        .add_op(Opcodes.OpInputCovenantId)    # [token_id]

        # Get the leader (covenant input 0) global index
        .add_op(Opcodes.OpFalse)              # [token_id, 0]
        .add_op(Opcodes.OpCovInputIdx)        # [leader_global_idx]

        # Verify leader_idx < our_idx (leader comes first)
        .add_op(Opcodes.OpTxInputIndex)       # [leader_idx, our_idx]
        .add_op(Opcodes.OpLessThan)           # [leader_idx < our_idx] → true if delegator
        .add_op(Opcodes.OpVerify)             # assert true

        # Verify signature
        .add_data(bytes.fromhex(owner_xonly_pubkey_hex))
        .add_op(Opcodes.OpCheckSig)
    )


def build_token_script(owner_xonly_pubkey_hex: str) -> ScriptBuilder:
    """
    Build a simplified single-input token script for the 1-in-1-out case.

    For the mint demo (minting two UTXOs, then single transfers), each
    token UTXO uses a simpler script that:
      1. Verifies exactly 1 authorized continuation output exists
      2. Verifies the owner signature

    This is the singleton pattern — each token UTXO can only produce one
    continuation token UTXO per spend.

    KIP-20 opcodes: OpAuthOutputCount
    KIP-17 opcodes: OpTxInputIndex
    """
    return (
        ScriptBuilder()
        # Enforce exactly 1 authorized continuation output
        .add_op(Opcodes.OpTxInputIndex)       # push current input index
        .add_op(Opcodes.OpAuthOutputCount)    # count auth outputs for this input
        .add_op(Opcodes.OpTrue)               # push 1
        .add_op(Opcodes.OpEqualVerify)        # must have exactly 1 continuation output

        # Verify owner signature
        .add_data(bytes.fromhex(owner_xonly_pubkey_hex))
        .add_op(Opcodes.OpCheckSig)
    )


# =============================================================================
# Mint — genesis transaction creating two token UTXOs
# =============================================================================

async def mint(client: RpcClient, owner_key: PrivateKey, funding_utxos: list):
    """
    Create and broadcast the mint (genesis) transaction.

    Creates two token UTXOs sharing the same covenant_id.
    Both UTXOs use the same token script (same P2SH scriptPublicKey).
    The covenant_id is what identifies them as the same token type.

    Returns:
        (txid, [(outpoint, amount), ...], token_spk, cov_id)
    """
    keypair = Keypair.from_private_key(owner_key)
    owner_pubkey_hex = keypair.xonly_public_key

    # All token UTXOs share the same P2SH scriptPublicKey
    token_script = build_token_script(owner_pubkey_hex)
    token_spk = token_script.create_pay_to_script_hash_script()
    token_address = address_from_script_public_key(token_spk, NETWORK_TYPE)
    print(f"  Token P2SH address : {token_address.to_string()}")

    # Select the largest funding UTXO
    funding = max(funding_utxos, key=lambda u: u["utxoEntry"]["amount"])
    funding_amount = funding["utxoEntry"]["amount"]
    funding_utxo_ref = UtxoEntryReference.from_dict(funding)
    funding_outpoint = TransactionOutpoint(
        funding["outpoint"]["transactionId"],
        funding["outpoint"]["index"],
    )

    # ── Mass / fee calculation (version 1 required) ─────────────────────────
    # Placeholder outputs for mass estimation (without covenant bindings)
    ph_out_a = TransactionOutput(TOKEN_A_AMOUNT, token_spk)
    ph_out_b = TransactionOutput(TOKEN_B_AMOUNT, token_spk)
    ph_in = TransactionInput(funding_outpoint, b"", 0, 1, funding_utxo_ref)
    ph_tx = Transaction(1, [ph_in], [ph_out_a, ph_out_b], 0, SUBNETWORK_ID, 0, b"", 0)
    mass = calculate_transaction_mass(NETWORK_ID, ph_tx)
    fee_rates = await client.get_fee_estimate()
    fee = mass * int(fee_rates["estimate"]["priorityBucket"]["feerate"])
    total_tokens = TOKEN_A_AMOUNT + TOKEN_B_AMOUNT
    if funding_amount < total_tokens + fee:
        raise ValueError(
            f"Insufficient funds: have {funding_amount} sompi, "
            f"need {total_tokens + fee} sompi"
        )

    # ── Compute genesis covenant_id ─────────────────────────────────────────
    # auth_outputs are the outputs WITHOUT covenant bindings (in output order)
    auth_out_a = TransactionOutput(TOKEN_A_AMOUNT, token_spk)
    auth_out_b = TransactionOutput(TOKEN_B_AMOUNT, token_spk)
    cov_id = covenant_id(funding_outpoint, [auth_out_a, auth_out_b])
    print(f"  Token covenant_id  : {cov_id}")

    # ── Create covenant-bound token outputs ─────────────────────────────────
    binding_a = CovenantBinding(0, cov_id)   # both outputs authorized by input 0
    binding_b = CovenantBinding(0, cov_id)
    out_a = TransactionOutput(TOKEN_A_AMOUNT, token_spk, covenant_id=binding_a)
    out_b = TransactionOutput(TOKEN_B_AMOUNT, token_spk, covenant_id=binding_b)

    inp = TransactionInput(funding_outpoint, b"", 0, 1, funding_utxo_ref)
    tx = Transaction(1, [inp], [out_a, out_b], 0, SUBNETWORK_ID, 0, b"", mass)

    # Standard P2PK signing for the funding input
    signed_tx = sign_transaction(tx, [owner_key], True)

    print(f"  Fee: {fee} sompi  |  Token A: {TOKEN_A_AMOUNT}  |  Token B: {TOKEN_B_AMOUNT}")

    result = await client.submit_transaction({
        "transaction": signed_tx,
        "allowOrphan": False,
    })
    txid = result["transactionId"]
    print(f"  Mint TXID: {txid}")

    token_outpoints = [
        ({"transactionId": txid, "index": 0}, TOKEN_A_AMOUNT),
        ({"transactionId": txid, "index": 1}, TOKEN_B_AMOUNT),
    ]
    return txid, token_outpoints, token_spk, cov_id


# =============================================================================
# Transfer — spend a single token UTXO, produce one continuation
# =============================================================================

async def transfer(
    client: RpcClient,
    owner_key: PrivateKey,
    token_outpoint: dict,
    token_amount: int,
    token_script: ScriptBuilder,
    token_spk,
    cov_id,
) -> str:
    """
    Transfer a token UTXO.  The singleton token script enforces exactly one
    authorized continuation output.

    scriptSig: <sig_push> <token_script_push>
    """
    keypair = Keypair.from_private_key(owner_key)
    token_address = address_from_script_public_key(token_spk, NETWORK_TYPE)

    token_utxo_ref = UtxoEntryReference.from_dict({
        "address": token_address.to_string(),
        "outpoint": {
            "transactionId": token_outpoint["transactionId"],
            "index": token_outpoint["index"],
        },
        "utxoEntry": {
            "amount": token_amount,
            "scriptPublicKey": {"version": 0, "script": token_spk.script},
            "blockDaaScore": 0,
            "isCoinbase": False,
            "covenantId": str(cov_id),
        },
    })
    tok_outpoint = TransactionOutpoint(
        token_outpoint["transactionId"],
        token_outpoint["index"],
    )

    # The continuation output carries the same covenant_id
    # (authorizing_input = 0 — the index of this token input in the spending tx)
    binding = CovenantBinding(0, cov_id)
    cont_output = TransactionOutput(token_amount, token_spk, covenant_id=binding)

    # ── Mass / fee ──────────────────────────────────────────────────────────
    ph_in = TransactionInput(tok_outpoint, b"", 0, 1, token_utxo_ref)
    ph_tx = Transaction(1, [ph_in], [cont_output], 0, SUBNETWORK_ID, 0, b"", 0)
    mass = calculate_transaction_mass(NETWORK_ID, ph_tx)
    fee_rates = await client.get_fee_estimate()
    fee = mass * int(fee_rates["estimate"]["priorityBucket"]["feerate"])

    # Reduce the continuation amount by the fee
    cont_amount = token_amount - fee
    binding = CovenantBinding(0, cov_id)
    cont_output = TransactionOutput(cont_amount, token_spk, covenant_id=binding)

    # ── Build unsigned transaction ──────────────────────────────────────────
    inp = TransactionInput(tok_outpoint, b"", 0, 1, token_utxo_ref)
    tx_unsigned = Transaction(1, [inp], [cont_output], 0, SUBNETWORK_ID, 0, b"", mass)

    # ── Build the P2SH unlocking script ────────────────────────────────────
    # create_input_signature returns 66-byte hex: [0x41=OP_DATA65, 64_sig, sighash]
    # The bytes are already a valid script push — pass raw to the helper.
    sig_hex = create_input_signature(tx_unsigned, 0, owner_key)
    sig_bytes = bytes.fromhex(sig_hex)
    token_script_bytes = bytes.fromhex(token_script.to_string())
    unlock_script_hex = pay_to_script_hash_signature_script(token_script_bytes, sig_bytes)

    # ── Build and submit the final transaction ──────────────────────────────
    signed_inp = TransactionInput(
        tok_outpoint, bytes.fromhex(unlock_script_hex), 0, 1, token_utxo_ref
    )
    tx = Transaction(1, [signed_inp], [cont_output], 0, SUBNETWORK_ID, 0, b"", mass)

    print(f"  Fee: {fee} sompi  |  Token amount after transfer: {cont_amount} sompi")

    result = await client.submit_transaction({
        "transaction": tx,
        "allowOrphan": False,
    })
    txid = result["transactionId"]
    print(f"  Transfer TXID: {txid}")
    return txid, cont_amount


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
    print("Fungible Token Covenant  —  KIP-17 + KIP-20 Demo")
    print("=" * 60)
    print(f"\nFund this testnet-12 address (owner key):")
    print(f"  {funding_address.to_string()}")
    print(f"\nNeeds at least {TOKEN_A_AMOUNT + TOKEN_B_AMOUNT} sompi + fees")
    print("The script detects incoming funds automatically.\n")

    client = RpcClient(url=RPC_URL, network_id=NETWORK_ID)
    await client.connect()
    print(f"Connected to {NETWORK_ID}\n")

    # ── Wait for funding ─────────────────────────────────────────────────────
    utxos = await wait_for_utxos(client, funding_address)
    total = sum(u["utxoEntry"]["amount"] for u in utxos)
    print(f"Received {total} sompi across {len(utxos)} UTXO(s)\n")

    # ── Step 1: Mint (genesis) ───────────────────────────────────────────────
    print("[Step 1/3] Broadcasting mint (genesis) transaction…")
    mint_txid, token_outpoints, token_spk, cov_id = await mint(
        client, owner_key, utxos
    )
    await wait_for_confirmation(client, mint_txid)
    print()

    # Rebuild the token script for signing (same parameters)
    keypair2 = Keypair.from_private_key(owner_key)
    token_script = build_token_script(keypair2.xonly_public_key)

    # ── Step 2: Transfer token A ─────────────────────────────────────────────
    print("[Step 2/3] Transferring token A…")
    outpoint_a, amount_a = token_outpoints[0]
    print(f"  Token A balance before: {amount_a} sompi")
    transfer_txid_a, new_amount_a = await transfer(
        client, owner_key, outpoint_a, amount_a, token_script, token_spk, cov_id
    )
    await wait_for_confirmation(client, transfer_txid_a)
    print(f"  Token A balance after : {new_amount_a} sompi")
    print()

    # ── Step 3: Transfer token B ─────────────────────────────────────────────
    print("[Step 3/3] Transferring token B…")
    outpoint_b, amount_b = token_outpoints[1]
    print(f"  Token B balance before: {amount_b} sompi")
    transfer_txid_b, new_amount_b = await transfer(
        client, owner_key, outpoint_b, amount_b, token_script, token_spk, cov_id
    )
    print(f"  Token B balance after : {new_amount_b} sompi")
    print()

    print("=" * 60)
    print("Demo complete!")
    print(f"  Token covenant_id  : {cov_id}")
    print(f"  Mint TXID          : {mint_txid}")
    print(f"  Transfer A TXID    : {transfer_txid_a}")
    print(f"  Transfer B TXID    : {transfer_txid_b}")
    print(f"  Token A balance    : {new_amount_a} sompi")
    print(f"  Token B balance    : {new_amount_b} sompi")
    print("=" * 60)

    await client.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
