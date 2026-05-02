# UTXO Context

A [`UtxoContext`](../../reference/Classes/UtxoContext.md) tracks UTXOs
for a fixed set of addresses. It's bound to a
[`UtxoProcessor`](../../reference/Classes/UtxoProcessor.md) (see [UTXO Processor](utxo-processor.md)) and fed by it: as the processor
receives notifications from the node, it routes changes to whichever
contexts have registered the relevant addresses. The context exposes
the resulting UTXO set, balance, and mature/pending splits.

The managed [`Wallet`](../../reference/Classes/Wallet.md) (see [Wallet](../wallet/overview.md)) creates one [`UtxoContext`](../../reference/Classes/UtxoContext.md)
per activated account internally.

[`UtxoContext`](../../reference/Classes/UtxoContext.md) can be used when you want UTXO tracking outside of [`Wallet`](../../reference/Classes/Wallet.md).

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
print(context.is_active)        # bool — see "Lifecycle" below
print(context.balance)          # Balance | None
print(context.balance_strings)  # BalanceStrings | None (formatted)
print(context.mature_length)    # int — number of spendable UTXOs

mature = context.mature_range(from_=0, to=10)   # list[UtxoEntryReference]
pending = context.pending()                     # list[UtxoEntryReference]
```

`balance` is `None` until the first notification arrives; after that
it's a [`Balance(mature, pending, outgoing)`](../../reference/Classes/Balance.md) in sompi
([`balance_strings`](../../reference/Classes/BalanceStrings.md) returns the formatted form);
[`mature_range`](../../reference/Classes/UtxoContext.md) and [`pending`](../../reference/Classes/UtxoContext.md) return [`UtxoEntryReference`](../../reference/Classes/UtxoEntryReference.md)s.

## Lifecycle

A [`UtxoContext`](../../reference/Classes/UtxoContext.md) has no separate active/inactive state of its own —
`context.is_active` mirrors the bound processor. Implications:

- The context's address set and in-memory UTXOs persist as long as
  the Python object lives, even if the processor is stopped or the
  socket dropped. On reconnect the processor re-registers them
  automatically.
- [`track_addresses(addresses, current_daa_score=None)`](../../reference/Classes/UtxoContext.md) adds addresses,
  subscribes the processor to `UtxosChanged` for them, and seeds the
  mature set via a single [`get_utxos_by_addresses`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.get_utxos_by_addresses) RPC. Pass
  `current_daa_score` only if you need the maturity classification of
  the seeded UTXOs to use a DAA score other than the processor's
  current one — for example, when reconstructing balances at a
  specific historical point.
- [`unregister_addresses([...])`](../../reference/Classes/UtxoContext.md) removes those addresses and stops the
  matching `UtxosChanged` subscription. UTXOs they contributed remain
  in the in-memory set until evicted by a notification or by [`clear()`](../../reference/Classes/UtxoContext.md).
- [`clear()`](../../reference/Classes/UtxoContext.md) unregisters every tracked address (unsubscribing from the
  node) and drops every cached UTXO. The context is reusable —
  [`track_addresses`](../../reference/Classes/UtxoContext.md) again to rehydrate it.

```python
await context.track_addresses(["kaspatest:..."])
await context.unregister_addresses(["kaspatest:..."])
await context.clear()
```

[`track_addresses`](../../reference/Classes/UtxoContext.md) accepts [`Address`](../../reference/Classes/Address.md) instances or their string forms.

## Use as `Generator` input

The [`Generator`](../../reference/Classes/Generator.md) (see [Transaction Generator](tx-generator.md)) accepts a [`UtxoContext`](../../reference/Classes/UtxoContext.md)
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

- [Transaction Generator](tx-generator.md) — sending using a
  `UtxoContext` as input.
- [UTXO Processor](utxo-processor.md) — the engine the context is
  bound to.
- [Wallet → Architecture](../wallet/architecture.md) — how the managed
  Wallet uses `UtxoContext`s internally.
