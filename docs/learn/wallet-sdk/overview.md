---
search:
  boost: 5
---

# Wallet SDK

The **Wallet SDK** is the set of primitives the managed
[`Wallet`](../../reference/Classes/Wallet.md) (see [Wallet](../wallet/overview.md)) is built on — and the toolkit you
reach for whenever you don't want a managed wallet at all (custom
signers, indexers, watch-only flows, hot-path services that keep
everything in memory).

This page is your section index. Every page below is independently
useful; read in order if you're new.

## What lives here

| Page | What it covers |
| --- | --- |
| [Key Management](key-management.md) | [`Mnemonic`](../../reference/Classes/Mnemonic.md), BIP-39 seed, [`XPrv`](../../reference/Classes/XPrv.md), hex import/export. |
| [Derivation](derivation.md) | [`PrivateKeyGenerator`](../../reference/Classes/PrivateKeyGenerator.md), [`PublicKeyGenerator`](../../reference/Classes/PublicKeyGenerator.md), BIP-44 paths. |
| [UTXO Processor](utxo-processor.md) | The engine: opens a node listener and dispatches events to contexts. |
| [UTXO Context](utxo-context.md) | Per-address UTXO tracking on top of a processor. |
| [Transaction Generator](tx-generator.md) | UTXO selection, fees, signing, submission. |

## End-to-end without a managed wallet

This is the canonical "primitives only" flow — a mnemonic, a derived
address, live UTXO tracking, and a signed-and-submitted send. No
`Wallet`, no on-disk file.

```python
import asyncio
from kaspa import (
    Generator, Mnemonic, NetworkId, NetworkType, PaymentOutput,
    PrivateKeyGenerator, Resolver, RpcClient, UtxoContext, UtxoProcessor,
    XPrv,
)

async def main():
    network = NetworkId("testnet-10")

    # 1. Key material
    xprv = XPrv(Mnemonic("<24-word phrase>").to_seed())
    keys = PrivateKeyGenerator(xprv=xprv, is_multisig=False, account_index=0)
    receive_key = keys.receive_key(0)
    receive_addr = receive_key.to_address(NetworkType.Testnet)

    # 2. Connect and start a processor
    client = RpcClient(resolver=Resolver(), network_id="testnet-10")
    await client.connect()
    try:
        processor = UtxoProcessor(client, network)
        await processor.start()

        # 3. Track the address — seeds the mature set and subscribes
        context = UtxoContext(processor)
        await context.track_addresses([receive_addr])

        # 4. Build, sign, submit
        gen = Generator(
            network_id=network,
            entries=context,                  # mature UTXOs flow from here
            change_address=receive_addr,
            outputs=[PaymentOutput(receive_addr, 100_000_000)],   # 1 KAS
        )
        for pending in gen:
            pending.sign([receive_key])
            print("submitted:", await pending.submit(client))

        await processor.stop()
    finally:
        await client.disconnect()

asyncio.run(main())
```

For the same flow with the managed [`Wallet`](../../reference/Classes/Wallet.md) (mnemonic stored on disk,
account state persisted), see
[Wallet → Send Transaction](../wallet/send-transaction.md).

## Where to next

- [Key Management](key-management.md) — start here if you have only a
  mnemonic or hex private key.
- [Derivation](derivation.md) — turn an `XPrv` into addresses.
- [UTXO Processor](utxo-processor.md) — set up the live UTXO event
  pipeline.
- [UTXO Context](utxo-context.md) — track UTXOs for a specific address
  set.
- [Transaction Generator](tx-generator.md) — build, sign, and submit.
