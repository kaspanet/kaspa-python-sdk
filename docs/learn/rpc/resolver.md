# Resolver

A `Resolver` finds a Kaspa node for a given [network](network.md). An `RpcClient` can be configured to use a Resolver instance instead of a hard-coded URL. On `connect()` the resolver (attempts) to connect to an available node on the public node-network (PNN).

## When to use it

- **Building an application that talks to mainnet or testnet** without
  shipping a node alongside.
- **Quick scripts and notebooks** where "just give me a node" is the right
  default.
- **Failover.** The resolver picks a different node if its first choice is
  unreachable.

If you need a deterministic URL — for example a load-balanced internal
node, or a node with an authenticated endpoint — point `RpcClient` at it
directly and don't construct a `Resolver` at all.

## The basic shape

```python
from kaspa import Resolver, RpcClient

resolver = Resolver()
client = RpcClient(resolver=resolver, network_id="mainnet")
await client.connect()
```

The `network_id` argument is what the resolver uses to filter candidate
nodes — `"mainnet"`, `"testnet-10"`, or `"testnet-11"`. The same `Resolver`
instance can be reused across networks; the network is a property of the
*client*, not the resolver.

## Configuring the resolver

```python
# Default: uses the public PNN endpoints baked into the SDK
resolver = Resolver()

# Override with explicit resolver URLs (advanced)
resolver = Resolver(urls=["https://resolver1.example.org"])

# Force TLS on resolver requests
resolver = Resolver(tls=True)
```

The default constructor is the right choice for almost everyone. Override
the URLs only if you're operating a private resolver fleet.

## Where to next

- [Connecting](connecting.md) — direct URLs, retry/timeout options, the
  encoding choice.
- [Calls](calls.md) — what to do once you're connected.
