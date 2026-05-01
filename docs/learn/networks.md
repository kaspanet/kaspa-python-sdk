# Networks

Kaspa runs three live networks: a production mainnet and two testnets.
Every SDK entry point that hits the chain — `RpcClient`, `Wallet`,
`Address`, derivation — needs a network identifier to pick the right
chain and address prefix.

## The networks

| Network | `network_id` | Address prefix | When to use |
| --- | --- | --- | --- |
| Mainnet | `"mainnet"` | `kaspa:` | Production. Real KAS. |
| Testnet 10 | `"testnet-10"` | `kaspatest:` | Mature testnet. Default for SDK examples; faucets available. |
| Testnet 11 | `"testnet-11"` | `kaspatest:` | Higher block-rate testnet for performance work. |
| Devnet | (operator-defined) | `kaspadev:` | A developer-run private chain. |
| Simnet | (operator-defined) | `kaspasim:` | Simulation / unit tests against a local sim. |

Most readers will only touch mainnet and testnet-10. Use testnet-11
only when you need its higher block rate — the SDK behaves identically
on it.

## `network_id` strings vs `NetworkId`

Most APIs accept the string form (`"mainnet"`, `"testnet-10"`).
[`NetworkId`](../reference/Classes/NetworkId.md) is the typed form —
useful when you want to hold a value without re-parsing:

```python
from kaspa import NetworkId

mainnet = NetworkId("mainnet")
testnet = NetworkId("testnet-10")
```

`NetworkId.network_type` and `NetworkId.suffix` return the parts.
[`NetworkType`](../reference/Enums/NetworkType.md) is a third form
some APIs accept (`NetworkType.Mainnet`, `NetworkType.Testnet`). All
three describe the same thing; pick whichever reads cleanly at the
call site.

## What changes between networks

- **Address prefix.** A key derived under `mainnet` produces
  `kaspa:...`; the same key under `testnet-10` produces
  `kaspatest:...`. See [Addresses](addresses.md).
- **Genesis and chain state.** Each network has its own UTXO set; funds
  on one don't exist on another.
- **Resolver pool.** A
  [`Resolver`](rpc/resolver.md) only returns nodes for the configured
  `network_id`.
- **Maturity depths.** Coinbase maturity differs by network; the SDK
  applies the right value automatically.

## Picking one for development

- **Writing examples / docs / tests:** use `testnet-10`. It's stable,
  has a faucet, and addresses are obviously test-shaped
  (`kaspatest:...`).
- **Performance experiments:** use `testnet-11`. Higher block rate
  means UTXO churn and event volume resemble a stress test.
- **Production code paths under CI:** parametrise the network — keep
  test runs on testnet, mainnet only on a release pipeline.
- **Anything touching mainnet:** read
  [Security](../getting-started/security.md) first.

## Where to next

- [Addresses](addresses.md) — what the prefix encodes and how versions
  fit in.
- [Wallet → Lifecycle](wallet/lifecycle.md#construct) — `network_id`
  is a required constructor argument.
- [RPC → Resolver](rpc/resolver.md) — how a `Resolver` finds a node for
  the configured network.
