# Metadata fields

Beyond inputs and outputs, a
[`Transaction`](../../reference/Classes/Transaction.md) carries five
fields that affect how it's interpreted on-chain. Typical sends take
the defaults — this page documents what they are so you know what to
leave alone and what to set deliberately.

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
version on Kaspa. The field exists for future formats; until then,
there's nothing to choose.

## `lock_time`

The earliest moment a transaction is allowed into a block, encoded as
a DAA-score threshold. `0` means "no lock" — what you want unless
building a time-locked construct (e.g. a refund branch).

```python
Transaction(..., lock_time=0, ...)
```

A non-zero value rejects the transaction from blocks whose DAA score
is below the threshold. See
[Kaspa Concepts → Virtual chain and DAA score](../concepts.md#virtual-chain-and-daa-score).

## `subnetwork_id`

The subnetwork the transaction belongs to. Most transactions live on
the default subnetwork — id all zeros — and that's what you pass when
building manually:

```python
subnetwork_id="0000000000000000000000000000000000000000"
```

Non-default subnetwork IDs are reserved for protocol-level transaction
kinds (coinbase, etc.) that you generally don't construct from the
SDK.

## `gas`

Reserved for subnetwork transactions with a compute-cost component.
On the default subnetwork it must be `0`. Pair with
`subnetwork_id="00...0"` and forget about it.

## `payload`

Arbitrary bytes attached to the transaction. The closest analog in
Bitcoin terms is `OP_RETURN`-style data, but `payload` lives at the
transaction level, not inside a script.

```python
Transaction(..., payload="68656c6c6f", ...)        # hex string
Transaction(..., payload=b"hello",     ...)        # or raw bytes
```

Use cases:

- **Application-level metadata** riding alongside a payment (invoice
  ID, memo, reference number).
- **Protocol-level data** for systems built on top of Kaspa
  transactions.

It's not a substitute for cryptographic state. Payload bytes are
hashed into the transaction ID and signed over, but they don't bind
the transaction to anything off-chain on their own.

The Generator accepts `payload=` directly:

```python
Generator(..., payload=b"invoice-12345")
```

## `mass`

The transaction's mass. `0` at construction; populate with
`update_transaction_mass(network_id, tx)` after inputs and outputs
are finalized and before signing or serializing. See
[Mass & fees](mass-and-fees.md).

## Where to next

- [Mass & fees](mass-and-fees.md) — the one metadata field you *must*
  update.
- [Serialization](serialization.md) — how these fields round-trip
  through `to_dict()` / `from_dict()`.
- [Kaspa Concepts](../concepts.md) — subnetworks, DAA score, virtual
  chain.
