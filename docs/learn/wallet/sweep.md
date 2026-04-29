# Sweep Funds

A sweep consolidates every UTXO in an account into one address. There are
two patterns, and the difference between them is whether you want any
leftover change.

## Pattern 1: sweep to your own change address

Omit `destination` entirely. The wallet routes the full sweepable balance
to the account's current change address and the resulting transaction has
no external recipient — useful for collapsing a long UTXO history into a
single mature output before a high-volume sending flow.

```python
from kaspa import Fees, FeeSource

await wallet.accounts_send(
    wallet_secret=secret,
    account_id=account.account_id,
    priority_fee_sompi=Fees(0, FeeSource.SenderPays),
    destination=None,
)
```

`SenderPays` is fine here because the change return absorbs the fee —
there's no external recipient to "subtract from."

## Pattern 2: sweep an exact balance to a fresh address

When you want to leave the account at zero and the destination receives
*the exact aggregate balance minus fees*, use `ReceiverPays`:

```python
from kaspa import Fees, FeeSource, PaymentOutput

utxos = await wallet.accounts_get_utxos(account_id=account.account_id)
total = sum(u["amount"] for u in utxos)

await wallet.accounts_send(
    wallet_secret=secret,
    account_id=account.account_id,
    priority_fee_sompi=Fees(0, FeeSource.ReceiverPays),
    destination=[PaymentOutput(sweep_address, total)],
)
```

The destination amount is the *gross* balance; `ReceiverPays` deducts the
network fee from that amount before broadcasting. The result: no
change output, no dust left behind.

## Which to use

| You want… | Use |
| --- | --- |
| To consolidate UTXOs and keep them in this account | Pattern 1 (no `destination`) |
| To move every sompi to an external address, leaving zero | Pattern 2 (`ReceiverPays`) |
| To sweep within the same wallet to a different account | [`accounts_transfer`](send-transaction.md#internal-transfers), not a sweep |

## Big sweeps come back as multiple transactions

If the input set is too large for one transaction's mass budget, the
underlying `Generator` produces a series of transactions: each one
consolidates some UTXOs into a single intermediate output, and the final
transaction sends that aggregate to the destination. `accounts_send`
returns the final `GeneratorSummary`. Watch the
[`Maturity` event](transaction-history.md) to know when the chain has
caught up — only the final output is what you'd hand off downstream.

## Where to next

- [Send Transaction](send-transaction.md) — non-sweep sends and fee
  modes.
- [Transaction History](transaction-history.md) — gating "wait until
  swept" on `Maturity`.
