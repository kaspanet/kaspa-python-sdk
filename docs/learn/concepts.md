# Kaspa Concepts

A fast tour of the protocol concepts you'll bump into while using the
SDK. It's deliberately surface-level — for the full picture, see the
[Kaspa MDBook](https://kaspa-mdbook.aspectron.com/).

## BlockDAG, not blockchain

Kaspa orders transactions in a **directed acyclic graph of blocks**,
not a linear chain. Multiple blocks can be produced in parallel and
reference the same parents; consensus emerges from a deterministic DAG
ordering (the *virtual chain*), not a single longest chain.

What this means in practice:

- **Block rate is high** (one or more blocks per second on testnet-10),
  so [`block-added`](rpc/subscriptions.md#available-events) events fire
  far more often than on a Bitcoin-shaped chain.
- **Transactions confirm via the virtual chain.** A transaction is
  "accepted" when it enters the virtual-chain ordering, not when it
  first lands in a block.
- **Reorgs happen at the DAG-ordering level.** A previously-accepted
  transaction can be re-ordered out; the SDK surfaces this as a
  [`Reorg` event](wallet/transaction-history.md).

## UTXO model

Every spendable balance is a set of *unspent transaction outputs*. To
spend, select UTXOs that sum to at least the amount you need, plus a
change output for the leftover.

The SDK never asks the chain "what's my balance" — it tracks UTXOs
locally, derives a balance, and updates as new ones land or existing
ones spend. See [Wallet → Architecture](wallet/architecture.md) and
[Wallet SDK → UTXO Context](wallet-sdk/utxo-context.md).

## Virtual chain and DAA score

Two ordering scalars show up in the SDK:

- **DAA score** (Difficulty Adjustment Algorithm) — a monotonic counter
  that grows roughly with wall-clock time. Use as an "age of block"
  comparator; appears on every UTXO via `block_daa_score`.
- **Virtual chain** — the canonical DAG ordering. The
  [`virtual-chain-changed`](rpc/subscriptions.md#virtual-chain-progression)
  notification reports updates; a transaction is confirmed by appearing
  in it.

For "wait N seconds", DAA score is roughly right. For "wait until this
transaction is confirmed", use a `Maturity` event (see below).

## Maturity

A UTXO moves through three states in the wallet's view:

- **Pending** — seen, not yet confirmed deeply enough to spend.
- **Mature** — confirmed past the maturity threshold; spendable.
- **Outgoing** — locked because the wallet just spent it; awaits the
  spend transaction maturing or being re-orged out.

[Coinbase outputs](https://kaspa-mdbook.aspectron.com/) have a longer
maturity than regular outputs. The SDK applies the right threshold
automatically — you observe via `Pending` / `Maturity` events on the
[managed Wallet](wallet/transaction-history.md) or the
[`UtxoProcessor`](wallet-sdk/utxo-processor.md).

The right "wait for confirmation" gate is the `Maturity` event for the
specific transaction — not `Pending`, and not "wait N seconds".

## Mass and the fee market

Every transaction has a *mass* — a number derived from byte size,
compute cost, and a storage cost (a function of input and output
values). Mass replaces the "size × byte rate" fee model used by some
other UTXO chains.

The required fee is `mass × fee_rate`, where `fee_rate` reflects current
congestion. Query the rate via
[`client.get_fee_estimate()`](rpc/calls.md#fees) or the wallet's
[`fee_rate_estimate()`](wallet/send-transaction.md#fees).

The [Transaction Generator](wallet-sdk/tx-generator.md) handles all of
this — it computes mass, picks a rate, and folds the leftover into
change. Set fees explicitly only for non-standard policies (priority
surcharges, exact-balance sweeps, multisig with custom signature
counts).

## Sompi and KAS

The atomic unit is the **sompi**: `1 KAS = 100_000_000 sompi`. Every
amount in the SDK — UTXO value, output amount, fee, balance — is in
sompi. Convert at the UI boundary with `sompi_to_kaspa(...)` /
`kaspa_to_sompi(...)`. Don't store KAS as a float internally; use
sompi ints.

## Subnetworks

Most transactions live on the default subnetwork (id all zeros). The
field exists for protocol extensions; leave
`subnetwork_id="0000...0"` unchanged when building manually.

## Where to next

- [Networks](networks.md) — picking a chain to talk to.
- [Addresses](addresses.md) and
  [Transactions](transactions/overview.md) — the on-chain primitives
  in Python.
- [Wallet → Architecture](wallet/architecture.md) — how the SDK turns
  these concepts into a working wallet.
