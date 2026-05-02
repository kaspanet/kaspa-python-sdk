# Connecting

[`RpcClient.connect()`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.connect)
opens the WebSocket;
[`disconnect()`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.disconnect)
closes it. While connected, every RPC method is callable and
notifications stream in.

You can connect directly to a node URL, or let the SDK discover one for
you via [`Resolver`](../../reference/Classes/Resolver.md) (covered on its own [page](resolver.md)).

## Connecting to a known node

Pass a URL directly. See [Networks](../networks.md) for the canonical
wRPC ports per network:

```python
import asyncio
from kaspa import RpcClient

async def main():
    client = RpcClient(
        url="ws://node.example.com:17110",
        network_id="mainnet",
        encoding="borsh",   # or "json"
    )
    await client.connect()
    try:
        info = await client.get_block_dag_info()
        print(info["blockCount"])
    finally:
        await client.disconnect()

asyncio.run(main())
```

## Connecting via Resolver

Skip the URL and let the SDK pick a Public Node Network (PNN) node for
the network you want:

```python
from kaspa import Resolver, RpcClient

client = RpcClient(resolver=Resolver(), network_id="mainnet")
await client.connect()
```

See [Resolver](resolver.md) for discovery details and when to run your
own node instead.

## URL schemes

- `ws://` — plaintext WebSocket
- `wss://` — TLS WebSocket

## Connection options

[`connect()`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.connect) takes a few behavioural overrides:

```python
await client.connect(
    block_async_connect=True,        # default
    strategy="retry",                # default
    url="ws://node.example.com:17110",
    timeout_duration=30000,
    retry_interval=1000,
)
```

- `block_async_connect` — `True` (default) makes `connect()` await
  until the socket is open. Set to `False` to return immediately and
  let the connection complete in the background; check
  `client.is_connected` or listen for the `connect` event to know when
  it's ready.
- `strategy` — `"retry"` (default) loops until a connection succeeds,
  pausing `retry_interval` between attempts; `"fallback"` returns on
  the first failure. `timeout_duration` caps each individual attempt
  (not the overall wall-clock); under `"retry"` there is no overall
  ceiling. Applies to both URL-based and [`Resolver`](../../reference/Classes/Resolver.md)-driven clients.
- `url` — overrides the constructor URL for this attempt only. Lets you
  retarget a long-lived client without rebuilding it.
- `timeout_duration` — per-attempt ceiling, in milliseconds.
- `retry_interval` — delay between attempts, in milliseconds.

## Inspecting the live client

```python
print(client.is_connected)   # bool
print(client.url)            # resolved or supplied node URL, or None
print(client.encoding)       # "borsh" or "json"
print(client.node_id)        # resolver-supplied node UID; None for direct URLs
print(client.resolver)       # the Resolver instance, or None
```

## Encoding: Borsh vs JSON

Borsh (the default) is a compact binary format used natively by the
node. The constructor accepts either the string form (`encoding="borsh"`,
`encoding="json"`) or the [`Encoding`](../../reference/Enums/Encoding.md)
enum (`Encoding.Borsh`, `Encoding.Json`) — pick whichever your codebase
prefers and stick to it.

Use `"json"` only to inspect raw frames in a tool that doesn't speak
Borsh, or when targeting a node that doesn't support it.

## Reconnects

If the WebSocket drops mid-session, the client reconnects on its own.
Calls made *during* the gap raise; calls made *after* a successful
reconnect work normally. To stop reconnect attempts, call
[`disconnect()`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.disconnect) — or use `strategy="fallback"`, which gives up after
one failed reconnect instead of looping. To track disruptions, listen
for the `connect` and `disconnect` events
(see [Subscriptions](subscriptions.md#available-events)).

## Where to next

- [Resolver](resolver.md) — node discovery details.
- [Calls](calls.md) — what to do once `is_connected` is `True`.
- [Subscriptions](subscriptions.md) — real-time notifications, including
  connection-state events.
