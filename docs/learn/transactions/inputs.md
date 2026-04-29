# Inputs

A transaction's inputs say *which UTXOs are being spent*. Each input
points at a previous output by `(transaction_id, index)` and carries the
script that proves the spender is allowed to claim it.

## Types involved

```
TransactionInput
  previous_outpoint: TransactionOutpoint(transaction_id, index)
  signature_script  (filled at sign time)
  sequence
  sig_op_count
  utxo: UtxoEntryReference   # optional, but you almost always want it set
```

- **`TransactionOutpoint`** — `(transaction_id, index)`. The pointer to
  the output you're spending.
- **`UtxoEntryReference`** — the cached copy of the *spent output*: its
  amount, its lockup script, the block DAA score it landed in, and
  whether it's a coinbase. See [UTXO Context](../wallet-sdk/utxo-context.md)
  for how the SDK tracks these.
- **`signature_script`** — the unlocking script. Empty string at build
  time; filled when you sign. See [Signing](signing.md).
- **`sequence`** — sequence number; leave at `0` unless you have a
  specific protocol-level reason.
- **`sig_op_count`** — how many signature operations this input
  performs (`1` for a normal Schnorr or ECDSA spend, `>1` for multisig).
  This feeds into mass calculation.

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
outpoint. The SDK can't sign an input correctly without that context, so
`TransactionInput.utxo` exists to *attach* it directly — no node
round-trip needed.

A few practical consequences:

- If you build inputs by hand and forget the `utxo=...` arg, signing
  will fail. Always set it.
- A signed transaction can be moved between processes (offline signer,
  co-signer, relay) without the receiving side needing access to the
  source node, because every input carries what's needed.
- The Generator does this for you — you hand it a list of
  `UtxoEntryReference`s (or a [`UtxoContext`](../wallet-sdk/utxo-context.md))
  and it picks and wraps inputs internally.

## UTXO selection

Selecting inputs that sum to at least `amount + fee` is what the
[Transaction Generator](../wallet-sdk/tx-generator.md) handles. When
building manually:

- Sum the input values you intend to spend.
- Subtract output amounts and fee — the leftover becomes the change
  output.
- Order matters only insofar as your downstream consumers care; protocol
  rules don't impose an order.

For input ordering rules, signature aggregation, or "spend exactly these
UTXOs first" semantics, see the Generator's `priority_entries` option.

## Reading inputs back

```python
for inp in tx.inputs:
    print(inp.previous_outpoint.transaction_id, inp.previous_outpoint.index)
    print(inp.signature_script_as_hex)   # hex string, or None pre-sign
    print(inp.sig_op_count, inp.sequence)
    if inp.utxo:
        print(inp.utxo.amount, inp.utxo.script_public_key)
```

`signature_script_as_hex` returns the unlocking script after signing as
a hex string (or `None` if the input hasn't been signed yet).

## Where to next

- [Outputs](outputs.md) — the other half of a transaction.
- [Signing](signing.md) — what "filled at sign time" actually does.
- [UTXO Context](../wallet-sdk/utxo-context.md) — managed UTXO state
  the SDK keeps in sync with the chain.
