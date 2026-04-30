# Connecting

[`RpcClient.connect()`](../../reference/Classes/RpcClient.md#connect)
opens the WebSocket;
[`disconnect()`](../../reference/Classes/RpcClient.md#disconnect)
closes it. While connected, every RPC method is callable and
notifications stream in.

You can connect via the Public Node Network (PNN) using
[`Resolver`](resolver.md), or directly to a node URL.

## Example

Connecting to a PNN mainnet node via `Resolver`:

```python
import asyncio
from kaspa import Resolver, RpcClient

async def main():
    client = RpcClient(resolver=Resolver(), network_id="mainnet")
    await client.connect()

    info = await client.get_block_dag_info()

    await client.disconnect()

asyncio.run(main())
```

## Connecting to a known node

Pass a URL directly. See [Networks](../networks.md) for the canonical
wRPC ports per network:

```python
client = RpcClient(
    url="ws://node.example.com:17110",
    network_id="mainnet",
    encoding="borsh",   # or "json"
)
```

## URL schemes

- `ws://` — plaintext WebSocket
- `wss://` — TLS WebSocket

## Connection options

`connect()` takes a few behavioural overrides:

```python
await client.connect(
    block_async_connect=True,
    strategy="fallback",
    url="ws://node.example.com:17110",
    timeout_duration=30000,
    retry_interval=1000,
)
```

- `block_async_connect` — if `False`, `connect()` returns immediately
  and the socket opens in the background.
- `strategy` — `"retry"` (default) loops until a connection succeeds;
  `"fallback"` returns on the first failure. Applies to both URL-based
  and `Resolver`-driven clients.
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
node.

Use `"json"` only to inspect raw frames in a tool that doesn't speak
Borsh, or when targeting a node that doesn't support it. See the
[`Encoding`](../../reference/Enums/Encoding.md) enum for accepted
values.

## Reconnects

If the WebSocket drops mid-session, the client reconnects on its own.
Calls made *during* the gap raise; calls made *after* a successful
reconnect work normally. To stop reconnect attempts, call
`disconnect()` — or use `strategy="fallback"`, which gives up after
one failed reconnect instead of looping. To track disruptions, listen
for the `connect` and `disconnect` events
(see [Subscriptions](subscriptions.md#available-events)).

## Where to next

- [Resolver](resolver.md) — node discovery details.
- [Calls](calls.md) — what to do once `is_connected` is `True`.
- [Subscriptions](subscriptions.md) — real-time notifications, including
  connection-state events.
