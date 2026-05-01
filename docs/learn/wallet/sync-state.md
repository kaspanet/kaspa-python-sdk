# Sync State

"Sync" in the managed wallet covers two distinct layers:

1. **Node sync** — has the kaspad you're connected to finished its
   own initial block download (IBD)?
2. **Processor sync** — has the wallet's embedded
   [`UtxoProcessor`](../../reference/Classes/UtxoProcessor.md)
   finished registering subscriptions and confirmed the node is in a
   usable state?

`accounts_*` calls — balances, UTXO snapshots, sends — wait on (2),
which is itself gated on (1). Treating them as one gives the right
answer most of the time but blurs *what* the wallet is actually
waiting for. This page splits them.

## Node sync state

A node that's still in IBD doesn't yet have all blocks/UTXOs, so it
can't answer wallet RPC calls authoritatively. Two surfaces report
this:

- **`ServerStatus`** event — emitted once after `connect()`, right
  after the initial `get_server_info` handshake. Payload:

    ```python
    {
        "type": "server-status",
        "data": {
            "networkId": "testnet-10",
            "serverVersion": "0.x.y",
            "isSynced": True,           # node-side flag
            "url": "wss://node:17110",
        },
    }
    ```

- **`SyncState`** event with an *IBD substate* — while the node is
  still catching up, the wallet derives progress from log lines the
  node prints and re-publishes them as `SyncState` events. See
  [Reading SyncState payloads](#reading-syncstate-payloads).

If the node is missing its UTXO index entirely, the processor
short-circuits with a **`UtxoIndexNotEnabled`** event and refuses to
proceed — the only fix is to point at a node that has it.

## Processor sync state

Once the node reports synced, the wallet's `UtxoProcessor` does its
own setup: registers UTXO/virtual-chain notification listeners, marks
itself ready, and starts forwarding UTXO changes to per-account
[`UtxoContext`](../../reference/Classes/UtxoContext.md)s.

- **`wallet.is_synced`** — `True` once the processor finishes that
  setup. This is the flag every `accounts_*` call effectively waits
  on. Polling it (with `await asyncio.sleep(0.5)`) is fine for
  scripts.
- **`SyncState`** event with a *terminal substate* — `Synced` when
  the processor flips ready, `NotSynced` when it falls back (e.g. on
  reconnect).
- **`UtxoProcStart`** / **`UtxoProcStop`** — fire when the processor
  itself starts and stops (lifecycle, not sync per se), but useful as
  bookends in event logs.

The relationship is one-way: the processor cannot be synced if the
node isn't. So `wallet.is_synced` is the single condition you
actually need to gate work on — the node-level signals are useful for
*reporting progress*, not for unblocking calls.

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

The IBD substates make it easy to drive a progress bar without
parsing kaspad logs yourself:

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

If you want to surface node IBD separately from processor readiness:

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

await node_synced.wait()        # node finished IBD
await processor_ready.wait()    # processor registered + ready
assert wallet.is_synced
```

For most scripts, the simpler form is fine:

```python
await wallet.start()
await wallet.connect()
while not wallet.is_synced:
    await asyncio.sleep(0.5)
```

## Reconnects and `Disconnect`

A `Disconnect` event flips `wallet.is_synced` back to `False`; the
processor re-runs its handshake on the next `Connect` and re-emits a
fresh `ServerStatus` plus `SyncState` chain. Long-running listeners
should treat the gate as *re-arming*, not one-shot — gate every
`accounts_*` batch on `is_synced`, not on a once-set flag.

## Where to next

- [Lifecycle](lifecycle.md#sync-gate) — the polling form of the sync gate, in context of the boot sequence.
- [Architecture](architecture.md) — where the processor and RPC client
  sit in the component graph.
- [Transaction History](transaction-history.md) — the full event
  taxonomy these events live in.
