# Kaspa Python SDK

> [!WARNING]
> **This project is beta status, not intended for use in critical applications/environments.**
> 
> Kasa Python SDK source has historically lived [here](https://github.com/aspectron/rusty-kaspa/tree/python). The current `kaspa` PyPi package is built from that source.
>
> This repository is the potential future home of Kaspa Python SDK source. It has reached parity (feature, stability, etc.) with the existing codebase.
> 
> **You can build/install the `kaspa` Python package from this repository. See instructions [here](https://kaspanet.github.io/kaspa-python-sdk/dev/getting-started/installation/#installation-from-source).**

---

The Kaspa Python SDK provides bindings to Rust & [Rusty-Kaspa](https://github.com/kaspanet/rusty-kaspa) source for use in Python applications. Allowing Python developers to interact with the Kaspa BlockDAG.

A native extension module, `kaspa`, is built from these bindings using [PyO3](https://pyo3.rs/v0.20.0/) and [Maturin](https://www.maturin.rs).

## Features

This SDK provides features in two primary categories:

- RPC Client - Connect to Kaspa nodes via RPC.
- Wallet Management - Wallet related functionality (key management, derivation, addresses, transactions, etc.).

Most feature gaps with Kaspa WASM SDK exist around Wallet functionality. Over time, features will be added to the Kaspa Python SDK to bring it as close as possible.

## Documentation

Full documentation is available on the [documentation site](https://kaspanet.github.io/kaspa-python-sdk/dev/), including:

- [Installation Guide](https://kaspanet.github.io/kaspa-python-sdk/dev/getting-started/installation/)
- [Examples](https://kaspanet.github.io/kaspa-python-sdk/dev/getting-started/examples/)
- [API Reference](https://kaspanet.github.io/kaspa-python-sdk/dev/reference/)

And more.

## Quick Install

```bash
pip install kaspa
```

## Example

A very basic example:

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


## Contributing & Core Concepts

The [Contributing Guide](https://kaspanet.github.io/kaspa-python-sdk/dev/contributing/) details various technical core concepts and workflows used by this project.

## License

This project is licensed under the ISC License.
