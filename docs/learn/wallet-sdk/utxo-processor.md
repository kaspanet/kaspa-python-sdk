# UTXO Processor

A `UtxoProcessor` subscribes to a node's UTXO and virtual-chain
notifications and dispatches them to one or more
[UTXO Contexts](utxo-context.md). It's the engine that makes context
tracking work. If you're using the managed
[Wallet](../wallet/overview.md), one is constructed for you. If you're not,
you build one yourself and bind contexts to it.

## Construction

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

`start()` is what activates the processor — it begins subscribing to
node notifications and forwarding them. `stop()` is the matching
shutdown. Without `start()`, contexts bound to it stay empty.

## Properties

| Property | Meaning |
| --- | --- |
| `processor.rpc` | The `RpcClient` it's reading from. |
| `processor.network_id` | The `NetworkId` it was constructed with. |
| `processor.is_active` | `True` after `start()`, `False` after `stop()` or before. |

## Events

The processor has its own event surface — a smaller, lower-level cousin
of the [managed Wallet's events](../wallet/transaction-history.md).
Listeners use the same shape:

```python
def on_event(event):
    print(event["type"], event.get("data"))

processor.add_event_listener(
    ["utxo-proc-start", "utxo-proc-stop", "pending", "maturity",
     "reorg", "stasis", "discovery", "balance", "utxo-proc-error", "error"],
    on_event,
)
```

Common events:

| Event | When it fires |
| --- | --- |
| `utxo-proc-start` / `utxo-proc-stop` | Processor entered / left the active state. |
| `pending` | A UTXO landed for a tracked address but isn't mature yet. |
| `maturity` | A previously-pending UTXO crossed the maturity depth. |
| `reorg`, `stasis` | A UTXO was unwound or coinbase-locked. |
| `discovery` | A scan-time discovery hit. |
| `balance` | A bound `UtxoContext`'s balance changed. |
| `utxo-proc-error`, `error` | Something went wrong. |

`UtxoProcessorEvent` is the enum form if you'd rather pass it as a
typed value than a string.

## Coordinating with `asyncio`

Listener callbacks may run on a background thread. If you need to
signal an `asyncio.Event` from one, bridge through the loop:

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

This is the same pattern the managed Wallet uses internally.

## Where to next

- [UTXO Context](utxo-context.md) — bind a context to the processor.
- [Transaction Generator](tx-generator.md) — pass that context as
  `entries`.
- [Wallet → Architecture](../wallet/architecture.md) — how the managed
  Wallet drives a `UtxoProcessor` for you.
