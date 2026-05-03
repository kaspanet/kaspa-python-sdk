---
search:
  boost: 3
---

# Transaction Generator

The [`Generator`](../../reference/Classes/Generator.md) is the SDK's
built-in coin selector and fee calculator. You hand it UTXOs, a change
address, and the outputs you want; it picks inputs, computes mass and
fees, and yields one or more
[`PendingTransaction`](../../reference/Classes/PendingTransaction.md)s
ready to sign and submit.

## Idiomatic: feed it a `UtxoContext`

In a long-running process, build a [`UtxoContext`](../../reference/Classes/UtxoContext.md) (see [UTXO Context](utxo-context.md))
once and pass it as `entries`. The generator pulls the current mature
set on iteration — no manual UTXO snapshot, no stale data. [`PaymentOutput`](../../reference/Classes/PaymentOutput.md) and [`NetworkId`](../../reference/Classes/NetworkId.md) describe the destination.

```python
gen = Generator(
    network_id=NetworkId("mainnet"),
    entries=context,                         # UtxoContext
    change_address=my_addr,
    outputs=[PaymentOutput(recipient, 100_000_000)],   # 1 KAS
)
for pending in gen:
    pending.sign([key])
    print("submitted:", await pending.submit(client))
```

A [`Generator`](../../reference/Classes/Generator.md) is *iterable* — when the input set is too large for one
transaction's mass budget, it yields a chain of consolidating
transactions followed by the final payment. Loop and [`submit`](../../reference/Classes/PendingTransaction.md#kaspa.PendingTransaction.submit) each.

For the full processor → context → generator wiring, see
[Wallet SDK → Overview](overview.md#end-to-end-without-a-managed-wallet).

## One-shot: raw UTXO list

Without a context (one-shot scripts, ad-hoc tools), pass the entries
list straight from
[`get_utxos_by_addresses`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.get_utxos_by_addresses) (see [RPC → Calls](../rpc/calls.md#balances-and-utxos)):

```python
import asyncio
from kaspa import (
    RpcClient, Resolver, Generator, PaymentOutput,
    PrivateKey, NetworkId,
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

        gen = Generator(
            network_id=NetworkId("mainnet"),
            entries=utxos["entries"],
            change_address=my_addr,
            outputs=[PaymentOutput(my_addr, 100_000_000)],   # 1 KAS
        )
        for pending in gen:
            pending.sign([key])
            print("submitted:", await pending.submit(client))

        print(gen.summary().fees, gen.summary().transactions)
    finally:
        await client.disconnect()

asyncio.run(main())
```

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

`entries` accepts a [`UtxoContext`](../../reference/Classes/UtxoContext.md) directly — pass
the context and it consumes from the mature set without you copying
the list.

## Estimate before signing

Two entry points, same answer. Use [`gen.estimate()`](../../reference/Classes/Generator.md) if you already
have a [`Generator`](../../reference/Classes/Generator.md); use [`estimate_transactions()`](../../reference/Functions/estimate_transactions.md) to quote a hypothetical
send without constructing one.

```python
# You already have a Generator — most common case.
summary = gen.estimate()
print(summary.fees, summary.transactions, summary.utxos)

# Standalone — no Generator yet.
from kaspa import estimate_transactions

summary = estimate_transactions(
    network_id="mainnet",
    entries=utxos,
    change_address=my_addr,
    outputs=[{"address": recipient, "amount": amount}],
)
```

[`estimate()`](../../reference/Classes/Generator.md) doesn't consume the generator — you can iterate it for
real afterwards.

## Pending transactions

Each yielded item exposes the proposed transaction's metadata:

```python
for pending in gen:
    print(pending.id, pending.fee_amount, pending.mass)
    print(pending.payment_amount, pending.change_amount, pending.transaction_type)
    inputs = pending.get_utxo_entries()
    addrs = pending.addresses()
    raw_tx = pending.transaction
```

Use this to surface fee / mass to a user before they authorize a
signature.

## Signing

The everyday path: sign every input with one or more keys.

```python
pending.sign([key])
pending.sign([key1, key2, key3])     # multisig
```

### Advanced: per-input and custom scripts

Reach for these only when single-call signing isn't enough — usually
mixed-key sets, hardware-signer integrations, or non-standard scripts.
See [`examples/transactions/multisig.py`](https://github.com/kaspanet/kaspa-python-sdk/blob/main/examples/transactions/multisig.py)
for a full multisig run.

[`SighashType`](../../reference/Enums/SighashType.md) selects which parts of the transaction the signature commits to:

```python
from kaspa import SighashType

# Per-input control with different keys
for i, _ in enumerate(pending.get_utxo_entries()):
    pending.sign_input(i, key)

# Custom signature scripts
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
    "transaction": pending.transaction,
    "allowOrphan": False,
})
```

[`pending.submit(client)`](../../reference/Classes/PendingTransaction.md#kaspa.PendingTransaction.submit) is the right path. The manual route is for
round-tripping the transaction through another system before
submission. See
[Transactions → Submission](../transactions/submission.md) for the
`allowOrphan` semantics and confirmation states.

## One-shot helpers

Two free functions for when the loop-and-submit pattern is more code
than you need:

- **[`create_transaction`](../../reference/Functions/create_transaction.md)** (singular) — builds a single
  [`Transaction`](../../reference/Classes/Transaction.md). Use it when you know the input set fits in one tx.
- **[`create_transactions`](../../reference/Functions/create_transactions.md)** (plural) — wraps [`Generator`](../../reference/Classes/Generator.md) end-to-end
  and returns `{"transactions": [...], "summary":`[`GeneratorSummary`](../../reference/Classes/GeneratorSummary.md)`}`,
  matching the chain you'd get from iterating [`Generator`](../../reference/Classes/Generator.md) yourself.

```python
from kaspa import create_transaction, create_transactions

# Singular — one transaction.
tx = create_transaction(
    utxo_entry_source=utxos,
    outputs=[{"address": "kaspa:...", "amount": 100_000_000}],   # 1 KAS
    priority_fee=1000,
)

# Plural — Generator-equivalent.
result = create_transactions(
    network_id="mainnet",
    entries=utxos,
    change_address=my_addr,
    outputs=[{"address": "kaspa:...", "amount": 100_000_000}],   # 1 KAS
    priority_fee=1000,
)
for pending in result["transactions"]:
    pending.sign([key])
    await pending.submit(client)

print(result["summary"])
```

## Where to next

- [UTXO Context](utxo-context.md) — what to pass as `entries` in a
  long-running process.
- [Wallet → Send Transaction](../wallet/send-transaction.md) — the
  managed [`Wallet`](../../reference/Classes/Wallet.md) wraps [`Generator`](../../reference/Classes/Generator.md) with sensible defaults.
- [Transactions → Submission](../transactions/submission.md) —
  `allowOrphan` semantics, confirmation states.
- [Examples](../../examples.md) — runnable scripts including a full
  multisig flow.
