---
search:
  boost: 3
---

# Resolver

A [`Resolver`](../../reference/Classes/Resolver.md) finds a public Kaspa
node so you don't need a URL up front. Pass one to
[`RpcClient`](../../reference/Classes/RpcClient.md) instead of `url=`
and `client.connect()` picks a live node from the Public Node Network
(PNN).

For most apps, this is all you need:

```python
from kaspa import Resolver, RpcClient

client = RpcClient(resolver=Resolver(), network_id="mainnet")
await client.connect()
```

**For security-critical applications, you should connect to your own
node.**

`network_id` selects the network — `"mainnet"` or a testnet
(e.g. `"testnet-10"`). It takes a string or a
[`NetworkId`](../../reference/Classes/NetworkId.md); see
[Networks](../networks.md) for the full list. Not every testnet has PNN
nodes.

## Constructor options

[`Resolver`](../../reference/Classes/Resolver.md) accepts a few keyword arguments at construction time:

```python
# Default
resolver = Resolver()

# Require a TLS-capable node (wss://)
resolver = Resolver(tls=True)

# Point at your own resolver fleet (advanced — see "Under the hood")
resolver = Resolver(urls=["https://resolver1.example.org"])
```

- `tls=True` — restrict to `wss://` nodes. Default `False` allows any
  reachable node.
- `urls=` — replaces the default resolver-service list with your own
  (see [Under the hood](#under-the-hood)).

## Querying the resolver directly

You can fetch a URL without constructing an [`RpcClient`](../../reference/Classes/RpcClient.md):

```python
from kaspa import Resolver

resolver = Resolver()

url = await resolver.get_url("borsh", "mainnet")
descriptor = await resolver.get_node("borsh", "mainnet")
```

Both arguments accept the string form shown above or the typed
equivalents (`Encoding.Borsh`, `NetworkId("mainnet")`) — see
[`Encoding`](../../reference/Enums/Encoding.md) and
[`NetworkId`](../../reference/Classes/NetworkId.md).

[`get_url`](../../reference/Classes/Resolver.md#kaspa.Resolver.get_url) returns a
WebSocket URL ready for [`RpcClient(url=...)`](../../reference/Classes/RpcClient.md).
[`get_node`](../../reference/Classes/Resolver.md#kaspa.Resolver.get_node) returns a
dict with the node's `uid`, `url`, and other metadata.

## Under the hood

You don't need any of this to use `Resolver` — it's here for anyone
running their own infrastructure or debugging connectivity.

A [`Resolver`](../../reference/Classes/Resolver.md) doesn't open WebSockets or hold Kaspa node URLs. It holds
a list of *resolver service* HTTP endpoints (see
[aspectron/kaspa-resolver](https://github.com/aspectron/kaspa-resolver))
that track live PNN nodes and load-balance across them.

On [`get_url`](../../reference/Classes/Resolver.md#kaspa.Resolver.get_url) / [`get_node`](../../reference/Classes/Resolver.md#kaspa.Resolver.get_node) (called internally by [`client.connect()`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.connect)):

1. Pick a configured resolver-service URL at random.
2. `GET {url}/v2/kaspa/{network_id}/{tls_or_any}/wrpc/{encoding}`.
3. Parse the response as a node descriptor and return the URL.
4. On failure, try the next service; raise if all fail.

The default `Resolver()` ships with the public resolver-service list
embedded in the SDK (sourced from
[`Resolvers.toml`](https://github.com/kaspanet/rusty-kaspa/blob/master/rpc/wrpc/client/Resolvers.toml)
in `kaspa-wrpc-client`). `Resolver(urls=...)` replaces that list —
useful for a private node cluster behind your own resolver fleet.
`resolver.urls()` returns the configured list, or an empty list when
using the embedded defaults (concrete URLs are hidden so they can
rotate without breaking SDK consumers).

## Where to next

- [Connecting](connecting.md) — direct URLs, retry/timeout options,
  encoding.
- [Calls](calls.md) — what to do once connected.
- [Subscriptions](subscriptions.md) — real-time notifications.
