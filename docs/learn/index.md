# Learn

- **[RPC](rpc/overview.md)** — the `RpcClient`: resolver, connection, calls,
  notifications.
- **[Wallet](wallet/overview.md)** — the managed high-level `Wallet` API: lifecycle, file
  storage, accounts, addresses, sending, history.
- **[Wallet SDK](wallet-sdk/overview.md)** — the lower-level primitives that the
  managed `Wallet` is built on: key management, transaction `Generator`,
  derivation, `UtxoContext`, `UtxoProcessor`, etc.
- **[Networks](networks.md)** - working with the various Kaspa networks (mainnet, testnets, etc.) from this SDK.
- **[Addresses](addresses.md)** - a quick primer on Kaspa addresses and handling in this SDK.
- **[Transactions](transactions/overview.md)** — the on-chain primitives:
  inputs, outputs, mass and fees, signing, submission, metadata fields,
  and serialization.
- **[Kaspa Concepts](concepts.md)** — explanation of the BlockDAG, UTXO
  model, mass-based fees, and maturity. Read this if any of those terms feel
  fuzzy.

## Before you get started

Read [Security](../getting-started/security.md). The Learn snippets use
literal mnemonics, hex strings, and short passwords for readability — **that
is not how production code should handle secret material.**
