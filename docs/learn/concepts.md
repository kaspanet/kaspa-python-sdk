# Kaspa Concepts

This page is a fast tour of the protocol concepts you bump into when
using the SDK. It's deliberately surface-level — for the protocol-level
details, see the [Kaspa MDBook](https://kaspa-mdbook.aspectron.com/).

## BlockDAG, not blockchain

Kaspa orders transactions in a **directed acyclic graph of blocks**, not
a linear chain. Multiple blocks can be produced in parallel and reference
the same parents; consensus emerges from a deterministic ordering of
the DAG (the *virtual chain*) rather than from a single longest chain.

The practical consequence for SDK users:

- **Block rate is high** (one or more blocks per second on testnet-10).
  You see far more `block-added` events than you would on a Bitcoin-shaped
  chain.
- **Transactions confirm via the virtual chain.** A transaction is
  "accepted" once it appears in the virtual-chain ordering, not when it
  first lands in a block.
- **Reorgs happen at the DAG-ordering level.** A previously-accepted
  transaction can be re-ordered out; the SDK surfaces this as a
  [`Reorg` event](wallet/transaction-history.md).

## UTXO model

Every spendable balance is a set of *unspent transaction outputs*. To
spend, you select UTXOs that sum to at least the amount you need, plus a
change output for the leftover.

The SDK never asks "what's my balance" of the chain directly — it tracks
UTXOs locally, derives a balance from them, and updates as new ones land
or existing ones spend. See
[Wallet → Architecture](wallet/architecture.md) and
[Wallet SDK → UTXO Context](wallet-sdk/utxo-context.md).

## Virtual chain and DAA score

Two ordering scalars come up in the SDK:

- **DAA score** (Difficulty Adjustment Algorithm) — a monotonic counter
  that increases roughly with wall-clock time. Used as a "what age is
  this block" comparator; surfaces on every UTXO via
  `block_daa_score`.
- **Virtual chain** — the canonical DAG ordering. The
  `virtual-chain-changed` notification reports updates to it; transactions
  are confirmed by appearing in it.

For "wait until N seconds have passed", DAA score is roughly the right
metric. For "wait until this transaction is confirmed", a `Maturity`
event is the right gate (see below).

## Maturity

A UTXO moves through three states in the wallet's view:

- **Pending** — seen, but not deeply enough confirmed to be spent.
- **Mature** — confirmed past the maturity threshold; spendable.
- **Outgoing** — locked because the wallet just spent it; awaits its
  spend transaction maturing or being re-orged out.

[Coinbase outputs](https://kaspa-mdbook.aspectron.com/) have a longer
maturity than regular transaction outputs. The SDK applies the right
threshold automatically — you observe the result via `Pending` /
`Maturity` events on either the [managed
Wallet](wallet/transaction-history.md) or the
[`UtxoProcessor`](wallet-sdk/utxo-processor.md).

The right "wait for confirmation" gate is the `Maturity` event for the
specific transaction you care about — not `Pending`, and not "wait N
seconds".

## Mass and the fee market

Every transaction has a *mass* — a number derived from the transaction's
byte size, its compute cost, and its storage cost (a function of input
and output values). Mass replaces the simple "size × byte rate" fee
model some other UTXO chains use.

The required fee for a transaction is `mass × fee_rate`, where
`fee_rate` is set by the network's current congestion. You query the
prevailing rate via `client.get_fee_estimate()` (see
[RPC → Calls](rpc/calls.md)) or via the wallet's
`fee_rate_estimate()` (see
[Wallet → Send Transaction](wallet/send-transaction.md)).

The [Transaction Generator](wallet-sdk/tx-generator.md) handles all of
this for you — it computes mass, picks a fee rate, and adds a change
output that absorbs the leftover. You only set fees explicitly when you
want a non-standard policy (priority surcharges, exact-balance sweeps,
multisig with custom signature counts).

## Sompi and KAS

The atomic unit is the **sompi**: 1 KAS = 100 000 000 sompi. Every
amount in the SDK — UTXO value, output amount, fee, balance — is in
sompi. Convert at the UI boundary with `sompi_to_kaspa(...)` /
`kaspa_to_sompi(...)`. Don't store KAS-as-float anywhere internal; use
ints in sompi.

## Subnetworks

Most transactions live on the default subnetwork (id all zeros). The
field exists for protocol extensions; you'll usually leave
`subnetwork_id="0000...0"` unchanged when building manually.

## Where to next

- [Networks](networks.md) — picking a chain to talk to.
- [Addresses](addresses.md) and [Transactions](transactions/overview.md) — the
  on-chain primitives in Python.
- [Wallet → Architecture](wallet/architecture.md) — how the SDK turns
  these concepts into a wallet you can actually use.
