# Mass & fees

Kaspa uses a **mass-based fee model**. Every transaction has a *mass* —
a number derived from its byte size, its compute cost, and a storage
component that depends on input and output values. The required fee is
`mass × fee_rate`, where `fee_rate` is the prevailing rate the network
sets based on congestion.

For the protocol-level view of why mass exists, see
[Kaspa Concepts](../concepts.md). This page covers the SDK helpers.

## The two kinds of mass

- **Compute / size mass** — derived from the transaction's serialized
  size and signature operations.
- **Storage mass** — derived from input and output *values*, specifically
  to discourage outputs that bloat the UTXO set (many tiny outputs from
  one large input).

The total mass is the larger of the two. You don't compute the parts
separately for normal use — `calculate_transaction_mass` and
`update_transaction_mass` handle it. `calculate_storage_mass` is exposed
for when you want to inspect the storage component on its own.

## Compute mass for a transaction

```python
from kaspa import (
    calculate_transaction_mass,
    update_transaction_mass,
    maximum_standard_transaction_mass,
)

mass = calculate_transaction_mass("mainnet", tx)
print(mass)
print(maximum_standard_transaction_mass())   # protocol upper bound
```

`calculate_transaction_mass(network_id, tx)` returns the mass without
mutating the transaction. To write it onto the transaction itself
(required before signing or serializing), use:

```python
update_transaction_mass("mainnet", tx)
print(tx.mass)
```

**Order matters.** Run `update_transaction_mass` after your inputs and
outputs are settled but *before* signing or serializing. The mass field
is part of the signed payload — sign first and you'll be signing over
mass=0.

For multisig estimation, both calls take an optional
`minimum_signatures` to size the signature script correctly:

```python
update_transaction_mass("mainnet", tx, minimum_signatures=2)
```

## Storage mass on its own

```python
from kaspa import calculate_storage_mass

storage_mass = calculate_storage_mass(
    network_id="mainnet",
    input_values=[1_000_000, 2_000_000],
    output_values=[2_500_000, 400_000],
)
```

Useful for sizing change outputs: if your "tiny change output" pushes
storage mass through the roof, fold it into the fee instead.

## Fees

```python
from kaspa import calculate_transaction_fee

fee = calculate_transaction_fee("mainnet", tx)
print(fee)   # required fee in sompi
```

`calculate_transaction_fee` returns the minimum required fee for the
transaction at the network's current rate. The result is a sompi int (or
`None` if the calculation can't be performed — typically because the
transaction is malformed).

## Querying the fee rate

The network exposes a fee estimator over RPC:

```python
estimate = await client.get_fee_estimate({})
print(estimate["estimate"]["priorityBucket"]["feerate"])
print(estimate["estimate"]["normalBuckets"])
print(estimate["estimate"]["lowBuckets"])
```

Each bucket carries a `feerate` (sompi-per-gram-of-mass) and an
`estimatedSeconds` for "how long until this rate gets you confirmed".
Pick a bucket based on how much you care about latency, multiply by
mass, and you have a fee.

The Wallet wraps this as `fee_rate_estimate()` (see
[Send Transaction](../wallet/send-transaction.md)) and the Generator
picks a sensible default if you don't pass `fee_rate=` explicitly.

## When to set fees explicitly

The [Generator](../wallet-sdk/tx-generator.md) picks a fee rate, computes
mass, and folds the leftover into a change output. You only need to
override:

- **`fee_rate=`** — when you have a specific sompi-per-gram you want to
  pay.
- **`priority_fee=`** — when you want to add a flat surcharge on top of
  the computed fee.
- **Manual path** — when you're building the transaction yourself and
  need to size change outputs around the fee.

For typical sends, the defaults are fine.

## Where to next

- [Signing](signing.md) — runs after mass, before submission.
- [Submission](submission.md) — `submit_transaction` and what counts as
  confirmed.
- [Kaspa Concepts → Mass and the fee market](../concepts.md) — protocol
  background on why mass is shaped this way.
