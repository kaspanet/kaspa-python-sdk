# Initialize

Constructing a `Wallet` does no I/O. It builds the local file store and an
internal wRPC client, and that's it — `start()` is the next step.

## Constructor

```python
from kaspa import Resolver, Wallet

wallet = Wallet(
    network_id="testnet-10",     # required
    resolver=Resolver(),         # discover a public node
    # url=...                    # OR a known node URL
    # encoding="borsh",          # default; "json" also accepted
    # storage_folder=None,       # override default ~/.kaspa
)
```

| Argument | Required | Notes |
| --- | --- | --- |
| `network_id` | yes | `"mainnet"`, `"testnet-10"`, `"testnet-11"`. Drives both the resolver query and the address encoding. |
| `resolver` | one of | A `Resolver` instance — see [RPC → Resolver](../rpc/resolver.md). |
| `url` | resolver/url | A known wRPC URL (`wss://node.example:17110`). Skip the resolver if you set this. |
| `encoding` | optional | `"borsh"` (default) or `"json"`. Borsh is right for almost everything. |

`network_id` is **required** — addresses derived from this wallet will be
encoded for that network and the resolver only returns nodes on that
network.

## Switching networks

`set_network_id` raises if the wallet is currently connected:

```python
await wallet.disconnect()
wallet.set_network_id("mainnet")
await wallet.connect()
```

Switching network does not invalidate the file store on disk — but the
*addresses* derived from a BIP32 account are network-specific, so a key
created under `testnet-10` produces different (testnet) addresses than the
same key under `mainnet`.

## Storage folder

By default the wallet stores files under `~/.kaspa/`. Override with
`storage_folder=...` when:

- Running tests against a temp directory.
- Running multiple isolated wallets in the same process.
- Sandboxing per-user wallet stores in a multi-tenant service.

The folder is created on first write; nothing is done at construction
time.

## Where to next

- [Start](start.md) — boot the runtime and connect to the node.
- [Open](open.md) — create or open a wallet file.
- [Lifecycle](lifecycle.md) — the full state machine.
