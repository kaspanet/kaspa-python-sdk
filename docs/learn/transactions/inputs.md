---
search:
  boost: 3
---

# Inputs

A transaction's inputs say *which UTXOs are being spent*. Each input
points at a previous output by `(transaction_id, index)` and carries
the script that proves the spender is allowed to claim it.

## Types involved

```
TransactionInput
  previous_outpoint: TransactionOutpoint(transaction_id, index)
  signature_script  (filled at sign time)
  sequence
  sig_op_count
  utxo: UtxoEntryReference   # optional, but you almost always want it set
```

- **[`TransactionOutpoint`](../../reference/Classes/TransactionOutpoint.md)** — `(transaction_id, index)`. The pointer
  to the output being spent.
- **[`UtxoEntryReference`](../../reference/Classes/UtxoEntryReference.md)** — a cached copy of the *spent output*: its
  amount, lockup script, block DAA score, and coinbase flag.
- **`signature_script`** — the unlocking script. Empty (`""`) at
  build time; filled when you sign. See [Signing](signing.md).
- **`sequence`** — sequence number. Leave at `0` unless you have a
  specific protocol-level reason.
- **`sig_op_count`** — number of signature operations this input
  performs (`1` for Schnorr/ECDSA, `>1` for multisig). Feeds into
  mass calculation.

## Build an input

From a UTXO dict returned by [`client.get_utxos_by_addresses(...)`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.get_utxos_by_addresses):

```python
from kaspa import TransactionInput, TransactionOutpoint, UtxoEntryReference

resp = await client.get_utxos_by_addresses({"addresses": [my_address]})
utxo = resp["entries"][0]

inp = TransactionInput(
    previous_outpoint=TransactionOutpoint(
        transaction_id=utxo["outpoint"]["transactionId"],
        index=utxo["outpoint"]["index"],
    ),
    signature_script="",          # filled at sign time
    sequence=0,
    sig_op_count=1,
    utxo=UtxoEntryReference(utxo),
)
```

## Why inputs carry a UtxoEntryReference

Kaspa signs over the spent output's amount and lockup, not just the
outpoint. The SDK can't sign correctly without that context, so
`TransactionInput.utxo` *attaches* it directly — no node round-trip
needed.

Consequences:

- Forgetting `utxo=...` when building manually breaks signing.
- A signed transaction can move between processes (offline signer,
  co-signer, relay) without the receiver needing the source node —
  every input carries what's needed.
- The [`Generator`](../../reference/Classes/Generator.md) handles this for you. Pass [`UtxoEntryReference`](../../reference/Classes/UtxoEntryReference.md)s (or
  a [`UtxoContext`](../../reference/Classes/UtxoContext.md) — see [UTXO Context](../wallet-sdk/utxo-context.md)) and it picks and
  wraps inputs internally.

## Selecting which UTXOs to spend

The [`Generator`](../../reference/Classes/Generator.md) (see [Transaction Generator](../wallet-sdk/tx-generator.md)) handles
selection. When building manually, sum input values until the total
covers `outputs + fee`, then route the remainder to a change output.
See [Outputs → Change](outputs.md#change-outputs).

## Reading inputs back

```python
for inp in tx.inputs:
    print(inp.previous_outpoint.transaction_id, inp.previous_outpoint.index)
    print(inp.signature_script_as_hex)   # hex string after signing, None before
    print(inp.sig_op_count, inp.sequence)
    if inp.utxo:
        print(inp.utxo.amount, inp.utxo.script_public_key)
```
