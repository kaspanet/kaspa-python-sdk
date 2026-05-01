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

- **`TransactionOutpoint`** — `(transaction_id, index)`. The pointer
  to the output being spent.
- **`UtxoEntryReference`** — a cached copy of the *spent output*: its
  amount, lockup script, block DAA score, and coinbase flag. See
  [UTXO Context](../wallet-sdk/utxo-context.md) for how the SDK
  tracks these.
- **`signature_script`** — the unlocking script. Empty string at
  build time; filled when you sign. See [Signing](signing.md).
- **`sequence`** — sequence number. Leave at `0` unless you have a
  specific protocol-level reason.
- **`sig_op_count`** — number of signature operations this input
  performs (`1` for Schnorr/ECDSA, `>1` for multisig). Feeds into
  mass calculation.

## Build an input

From a UTXO dict returned by `client.get_utxos_by_addresses(...)`:

```python
from kaspa import TransactionInput, TransactionOutpoint, UtxoEntryReference

utxo = utxos["entries"][0]

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

Practical consequences:

- Forgetting `utxo=...` when building manually breaks signing. Always
  set it.
- A signed transaction can move between processes (offline signer,
  co-signer, relay) without the receiver needing the source node —
  every input carries what's needed.
- The Generator handles this — pass a list of `UtxoEntryReference`s
  (or a [`UtxoContext`](../wallet-sdk/utxo-context.md)) and it picks
  and wraps inputs internally.

## UTXO selection

Selecting inputs that sum to at least `amount + fee` is what the
[Transaction Generator](../wallet-sdk/tx-generator.md) handles. When
building manually:

- Sum the input values you intend to spend.
- Subtract output amounts and fee — the leftover becomes the change
  output.
- Order only matters to your downstream consumers; the protocol
  doesn't impose one.

For input ordering rules, signature aggregation, or "spend exactly
these UTXOs first", see the Generator's `priority_entries` option.

## Reading inputs back

```python
for inp in tx.inputs:
    print(inp.previous_outpoint.transaction_id, inp.previous_outpoint.index)
    print(inp.signature_script_as_hex)   # hex string, or None pre-sign
    print(inp.sig_op_count, inp.sequence)
    if inp.utxo:
        print(inp.utxo.amount, inp.utxo.script_public_key)
```

`signature_script_as_hex` returns the unlocking script after signing
as a hex string, or `None` if not yet signed.

## Where to next

- [Outputs](outputs.md) — the other half of a transaction.
- [Signing](signing.md) — what "filled at sign time" actually does.
- [Mass & Fees](mass-and-fees.md) — `sig_op_count` feeds the mass
  calculator.
- [UTXO Context](../wallet-sdk/utxo-context.md) — managed UTXO state
  the SDK keeps in sync with the chain.
