---
search:
  boost: 3
---

# UTXO Processor

A [`UtxoProcessor`](../../reference/Classes/UtxoProcessor.md) is the
engine that drives live UTXO tracking: it owns the wRPC subscription
and dispatches address-scoped UTXO events to one or more
[`UtxoContext`](../../reference/Classes/UtxoContext.md)s (see [UTXO Context](utxo-context.md)). The managed
[`Wallet`](../../reference/Classes/Wallet.md) (see [Wallet](../wallet/overview.md)) builds one internally; otherwise, you
build one and bind contexts to it.

**Read this page first**, then [UTXO Context](utxo-context.md).

## Construction

Build a [`UtxoProcessor`](../../reference/Classes/UtxoProcessor.md) on top of an [`RpcClient`](../../reference/Classes/RpcClient.md) and a [`NetworkId`](../../reference/Classes/NetworkId.md):

```python
from kaspa import NetworkId, Resolver, RpcClient, UtxoProcessor

client = RpcClient(resolver=Resolver(), network_id="testnet-10")
await client.connect()

processor = UtxoProcessor(client, NetworkId("testnet-10"))
await processor.start()

# ...later...
await processor.stop()
await client.disconnect()
```

`start()` runs a short handshake against the node:

1. Calls `get_server_info`, validates RPC version + network match,
   bails with `UtxoIndexNotEnabled` if the node has no UTXO index.
2. Opens a single wRPC listener and subscribes to
   `VirtualDaaScoreChanged` only — **no** `UtxosChanged` subscription
   yet. UTXO subscriptions are opened later, per-address, when a
   [`UtxoContext`](../../reference/Classes/UtxoContext.md) calls `track_addresses(...)`.
3. Emits `utxo-proc-start`.

`stop()` is the matching shutdown. Without `start()`, bound contexts
stay empty.

## Properties

| Property | Meaning |
| --- | --- |
| `processor.rpc` | The [`RpcClient`](../../reference/Classes/RpcClient.md) it's reading from. |
| `processor.network_id` | The [`NetworkId`](../../reference/Classes/NetworkId.md) it was constructed with. |
| `processor.is_active` | `True` after `start()`, `False` after `stop()` or before. |

## Events

The processor has its own event surface — a smaller, lower-level
cousin of the
[managed Wallet's events](../wallet/events.md).

```python
def on_event(event):
    print(event["type"], event.get("data"))

# Single event
processor.add_event_listener("maturity", on_event)

# A list of events — supported here (the wallet's listener takes one
# event at a time)
processor.add_event_listener(
    ["utxo-proc-start", "utxo-proc-stop", "pending", "maturity",
     "reorg", "stasis", "discovery", "balance", "utxo-proc-error", "error"],
    on_event,
)

# Every event
processor.add_event_listener("all", on_event)
```

| Event | When it fires |
| --- | --- |
| `utxo-proc-start` / `utxo-proc-stop` | Processor entered / left the active state. |
| `pending` | A UTXO landed for a tracked address but isn't mature yet. |
| `maturity` | A previously-pending UTXO crossed the maturity depth. |
| `reorg`, `stasis` | A UTXO was unwound or coinbase-locked. |
| `discovery` | A scan-time discovery hit. |
| `balance` | A bound [`UtxoContext`](../../reference/Classes/UtxoContext.md)'s balance changed. |
| `utxo-proc-error`, `error` | Something went wrong. |

[`UtxoProcessorEvent`](../../reference/Enums/UtxoProcessorEvent.md)
is the enum form if you prefer typed values over kebab strings.

## Reconnects

The underlying [`RpcClient`](../../reference/Classes/RpcClient.md) reconnects automatically when the WebSocket
drops. The processor reacts:

- On disconnect: emits `utxo-proc-stop`. Bound contexts stay alive
  — their address sets and in-memory UTXOs are kept.
- On the next successful reconnect: re-runs the handshake, re-opens
  the DAA-score subscription, and re-registers `UtxosChanged` for
  every address that's still tracked across all bound contexts.
  Re-emits `utxo-proc-start`.

You don't need to rebuild contexts on reconnect. Gate work that needs
fresh state on `processor.is_active` (or on `utxo-proc-start`).

## Coordinating with `asyncio`

Listener callbacks may run on a background thread. To signal an
`asyncio.Event` from one, bridge through the loop:

```python
loop = asyncio.get_running_loop()
got_start = asyncio.Event()

def on_event(event):
    if event.get("type") == "utxo-proc-start":
        loop.call_soon_threadsafe(got_start.set)

processor.add_event_listener("utxo-proc-start", on_event)
await processor.start()
await got_start.wait()
```

This is the same pattern the managed [`Wallet`](../../reference/Classes/Wallet.md) uses internally.

## Where to next

- [UTXO Context](utxo-context.md) — bind a context to this processor.
- [Transaction Generator](tx-generator.md) — pass a bound context as
  `entries`.
- [Wallet → Sync State](../wallet/sync-state.md) — the same handshake
  surfaced one level up, with the full `SyncState` payload reference.
