# Networks

Kaspa runs three live networks: a production mainnet and two testnets.
Every SDK call that hits the chain — `RpcClient`, `Wallet`,
`Address`, derivation — needs a network identifier so it knows which
chain it's targeting and which address prefix to encode.

## The networks

| Network | `network_id` | Address prefix | When to use |
| --- | --- | --- | --- |
| Mainnet | `"mainnet"` | `kaspa:` | Production. Real KAS. |
| Testnet 10 | `"testnet-10"` | `kaspatest:` | The mature testnet. Default for SDK examples; faucets exist. |
| Testnet 11 | `"testnet-11"` | `kaspatest:` | Higher block-rate testnet for performance work. |
| Devnet | (operator-defined) | `kaspadev:` | A developer-run private chain. |
| Simnet | (operator-defined) | `kaspasim:` | Simulation / unit tests against a local sim. |

Mainnet and testnet-10 are what almost every reader of these docs will
touch. Reach for testnet-11 when you specifically need its higher block
rate; the SDK behaves identically on it.

## `network_id` strings vs `NetworkId`

Most APIs accept the string form (`"mainnet"`, `"testnet-10"`). The
`NetworkId` class is the typed form — useful when you want a value to
hold and pass around without re-parsing:

```python
from kaspa import NetworkId

mainnet = NetworkId("mainnet")
testnet = NetworkId("testnet-10")
```

`NetworkId.network_type` and `NetworkId.suffix` give you the parts back.
`NetworkType` (the enum) is what some APIs accept as a third form —
`NetworkType.Mainnet`, `NetworkType.Testnet`, etc. They all describe the
same thing; pick whichever the call site reads cleanly with.

## What changes between networks

- **Address prefix.** A key derived under `mainnet` produces
  `kaspa:...`; the same key under `testnet-10` produces `kaspatest:...`.
  See [Addresses](addresses.md).
- **Genesis and chain state.** Every network has its own UTXO set;
  funds on one network don't exist on another.
- **Resolver pool.** A `Resolver` only returns nodes for the
  `network_id` of the client it was given to.
- **Maturity depths.** Coinbase maturity differs by network; the SDK
  applies the right value automatically.

## Picking one for development

- **Writing examples / docs / tests:** use `testnet-10`. It's stable,
  there's a faucet, and addresses look obviously test-shaped
  (`kaspatest:...`).
- **Performance experiments:** use `testnet-11`. Block rate is higher,
  so UTXO churn and event volume look more like a stress test.
- **Production code paths under CI:** parametrise the network — keep
  test runs on testnet, mainnet only on a release pipeline.
- **Anything that ever touches mainnet:** read
  [Security](../getting-started/security.md) first.

## Where to next

- [Addresses](addresses.md) — what the prefix encodes and how versions
  fit in.
- [Wallet → Initialize](wallet/initialize.md) — `network_id` is a
  required constructor argument.
- [RPC → Resolver](rpc/resolver.md) — how a `Resolver` finds a node for
  the configured network.
