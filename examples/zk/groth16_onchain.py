"""Verify a RISC Zero Groth16 proof on-chain via a zk-to-script round-trip (testnet-10).

`kaspa.ZkScriptBuilder` turns a RISC Zero Groth16 receipt into a Kaspa P2SH
redeem script that only unlocks when the on-chain `OpZkPrecompile` verifies the
proof for a fixed image id and journal hash. This script runs the full lifecycle
against a live node:

    commit   lock a funding UTXO into P2SH(redeem_script)
    redeem   spend that P2SH UTXO by presenting the proof (the sig script)

The redeem is permissionless: the zk proof *is* the authorization, so the
covenant input carries no Schnorr signature (exactly like a covenant transition
in examples/silverscript/counter.py). Only the normal P2PK funding input of the
commit transaction is signed.

`OpZkPrecompile` is gated by the same Toccata activation as the covenant
opcodes, so this needs a network where Toccata is active — testnet-10. The proof
fixtures under examples/zk/data/ are a matching set (image id, journal hash, and
the borsh-encoded receipt), so the redeem genuinely verifies on-chain.

Setup:
    export KASPA_RPC_URL=<your testnet-10 node, e.g. ws://127.0.0.1:17210>
    python examples/zk/groth16_onchain.py

Prints a funding address and waits for you to send it testnet KAS (a fraction of
a TKAS is plenty — the zk redeem's compute budget makes it the costlier tx).
"""

import asyncio
import os
import pathlib

from kaspa import (
    Hash,
    Keypair,
    PrivateKey,
    RpcClient,
    Transaction,
    TransactionInput,
    TransactionOutpoint,
    TransactionOutput,
    UtxoEntryReference,
    ZkScriptBuilder,
    address_from_script_public_key,
    calculate_transaction_mass,
    pay_to_address_script,
    pay_to_script_hash_script,
    sign_transaction,
)

NETWORK_ID = "testnet-10"
NETWORK_TYPE = "testnet"
RPC_URL = os.environ.get("KASPA_RPC_URL")
SUBNETWORK_ID = bytes(20)
TX_VERSION = 1

# Compute budget per input. The P2PK funding input is cheap; the zk redeem input
# must cover the Groth16 precompile (≈140k grams ≈ 1400 budget units) plus the
# surrounding script opcodes — 1600 leaves headroom.
FUNDING_COMPUTE_BUDGET = 10
ZK_COMPUTE_BUDGET = 1600

# Toccata activation DAA score for testnet-10. At/above this score, covenants and
# the zk precompile are active. Below it, the redeem would be rejected.
TOCCATA_TESTNET10 = 467_579_632

# A matching RISC Zero Groth16 proof set (image id, journal hash, receipt) shipped
# with rusty-kaspa's zk-sdk. The receipt is read from a file rather than inlined.
IMAGE_ID = "75641a540ee2ad9ee5902bcdcdb8b55c0bef4a28287309b858f97b1356c6c2e0"
JOURNAL_HASH = "5df6e0e2761359d30a8275058e299fcc0381534545f55cf43e41983f5d4c9456"
DATA = pathlib.Path(__file__).resolve().parent / "data"


# =============================================================================
# Building the zk scripts (the new bindings)
# =============================================================================

def build_zk_scripts() -> tuple[str, str]:
    """Build the P2SH redeem script and the proof-bearing signature script.

    Returns:
        A tuple of (redeem_script, sig_script), both hex. The redeem script is
        hashed into the P2SH locking script; the sig script unlocks it.
    """
    receipt = (DATA / "groth.rcpt.hex").read_text().strip()

    builder = ZkScriptBuilder.new_r0(covenants_enabled=True)
    builder.commit_to_groth16(IMAGE_ID)
    finalized = builder.finalize_with_groth16_proof(receipt, JOURNAL_HASH)
    return finalized.redeem_script, finalized.sig_script


# =============================================================================
# Building transactions
# =============================================================================

async def build_tx(
    client: RpcClient,
    spend: TransactionInput,
    value_in: int,
    output_spk,
) -> tuple[Transaction, int]:
    """Size and build a 1-in/1-out tx, paying the fee out of `value_in`.

    Mass doesn't depend on the output amount, so it's measured on a draft first,
    then the fee is deducted and the tx rebuilt with the measured mass.

    Args:
        client: RPC client used to fetch the fee estimate.
        spend: The (fully formed) input being spent.
        value_in: The input's value, in sompi; the fee comes out of this.
        output_spk: The output's script public key.

    Returns:
        A tuple of (transaction, output_value).
    """
    draft = Transaction(
        TX_VERSION, [spend], [TransactionOutput(value_in, output_spk)],
        lock_time=0, subnetwork_id=SUBNETWORK_ID, gas=0, payload=b"", mass=0,
    )
    mass = calculate_transaction_mass(NETWORK_ID, draft)
    estimate = await client.get_fee_estimate()
    fee = mass * int(estimate["estimate"]["priorityBucket"]["feerate"])
    value_out = value_in - fee

    tx = Transaction(
        TX_VERSION, [spend], [TransactionOutput(value_out, output_spk)],
        lock_time=0, subnetwork_id=SUBNETWORK_ID, gas=0, payload=b"", mass=mass,
    )
    return tx, value_out


async def commit(
    client: RpcClient,
    funder_key: PrivateKey,
    funding_utxos: list[dict],
    redeem_script: str,
) -> tuple[str, int]:
    """Lock the largest funding UTXO into P2SH(redeem_script).

    Args:
        client: RPC client used to size and submit the tx.
        funder_key: Private key for the P2PK funding input.
        funding_utxos: Candidate funding UTXOs; the largest is spent.
        redeem_script: The zk redeem script (hex) to lock funds behind.

    Returns:
        A tuple of (commit_txid, p2sh_output_value).
    """
    funding = max(funding_utxos, key=lambda u: u["utxoEntry"]["amount"])
    spend = TransactionInput(
        TransactionOutpoint(Hash(funding["outpoint"]["transactionId"]), funding["outpoint"]["index"]),
        b"",
        sequence=0,
        sig_op_count=0,
        compute_budget=FUNDING_COMPUTE_BUDGET,
        utxo=UtxoEntryReference.from_dict(funding),
    )

    p2sh_spk = pay_to_script_hash_script(redeem_script)
    tx, value = await build_tx(client, spend, funding["utxoEntry"]["amount"], p2sh_spk)

    signed = sign_transaction(tx, [funder_key], True)
    result = await client.submit_transaction({"transaction": signed, "allowOrphan": False})
    return result["transactionId"], value


async def redeem(
    client: RpcClient,
    commit_txid: str,
    p2sh_value: int,
    redeem_script: str,
    sig_script: str,
    payout_address,
) -> tuple[str, int]:
    """Spend the P2SH UTXO by presenting the zk proof, paying out to `payout_address`.

    The input is unlocked by the proof alone — no Schnorr signature — so the tx
    is submitted unsigned.

    Args:
        client: RPC client used to size and submit the tx.
        commit_txid: Txid of the commit transaction (output 0 is the P2SH UTXO).
        p2sh_value: Value of the P2SH UTXO, in sompi.
        redeem_script: The zk redeem script (hex) the P2SH UTXO is locked behind.
        sig_script: The proof-bearing signature script (hex) that unlocks it.
        payout_address: Where the redeemed funds (minus fee) are sent.

    Returns:
        A tuple of (redeem_txid, payout_value).
    """
    p2sh_spk = pay_to_script_hash_script(redeem_script)
    p2sh_utxo = UtxoEntryReference.from_dict({
        "address": address_from_script_public_key(p2sh_spk, NETWORK_TYPE).to_string(),
        "outpoint": {"transactionId": commit_txid, "index": 0},
        "utxoEntry": {
            "amount": p2sh_value,
            "scriptPublicKey": {"version": p2sh_spk.version, "script": p2sh_spk.script},
            "blockDaaScore": 0,
            "isCoinbase": False,
            "covenantId": None,  # a plain zk P2SH UTXO carries no covenant
        },
    })

    # The proof is the authorization: set the sig script, allocate the zk compute
    # budget, and do NOT sign.
    spend = TransactionInput(
        TransactionOutpoint(Hash(commit_txid), 0),
        sig_script,
        sequence=0,
        sig_op_count=0,
        compute_budget=ZK_COMPUTE_BUDGET,
        utxo=p2sh_utxo,
    )

    tx, value = await build_tx(client, spend, p2sh_value, pay_to_address_script(payout_address))
    result = await client.submit_transaction({"transaction": tx, "allowOrphan": False})
    return result["transactionId"], value


# =============================================================================
# RPC helpers
# =============================================================================

async def wait_for_funds(client: RpcClient, addr) -> list[dict]:
    """Poll until `addr` has at least one UTXO, then return them."""
    while True:
        result = await client.get_utxos_by_addresses({"addresses": [addr]})
        if result["entries"]:
            return result["entries"]
        await asyncio.sleep(5)


async def wait_until_accepted(client: RpcClient, addr, txid: str) -> None:
    """Poll `addr` until an output of `txid` appears there."""
    while True:
        result = await client.get_utxos_by_addresses({"addresses": [addr]})
        if any(e["outpoint"]["transactionId"] == txid for e in result["entries"]):
            return
        await asyncio.sleep(1)


# =============================================================================
# Main
# =============================================================================

async def main() -> None:
    if not RPC_URL:
        raise SystemExit("Set KASPA_RPC_URL to your testnet-10 node URL (e.g. ws://127.0.0.1:17210).")

    print(f"RISC Zero Groth16 zk-to-script — live on {NETWORK_ID}\n")

    client = RpcClient(url=RPC_URL, network_id=NETWORK_ID)
    print(f"Connecting to RPC host {RPC_URL}...")
    await client.connect(strategy="fallback")
    print("Connected\n")

    try:
        # Activation precheck: the zk precompile only runs where Toccata is active.
        info = await client.get_block_dag_info()
        daa = int(info["virtualDaaScore"])
        if daa < TOCCATA_TESTNET10:
            raise SystemExit(
                f"Toccata is not active yet on {NETWORK_ID} "
                f"(virtual DAA {daa:,} < activation {TOCCATA_TESTNET10:,}); "
                "the zk redeem would be rejected. Try again later."
            )

        # Build the zk scripts up front so we can show the P2SH address.
        redeem_script, sig_script = build_zk_scripts()
        p2sh_spk = pay_to_script_hash_script(redeem_script)
        p2sh_address = address_from_script_public_key(p2sh_spk, NETWORK_TYPE)
        print(f"Redeem script   {len(redeem_script) // 2} bytes")
        print(f"Sig script      {len(sig_script) // 2} bytes (carries the proof)")
        print(f"P2SH address    {p2sh_address.to_string()}\n")

        # Every run uses a fresh random key for funding.
        keypair = Keypair.random()
        funder_key = PrivateKey(keypair.private_key)
        funding_address = keypair.to_address(NETWORK_TYPE)
        print("Fund this address with testnet KAS (TKAS):")
        print(f"{funding_address.to_string()}\n")
        print("Polling for funds (continues automatically within a few seconds of funding)...\n")
        funding_utxos = await wait_for_funds(client, funding_address)

        # Commit: lock the funds into the zk P2SH.
        commit_txid, p2sh_value = await commit(client, funder_key, funding_utxos, redeem_script)
        await wait_until_accepted(client, p2sh_address, commit_txid)
        print("commit       funds locked behind the zk proof")
        print(f"  value      {p2sh_value:,} sompi")
        print(f"  tx         {commit_txid}\n")

        # Redeem: present the proof to unlock the P2SH (permissionless, unsigned).
        redeem_txid, payout = await redeem(
            client, commit_txid, p2sh_value, redeem_script, sig_script, funding_address
        )
        await wait_until_accepted(client, funding_address, redeem_txid)
        print("redeem       zk proof verified on-chain, funds released")
        print(f"  payout     {payout:,} sompi -> {funding_address.to_string()}")
        print(f"  tx         {redeem_txid}\n")

        print("Groth16 proof verified on-chain.")
    finally:
        await client.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
