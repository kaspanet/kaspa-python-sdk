# Calls

Once connected, every RPC method is `await client.<name>(...)`. Most
take no arguments or a single dict, and return a dict (or list of
dicts) shaped like the rusty-kaspa wire protocol.

This page is a brief tour of the available RPC calls, grouped by category. This is not an
exhaustive reference. For every call and its request/response model,
see [`RpcClient`](../../reference/Classes/RpcClient.md) in the API
Reference.

## Network information

```python
info       = await client.get_info()
dag_info   = await client.get_block_dag_info()
count      = await client.get_block_count()
supply     = await client.get_coin_supply()
network    = await client.get_current_network()
```

`get_block_dag_info` returns:

```python
{
    "network": "kaspa-mainnet",
    "blockCount": 12345678,
    "headerCount": 12345678,
    "tipHashes": ["..."],
    "difficulty": 1.23e15,
    "pastMedianTime": 1700000000000,
    "virtualParentHashes": ["..."],
    # ...plus virtualDaaScore, sink, pruningPointHash, etc.
}
```

## Balances and UTXOs

```python
balance  = await client.get_balance_by_address({"address": "kaspa:qz..."})
# {"balance": 100000000}

balances = await client.get_balances_by_addresses({
    "addresses": ["kaspa:qz...", "kaspa:qr..."],
})
# {"entries": [{"address": "kaspa:qz...", "balance": 100000000}, ...]}

utxos = await client.get_utxos_by_addresses({"addresses": ["kaspa:qz..."]})
for entry in utxos.get("entries", []):
    print(entry["outpoint"], entry["utxoEntry"]["amount"])
```

Balance amounts are in sompi (1 KAS = 100,000,000 sompi).

Use [`get_utxos_by_addresses`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.get_utxos_by_addresses) for one-shot queries or polling. For
continuous tracking, subscribe to
[`utxos-changed`](subscriptions.md#available-events), or use
[`UtxoContext`](../../reference/Classes/UtxoContext.md) for per-address tracking
on top of that subscription (see [UTXO Context](../wallet-sdk/utxo-context.md)).

## Blocks

```python
block  = await client.get_block({"hash": "...", "includeTransactions": True})
blocks = await client.get_blocks({
    "lowHash": "...",
    "includeBlocks": True,
    "includeTransactions": False,
})
template = await client.get_block_template({
    "payAddress": "kaspa:...",
    "extraData": [],
})
```

`get_block` returns `{"block": {...}}` where the inner block has
`header`, `transactions`, and `verboseData` keys; verbose data adds the
block hash, child hashes, and merge-set info.

## Virtual chain

Walk the selected-parent chain forward from a known block. Useful for
indexers and confirmation tracking — every accepted transaction id
passes through this stream.

```python
chain = await client.get_virtual_chain_from_block({
    "startHash": "...",
    "includeAcceptedTransactionIds": True,
    "minConfirmationCount": 10,        # optional; skip blocks with fewer confirmations
})

for h in chain["addedChainBlockHashes"]:
    print("+", h)
for h in chain["removedChainBlockHashes"]:
    print("-", h)
for entry in chain["acceptedTransactionIds"]:
    print(entry["acceptingBlockHash"], entry["acceptedTransactionIds"])
```

`addedChainBlockHashes` and `removedChainBlockHashes` give the chain
delta from `startHash` to the current sink. With
`includeAcceptedTransactionIds=True`, each entry maps an accepting
block hash to the transaction ids it accepted.

For a streaming version, subscribe to
[`virtual-chain-changed`](subscriptions.md#virtual-chain-progression).

### `get_virtual_chain_from_block_v2`

**Prefer V2 for new code.** It supersedes the V1 call above: it swaps
the boolean flag for a verbosity level and returns richer per-block
data. V1 is kept for backward compatibility.

```python
chain = await client.get_virtual_chain_from_block_v2({
    "startHash": "...",
    "dataVerbosityLevel": "Low",       # "None" | "Low" | "High" | "Full"
    "minConfirmationCount": 10,        # optional
})

for entry in chain["chainBlockAcceptedTransactions"]:
    header = entry.get("chainBlockHeader")        # populated at higher verbosity
    txs    = entry.get("acceptedTransactions", [])
    print(len(txs), "txs accepted under header:", header)
```

Verbosity levels:

- `"None"` — chain delta only (`addedChainBlockHashes`,
  `removedChainBlockHashes`).
- `"Low"` — adds accepted-transaction ids per chain block.
- `"High"` — adds the chain block header and per-transaction metadata.
- `"Full"` — full headers and full transactions.

Pick the lowest level that meets your needs; higher verbosity costs
bandwidth and node CPU.

## Transactions and mempool

```python
result  = await client.submit_transaction({
    "transaction": signed_tx,        # required: a Transaction instance, NOT a dict
    "allowOrphan": False,
})
# {"transactionId": "..."}

mempool = await client.get_mempool_entries({
    "includeOrphanPool": False,
    "filterTransactionPool": True,
})
entry   = await client.get_mempool_entry({
    "transactionId": "...",
    "includeOrphanPool": False,
    "filterTransactionPool": True,
})
```

[`submit_transaction`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.submit_transaction) is the one call where the request dict embeds a
real Python object: the `transaction` value must be a
[`Transaction`](../../reference/Classes/Transaction.md) instance
(passing a dict raises). Every other call on this page is dict-in,
dict-out.

If you have a
[`PendingTransaction`](../../reference/Classes/PendingTransaction.md)
from the [Transaction Generator](../wallet-sdk/tx-generator.md),
prefer [`pending_tx.submit(client)`](../../reference/Classes/PendingTransaction.md#kaspa.PendingTransaction.submit) — it serialises and submits in one
call. See [Submission](../transactions/submission.md) for the full
flow and `allowOrphan` semantics.

## Fees

```python
fee = await client.get_fee_estimate()
# {
#     "estimate": {
#         "priorityBucket": {"feerate": 1.0, "estimatedSeconds": 1.0},
#         "normalBuckets": [...],
#         "lowBuckets": [...],
#     }
# }

fee_x = await client.get_fee_estimate_experimental({"verbose": True})
```

See [Mass & Fees](../transactions/mass-and-fees.md) for how these
estimates feed into transaction construction.

## Peers

```python
peers     = await client.get_connected_peer_info()
addresses = await client.get_peer_addresses()

await client.add_peer({"peerAddress": "192.168.1.1:16111", "isPermanent": False})
await client.ban({"ip": "192.168.1.1"})
await client.unban({"ip": "192.168.1.1"})
```

These are administrative — for node operators, not clients.

## System

```python
await client.ping()
sync    = await client.get_sync_status()
server  = await client.get_server_info()
system  = await client.get_system_info()
metrics = await client.get_metrics({
    "processMetrics": True,
    "connectionMetrics": True,
    "bandwidthMetrics": True,
    "consensusMetrics": True,
    "storageMetrics": False,
    "customMetrics": False,
})
```

## Errors

Protocol-level failures (invalid address, malformed request, node-side
errors) raise a plain `Exception` — see
[Errors in the overview](overview.md#errors) for the full picture.
Connection-level failures retry automatically (see
[Connecting → Reconnects](connecting.md#reconnects)).

## Where to next

- [Subscriptions](subscriptions.md) — server-pushed notifications.
- [Wallet → Send Transaction](../wallet/send-transaction.md) — the
  managed [`Wallet`](../../reference/Classes/Wallet.md) wraps [`submit_transaction`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.submit_transaction) with sensible defaults.
- [Wallet SDK → Transaction Generator](../wallet-sdk/tx-generator.md) —
  build the transactions you submit via [`submit_transaction`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.submit_transaction).
