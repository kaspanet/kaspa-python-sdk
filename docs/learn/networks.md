---
search:
  boost: 3
---

# Networks

The Kaspa community runs various public networks: a production mainnet and a few testnets.
Every SDK entry point that hits the chain
([`RpcClient`](../reference/Classes/RpcClient.md),
[`Wallet`](../reference/Classes/Wallet.md),
[`Address`](../reference/Classes/Address.md),
[derivation](wallet-sdk/derivation.md), etc.) needs a network identifier to pick the right
chain and address prefix.

## The networks

| Network | `network_id` | Address prefix |
| --- | --- | --- |
| Mainnet | `"mainnet"` | `kaspa:` |
| Testnet 10 | `"testnet-10"` | `kaspatest:` |
| Testnet 11 | `"testnet-11"` | `kaspatest:` |

Operator-run **devnet** (`kaspadev:`) and **simnet**
(`kaspasim:`) also exist for private chains and simulators.

## Using the identifier

```python
from kaspa import RpcClient, Resolver

client = RpcClient(resolver=Resolver(), network_id="testnet-10")
```

Most APIs accept the string form (`"mainnet"`, `"testnet-10"`).

## `network_id` strings, `NetworkId`, and `NetworkType`

Three forms turn up in different APIs:

- **Plain strings** (`"testnet-10"`) — what most call sites accept.
  Carries the suffix.
- **[`NetworkId`](../reference/Classes/NetworkId.md)** — typed, also
  carries the suffix. Useful when you want to hold a value without
  re-parsing it. Build with `NetworkId("testnet-10")`; read parts with
  `.network_type` and `.suffix`.
- **[`NetworkType`](../reference/Enums/NetworkType.md)** — just the
  base kind (`Mainnet`, `Testnet`, `Devnet`, `Simnet`). **No suffix.**
  Sufficient anywhere only the address prefix matters (key-to-address
  derivation, multisig address creation), since testnet-10 and
  testnet-11 share the same `kaspatest:` prefix. Not enough to pick a
  specific testnet for an RPC client.

Rule of thumb: pass strings or [`NetworkId`](../reference/Classes/NetworkId.md) to anything that talks to
the chain; [`NetworkType`](../reference/Enums/NetworkType.md) is fine for derivation and address encoding.

## What changes between networks

- **Address prefix.** A key derived under `mainnet` produces
  `kaspa:...`; the same key under `testnet-10` produces
  `kaspatest:...`. See [Addresses](addresses.md).
- **Genesis and chain state.** Each network has its own UTXO set; funds
  on one don't exist on another.
- **Resolver pool.** A
  [`Resolver`](rpc/resolver.md) only returns nodes for the configured
  `network_id`.
- **Maturity depths and block rate.** Coinbase maturity and block rate
  differ by network; the SDK applies the right values automatically.
  See [UTXO maturity](wallet/send-transaction.md#utxo-maturity).
