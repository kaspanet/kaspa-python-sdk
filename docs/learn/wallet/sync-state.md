# Sync State

"Sync" in the managed wallet covers two distinct layers:

1. **Node sync** — has the kaspad you're connected to finished its
   own initial block download (IBD)?
2. **Processor sync** — has the wallet's embedded
   [`UtxoProcessor`](../../reference/Classes/UtxoProcessor.md)
   finished registering subscriptions and confirmed the node is in a
   usable state?

`accounts_*` calls (balances, UTXO snapshots, sends) wait on (2),
which is itself gated on (1). Treating them as one gives the right
answer most of the time but obscurs *what* the wallet is waiting for.

## Node sync state

A node still in IBD doesn't have all blocks/UTXOs and can't answer
wallet RPC calls authoritatively. Two surfaces report this:

- **`ServerStatus`** [event](events.md) — emitted once after [`connect()`](../../reference/Classes/Wallet.md#kaspa.Wallet.connect), right
  after the initial `get_server_info` handshake. Payload includes
  `isSynced` (the node-side flag), `networkId`, `serverVersion`, and
  `url`.
- **`SyncState`** [event](events.md) with an *IBD substate* — see
  [Reading SyncState payloads](#reading-syncstate-payloads).

If the node is missing its UTXO index entirely, the processor
short-circuits with a **`UtxoIndexNotEnabled`** [event](events.md) and refuses to
proceed — the only fix is to point at a node that has it.

## Processor sync state

After [`connect()`](../../reference/Classes/Wallet.md#kaspa.Wallet.connect), the wallet's [`UtxoProcessor`](../../reference/Classes/UtxoProcessor.md) runs a short handshake
against the node:

1. One `get_server_info` round trip — validates RPC API version,
   network-ID match, and that the node has its UTXO index enabled
   (otherwise it emits `UtxoIndexNotEnabled` and stops). Reads the
   node's `is_synced` flag and current `virtualDaaScore`. Emits
   `ServerStatus`.
2. Registers a single wRPC listener and subscribes to
   `VirtualDaaScoreChanged`. No `UtxosChanged` subscription is opened
   at this stage — the processor doesn't know about any addresses yet.
3. Tracks the node's IBD progress (the `proof` / `headers` / `blocks`
   / `utxo-sync` substates flow through here), then flips ready.

Per-address `UtxosChanged` subscriptions and the initial UTXO seed
happen later, inside [`accounts_activate`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_activate): each account's
[`UtxoContext`](../../reference/Classes/UtxoContext.md) issues a single
`get_utxos_by_addresses` call to populate its mature set and starts a
`UtxosChanged` subscription scoped to its addresses. After that the
context is updated purely from streamed notifications — there's no
periodic re-poll.

- **[`wallet.is_synced`](../../reference/Classes/Wallet.md#kaspa.Wallet.is_synced)** — `True` once the handshake above completes.
  This is the flag every `accounts_*` call effectively waits on.
- **`SyncState`** [event](events.md) with a *terminal substate* — `Synced` when
  the processor flips ready, `NotSynced` when it falls back (e.g. on
  reconnect).
- **`UtxoProcStart`** / **`UtxoProcStop`** — fire when the processor
  itself starts and stops. Useful as bookends in event logs.

The processor cannot be synced if the node isn't, so [`wallet.is_synced`](../../reference/Classes/Wallet.md#kaspa.Wallet.is_synced)
is the single condition you actually need to gate work on. The
node-level signals are useful for *reporting progress*, not for
unblocking calls.

## Reading SyncState payloads

The `SyncState` event carries one substate per emission. Payload
shape:

```python
{
    "type": "sync-state",
    "data": {
        "syncState": {
            "type": "<substate>",
            "data": { ... },             # variant-specific
        },
    },
}
```

The substates fall into three groups:

| Group | `type` | `data` fields | Layer |
| --- | --- | --- | --- |
| IBD progress | `proof` | `level: int` | Node |
| | `headers` | `headers: int, progress: int` | Node |
| | `blocks` | `blocks: int, progress: int` | Node |
| | `utxo-sync` | `chunks: int, total: int` | Node |
| | `trust-sync` | `processed: int, total: int` | Node |
| Resync | `utxo-resync` | *(none)* | Node |
| Terminal | `not-synced` | *(none)* | Processor |
| | `synced` | *(none)* | Processor |

The IBD substates make it easy to drive a progress bar:

```python
def on_event(event):
    if event["type"] != "sync-state":
        return
    state = event["data"]["syncState"]
    kind = state["type"]
    if kind == "headers":
        print(f"headers {state['data']['headers']:,} ({state['data']['progress']}%)")
    elif kind == "blocks":
        print(f"blocks  {state['data']['blocks']:,} ({state['data']['progress']}%)")
    elif kind == "synced":
        print("processor ready")
```

## A staged wait

To surface node IBD separately from processor readiness:

Drive `node_synced` and `processor_ready` events with [`add_event_listener`](../../reference/Classes/Wallet.md#kaspa.Wallet.add_event_listener) and the [`WalletEventType`](../../reference/Enums/WalletEventType.md) enum:

```python
import asyncio
from kaspa import Resolver, Wallet, WalletEventType

node_synced = asyncio.Event()
processor_ready = asyncio.Event()

def on_event(event):
    t = event["type"]
    if t == "server-status" and event["data"]["isSynced"]:
        node_synced.set()
    elif t == "sync-state" and event["data"]["syncState"]["type"] == "synced":
        processor_ready.set()

wallet = Wallet(network_id="testnet-10", resolver=Resolver())
wallet.add_event_listener(WalletEventType.All, on_event)

await wallet.start()
await wallet.connect()
await node_synced.wait()
await processor_ready.wait()
assert wallet.is_synced
```

For most scripts the polling form in
[Lifecycle → Sync gate](lifecycle.md#sync-gate) is enough.

## Reconnects and `Disconnect`

A `Disconnect` [event](events.md) flips [`wallet.is_synced`](../../reference/Classes/Wallet.md#kaspa.Wallet.is_synced) back to `False`; the
processor re-runs its handshake on the next `Connect` and re-emits a
fresh `ServerStatus` plus `SyncState` chain. Long-running listeners
should treat the gate as *re-arming*, not one-shot — gate every
`accounts_*` batch on [`is_synced`](../../reference/Classes/Wallet.md#kaspa.Wallet.is_synced), not on a once-set flag.

## Where to next

- [Architecture](architecture.md) — where the processor and RPC
  client sit in the component graph.
- [Events](events.md) — the full event taxonomy these signals live in.
