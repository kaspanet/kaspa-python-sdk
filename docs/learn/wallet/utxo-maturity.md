# UTXO Maturity

Every UTXO the [`UtxoProcessor`](../../reference/Classes/UtxoProcessor.md)
sees moves through three states:

- **Pending** — seen, but confirmation depth is below the maturity
  threshold. Counted in `Balance.pending`. *Not* spendable.
- **Mature** — confirmed deeply enough to spend. Counted in
  `Balance.mature`. Returned by `accounts_get_utxos`. Selectable.
- **Outgoing** — locked because the wallet just spent it in a
  transaction it generated. Counted in `Balance.outgoing` until the
  spend matures or is reorged out.

[Send Transaction](send-transaction.md) waits on `Maturity` for this
reason: a `Pending` UTXO is real, but the next `accounts_send` won't
see it as spendable.

## Why `accounts_get_utxos` can return `[]`

`accounts_get_utxos` reads the in-memory `UtxoContext`. It returns
`[]` when:

1. The wallet isn't synced yet — see [Sync State](sync-state.md).
2. The account hasn't been activated.
3. No notification for a funding tx has reached the processor yet.

None of these mean "the address has no funds" — they mean "the wallet
hasn't been told yet." Listen for `Maturity` instead of polling.

## Where to next

- [Sync State](sync-state.md) — the gate that controls when UTXOs
  start flowing.
- [Send Transaction](send-transaction.md) — `Maturity` as the gate for
  send-then-wait flows.
- [Transaction History](transaction-history.md) — `Pending`,
  `Maturity`, `Reorg`, and `Stasis` events.
