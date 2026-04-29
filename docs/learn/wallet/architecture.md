# Architecture

A `Wallet` is not a single object — it's a small system of cooperating
pieces. Knowing how they fit together is what makes the rest of this
section make sense, especially the [sync gate](start.md) and the
[transaction-history events](transaction-history.md).

## The pieces

```
                ┌──────────────────────────────────────────┐
                │                  Wallet                  │
                │   (lifecycle, file storage, accounts)    │
                └───────────┬───────────────┬──────────────┘
                            │               │
              owns          │               │     owns
                            ▼               ▼
                    ┌────────────────┐  ┌──────────────┐
                    │ UtxoProcessor  │  │  RpcClient   │
                    └────────┬───────┘  └──────┬───────┘
                             │                 │
                       fans out to             │  pushes notifications
                             │                 │
                             ▼                 ▼
                    ┌────────────────────────────────┐
                    │   UtxoContext  (one per          │
                    │   activated account)             │
                    └────────────────────────────────┘
```

| Component | Job |
| --- | --- |
| **`Wallet`** | Lifecycle, on-disk file storage, account list, event multiplexer. The thing your code calls. |
| **`RpcClient`** | The wRPC connection. Used internally for calls and as the source of node-pushed notifications. |
| **`UtxoProcessor`** | Subscribes to virtual-chain / UTXO notifications, tracks `synced` state, routes incoming UTXO changes to the right `UtxoContext`. |
| **`UtxoContext`** | One per activated account. Holds the tracked addresses, the per-state balance (`mature`, `pending`, `outgoing`), and the mature UTXO set the coin selector pulls from. |

The wallet *does not poll* the node for UTXO state. It is **fed**, by the
processor, from notifications. This is why the [sync gate](start.md)
matters — before sync, the processor isn't forwarding anything, so the
contexts stay empty.

## UTXO maturity

Every UTXO the processor sees moves through three states:

- **Pending** — seen, but the chain confirmation depth is below the
  maturity threshold. Counted in `Balance.pending`. *Not* spendable.
- **Mature** — confirmed deeply enough to spend. Counted in
  `Balance.mature`. Returned by `accounts_get_utxos`. Selectable.
- **Outgoing** — locked because the wallet just spent it in a transaction
  it generated. Counted in `Balance.outgoing` until the spend matures or is
  reorged out.

[Send Transaction](send-transaction.md) waits on `Maturity` for this
reason: a `Pending` UTXO is real, but the next `accounts_send` won't see
it as spendable.

## Why `accounts_get_utxos` can return `[]`

`accounts_get_utxos` is a read of the in-memory `UtxoContext`. It returns
`[]` when:

1. The wallet isn't synced yet (the processor isn't forwarding).
2. The account hasn't been activated.
3. No notification for a funding tx has reached the processor yet.

None of these are "the address has no funds" — they're "the wallet hasn't
been told yet." The fix is to gate UTXO-dependent code on `is_synced` and
to listen for `Maturity` rather than polling. See [Start](start.md) and
[Transaction History](transaction-history.md).

## Where to next

- [Lifecycle](lifecycle.md) — the state machine.
- [Start](start.md) — `start → connect → is_synced` and why each step matters.
- [Transaction History](transaction-history.md) — the event surface.
