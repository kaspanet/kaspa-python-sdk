# Submission & confirmation

Submitting a signed transaction hands it to a node, which gossips it
to the network and includes it in a block when capacity allows. Two
surfaces: [`pending.submit(client)`](../../reference/Classes/PendingTransaction.md#kaspa.PendingTransaction.submit) for transactions produced by the
[`Generator`](../../reference/Classes/Generator.md) (see [Transaction Generator](../wallet-sdk/tx-generator.md)), and
[`client.submit_transaction(...)`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.submit_transaction) (see [RPC → Calls](../rpc/calls.md#transactions-and-mempool))
for everything else.

## From a PendingTransaction

```python
tx_id = await pending.submit(client)
print(tx_id)
```

[`pending.submit`](../../reference/Classes/PendingTransaction.md#kaspa.PendingTransaction.submit) calls [`submit_transaction`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.submit_transaction) for you. The right path
for [`Generator`](../../reference/Classes/Generator.md)-built transactions — including the managed
[`Wallet`](../../reference/Classes/Wallet.md) (see [Send Transaction](../wallet/send-transaction.md)), which is built on the
[`Generator`](../../reference/Classes/Generator.md).

## Manual submission

```python
result = await client.submit_transaction({
    "transaction": signed_tx,
    "allowOrphan": False,
})
print(result["transactionId"])
```

The request takes:

- **`transaction`** — a signed [`Transaction`](../../reference/Classes/Transaction.md) (or its dict form via
  [`Transaction.to_dict()`](../../reference/Classes/Transaction.md) when shipping through another system; see
  [Serialization](serialization.md)).
- **`allowOrphan`** — whether to keep the transaction in the mempool
  when an input hasn't been seen yet (e.g. submitting a chain out of
  order). Default `False` unless you know you're submitting a chain.

## What "submitted" means

Submission is *acceptance into the mempool*, not confirmation. The
return value is the transaction ID; the transaction is now eligible
for inclusion in a block.

A Kaspa transaction lifecycle has three observable states:

- **In mempool** — accepted by the node, waiting for inclusion. ID
  returned from [`submit_transaction`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.submit_transaction).
- **Virtual-chain accepted** — included in a block that's part of the
  canonical DAG ordering. Surfaces via the
  [`virtual-chain-changed`](../rpc/subscriptions.md#virtual-chain-progression)
  notification.
- **Mature** — confirmed past the maturity threshold; new UTXOs are
  spendable. Surfaces via a `Maturity` event on the managed
  [`Wallet`](../../reference/Classes/Wallet.md) (see [Wallet → Events](../wallet/events.md)) or the
  [`UtxoProcessor`](../../reference/Classes/UtxoProcessor.md) (see [UTXO Processor](../wallet-sdk/utxo-processor.md)).

The right gate for confirmation is the `Maturity` event for the
specific transaction, not "wait N seconds" and not the first time it
appears in a block.

## Failures and retries

[`submit_transaction`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.submit_transaction) raises if the node rejects the transaction.
Common reasons:

- **`fee too low`** — recompute mass with [`update_transaction_mass`](../../reference/Functions/update_transaction_mass.md)
  *after* any input/output change, then re-sign.
- **`orphan`** — an input references a transaction the node hasn't
  seen. Wait for the parent to land, or set `allowOrphan=True` when
  intentionally submitting a chain.
- **`already in mempool`** — the same `transaction_id` is already
  pending. Safe to ignore.
- **`mass exceeded`** — the transaction is over
  [`maximum_standard_transaction_mass()`](../../reference/Functions/maximum_standard_transaction_mass.md). Split the inputs across
  multiple transactions; the [`Generator`](../../reference/Classes/Generator.md) does this automatically.

A virtual-chain-accepted transaction *can* be reorged out — at which
point its outputs are no longer mature. The SDK surfaces this as a
`Reorg` event; see
[Wallet → Events](../wallet/events.md).
