# Wallet SDK

The **Wallet SDK** section is the layer beneath the managed
[Wallet](../wallet/index.md). When you don't need on-disk file storage,
multi-account management, or the wallet's event multiplexer — when you
just want to derive a key, build a transaction, or track UTXOs for a few
addresses — drop down here.

## What lives here

| Page | What it covers |
| --- | --- |
| [Key Management](key-management.md) | `Mnemonic`, BIP-39 seed, `XPrv`, hex import/export. |
| [Derivation](derivation.md) | `PrivateKeyGenerator`, `PublicKeyGenerator`, `DerivationPath`, BIP-44. |
| [Transaction Generator](tx-generator.md) | The `Generator` class — UTXO selection, fees, signing, submission. |
| [UTXO Context](utxo-context.md) | `UtxoContext`: per-address UTXO tracking. |
| [UTXO Processor](utxo-processor.md) | `UtxoProcessor`: the engine that drives `UtxoContext`s. |

## Wallet vs. Wallet SDK in one table

| You want to... | Use |
| --- | --- |
| Open a file, manage many accounts, track them long-term | [Wallet](../wallet/index.md) |
| Sign one transaction in a script with a key you already have | Wallet SDK ([Transaction Generator](tx-generator.md)) |
| Derive an address from a mnemonic without persisting anything | Wallet SDK ([Key Management](key-management.md), [Derivation](derivation.md)) |
| Watch a fixed set of addresses for incoming UTXOs without a wallet file | Wallet SDK ([UTXO Processor](utxo-processor.md), [UTXO Context](utxo-context.md)) |

The managed `Wallet` is built from these pieces — every primitive on this
page is what `Wallet` wraps internally.

## Where to next

If you're new here, start with [Key Management](key-management.md) →
[Derivation](derivation.md) → [Transaction Generator](tx-generator.md).
That sequence walks the typical "make a key, build a transaction, send
it" flow without any file I/O.
