---
search:
  boost: 3
---

# Subscriptions

Subscriptions let
[`RpcClient`](../../reference/Classes/RpcClient.md) receive a live feed
of node events. The node pushes events as they happen; the client
invokes the callbacks you registered.

## The two-step pattern

Every subscription has two parts:

1. **A listener** — a Python callback registered via
   [`add_event_listener("<event>", callback)`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.add_event_listener).
2. **A subscription** — `await client.subscribe_<event>(...)` tells the
   node to start streaming.

Both halves are required. A listener with no subscription receives
nothing; a subscription with no listener silently drops events.

```python
def on_utxo_change(event):
    print("UTXO change:", event)

client.add_event_listener("utxos-changed", on_utxo_change)
await client.subscribe_utxos_changed([Address("kaspa:qz...")])
```

## Available events

Each `subscribe_*` has a matching `unsubscribe_*` with the same
argument shape.

| Event name | Subscribe call | Arguments | Event payload |
| --- | --- | --- | --- |
| `utxos-changed` | [`subscribe_utxos_changed`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.subscribe_utxos_changed) | `addresses: list[Address]` | [`UtxosChangedEvent`](../../reference/TypedDicts/UtxosChangedEvent.md) |
| `block-added` | [`subscribe_block_added`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.subscribe_block_added) | — | [`BlockAddedEvent`](../../reference/TypedDicts/BlockAddedEvent.md) |
| `virtual-chain-changed` | [`subscribe_virtual_chain_changed`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.subscribe_virtual_chain_changed) | `include_accepted_transaction_ids: bool` | [`VirtualChainChangedEvent`](../../reference/TypedDicts/VirtualChainChangedEvent.md) |
| `virtual-daa-score-changed` | [`subscribe_virtual_daa_score_changed`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.subscribe_virtual_daa_score_changed) | — | [`VirtualDaaScoreChangedEvent`](../../reference/TypedDicts/VirtualDaaScoreChangedEvent.md) |
| `sink-blue-score-changed` | [`subscribe_sink_blue_score_changed`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.subscribe_sink_blue_score_changed) | — | [`SinkBlueScoreChangedEvent`](../../reference/TypedDicts/SinkBlueScoreChangedEvent.md) |
| `finality-conflict` | [`subscribe_finality_conflict`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.subscribe_finality_conflict) | — | [`FinalityConflictEvent`](../../reference/TypedDicts/FinalityConflictEvent.md) |
| `finality-conflict-resolved` | [`subscribe_finality_conflict_resolved`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.subscribe_finality_conflict_resolved) | — | [`FinalityConflictResolvedEvent`](../../reference/TypedDicts/FinalityConflictResolvedEvent.md) |
| `new-block-template` | [`subscribe_new_block_template`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.subscribe_new_block_template) | — | [`NewBlockTemplateEvent`](../../reference/TypedDicts/NewBlockTemplateEvent.md) |
| `pruning-point-utxo-set-override` | [`subscribe_pruning_point_utxo_set_override`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.subscribe_pruning_point_utxo_set_override) | — | [`PruningPointUtxoSetOverrideEvent`](../../reference/TypedDicts/PruningPointUtxoSetOverrideEvent.md) |

Event names also map to the
[`NotificationEvent`](../../reference/Enums/NotificationEvent.md) enum
if you prefer typed variants over kebab-case strings.

The client also emits two control events that don't require a
`subscribe_*` call — just register a listener:

| Event name | Fires when | Event payload |
| --- | --- | --- |
| `connect` | The WebSocket has connected (including after a reconnect). | [`ConnectEvent`](../../reference/TypedDicts/ConnectEvent.md) |
| `disconnect` | The WebSocket has dropped. | [`DisconnectEvent`](../../reference/TypedDicts/DisconnectEvent.md) |

Use these to track connection state without polling `client.is_connected`.

### Listening to all events

Pass the special `"all"` event name (or `NotificationEvent.All`) to
register one callback for every notification — node-pushed events plus
`connect` / `disconnect`. You still need `subscribe_*` for any
node-pushed event you want to receive; `"all"` only multiplexes
delivery, it doesn't subscribe on your behalf.

```python
def on_any(event):
    print(event["type"], event)

client.add_event_listener("all", on_any)
await client.subscribe_block_added()
await client.subscribe_virtual_daa_score_changed()
```

## Event payload shape

Every callback receives a `dict` with a `"type"` key naming the
event. The remaining keys depend on the event.

### utxos-changed

[`UtxosChangedEvent`](../../reference/TypedDicts/UtxosChangedEvent.md)
is the only event that does *not* nest its body under `"data"` — it's
flattened so callbacks can read `event["added"]` directly. The
`"added"` and `"removed"` lists hold
[`RpcUtxosByAddressesEntry`](../../reference/TypedDicts/RpcUtxosByAddressesEntry.md)
items.

The `"type"` value is the PascalCase variant name (`"UtxosChanged"`,
`"BlockAdded"`, …) — the kebab-case form (`"utxos-changed"`,
`"block-added"`, …) is only used when registering listeners.

```python
{
    "type": "UtxosChanged",
    "added": [
        {
            "address": "kaspa:qz...",
            "outpoint": {"transactionId": "...", "index": 0},
            "utxoEntry": {
                "amount": 100000000,
                "scriptPublicKey": {"version": 0, "script": "..."},
                "blockDaaScore": 123456789,
                "isCoinbase": False,
            },
        },
    ],
    "removed": [],
}
```

### All other node-pushed events

A `"data"` key holds the notification body. Each event has a wrapper
TypedDict (e.g.
[`BlockAddedEvent`](../../reference/TypedDicts/BlockAddedEvent.md)) and
a body TypedDict (e.g.
[`RpcBlockAddedNotification`](../../reference/TypedDicts/RpcBlockAddedNotification.md)).
See the [Available events](#available-events) table for the full list.
For example, a `virtual-daa-score-changed` callback receives:

```python
{
    "type": "VirtualDaaScoreChanged",
    "data": {
        "virtualDaaScore": 123456789,
    },
}
```

### connect / disconnect

[`ConnectEvent`](../../reference/TypedDicts/ConnectEvent.md) and
[`DisconnectEvent`](../../reference/TypedDicts/DisconnectEvent.md) carry
a single `"rpc"` key holding the node URL as a string:

```python
{"type": "connect", "rpc": "wss://node.example.com:17110"}
```

The bundled
[`Notification`](../../reference/Classes/Notification.md) class wraps
notifications internally; for callback type hints, use the per-event
TypedDicts above.

## Examples

### Watching addresses for UTXO changes

Pass [`Address`](../../reference/Classes/Address.md) instances (or
strings parsed by `Address(...)`):

```python
from kaspa import Address

addresses = [Address("kaspa:qz...")]

def on_change(event):
    for added in event.get("added", []):
        print("+", added["utxoEntry"]["amount"])
    for removed in event.get("removed", []):
        print("-", removed["utxoEntry"]["amount"])

client.add_event_listener("utxos-changed", on_change)
await client.subscribe_utxos_changed(addresses)

# ... later ...
await client.unsubscribe_utxos_changed(addresses)
```

To watch a managed-wallet account instead of raw addresses, use the
[`Wallet`](../../reference/Classes/Wallet.md)'s `Balance` and `Maturity` events — see
[Wallet → Events](../wallet/events.md).

### Block events

```python
def on_block(event):
    print("new block:", event["data"]["block"]["header"]["hash"])

client.add_event_listener("block-added", on_block)
await client.subscribe_block_added()
```

### Virtual chain progression

```python
def on_chain(event):
    data = event["data"]
    print("added:", data["addedChainBlockHashes"])
    print("removed:", data["removedChainBlockHashes"])

client.add_event_listener("virtual-chain-changed", on_chain)
await client.subscribe_virtual_chain_changed(include_accepted_transaction_ids=True)
```

With `include_accepted_transaction_ids=True`, the payload doubles as a
confirmation feed — every accepted transaction id appears in
`event["data"]["acceptedTransactionIds"]`. For the one-shot equivalent,
see [`get_virtual_chain_from_block`](calls.md#virtual-chain).

### Connection state

```python
def on_connect(event):
    print("connected to", event["rpc"])

def on_disconnect(event):
    print("disconnected from", event["rpc"])

client.add_event_listener("connect", on_connect)
client.add_event_listener("disconnect", on_disconnect)
```

No `subscribe_*` is needed — the client emits these itself when the
WebSocket transitions.

## Listener bookkeeping

```python
client.add_event_listener("block-added", callback)             # add
client.add_event_listener("block-added", callback, extra)      # forward extra arg

client.remove_event_listener("block-added", callback)          # remove specific
client.remove_event_listener("block-added")                    # remove all for event
client.remove_all_event_listeners()                            # remove all globally
```

Listeners outlive a single subscription cycle. Re-subscribing after an
unsubscribe does *not* re-fire previously delivered events. To catch
up, do a one-shot
[`get_utxos_by_addresses`](calls.md#balances-and-utxos) (or equivalent)
before re-subscribing.

## Where to next

- [Calls](calls.md) — the request/response side of the API.
- [`RpcClient`](../../reference/Classes/RpcClient.md) — full
  `subscribe_*` / `unsubscribe_*` / [`add_event_listener`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.add_event_listener) reference.
- [`UtxoProcessor`](../../reference/Classes/UtxoProcessor.md) (see [UTXO Processor](../wallet-sdk/utxo-processor.md)) and
  [`UtxoContext`](../../reference/Classes/UtxoContext.md) (see [UTXO Context](../wallet-sdk/utxo-context.md)) — higher-level UTXO
  tracking built on `utxos-changed`.
- [Wallet → Events](../wallet/events.md) — the managed [`Wallet`](../../reference/Classes/Wallet.md)'s
  higher-level event surface.
