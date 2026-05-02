---
search:
  boost: 5
---

# RPC

Kaspa nodes expose an RPC API. This SDK provides
[`RpcClient`](../../reference/Classes/RpcClient.md) for interacting with the RPC API. One class for
connection management, request/response calls, and event subscriptions.

## An example

A complete script — connect to a public mainnet node, fetch DAG state,
disconnect:

```python
import asyncio
from kaspa import Resolver, RpcClient

async def main():
    client = RpcClient(resolver=Resolver(), network_id="mainnet")
    await client.connect()
    try:
        info = await client.get_block_dag_info()
        print(f"network={info['network']} blocks={info['blockCount']}")
    finally:
        await client.disconnect()

asyncio.run(main())
```

## Overview

[`RpcClient`](../../reference/Classes/RpcClient.md) is an async WebSocket client. Each instance:

- Connects to one node at a time.
- Reconnects automatically if the socket drops.
- Uses Borsh encoding by default — compact and faster to parse than
  JSON. Pass `encoding="json"` for the JSON wire format. See the
  [`Encoding`](../../reference/Enums/Encoding.md) enum.

[`RpcClient`](../../reference/Classes/RpcClient.md) is **not** an async context manager — wrap calls in
`try/finally` (or your own helper) to guarantee [`disconnect()`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.disconnect) runs.

## Two ways to point a client at a node

```python
# Resolver: let the SDK pick a public node for the network you want
client = RpcClient(resolver=Resolver(), network_id="mainnet")

# Direct URL: a known node you control or trust
client = RpcClient(url="wss://node.example.com:17110", network_id="mainnet")
```

See [Resolver](resolver.md) for node discovery and
[Connecting](connecting.md) for the connection lifecycle.

## Naming conventions

Two styles meet at this API and the seam is worth knowing about up
front:

- **Method names are Python snake_case**: `get_block_dag_info`,
  `subscribe_utxos_changed`.
- **Request and response dict keys are camelCase**: `includeTransactions`,
  `addedChainBlockHashes`, `payAddress`.

The camelCase keys mirror the rusty-kaspa wire protocol. Misspelled or
snake_cased keys (`include_transactions`) raise a `KeyError` at the
binding layer rather than being silently ignored. The one historical
exception is `submit_transaction`'s `allow_orphan`, which is accepted
with a `DeprecationWarning` — prefer `allowOrphan`.

Request and response shapes are `TypedDict`s in the bundled type
stubs (e.g. `GetBlockDagInfoResponse`), so an IDE will autocomplete
the camelCase keys for you.

## Errors

[`RpcClient`](../../reference/Classes/RpcClient.md) raises a plain `Exception` for protocol-, network-, and
validation-level failures (the binding layer doesn't currently expose
typed RPC error subclasses). Catch broadly and inspect the message:

```python
try:
    balance = await client.get_balance_by_address({"address": addr})
except Exception as exc:
    print("rpc call failed:", exc)
```

Connection drops are handled automatically — see
[Connecting → Reconnects](connecting.md#reconnects). The typed
exception classes under [`kaspa.exceptions`](../../reference/SUMMARY.md)
([`WalletRpcError`](../../reference/Exceptions/WalletRpcError.md),
[`WalletNotConnectedError`](../../reference/Exceptions/WalletNotConnectedError.md), etc.) come from the wallet layer, not raw
[`RpcClient`](../../reference/Classes/RpcClient.md) calls.

## RPC methods

- **[Calls](calls.md)** — request/response RPCs for network info,
  balances, blocks, mempool, fees, and peers.
- **[Subscriptions](subscriptions.md)** — node-pushed notifications for
  UTXO changes, new blocks, virtual chain updates, and DAA score
  changes. Each subscription pairs with an event listener.

## Where to next

- [Connecting](connecting.md) — connect, disconnect, retries, encoding.
- [Resolver](resolver.md) — node discovery.
- [Calls](calls.md) — RPC method catalog.
- [Subscriptions](subscriptions.md) — real-time notifications.
