# Serialization

Most transaction-shaped types — `Transaction`, `TransactionInput`,
`TransactionOutput`, `TransactionOutpoint`, `UtxoEntryReference` —
support `to_dict()` and `from_dict()`. The dict shape matches the
wRPC wire format the node accepts and produces.

## Round-tripping

```python
tx_dict = tx.to_dict()
restored = Transaction.from_dict(tx_dict)
assert restored == tx
```

The same shape works for the component types:

```python
inp_dict = inputs[0].to_dict()
restored_inp = TransactionInput.from_dict(inp_dict)

out_dict = outputs[0].to_dict()
restored_out = TransactionOutput.from_dict(out_dict)

ref_dict = utxo_ref.to_dict()
restored_ref = UtxoEntryReference.from_dict(ref_dict)
```

`to_dict()` produces a fresh Python dict — modifying it doesn't mutate
the source object. `from_dict()` raises on malformed input (missing
required keys, wrong types, invalid values).

## When you need this

Within a single process, you rarely need to round-trip — pass the
typed objects around. The dict form earns its place at process
boundaries:

- **Submission** — `client.submit_transaction({"transaction":
  tx.serialize_to_dict(), ...})` takes a dict, not a `Transaction`. See
  [Submission](submission.md).
- **Offline signing** — build on an online machine, serialize, sign on
  an air-gapped one, serialize again, send back, submit. The dict form
  is the natural transport.
- **Co-signer flows** — multisig where each cosigner signs in turn.
  Each step ships a dict; the next signer reads it back, signs, and
  forwards.
- **Persistence** — saving a pending transaction to disk or a queue for
  later submission. Store the dict (as JSON), not the typed object.

## `serialize_to_dict` vs `to_dict`

Both produce a dict matching the wRPC wire shape. `to_dict` is the
general-purpose Python conversion; `serialize_to_dict` (on
`Transaction`) is the form `submit_transaction` expects. In practice
they produce equivalent shapes — use `serialize_to_dict` when you're
about to submit, `to_dict` when you're shuttling the object somewhere
else.

## Where to next

- [Submission](submission.md) — where the dict form actually goes.
- [Inputs](inputs.md) and [Outputs](outputs.md) — the typed objects
  these dicts represent.
