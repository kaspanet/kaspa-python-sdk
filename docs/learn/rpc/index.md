# RPC

The `RpcClient` provides a connection to a Kaspa node, allowing you to interact via Kaspa's RPC calls. This includes query methods, submission methods (e.g. submitting a transaction), and event subscriptions.

Other layers in the SDK depend on RPC Client. For example, the `Wallet` class submits transactions and monitors UTXO state via RPC.

## Overview

A persistent, asynchronous WebSocket client. Each instance:

- Connects to one node at a time.
- Is async-first - every method is a coroutine.
- Handles connection state - when a connection drops the client attempts to reconnect automatically.
- Encodes in Borsh by default - Borsh is more compact and faster to parse than JSON. Pass `encoding="json"` if you need the JSON wire format.

## Two ways to point a client at a node

```python
# Resolver: let the SDK pick a public node for the network you want
client = RpcClient(resolver=Resolver(), network_id="mainnet")

# Direct URL: a known node you control or trust
client = RpcClient(url="wss://node.example.com:17110", network_id="mainnet")
```

See [Resolver](resolver.md) for the discovery mechanism, and
[Connecting](connecting.md) for the connection lifecycle and options.

## What you can do once connected

- **[Calls](calls.md)** — request/response RPCs for network info, balances,
  blocks, mempool, fee estimation, peer management, etc.
- **[Subscriptions](subscriptions.md)** — node-pushed notifications for
  UTXO changes, new blocks, virtual chain updates, and DAA score
  changes. Each subscription is paired with an event listener you
  register on the client.

## Where to next

- [Connecting](connecting.md) — connect, disconnect, retries, encoding.
- [Resolver](resolver.md) — how node discovery works.
- [Calls](calls.md) — the RPC method catalog.
- [Subscriptions](subscriptions.md) — real-time notifications.
