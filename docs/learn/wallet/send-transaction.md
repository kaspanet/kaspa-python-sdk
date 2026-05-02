# Send Transaction

Outgoing flows from an activated account. Every method on this page
requires an open wallet, a connected wRPC client, an activated source
account, and `wallet.is_synced == True` — see
[Sync State](sync-state.md).

## Surface

| Method | Returns | Purpose |
| --- | --- | --- |
| [`accounts_estimate(...)`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_estimate) | [`GeneratorSummary`](../../reference/Classes/GeneratorSummary.md) | Dry-run a send without submitting. |
| [`accounts_send(...)`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_send) | [`GeneratorSummary`](../../reference/Classes/GeneratorSummary.md) | Sign and submit a send. |
| [`accounts_transfer(...)`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_transfer) | [`GeneratorSummary`](../../reference/Classes/GeneratorSummary.md) | Internal transfer between two accounts in the same wallet. |
| [`accounts_get_utxos(...)`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_get_utxos) | `list[dict]` | Snapshot of an account's tracked UTXOs (post-sync). |
| [`fee_rate_estimate()`](../../reference/Classes/Wallet.md#kaspa.Wallet.fee_rate_estimate) | `dict` | Current low / normal / priority fee rates from the node. |
| [`fee_rate_poller_enable(seconds)`](../../reference/Classes/Wallet.md#kaspa.Wallet.fee_rate_poller_enable) / [`_disable()`](../../reference/Classes/Wallet.md#kaspa.Wallet.fee_rate_poller_disable) | — | Background fee-rate refresh. |

For sweeping (consolidating every UTXO), see [Sweep Funds](sweep.md).

## Send a single output

[`accounts_send`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_send) takes a [`Fees`](../../reference/Classes/Fees.md) object with a [`FeeSource`](../../reference/Enums/FeeSource.md) and a list of [`PaymentOutput`](../../reference/Classes/PaymentOutput.md):

```python
from kaspa import Fees, FeeSource, PaymentOutput

result = await wallet.accounts_send(
    wallet_secret=wallet_secret,
    account_id=account.account_id,
    priority_fee_sompi=Fees(0, FeeSource.SenderPays),
    destination=[PaymentOutput("kaspatest:...", 100_000_000)],   # 1 KAS
)
print(result.final_transaction_id, result.fees, result.final_amount)
```

[`Fees(0, ...)`](../../reference/Classes/Fees.md) is fine here: with priority `0` the wallet still pays
the network minimum (mass × `fee_rate`). See
[Fee model](#fee-model) below.

## Multi-output send

A single `destination` list of N outputs becomes one transaction with
N + 1 outputs (the +1 is the change return).

```python
outputs = [PaymentOutput(addr, 100_000_000) for addr in addresses]
result = await wallet.accounts_send(
    wallet_secret=wallet_secret,
    account_id=account.account_id,
    priority_fee_sompi=Fees(0, FeeSource.SenderPays),
    destination=outputs,
)
```

## Estimate before sending

```python
estimate = await wallet.accounts_estimate(
    account_id=account.account_id,
    priority_fee_sompi=Fees(0, FeeSource.SenderPays),
    destination=outputs,
)
print(estimate.fees, estimate.final_amount, estimate.utxos)
```

[`accounts_estimate`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_estimate) and [`accounts_send`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_send) take the same arguments.
Estimating first surfaces fees and UTXO selection before signing.

## `GeneratorSummary` fields

Both [`accounts_estimate`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_estimate) and [`accounts_send`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_send) return a
[`GeneratorSummary`](../../reference/Classes/GeneratorSummary.md):

| Field | Type | Meaning |
| --- | --- | --- |
| `network_id` | `str` | Network the summary applies to. |
| `network_type` | `str` | Network type identifier. |
| `utxos` | `int` | Total inputs consumed across the chain. |
| `mass` | `int` | Total transaction mass. |
| `fees` | `int` | Aggregate fee paid in sompi (network + priority). |
| `transactions` | `int` | Number of transactions generated. |
| `stages` | `int` | Number of compounding stages. |
| `final_amount` | `int \| None` | Final output amount in sompi (None if not yet generated). |
| `final_transaction_id` | `str \| None` | ID of the final transaction in the chain. |

For most sends the chain is one transaction; large sweeps produce
multi-transaction chains (see [Sweep Funds](sweep.md)).

## Fee model

A submitted transaction pays:

```
total_fee = network_fee + priority_fee
network_fee = mass * fee_rate
```

Both arguments are total sompi values, never per-gram on the Python
side:

| Argument | Type | What it controls |
| --- | --- | --- |
| `priority_fee_sompi` | [`Fees`](../../reference/Classes/Fees.md)`(amount_sompi, source)` | A flat top-up in sompi above the network minimum. `0` is a valid (minimum-priority) value. |
| `fee_rate` | `float \| None` | Network rate in sompi per gram. `None` resolves to the node's suggested rate. |

[`FeeSource`](../../reference/Enums/FeeSource.md) decides who absorbs the fee:

- **`FeeSource.SenderPays`** — fee is added on top of the destination
  amount. Standard sends.
- **`FeeSource.ReceiverPays`** — fee is deducted from the destination
  amount. Used to sweep an exact balance with no leftover change (see
  [Sweep Funds](sweep.md)).

See [Mass & Fees](../transactions/mass-and-fees.md) for the underlying
mass model.

```python
rates = await wallet.fee_rate_estimate()
# {"priority": {"feerate": ..., "seconds": ...},
#  "normal":   {"feerate": ..., "seconds": ...},
#  "low":      {"feerate": ..., "seconds": ...}}
```

[`fee_rate_estimate`](../../reference/Classes/Wallet.md#kaspa.Wallet.fee_rate_estimate) returns a one-shot dict.

For latency-sensitive flows, run a background poller via [`fee_rate_poller_enable`](../../reference/Classes/Wallet.md#kaspa.Wallet.fee_rate_poller_enable) /
[`fee_rate_poller_disable`](../../reference/Classes/Wallet.md#kaspa.Wallet.fee_rate_poller_disable):

```python
wallet.fee_rate_poller_enable(15)   # refresh every 15 seconds
# ... later ...
wallet.fee_rate_poller_disable()
```

## Internal transfers

Funds moved between two accounts in the **same wallet** are spendable
immediately on transaction acceptance — no maturity wait:

```python
await wallet.accounts_transfer(
    wallet_secret=wallet_secret,
    source_account_id=src.account_id,
    destination_account_id=dst.account_id,
    transfer_amount_sompi=500_000_000,
)
```

Use [`accounts_transfer`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_transfer) for in-wallet movement; use [`accounts_send`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_send)
for external addresses.

## UTXO maturity

Every UTXO the
[`UtxoProcessor`](../../reference/Classes/UtxoProcessor.md) sees moves
through three states:

- **Pending** — seen, but confirmation depth is below the maturity
  threshold. Counted in [`Balance.pending`](../../reference/Classes/Balance.md). *Not* spendable.
- **Mature** — confirmed deeply enough to spend. Counted in
  [`Balance.mature`](../../reference/Classes/Balance.md). Returned by [`accounts_get_utxos`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_get_utxos). Selectable.
- **Outgoing** — locked because the wallet just spent it in a
  transaction it generated. Counted in [`Balance.outgoing`](../../reference/Classes/Balance.md) until the
  spend matures or is reorged out.

The default thresholds (mainnet and testnet-10):

| | DAA score depth |
| --- | --- |
| User transactions → Mature | 100 |
| Coinbase transactions → Mature | 1000 |
| Coinbase stasis (UTXOs hidden) | 500 |

Override per network with
[`UtxoProcessor.set_user_transaction_maturity_period_daa(network_id, value)`](../../reference/Classes/UtxoProcessor.md)
and the matching coinbase setter; defaults usually want to stay.

### Why `accounts_get_utxos` can return `[]`

[`accounts_get_utxos`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_get_utxos) reads the in-memory [`UtxoContext`](../../reference/Classes/UtxoContext.md). It returns
`[]` when:

1. The wallet isn't synced yet — see [Sync State](sync-state.md).
2. The account hasn't been activated.
3. No notification for a funding tx has reached the processor yet.

Each UTXO is a `dict` — read amounts as `u["amount"]`, not
`u.amount`.

### Waiting for confirmations

Sends submit immediately, but spent UTXOs need to mature before the
next [`accounts_send`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_send) will see them. Two correct waits, both via
[Events](events.md):

- **Pending** — fires when a UTXO lands but isn't yet spendable.
  Useful for UI.
- **Maturity** — fires when a UTXO crosses the maturity depth and is
  spendable. The right gate for "send → wait → send again" flows.

Polling [`accounts_get_utxos`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_get_utxos) works for one-shot scripts; a `Maturity`
listener is the production pattern.

## Where to next

- [Sweep Funds](sweep.md) — consolidating every UTXO.
- [Events](events.md) — `Pending`, `Maturity`, and listener
  registration.
- [Wallet SDK → Transaction Generator](../wallet-sdk/tx-generator.md) —
  the lower-level primitive `accounts_send` is built on.
