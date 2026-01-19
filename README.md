# Kaspa Python SDK

The Kaspa Python SDK provides bindings to Rust & [Rusty-Kaspa](https://github.com/kaspanet/rusty-kaspa) source, allowing Python developers to interact with the Kaspa BlockDAG.

A native extension module, `kaspa`, is built from these bindings using [PyO3](https://pyo3.rs/) and [Maturin](https://www.maturin.rs).

> [!IMPORTANT]
> This project has moved! Welcome to the new home of Kaspa Python SDK, which historically lived [here](https://github.com/aspectron/rusty-kaspa/tree/python).
> 
> Versions < 1.1.0 were built from the old repository.
>
> Versions >= 1.1.0 are/will be built from this repository.


## Features

This SDK provides features in two primary categories:

- RPC Client - Connect to Kaspa nodes & PNN, perform calls, subscriptions, etc.
- Wallet Management - Wallet related functionality (key management, derivation, addresses, transactions, etc.).

This package strives to mirror the [Kaspa WASM32 SDK](https://kaspa.aspectron.org/docs/) from a feature and API perspective, while respecting Python conventions.

Most feature gaps with WASM32 SDK exist around Wallet functionality. Over time, features will be added to the Python SDK to bring it as close as possible.

## Documentation

Full documentation is available on the [documentation site](https://kaspanet.github.io/kaspa-python-sdk/dev/), including (but not limited to):

- [Installation Guide](https://kaspanet.github.io/kaspa-python-sdk/dev/getting-started/installation/)
- [Examples](https://kaspanet.github.io/kaspa-python-sdk/dev/getting-started/examples/)
- [API Reference](https://kaspanet.github.io/kaspa-python-sdk/dev/reference/)

The documentation site is versioned:
- `dev` refers to the latest in `main` branch
- `latest` refers to the most recent production release
- Specific version tags are available


Documentation is not available for versions prior to 1.1.0. However, the API is very close, if not the exact same, for those versions and 1.1.0.

## Quick Install

The Kaspa Python SDK is available on PyPi ([link](https://pypi.org/project/kaspa/)). As such, it can be installed from PyPi via:

```bash
pip install kaspa
```

Security-critical applications should consider building and installing from source. Instructions can be found [here](https://kaspanet.github.io/kaspa-python-sdk/dev/getting-started/installation/#installation-from-source) on the documentation site.

## Example

A very basic RPC example:

```python
import asyncio
from kaspa import Resolver, RpcClient

async def main():
    # Connect to Public Node Network (PNN) with Resolver
    client = RpcClient(resolver=Resolver())
    await client.connect()

    # Execute RPC Call
    print(await client.get_block_dag_info())

if __name__ == "__main__":
    asyncio.run(main())
```

Additional detailed examples can be found in the following locations:
- The [Examples section of the documentation site](https://kaspanet.github.io/kaspa-python-sdk/dev/getting-started/examples/)
- The [examples directory of this project](https://github.com/kaspanet/kaspa-python-sdk/tree/main/examples)


## Core Concepts & Contributing

The [Contributing Guide](https://kaspanet.github.io/kaspa-python-sdk/dev/contributing/) details various technical core concepts and information about this project.

## License

This project is licensed under the ISC License.
