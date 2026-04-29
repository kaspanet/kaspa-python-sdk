# Kaspa Python SDK

This Python package, `kaspa`, provides an SDK for interacting with the
Kaspa network from Python.

`kaspa` is a native extension module built from bindings to Rust and
[rusty-kaspa](https://github.com/kaspanet/rusty-kaspa) source.
[PyO3](https://pyo3.rs/) and [Maturin](https://www.maturin.rs/) are used
to create bindings and build the extension module. More information on
the inner workings can be found in the
[Contributing section](contributing/index.md).

!!! warning "Beta Status"
    This project is in beta status.

This project closely mirrors
[Kaspa's WASM SDK](https://kaspa.aspectron.org/docs/), while trying to
respect Python conventions. Feature parity with the WASM SDK is a work
in progress; not every feature is available yet in Python.

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

This site follows the [Diataxis](https://diataxis.fr) framework. Each
section answers a different question:

<div class="grid cards" markdown>

- **[Getting Started](getting-started/installation.md)**  
  Install the SDK, run the first script, read the security note before
  generating real keys.

- **[Learn](learn/index.md)**  
  How the SDK is shaped, taught topic by topic. Connections, wallets,
  derivation, transactions, the Kaspa concepts behind them.

- **[Guides](guides/mnemonics.md)**  
  Recipes for specific tasks — mnemonic restore, message signing,
  multisig, wallet recovery, custom derivation.

- **[API Reference](reference/index.md)**  
  Every public class, method, and signature. Auto-generated.

</div>

## Where to start

- **New to the SDK:** [Installation](getting-started/installation.md) →
  [Learn → RPC](learn/rpc/index.md) →
  [Learn → Wallet SDK → Key Management](learn/wallet-sdk/key-management.md).
- **Looking for a recipe:** jump to [Guides](guides/mnemonics.md).
- **Looking up an API:** [API Reference](reference/index.md).
- **Generating real keys:** read
  [Security](getting-started/security.md) first.

## License

This project is licensed under the ISC License.
