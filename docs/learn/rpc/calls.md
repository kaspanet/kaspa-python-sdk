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

## Balances and UTXOs

```python
balance  = await client.get_balance_by_address({"address": "kaspa:qz..."})
balances = await client.get_balances_by_addresses({
    "addresses": ["kaspa:qz...", "kaspa:qr..."],
})

utxos = await client.get_utxos_by_addresses({"addresses": ["kaspa:qz..."]})
for entry in utxos.get("entries", []):
    print(entry["outpoint"], entry["utxoEntry"]["amount"])
```

Use `get_utxos_by_addresses` for one-shot queries or polling. For
continuous tracking, subscribe to
[`utxos-changed`](subscriptions.md#available-events), or use
[`UtxoContext`](../wallet-sdk/utxo-context.md) for per-address tracking
on top of that subscription.

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

V2 swaps the boolean flag for a verbosity level and returns richer
per-block data:

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
    "transaction": signed_tx,        # a Transaction instance
    "allowOrphan": False,
})
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

If you have a
[`PendingTransaction`](../../reference/Classes/PendingTransaction.md)
from the [Transaction Generator](../wallet-sdk/tx-generator.md),
prefer `pending_tx.submit(client)` — it serialises and submits in one
call. See [Submission](../transactions/submission.md) for the full
flow and `allowOrphan` semantics.

## Fees

```python
fee = await client.get_fee_estimate()
# fee["estimate"]["priorityBucket"] etc.

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

A failing RPC call raises. Handle it like any other coroutine
exception:

```python
try:
    info = await client.get_balance_by_address({"address": addr})
except Exception as exc:
    print("balance lookup failed:", exc)
```

Connection-level failures retry automatically (see
[Connecting](connecting.md#reconnects)). The exception surface is for
protocol-level failures: invalid address, malformed request, node-side
errors.

## Where to next

- [Subscriptions](subscriptions.md) — server-pushed notifications.
- [Wallet → Send Transaction](../wallet/send-transaction.md) — the
  managed Wallet wraps `submit_transaction` with sensible defaults.
- [Wallet SDK → Transaction Generator](../wallet-sdk/tx-generator.md) —
  build the transactions you submit via `submit_transaction`.
