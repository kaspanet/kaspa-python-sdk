# Events

The wallet emits events for every state change the node pushes
through. Register Python callbacks on the wallet; its event
multiplexer forwards relevant events to them, and you react — update
a UI, trigger the next send, log a maturity, handle a reorg.

For the "what happened in this account?" view that reads stored
transaction records, see [Transaction History](transaction-history.md).

!!! warning "Callbacks must be synchronous"
    The dispatcher invokes callbacks via `callback(*args, event, **kwargs)` —
    a direct synchronous call. If you pass an `async def` function, it
    returns a coroutine that is **never awaited**: the body silently does
    not run. Use a regular `def` and offload async work with
    `asyncio.create_task(...)` from inside the callback.

## Listener API

The wallet exposes [`add_event_listener`](../../reference/Classes/Wallet.md#kaspa.Wallet.add_event_listener) and [`remove_event_listener`](../../reference/Classes/Wallet.md#kaspa.Wallet.remove_event_listener):

```python
def add_event_listener(event, callback, *args, **kwargs) -> None
def remove_event_listener(event, callback=None) -> None
```

- `event` — a
  [`WalletEventType`](../../reference/Enums/WalletEventType.md), its
  kebab-case string name (`"balance"`, `"sync-state"`), or `"all"` /
  [`WalletEventType.All`](../../reference/Enums/WalletEventType.md) for every event.
- `callback` — invoked as `callback(*args, event, **kwargs)`. Must be
  synchronous (see warning above).
- `args` / `kwargs` — forwarded verbatim to every invocation. Handy
  for routing context (account id, channel) without closures.
- [`remove_event_listener(event)`](../../reference/Classes/Wallet.md#kaspa.Wallet.remove_event_listener) with no callback clears every
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

## Event payloads

Every event is `{"type": <kebab-case>, "data": <dict>}`. The `data`
shapes below come from `kaspa-wallet-core::events`. Field names that
appear in `camelCase` are camel-cased on the wire; `snake_case` fields
are passed through as-is.

### UTXO movement

| Event | `data` fields |
| --- | --- |
| `Balance` | `balance: {mature, pending, outgoing, mature_utxo_count, pending_utxo_count, stasis_utxo_count} \| None`, `id: <utxo_context_id>` (see [`Balance`](../../reference/Classes/Balance.md)) |
| `Pending` | `record: TransactionRecord` — UTXO seen, not yet mature. |
| `Maturity` | `record: TransactionRecord` — UTXO crossed the maturity threshold; spendable. |
| `Reorg` | `record: TransactionRecord` — pending UTXO unwound by a reorg. |
| `Stasis` | `record: TransactionRecord` — coinbase output unwound during stasis. Safe to ignore. |
| `Discovery` | `record: TransactionRecord` — UTXO discovered during the initial scan of an account. When using the runtime [`Wallet`](../../reference/Classes/Wallet.md), you can usually rely on the [transaction history](transaction-history.md) instead. |

A `TransactionRecord` carries `id`, `unixtimeMsec`, `value`,
`binding`, `blockDaaScore`, `network`, `data` (the per-kind
transaction body), and optional `note` / `metadata` strings.

### Sync & runtime

| Event | `data` fields |
| --- | --- |
| `SyncState` | `syncState: {type, data}` — see [Sync State → payloads](sync-state.md#reading-syncstate-payloads). |
| `UtxoProcStart` | *(no data)* |
| `UtxoProcStop` | *(no data)* |
| `UtxoProcError` | `message: str` |
| `DaaScoreChange` | `currentDaaScore: int` |
| `Metrics` | `networkId: str`, `metrics: MetricsUpdate` |
| `FeeRate` | `priority: {feerate, seconds}`, `normal: {...}`, `low: {...}` |

### Key & account state

| Event | `data` fields |
| --- | --- |
| `PrvKeyDataCreate` | `prvKeyDataInfo:`[`PrvKeyDataInfo`](../../reference/Classes/PrvKeyDataInfo.md) |
| `AccountCreate` | `accountDescriptor:`[`AccountDescriptor`](../../reference/Classes/AccountDescriptor.md) |
| `AccountActivation` | `ids: list[`[`AccountId`](../../reference/Classes/AccountId.md)`]` |
| `AccountDeactivation` | `ids: list[`[`AccountId`](../../reference/Classes/AccountId.md)`]` |
| `AccountSelection` | `id:`[`AccountId`](../../reference/Classes/AccountId.md)`\| None` |
| `AccountUpdate` | `accountDescriptor:`[`AccountDescriptor`](../../reference/Classes/AccountDescriptor.md) — fires when a new address is generated, etc. |

### Wallet file

| Event | `data` fields |
| --- | --- |
| `WalletList` | `walletDescriptors: list[`[`WalletDescriptor`](../../reference/Classes/WalletDescriptor.md)`]` |
| `WalletStart` | *(no data)* — fires once after [`start()`](../../reference/Classes/Wallet.md#kaspa.Wallet.start). |
| `WalletHint` | `hint: str \| None` — anti-phishing hint stored on the wallet file. |
| `WalletOpen` | `walletDescriptor:`[`WalletDescriptor`](../../reference/Classes/WalletDescriptor.md)`\| None`, `accountDescriptors: list[`[`AccountDescriptor`](../../reference/Classes/AccountDescriptor.md)`] \| None` |
| `WalletCreate` | `walletDescriptor:`[`WalletDescriptor`](../../reference/Classes/WalletDescriptor.md), `storageDescriptor: StorageDescriptor` |
| `WalletReload` | `walletDescriptor:`[`WalletDescriptor`](../../reference/Classes/WalletDescriptor.md)`\| None`, `accountDescriptors: list[`[`AccountDescriptor`](../../reference/Classes/AccountDescriptor.md)`] \| None` |
| `WalletClose` | *(no data)* |
| `WalletError` | `message: str` |

### Connection

| Event | `data` fields |
| --- | --- |
| `Connect` | `networkId: str`, `url: str \| None` |
| `Disconnect` | `networkId: str`, `url: str \| None` |
| `ServerStatus` | `networkId: str`, `serverVersion: str`, `isSynced: bool`, `url: str \| None` |
| `UtxoIndexNotEnabled` | `url: str \| None` |

## When each event fires

Common subscriptions:

- **`SyncState`** — progress while the [`UtxoProcessor`](../../reference/Classes/UtxoProcessor.md) catches up.
  Pair with [`wallet.is_synced`](../../reference/Classes/Wallet.md#kaspa.Wallet.is_synced) — see [Sync State](sync-state.md).
- **`Balance`** — fires when a [`UtxoContext`](../../reference/Classes/UtxoContext.md) balance changes. The
  right signal for live UI updates.
- **`Pending`** — a new UTXO landed for a tracked address but isn't
  yet spendable.
- **`Maturity`** — a previously-pending UTXO crossed the maturity
  depth and is now spendable. The strongest gate for "send-then-wait"
  flows — don't trigger the next [`accounts_send`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_send) on `Pending` alone.
- **`Reorg`** / **`Stasis`** — a UTXO was unwound or coinbase-locked.
  Defensive code for high-value flows.
- **`AccountActivation`** / **`AccountDeactivation`** — react to
  [`accounts_activate`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_activate) / [`wallet_close`](../../reference/Classes/Wallet.md#kaspa.Wallet.wallet_close).

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

## Cleanup

Listeners outlive the wallet's open file but not its runtime. Pair a
permanent registration with an explicit removal on shutdown, or use
`"all"` to clear in one call via [`remove_event_listener`](../../reference/Classes/Wallet.md#kaspa.Wallet.remove_event_listener):

```python
wallet.remove_event_listener(WalletEventType.All)
await wallet.stop()
```

## Where to next

- [Transaction History](transaction-history.md) — stored records,
  notes, and metadata.
- [Send Transaction](send-transaction.md) — `Maturity` as the right
  wait condition.
- [Architecture](architecture.md) — what's actually generating these
  events.
- [Lifecycle](lifecycle.md) — when each event group fires.
- [Wallet SDK → UTXO Processor](../wallet-sdk/utxo-processor.md) —
  the lower-level event surface beneath the managed wallet.
