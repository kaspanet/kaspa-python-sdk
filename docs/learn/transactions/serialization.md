---
search:
  boost: 3
---

# Serialization

The transaction-shaped types —
[`Transaction`](../../reference/Classes/Transaction.md),
[`TransactionInput`](../../reference/Classes/TransactionInput.md),
[`TransactionOutput`](../../reference/Classes/TransactionOutput.md),
[`TransactionOutpoint`](../../reference/Classes/TransactionOutpoint.md),
[`UtxoEntryReference`](../../reference/Classes/UtxoEntryReference.md)
— support `to_dict()` and `from_dict()`. The dict shape matches the
wRPC wire format the node accepts and produces.

## Round-tripping

```python
tx_dict = tx.to_dict()
restored = Transaction.from_dict(tx_dict)
assert restored == tx
```

[`to_dict()`](../../reference/Classes/Transaction.md) returns a fresh Python dict — modifying it doesn't
mutate the source object. [`from_dict()`](../../reference/Classes/Transaction.md) raises on malformed input
(missing required keys, wrong types, invalid values).

## What the dict looks like

```python
{
  "id":           "ab12...",          # transaction id, hex
  "version":      0,
  "inputs":       [{ "previousOutpoint": {...}, "signatureScript": "...", "sequence": 0, "sigOpCount": 1, "computeBudget": 0 }, ...],
                  # computeBudget is non-zero only on version-1 (covenant) transactions, where sigOpCount must be 0
  "outputs":      [{ "value": 500000000, "scriptPublicKey": {"version": 0, "script": "..."} }, ...],
  "lockTime":     0,
  "subnetworkId": "0000000000000000000000000000000000000000",
  "gas":          0,
  "payload":      "",                   # hex string
  "mass":         12345,
}
```

Field names use camelCase (wRPC convention), unlike the snake_case
Python class attributes.

## When you need this

Within a single process, you rarely need to round-trip — pass typed
objects around. The dict form earns its place at process boundaries:

- **Offline signing** — build on an online machine, serialize, sign
  on an air-gapped one, serialize again, send back, submit. The dict
  is the natural transport.
- **Co-signer flows** — multisig where each cosigner signs in turn.
  Each step ships a dict; the next signer reads, signs, and forwards.
- **Persistence** — saving a pending transaction to disk or a queue.
  Store the dict (as JSON), not the typed object.

For submission itself, [`client.submit_transaction({"transaction": ...})`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.submit_transaction)
requires a [`Transaction`](../../reference/Classes/Transaction.md) instance; passing a dict raises.
If the transaction arrives as a dict (from another process, a queue, or disk), rebuild it with
[`Transaction.from_dict()`](../../reference/Classes/Transaction.md) first. See [Submission](submission.md).
