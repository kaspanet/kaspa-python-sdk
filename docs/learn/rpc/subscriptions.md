# Subscriptions

Subscriptions turn the `RpcClient` from a request/response API into a live
feed. The node pushes events; the client invokes callbacks you registered.
You typically use them to react to UTXO changes for an address you care
about, or to track block / virtual-chain progression for an indexer.

## The two-step pattern

Every subscription has two parts:

1. **A listener** — a Python callback registered with
   `client.add_event_listener("<event>", callback)`.
2. **A subscription** — `await client.subscribe_<event>(...)` that tells the
   node to start streaming.

Both halves are required. A listener with no subscription receives nothing;
a subscription with no listener silently drops events.

```python
def on_utxo_change(event):
    print("UTXO change:", event)

client.add_event_listener("utxos-changed", on_utxo_change)
await client.subscribe_utxos_changed([Address("kaspa:qz...")])
```

## Available events

| Event name | Subscribe with |
| --- | --- |
| `utxos-changed` | `subscribe_utxos_changed(addresses)` |
| `block-added` | `subscribe_block_added()` |
| `virtual-chain-changed` | `subscribe_virtual_chain_changed(include_accepted_transaction_ids=...)` |
| `virtual-daa-score-changed` | `subscribe_virtual_daa_score_changed()` |
| `sink-blue-score-changed` | `subscribe_sink_blue_score_changed()` |
| `finality-conflict` | `subscribe_finality_conflict()` |
| `finality-conflict-resolved` | `subscribe_finality_conflict_resolved()` |
| `new-block-template` | `subscribe_new_block_template()` |
| `pruning-point-utxo-set-override` | `subscribe_pruning_point_utxo_set_override()` |

Each `subscribe_*` has a matching `unsubscribe_*` that takes the same
argument shape.

## Watching addresses for UTXO changes

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

For watching a managed-wallet account rather than raw addresses, use the
wallet's `Balance` and `Maturity` events instead — see
[Wallet → Transaction History](../wallet/transaction-history.md).

## Block events

```python
def on_block(event):
    print("new block:", event["block"]["header"]["hash"])

client.add_event_listener("block-added", on_block)
await client.subscribe_block_added()
```

## Virtual chain progression

```python
def on_chain(event):
    print("chain update:", event)

client.add_event_listener("virtual-chain-changed", on_chain)
await client.subscribe_virtual_chain_changed(include_accepted_transaction_ids=True)
```

`include_accepted_transaction_ids=True` makes the event payload usable as
a confirmation feed — every accepted transaction id appears in the stream.

## Listener bookkeeping

```python
client.add_event_listener("block-added", callback)             # add
client.add_event_listener("block-added", callback, extra)      # forward extra arg

client.remove_event_listener("block-added", callback)          # remove specific
client.remove_event_listener("block-added")                    # remove all for event
client.remove_all_event_listeners()                            # remove all globally
```

Listeners outlive a single subscription cycle — re-subscribing after an
unsubscribe does *not* re-fire previously delivered events. If you need to
catch up, do a one-shot `get_utxos_by_addresses` (or equivalent) before
re-subscribing.

## Where to next

- [Calls](calls.md) — the request/response side of the API.
- [Wallet → Transaction History](../wallet/transaction-history.md) — the
  managed Wallet's higher-level event surface.
