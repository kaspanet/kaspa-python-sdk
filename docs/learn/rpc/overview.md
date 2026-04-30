# RPC

Kaspa nodes expose an RPC API. This SDK wraps it in
[`RpcClient`](../../reference/Classes/RpcClient.md) — one class for
connection management, request/response calls, and event subscriptions.

Higher-level SDK layers build on `RpcClient`. For example,
[`Wallet`](../wallet/overview.md) submits transactions and tracks UTXO
state through it.

## Overview

`RpcClient` is an async WebSocket client. Each instance:

- Connects to one node at a time.
- Reconnects automatically if the socket drops.
- Uses Borsh encoding by default — compact and faster to parse than
  JSON. Pass `encoding="json"` for the JSON wire format. See the
  [`Encoding`](../../reference/Enums/Encoding.md) enum.

## Two ways to point a client at a node

```python
# Resolver: let the SDK pick a public node for the network you want
client = RpcClient(resolver=Resolver(), network_id="mainnet")

# Direct URL: a known node you control or trust
client = RpcClient(url="wss://node.example.com:17110", network_id="mainnet")
```

See [Resolver](resolver.md) for node discovery and
[Connecting](connecting.md) for the connection lifecycle.

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
