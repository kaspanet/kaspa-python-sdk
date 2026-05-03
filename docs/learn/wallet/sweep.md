---
search:
  boost: 3
---

# Sweep Funds

A sweep consolidates every UTXO in an account into one address. Two
patterns; the difference is whether you want any leftover change.

Both patterns require a synced wallet — [`accounts_get_utxos`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_get_utxos) returns
`[]` until then. See [Sync State](sync-state.md).

## Pattern 1: sweep to your own change address

Omit `destination` entirely. The wallet routes the full sweepable
balance to the account's current change address — no external
recipient. Useful for collapsing a long UTXO history into a single
mature output before a high-volume send flow.

```python
from kaspa import Fees, FeeSource

await wallet.accounts_send(
    wallet_secret=wallet_secret,
    account_id=account.account_id,
    priority_fee_sompi=Fees(0, FeeSource.SenderPays),
    destination=None,
)
```

[`FeeSource.SenderPays`](../../reference/Enums/FeeSource.md) works here because the change return absorbs the fee —
there's no external recipient to subtract from. [`Fees(0, ...)`](../../reference/Classes/Fees.md) keeps
priority at the network minimum; raise it to bid for faster
inclusion.

## Pattern 2: sweep an exact balance to a fresh address

To leave the account at zero with the destination receiving *the exact
aggregate balance minus fees*, use [`FeeSource.ReceiverPays`](../../reference/Enums/FeeSource.md):

```python
from kaspa import Fees, FeeSource, PaymentOutput

utxos = await wallet.accounts_get_utxos(account_id=account.account_id)
total = sum(u["amount"] for u in utxos)

await wallet.accounts_send(
    wallet_secret=wallet_secret,
    account_id=account.account_id,
    priority_fee_sompi=Fees(0, FeeSource.ReceiverPays),
    destination=[PaymentOutput(sweep_address, total)],
)
```

[`accounts_get_utxos`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_get_utxos) returns `list[dict]`; each UTXO's amount is
`u["amount"]` (sompi).

The destination amount is the *gross* balance; [`FeeSource.ReceiverPays`](../../reference/Enums/FeeSource.md) deducts
the network fee from it before broadcasting. The result: no change
output, no dust.

## Big sweeps come back as multiple transactions

If the input set is too large for one transaction's mass budget, the
underlying [`Generator`](../../reference/Classes/Generator.md) (see [Transaction Generator](../wallet-sdk/tx-generator.md)) produces a
series of transactions: each consolidates some UTXOs into a single
intermediate output, and the final transaction sends the aggregate to
the destination. [`accounts_send`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_send) returns the final
[`GeneratorSummary`](../../reference/Classes/GeneratorSummary.md);
`summary.transactions` reports how many were submitted. Watch the
[`Maturity` event](events.md) to know when the chain has
caught up — only the final output is what you'd hand off downstream.

## Where to next

- [Send Transaction](send-transaction.md) — non-sweep sends and the
  fee model.
- [Events](events.md) — gating "wait until swept" on `Maturity`.
