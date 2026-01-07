# Kaspa Python SDK

This Python package, `kaspa`, provides an SDK for interacting with the Kaspa network from Python.

`kaspa` is a native extension module built from bindings to Rust and [rusty-kaspa](https://github.com/kaspanet/rusty-kaspa) source. [PyO3](https://pyo3.rs/) and [Maturin](https://www.maturin.rs/) are used to create bindings and build the extension module. More information can be found in 

!!! warning "Beta Status"
    This project is considered beta status.

This project very closely mirrors [Kaspa's WASM SDK](https://kaspa.aspectron.org/docs/), while trying to respect Python conventions. Feature parity with WASM SDK is a work in progress, not all features are available yet in Python.

This documentation site currently provides API reference and basic usage guides. General cryptocurrency concepts, development practices, and Kaspa specific concepts are not covered here.

## Features

This SDK provides features in two primary categories:

- **RPC Client** - Connect to Kaspa nodes via RPC.
- **Wallet Management** - Wallet related functionality (key management, derivation, addresses, transactions, etc.).

Most features gaps with Kaspa WASM SDK exist around Wallet functionality.

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

## Getting Started

1. [Installation](getting-started/installation.md) - Set up the SDK in your environment
2. [Examples](getting-started/examples.md) - Build your first Kaspa application

## Guides

<div class="grid cards" markdown>

- **[Addresses](guides/addresses.md)**  
  Create and validate Kaspa addresses

- **[Mnemonics](guides/mnemonics.md)**  
  Generate and use seed phrases

- **[Key Derivation](guides/key-derivation.md)**  
  HD wallet key generation

- **[Transactions](guides/transactions.md)**  
  Build and sign transactions

- **[Message Signing](guides/message-signing.md)**  
  Sign and verify messages

- **[RPC Client](guides/rpc-client.md)**  
  Connect to Kaspa nodes

</div>

## API Reference

For complete API documentation, see the [API Reference](reference/index.md).

## License

This project is licensed under the ISC License.

