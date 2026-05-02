# Mass & fees

Kaspa uses a **mass-based fee model**. Every transaction has a *mass*
— a number derived from its byte size, compute cost, and a storage
component tied to input and output values. The required fee is
`mass × fee_rate`, where `fee_rate` is the network's current rate.

## The two kinds of mass

- **Compute / size mass** — from the transaction's serialized size
  and signature operations.
- **Storage mass** — from input and output *values*, specifically to
  discourage UTXO-set bloat (many tiny outputs from one large input).

A transaction's overall mass combines the two; you don't compute the
parts separately for normal use.

## Compute mass for a transaction

The relevant helpers:
[`calculate_transaction_mass`](../../reference/Functions/calculate_transaction_mass.md),
[`update_transaction_mass`](../../reference/Functions/update_transaction_mass.md),
[`maximum_standard_transaction_mass`](../../reference/Functions/maximum_standard_transaction_mass.md):

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

[`calculate_transaction_mass(network_id, tx)`](../../reference/Functions/calculate_transaction_mass.md) returns the mass without
mutating the transaction. To write it onto the transaction itself
(required before signing or serializing):

```python
ok = update_transaction_mass("mainnet", tx)
print(tx.mass, ok)   # ok is False if mass exceeds the standard limit
```

!!! warning "Order matters"
    Run [`update_transaction_mass`](../../reference/Functions/update_transaction_mass.md) after inputs and outputs are
    settled but **before** signing or serializing. Mass is part of
    the signed payload — sign first and you'll be signing over
    `mass=0`.

For multisig estimation, both calls take an optional
`minimum_signatures` to size the signature script correctly:

```python
update_transaction_mass("mainnet", tx, minimum_signatures=2)
```

## Storage mass on its own

[`calculate_storage_mass`](../../reference/Functions/calculate_storage_mass.md) computes only the storage component:

```python
from kaspa import calculate_storage_mass

storage_mass = calculate_storage_mass(
    network_id="mainnet",
    input_values=[1_000_000, 2_000_000],
    output_values=[2_500_000, 400_000],
)
```

Useful for sizing change outputs: if a tiny change output pushes
storage mass through the roof, fold it into the fee instead.

## Fees

```python
from kaspa import calculate_transaction_fee

fee = calculate_transaction_fee("mainnet", tx)
print(fee)   # required fee in sompi, or None if mass exceeds the standard limit
```

[`calculate_transaction_fee`](../../reference/Functions/calculate_transaction_fee.md) returns the minimum required fee at the
network's current rate, or `None` if the transaction's mass exceeds
[`maximum_standard_transaction_mass()`](../../reference/Functions/maximum_standard_transaction_mass.md). Split the inputs across
multiple transactions in that case.

## Querying the fee rate

The network exposes a fee estimator over RPC — see
[RPC → Calls → Fees](../rpc/calls.md#fees) and
[`get_fee_estimate`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.get_fee_estimate):

```python
estimate = await client.get_fee_estimate({})
print(estimate["estimate"]["priorityBucket"]["feerate"])
print(estimate["estimate"]["normalBuckets"])
print(estimate["estimate"]["lowBuckets"])
```

Each bucket carries a `feerate` (sompi-per-gram-of-mass) and an
`estimatedSeconds` for time-to-confirmation at that rate. Pick a
bucket by how much you care about latency, multiply by mass, and
you have a fee.

The [`Wallet`](../../reference/Classes/Wallet.md) wraps this as
[`fee_rate_estimate()`](../../reference/Classes/Wallet.md#kaspa.Wallet.fee_rate_estimate) (see [the fee model](../wallet/send-transaction.md#fee-model)) and
the [`Generator`](../../reference/Classes/Generator.md) picks a sensible default if you don't pass `fee_rate=`
explicitly.

## When to set fees explicitly

The [`Generator`](../../reference/Classes/Generator.md) (see [Transaction Generator](../wallet-sdk/tx-generator.md)) picks a fee rate,
computes mass, and folds the leftover into change. Override only:

- **`fee_rate=`** — when you have a specific sompi-per-gram in mind.
- **`priority_fee=`** — to add a flat surcharge on top.
- **Manual path** — when sizing change around the fee yourself.

For typical sends, the defaults are fine.
