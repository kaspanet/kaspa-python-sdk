# Calls

Once connected, every RPC method is `await client.<name>(...)`. Most take
either no arguments or a single dict; all return a dict (or list of dicts)
shaped like the rusty-kaspa wire protocol.

This page is a tour of the surface, grouped by what you're trying to do.
For full request/response shapes use the [API Reference](../../reference/index.md).

## Network information

```python
info       = await client.get_info()
dag_info   = await client.get_block_dag_info()
count      = await client.get_block_count()       # blockCount, headerCount
supply     = await client.get_coin_supply()        # circulatingSompi, maxSompi
network    = await client.get_current_network()
sync       = await client.get_sync_status()        # {"isSynced": bool}
```

`get_block_dag_info` is the closest the SDK has to a "where is the chain
right now" call: it returns network name, block count, virtual DAA score,
tip hashes, and pruning point in one go.

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

`get_utxos_by_addresses` is the go-to call when you need a one-shot UTXO
snapshot. For *continuous* UTXO tracking, subscribe instead — see
[Subscriptions](subscriptions.md).

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

## Transactions and mempool

```python
result  = await client.submit_transaction({
    "transaction": tx.serialize_to_dict(),
    "allowOrphan": False,
})
mempool = await client.get_mempool_entries({
    "includeOrphanPool": False,
    "filterTransactionPool": True,
})
entry   = await client.get_mempool_entry({"transactionId": "..."})
```

If you have a `PendingTransaction` from the [Transaction
Generator](../wallet-sdk/tx-generator.md), prefer `pending_tx.submit(client)` —
it serialises and submits in one call.

## Fees

```python
fee = await client.get_fee_estimate()
# fee["estimate"]["priorityBucket"] etc.

fee_x = await client.get_fee_estimate_experimental({"verbose": True})
```

## Peers

```python
peers     = await client.get_connected_peer_info()
addresses = await client.get_peer_addresses()

await client.add_peer({"peerAddress": "192.168.1.1:16111", "isPermanent": False})
await client.ban({"ip": "192.168.1.1"})
await client.unban({"ip": "192.168.1.1"})
```

These are administrative — you use them when you operate the node, not
when you're a client.

## System

```python
await client.ping()
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

A failing RPC call raises. Handle it like any other coroutine exception:

```python
try:
    info = await client.get_balance_by_address({"address": addr})
except Exception as exc:
    print("balance lookup failed:", exc)
```

Connection-level failures retry automatically (see
[Connecting](connecting.md)); the exception surface is for protocol-level
failures (invalid address, malformed request, node-side errors).

## Where to next

- [Subscriptions](subscriptions.md) — server-pushed notifications.
- [Wallet → Send Transaction](../wallet/send-transaction.md) — the managed
  Wallet wraps `submit_transaction` with sensible defaults.
- [Wallet SDK → Transaction Generator](../wallet-sdk/tx-generator.md) —
  build the transactions you submit through `submit_transaction`.
