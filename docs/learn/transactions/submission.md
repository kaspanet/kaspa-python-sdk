# Submission & confirmation

Submitting a signed transaction hands it to a node, which gossips it to
the network and includes it in a block when capacity allows. The SDK
gives you two surfaces: `pending.submit(client)` for transactions
produced by the [Generator](../wallet-sdk/tx-generator.md), and
`client.submit_transaction(...)` for everything else.

## From a PendingTransaction

```python
tx_id = await pending.submit(client)
print(tx_id)
```

`pending.submit` serializes the underlying `Transaction` and calls
`submit_transaction` for you. This is the right path for transactions
the Generator built — including the managed
[Wallet](../wallet/send-transaction.md), which is built on the
Generator.

## Manual submission

```python
result = await client.submit_transaction({
    "transaction": signed_tx.serialize_to_dict(),
    "allowOrphan": False,
})
print(result["transactionId"])
```

The request takes:

- **`transaction`** — the wire-format dict you get from
  `Transaction.serialize_to_dict()` (or via the equivalent
  `pending.transaction.serialize_to_dict()`).
- **`allowOrphan`** — whether to keep the transaction in mempool when
  one of its inputs hasn't been seen yet (e.g. you submitted a chain of
  transactions out of order). Default to `False` unless you know you're
  submitting a chain.

The manual path is the right call when you need to ship the transaction
through another system — a co-signer, a relay, an offline signer —
before the node sees it. For round-tripping through that other system,
see [Serialization](serialization.md).

## What "submitted" means

Submission is *acceptance into the mempool*, not confirmation. The
return value is the transaction ID; the transaction is now eligible to
be included in a block.

A Kaspa transaction lifecycle has three observable states:

- **In mempool** — accepted by the node, waiting for inclusion. The
  transaction ID is returned from `submit_transaction`.
- **Virtual-chain accepted** — included in a block that's part of the
  canonical DAG ordering. Surfaces via the
  `virtual-chain-changed` notification.
- **Mature** — confirmed past the maturity threshold; the new UTXOs are
  spendable. Surfaces via a `Maturity` event on the managed
  [Wallet](../wallet/transaction-history.md) or the
  [`UtxoProcessor`](../wallet-sdk/utxo-processor.md).

For confirmation, the right gate is the `Maturity` event for the
specific transaction, not "wait N seconds" and not the first time it
appears in a block. See [Kaspa Concepts → Maturity](../concepts.md) for
the protocol view.

## Failures and retries

`submit_transaction` raises if the node rejects the transaction.
Common reasons:

- **`fee too low`** — recompute mass with `update_transaction_mass`
  *after* any input/output change, then re-sign.
- **`orphan`** — an input references a transaction the node hasn't seen.
  Either wait for the parent to land, or set `allowOrphan=True` when you
  intentionally submit a chain.
- **`already in mempool`** — the same `transaction_id` is already
  pending. Safe to ignore for retries.
- **`mass exceeded`** — the transaction is over
  `maximum_standard_transaction_mass()`. Split the inputs across
  multiple transactions; the Generator does this automatically when its
  input set is too large.

A transaction that was virtual-chain accepted *can* be reorged out — at
which point its outputs are no longer mature. The SDK surfaces this as
a `Reorg` event; see
[Wallet → Transaction History](../wallet/transaction-history.md).

## Where to next

- [Wallet → Send Transaction](../wallet/send-transaction.md) — the
  managed Wallet's send surface, which wraps all of this.
- [Wallet → Transaction History](../wallet/transaction-history.md) —
  observing maturity and reorgs.
- [Kaspa Concepts](../concepts.md) — virtual chain, DAA score, maturity.
