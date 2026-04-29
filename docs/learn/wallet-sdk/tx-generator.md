# Transaction Generator

The `Generator` is the SDK's built-in coin selector and fee calculator.
You hand it UTXOs, a change address, and the outputs you want; it picks
inputs, computes mass and fees, and yields one or more
`PendingTransaction`s ready to sign and submit.

## Send a payment, end to end

```python
import asyncio
from kaspa import (
    RpcClient, Resolver, Generator, PaymentOutput,
    Address, PrivateKey, NetworkId,
)

async def main():
    client = RpcClient(resolver=Resolver(), network_id="mainnet")
    await client.connect()
    try:
        key = PrivateKey("<64-char hex>")
        my_addr = key.to_address("mainnet")

        utxos = await client.get_utxos_by_addresses({
            "addresses": [my_addr.to_string()],
        })

        recipient = Address("kaspa:...")
        gen = Generator(
            network_id=NetworkId("mainnet"),
            entries=utxos["entries"],
            change_address=my_addr,
            outputs=[PaymentOutput(recipient, 500_000_000)],   # 5 KAS
        )

        for pending in gen:
            pending.sign([key])
            tx_id = await pending.submit(client)
            print("submitted:", tx_id)

        print(gen.summary().fees, gen.summary().transactions)
    finally:
        await client.disconnect()

asyncio.run(main())
```

A `Generator` is *iterable* — when the input set is too large for a single
transaction's mass budget, it yields a chain of consolidating transactions
followed by the final payment. Loop and submit each.

## Constructor options

```python
gen = Generator(
    network_id=NetworkId("mainnet"),
    entries=utxos,                 # list[UtxoEntry] OR a UtxoContext
    change_address=my_addr,
    outputs=[PaymentOutput(recipient, amount)],

    # optional
    payload=b"optional-data",      # OP_RETURN-equivalent payload
    priority_fee=1000,             # extra fee in sompi
    priority_entries=priority,     # UTXOs to consume first
    sig_op_count=1,                # signature ops per input
    minimum_signatures=1,          # for multisig mass estimation
    fee_rate=2.0,                  # explicit sompi/gram override
)
```

`entries` accepts a [`UtxoContext`](utxo-context.md) directly — pass the
context and it consumes from the mature set without you copying the list.

## Estimate before signing

```python
from kaspa import estimate_transactions

# via a Generator
summary = gen.estimate()
print(summary.fees, summary.transactions, summary.utxos)

# via the standalone function
summary = estimate_transactions(
    network_id="mainnet",
    entries=utxos,
    change_address=my_addr,
    outputs=[{"address": recipient, "amount": amount}],
)
```

`estimate()` does not consume the generator; you can iterate it for real
afterwards.

## Pending transactions

Each item the generator yields exposes the proposed transaction's
metadata:

```python
for pending in gen:
    print(pending.id, pending.fee_amount, pending.mass)
    print(pending.payment_amount, pending.change_amount, pending.transaction_type)
    inputs = pending.get_utxo_entries()
    addrs = pending.addresses()
    raw_tx = pending.transaction
```

Use this when you need to surface fee / mass to a user before they
authorize a signature.

## Signing

```python
# All inputs at once with one or more keys
pending.sign([key])
pending.sign([key1, key2, key3])     # multisig

# Per-input control
for i, _ in enumerate(pending.get_utxo_entries()):
    pending.sign_input(i, key)

# Custom signature scripts (advanced)
from kaspa import SighashType

sig = pending.create_input_signature(
    input_index=0,
    private_key=key,
    sighash_type=SighashType.All,
)
pending.fill_input(0, custom_script_bytes)
```

## Submit

```python
tx_id = await pending.submit(client)

# Or manually:
result = await client.submit_transaction({
    "transaction": pending.transaction.serialize_to_dict(),
    "allowOrphan": False,
})
```

`pending.submit(client)` is the right path. The manual route is for cases
where you need to round-trip the transaction through another system
before submission.

## One-shot helpers

When the loop-and-submit pattern is more code than you need:

```python
from kaspa import create_transaction, create_transactions

tx = create_transaction(
    utxo_entry_source=utxos,
    outputs=[{"address": "kaspa:...", "amount": 100_000_000}],
    priority_fee=1000,
)

result = create_transactions(
    network_id="mainnet",
    entries=utxos,
    change_address=my_addr,
    outputs=[{"address": "kaspa:...", "amount": 100_000_000}],
    priority_fee=1000,
)

for pending in result["transactions"]:
    pending.sign([key])
    await pending.submit(client)

print(result["summary"])
```

## Where to next

- [UTXO Context](utxo-context.md) — pass a context as `entries` instead
  of a raw list.
- [Wallet → Send Transaction](../wallet/send-transaction.md) — the
  managed Wallet wraps `Generator` with sensible defaults.
- [Multi-signature transactions](../../guides/multisig.md) — full multisig
  recipe including `minimum_signatures`.
