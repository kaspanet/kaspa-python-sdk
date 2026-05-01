# Transaction History

The wallet emits events for every state change the node pushes
through. Register Python callbacks on the wallet; its event
multiplexer forwards relevant events to them, and you react — update
a UI, trigger the next send, log a maturity, handle a reorg. The
history APIs (`transactions_data_get` and friends) cover the
"what happened in this account?" question.

## Listener API

```python
def add_event_listener(event, callback, *args, **kwargs) -> None
def remove_event_listener(event, callback=None) -> None
```

- `event` — a
  [`WalletEventType`](../../reference/Enums/WalletEventType.md), its
  kebab-case string name (`"balance"`, `"sync-state"`), or `"all"` /
  `WalletEventType.All` for every event.
- `callback` — invoked as `callback(*args, event, **kwargs)`. Must be
  a regular (synchronous) function; the dispatcher calls it inline and
  does not await coroutines, so an `async def` callback's body never
  runs.
- `args` / `kwargs` — forwarded verbatim to every invocation. Handy
  for routing context (account id, channel) without closures.
- `remove_event_listener(event)` with no callback clears every
  listener for that event. With `"all"` and no callback, clears every
  listener globally.

## A minimal subscriber

```python
from kaspa import Resolver, Wallet, WalletEventType

def on_event(event):
    print(event["type"], event.get("data"))

wallet = Wallet(network_id="testnet-10", resolver=Resolver())
wallet.add_event_listener(WalletEventType.All, on_event)

await wallet.start()
await wallet.connect()
# ... events stream in for the rest of the session ...
```

Each event is a dict with at least a `type` key (the kebab-case kind
name, e.g. `"balance"`, `"sync-state"`, `"fee-rate"`) and an optional
`data` payload specific to that event.

## Event taxonomy

| Group | Events |
| --- | --- |
| Connection | `Connect`, `Disconnect`, `ServerStatus`, `UtxoIndexNotEnabled` |
| Wallet file | `WalletList`, `WalletStart`, `WalletHint`, `WalletOpen`, `WalletCreate`, `WalletReload`, `WalletClose`, `WalletError` |
| Key & account state | `PrvKeyDataCreate`, `AccountCreate`, `AccountActivation`, `AccountDeactivation`, `AccountSelection`, `AccountUpdate` |
| Sync & runtime | `SyncState`, `UtxoProcStart`, `UtxoProcStop`, `UtxoProcError`, `DaaScoreChange`, `Metrics`, `FeeRate` |
| UTXO movement | `Pending`, `Maturity`, `Reorg`, `Stasis`, `Discovery`, `Balance` |
| Catch-all | `All`, `Error` |

The most common subscriptions:

- **`SyncState`** — progress while the `UtxoProcessor` catches up.
  Pair with `wallet.is_synced` — see [Sync State](sync-state.md) for
  the substate payload shape.
- **`Balance`** — fires when a `UtxoContext` balance changes. The
  right signal for live UI updates.
- **`Pending`** — a new UTXO landed for a tracked address but isn't
  yet spendable.
- **`Maturity`** — a previously-pending UTXO crossed the maturity
  depth and is now spendable. The strongest gate for "send-then-wait"
  flows — don't trigger the next `accounts_send` on `Pending` alone.
- **`Reorg`** / **`Stasis`** — a UTXO was unwound or coinbase-locked.
  Defensive code for high-value flows.
- **`AccountActivation`** / **`AccountDeactivation`** — react to
  `accounts_activate` / `wallet_close`.

## Targeted subscriptions

```python
wallet.add_event_listener("balance", on_balance)
wallet.add_event_listener("maturity", on_maturity)
wallet.add_event_listener(WalletEventType.SyncState, on_sync)
```

To pass context to a generic callback:

```python
wallet.add_event_listener("balance", on_change, account.account_id, label="primary")
# callback receives: on_change(account.account_id, event, label="primary")
```

## History queries

For the "what happened" view rather than the live stream:

```python
data = await wallet.transactions_data_get(
    account_id=account.account_id,
    network_id="testnet-10",
    start=0,
    end=20,
)
# Annotate / re-annotate:
await wallet.transactions_replace_note(
    account.account_id, "testnet-10", tx_id, "rent",
)
await wallet.transactions_replace_metadata(
    account.account_id, "testnet-10", tx_id, '{"tag": "ops"}',
)
```

Note and metadata are free-form strings stored in the wallet file —
not on chain. If you want structured metadata, encode it yourself
(JSON, etc.).

## Cleanup

Listeners outlive the wallet's open file but not its runtime. Pair a
permanent registration with an explicit removal on shutdown, or use
`"all"` to clear in one call:

```python
wallet.remove_event_listener(WalletEventType.All)
await wallet.stop()
```

## Where to next

- [Send Transaction](send-transaction.md) — `Maturity` as the right
  wait condition.
- [Architecture](architecture.md) — what's actually generating these
  events.
- [Lifecycle](lifecycle.md) — when each event group fires.
- [Wallet SDK → UTXO Processor](../wallet-sdk/utxo-processor.md) —
  the lower-level event surface beneath the managed wallet.
