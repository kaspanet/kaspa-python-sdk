# Kaspa Python SDK

> [!CAUTION]
> **This repository is under active development. Intended solely to assess feasibility of moving Kaspa Python SDK into its own repository.**
>
> **Do NOT use this repository.**
> 
> **The current `kaspa` Python package source can be found [here](https://github.com/aspectron/rusty-kaspa/tree/python).**

---

The Kaspa Python SDK provides bindings to Rust & [Rusty-Kaspa](https://github.com/kaspanet/rusty-kaspa) source for use in Python applications. Allowing Python developers to interact with the Kaspa BlockDAG.

A native extension module, `kaspa`, is built from these bindings using [PyO3](https://pyo3.rs/v0.20.0/) and [Maturin](https://www.maturin.rs).

## Documentation

Full documentation is available at the [documentation site](https://smartgoo.github.io/kaspa-python-sdk/dev/), including:

- [Installation Guide](https://smartgoo.github.io/kaspa-python-sdk/dev/getting-started/installation/)
- [Examples](https://smartgoo.github.io/kaspa-python-sdk/dev/getting-started/examples/)
- [API Reference](https://smartgoo.github.io/kaspa-python-sdk/dev/reference/)

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

The [Contributing Guide](https://smartgoo.github.io/kaspa-python-sdk/dev/contributing/) details various technical core concepts and workflows used by this project.

## License

This project is licensed under the ISC License.
