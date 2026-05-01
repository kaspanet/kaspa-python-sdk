# UTXO Context

A [`UtxoContext`](../../reference/Classes/UtxoContext.md) tracks UTXOs
for a fixed set of addresses. It's bound to a
[UTXO Processor](utxo-processor.md) and fed by it: as the processor
receives notifications from the node, it routes changes to whichever
contexts have registered the relevant addresses. The context exposes
the resulting UTXO set, balance, and mature/pending splits.

The managed [Wallet](../wallet/overview.md) creates a `UtxoContext`
per activated account internally — you usually don't construct one
yourself. Drop down here when you want UTXO tracking without an
on-disk wallet file.

## Build one

```python
from kaspa import NetworkId, Resolver, RpcClient, UtxoContext, UtxoProcessor

client = RpcClient(resolver=Resolver(), network_id="testnet-10")
await client.connect()

processor = UtxoProcessor(client, NetworkId("testnet-10"))
await processor.start()

context = UtxoContext(processor)
await context.track_addresses(["kaspatest:qr0lr4ml..."])
```

`UtxoContext(processor, id=...)` accepts an optional 32-byte hex id;
omitted ids are generated. Set an explicit id when you need the
context to be addressable across reconnects.

## What it exposes

```python
print(context.is_active)        # bool — processor running?
print(context.balance)          # Balance | None
print(context.balance_strings)  # BalanceStrings | None (formatted)
print(context.mature_length)    # int — number of spendable UTXOs

mature = context.mature_range(from_=0, to=10)   # list[UtxoEntryReference]
pending = context.pending()                     # list[UtxoEntryReference]
```

`balance` is `None` until the first notification arrives; after that
it's a `Balance(mature, pending, outgoing)` in sompi.

## Add and remove tracked addresses

```python
await context.track_addresses(["kaspatest:..."])
await context.unregister_addresses(["kaspatest:..."])
await context.clear()           # forget every address and UTXO
```

`track_addresses` accepts `Address` instances or their string forms.
`current_daa_score=...` is optional — supply it to ignore
confirmations older than that score.

## Use as `Generator` input

The [Transaction Generator](tx-generator.md) accepts a `UtxoContext`
as its `entries` argument:

```python
from kaspa import Generator, PaymentOutput

gen = Generator(
    entries=context,                 # mature UTXOs come from here
    change_address=my_addr,
    outputs=[PaymentOutput(recipient, 100_000_000)],
)
```

This avoids snapshotting the UTXO list yourself — the generator pulls
the current mature set when it iterates.

## Where to next

- [UTXO Processor](utxo-processor.md) — the engine the context is bound
  to.
- [Transaction Generator](tx-generator.md) — sending using a
  `UtxoContext` as input.
- [Wallet → Architecture](../wallet/architecture.md) — how the managed
  Wallet uses `UtxoContext`s internally.
