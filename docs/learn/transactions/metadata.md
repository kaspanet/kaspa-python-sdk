---
search:
  boost: 3
---

# Metadata fields

Beyond inputs and outputs, a
[`Transaction`](../../reference/Classes/Transaction.md) carries six
fields that affect how it's interpreted on-chain. Most take defaults
— this page documents what they are so you know what to leave alone
and what to set deliberately.

```python
Transaction(
    version=0,
    inputs=...,
    outputs=...,
    lock_time=0,
    subnetwork_id="0000000000000000000000000000000000000000",
    gas=0,
    payload="",
    mass=0,
)
```

## `version`

Transaction format version. Use `0` — the only currently-defined
version on Kaspa.

## `lock_time`

Earliest moment a transaction is allowed into a block, encoded as a
DAA-score threshold. `0` means "no lock" — what you want unless
building a time-locked construct (e.g. a refund branch).

## `subnetwork_id`

The subnetwork the transaction belongs to. Most transactions live on
the default subnetwork — id all zeros:

```python
subnetwork_id="0000000000000000000000000000000000000000"
```

Non-default IDs are reserved for protocol-level transaction kinds
(coinbase, etc.) that you generally don't construct from the SDK.

## `gas`

`0` on the default subnetwork. Reserved for subnetwork transactions
with a compute-cost component.

## `payload`

Arbitrary bytes attached to the transaction. The closest analog in
Bitcoin terms is `OP_RETURN`-style data, but `payload` lives at the
transaction level, not inside a script. The SDK accepts a hex
string, raw bytes, or a list of byte values:

```python
Transaction(..., payload="68656c6c6f", ...)        # hex string
Transaction(..., payload=b"hello",     ...)        # raw bytes
```

Use cases:

- **Application-level metadata** riding alongside a payment (invoice
  ID, memo, reference number).
- **Protocol-level data** for systems built on top of Kaspa
  transactions.

Payload bytes are hashed into the transaction ID and signed over,
but they don't bind the transaction to anything off-chain on their
own.

The [`Generator`](../../reference/Classes/Generator.md) (see [Transaction Generator](../wallet-sdk/tx-generator.md)) accepts `payload=` directly:

```python
Generator(..., payload=b"invoice-12345")
```

## `mass`

The transaction's mass. `0` at construction; populate with
[`update_transaction_mass(network_id, tx)`](../../reference/Functions/update_transaction_mass.md) after inputs and outputs
are finalized and **before** signing or serializing — mass is part
of the signed payload. See [Mass & fees](mass-and-fees.md).
