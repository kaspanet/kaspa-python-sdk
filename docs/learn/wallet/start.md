# Start

`start()` boots the wallet's runtime: the `UtxoProcessor`, the wRPC
notifier task, and the event-dispatch loop. `connect()` then attaches the
wRPC client to the node. After both, the wallet is ready to *open a file*
— but not yet ready to do anything that touches UTXO state.

## Boot sequence

```python
wallet = Wallet(network_id="testnet-10", resolver=Resolver())
await wallet.start()
await wallet.connect()
```

Both calls are required. `start()` without `connect()` leaves the runtime
running but unable to talk to the node; `connect()` without `start()`
raises.

## Connect options

`connect()` takes the same options as `RpcClient.connect`:

```python
await wallet.connect(
    block_async_connect=True,    # await readiness before returning
    strategy="retry",            # "retry" or "fallback"
    url=None,                    # override the resolver-discovered URL
    timeout_duration=10_000,     # ms
    retry_interval=1_000,        # ms
)
```

If you constructed with a `Resolver`, omit `url` and let the resolver pick
a public node. Pass `url=` to override for one connection (useful for
pinning to a specific node temporarily).

## The sync gate

`connect()` resolves as soon as the WebSocket is up — **not** when the
wallet's UTXO processor has caught up. UTXO-dependent state
(`AccountDescriptor.balance`, `accounts_get_utxos`, `accounts_send`) is
unusable in this gap.

```python
await wallet.connect(...)

while not wallet.is_synced:
    await asyncio.sleep(0.5)
```

After this loop, `accounts_activate` actually attaches a working
`UtxoContext` and node notifications start populating balances and UTXOs.

## Why the gate is necessary

Before `is_synced` flips to `True`:

- The `UtxoProcessor` is not driving per-account `UtxoContext`s.
- `accounts_activate` is a no-op for UTXO discovery — accounts are
  registered but the processor isn't pulling state.
- Notifications from the node are buffered or ignored.

So `accounts_get_utxos` returns `[]` and `AccountDescriptor.balance` is
`None` — not because the address is unfunded, but because the wallet
hasn't started tracking it. See [Architecture](architecture.md).

## Event-driven wait

If you've registered a listener (see
[Transaction History](transaction-history.md)), the `SyncState` event
reports progress and `is_synced` flips at the end:

```python
import asyncio
from kaspa import WalletEventType

ready = asyncio.Event()

async def on_event(event):
    if event["type"] == WalletEventType.SyncState.name and wallet.is_synced:
        ready.set()

wallet.add_event_listener(WalletEventType.All, on_event)
await wallet.connect(...)
await ready.wait()
```

Use this when you want a UI progress indicator alongside the gate.
Polling is fine for scripts.

## Shutdown

```python
await wallet.disconnect()    # drop the wRPC link; runtime stays alive
await wallet.stop()          # tear down the runtime and event task
```

Skipping `stop()` leaks the notification task. Skipping `disconnect()`
keeps the WebSocket open.

## Where to next

- [Open](open.md) — create or open a wallet file.
- [Architecture](architecture.md) — what `start` actually wires up.
- [Transaction History](transaction-history.md) — `SyncState` and the
  rest of the event taxonomy.
