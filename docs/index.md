# Kaspa Python SDK

This Python package, `kaspa`, provides an SDK for interacting with the
Kaspa network from Python. This SDK provides features in the following main categories:

- [RPC Client](learn/rpc/overview.md) — RPC API for the Kaspa node using WebSockets.
- [Wallet SDK](learn/wallet-sdk/overview.md) — Bindings for wallet-related primitives such as key management, derivation, and transactions.
- [Managed Wallet](learn/wallet/overview.md) — A high-level, single Python class interface to the Rusty Kaspa Wallet API. This provides full wallet functionality in the single Python class: `Wallet`.

This project closely mirrors
[Kaspa's WASM SDK](https://kaspa.aspectron.org/docs/), while trying to
respect Python conventions.

## Bindings to Rusty Kaspa

**`kaspa` is a Python-native extension module built from bindings to Rust and
[rusty-kaspa](https://github.com/kaspanet/rusty-kaspa) source.** [PyO3](https://pyo3.rs/) and [Maturin](https://www.maturin.rs/) are used
to create bindings and build the extension module.

!!! info "As-thin-as-possible"
      This project strives to provide as-thin-as-possible Python-compatible wrappers over rusty-kaspa source. Allowing Python developers leverage the features, stability, and security of rusty-kaspa directly, with minimal reimplementation in Python.

More information on bindings approach and development notes can be found in the
[Contributing section](contributing/index.md).

## A (Very) Basic Example

```python
import asyncio
from kaspa import Resolver, RpcClient

async def main():
    client = RpcClient(resolver=Resolver())
    await client.connect()
    print(await client.get_server_info())

if __name__ == "__main__":
    asyncio.run(main())
```

## How the docs are organised

<div class="grid cards" markdown>

- **[Getting Started](getting-started/installation.md)**  
  Install the SDK, run the first script, read the security note before
  generating real keys.

- **[Learn](learn/index.md)**  
  How the SDK is shaped, taught topic by topic. Connections, wallets,
  derivation, transactions, the Kaspa concepts behind them.

- **[Examples](examples.md)**  
  Runnable scripts on GitHub covering RPC, wallet, transactions,
  derivation, mnemonics, message signing, and addresses.

- **[API Reference](reference/index.md)**  
  Every public class, method, and signature. Auto-generated.

</div>

## License

This project is licensed under the ISC License.
