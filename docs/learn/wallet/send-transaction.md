# Send Transaction

Outgoing flows from an activated account. Every method on this page
requires the wallet to be open, the wRPC client connected, the source
account activated, and `wallet.is_synced` to be `True` — see
[Start](start.md).

## Surface

| Method | Purpose |
| --- | --- |
| `accounts_estimate(...)` | Dry-run a send; returns a `GeneratorSummary` without submitting. |
| `accounts_send(...)` | Sign and submit a send. Returns the same `GeneratorSummary` after submission. |
| `accounts_transfer(...)` | Internal transfer between two accounts in the same wallet. |
| `accounts_get_utxos(...)` | Snapshot of an account's tracked UTXOs (post-sync). |
| `fee_rate_estimate()` | Current low / normal / priority fee rates from the node. |
| `fee_rate_poller_enable(seconds)` / `_disable()` | Background fee-rate refresh. |

For sweeping (consolidating every UTXO), see [Sweep Funds](sweep.md).

## Send a single output

```python
from kaspa import Fees, FeeSource, PaymentOutput

result = await wallet.accounts_send(
    wallet_secret="example-secret",
    account_id=account.account_id,
    priority_fee_sompi=Fees(0, FeeSource.SenderPays),
    destination=[PaymentOutput("kaspatest:...", 100_000_000)],   # 1 KAS
)
print(result.final_transaction_id, result.fees, result.final_amount)
```

## Multi-output send

A single `destination` list with N outputs becomes one transaction with
N + 1 outputs (the +1 is the change return).

```python
outputs = [PaymentOutput(addr, 100_000_000) for addr in addresses]
result = await wallet.accounts_send(
    wallet_secret=secret,
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
print(estimate.fees, estimate.final_amount, estimate.aggregated_utxos)
```

`accounts_estimate` and `accounts_send` take the same arguments.
Estimating first is cheap — it surfaces fees and UTXO selection before
signing.

## Fees

`priority_fee_sompi` is a `Fees(amount, FeeSource)` (or equivalent dict):

- **`FeeSource.SenderPays`** — fee is added on top of the destination
  amount. Used in normal sends.
- **`FeeSource.ReceiverPays`** — fee is deducted from the destination
  amount. Used to sweep an exact balance with no leftover change (see
  [Sweep Funds](sweep.md)).

`fee_rate` overrides the resolved sompi-per-gram rate explicitly. Leave
it `None` to let the wallet pick the network-suggested rate.

```python
rates = await wallet.fee_rate_estimate()
# {"low": ..., "normal": ..., "priority": ...}
```

For latency-sensitive flows, run a background poller:

```python
wallet.fee_rate_poller_enable(15)   # refresh every 15 seconds
# ... later ...
wallet.fee_rate_poller_disable()
```

## Internal transfers

Funds moved between two accounts in the **same wallet** are immediately
spendable on transaction acceptance — no maturity wait:

```python
await wallet.accounts_transfer(
    wallet_secret=secret,
    source_account_id=src.account_id,
    destination_account_id=dst.account_id,
    transfer_amount_sompi=500_000_000,
)
```

Use `accounts_transfer` for in-wallet movement; use `accounts_send` for
external addresses.

## Waiting for funds and confirmations

Sends submit immediately, but spent UTXOs need to mature before the next
`accounts_send` will see them. Two correct waits, both via
[Transaction History](transaction-history.md):

- **Pending** fires when a UTXO lands but isn't spendable yet — useful
  for UI.
- **Maturity** fires when a UTXO crosses the maturity depth and is
  spendable. This is the right gate for "send → wait → send again" flows.

Polling `accounts_get_utxos` works for one-shot scripts, but a `Maturity`
listener is the production pattern.

## Where to next

- [Sweep Funds](sweep.md) — consolidating every UTXO.
- [Transaction History](transaction-history.md) — `Pending`, `Maturity`,
  and listener registration.
- [Wallet SDK → Transaction Generator](../wallet-sdk/tx-generator.md) —
  the lower-level primitive `accounts_send` is built on.
