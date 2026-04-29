# Connecting

`RpcClient.connect()` opens the WebSocket. `disconnect()` closes it. Between
those two calls, the client is "connected" — every RPC method is callable
and notifications stream in.

## A connected client, end to end

```python
import asyncio
from kaspa import Resolver, RpcClient

async def main():
    client = RpcClient(resolver=Resolver(), network_id="mainnet")
    await client.connect()
    try:
        info = await client.get_block_dag_info()
        print(info["networkName"], info["blockCount"])
    finally:
        await client.disconnect()

asyncio.run(main())
```

The `try`/`finally` matters: a Python exception before `disconnect()` would
otherwise leave the socket open and the event loop holding a reference.

## Connecting to a known node

Skip the resolver when you have a URL:

```python
client = RpcClient(
    url="wss://node.example.com:17110",
    network_id="mainnet",
    encoding="borsh",   # or "json"
)
```

Use this for nodes you operate, paid endpoints, or testnet runs against a
local `kaspad`.

## Connection options

`connect()` accepts a handful of behavioural overrides:

```python
await client.connect(
    block_async_connect=True,   # await until the socket is open (default True)
    strategy="fallback",        # "retry" or "fallback" — what to do if the first attempt fails
    timeout_duration=30000,     # per-attempt timeout, ms
    retry_interval=1000,        # delay between attempts, ms
)
```

The defaults are appropriate for production code. Lower `timeout_duration`
in fast-failure CI environments; raise `retry_interval` if you want to be
gentler on resolver back-ends during outages.

## Inspecting the live client

```python
print(client.is_connected)   # bool
print(client.url)            # the resolved or supplied node URL
print(client.encoding)       # "borsh" or "json"
print(client.node_id)        # node-reported identifier
print(client.resolver)       # the Resolver instance, or None
```

These are all property reads — no I/O, no `await`.

## Encoding: Borsh vs JSON

`encoding="borsh"` is the default and the right choice. Borsh is the binary
format the node speaks natively; payloads are smaller and parsing is
faster. Pick `"json"` only when you need to inspect raw frames in a tool
that doesn't speak Borsh, or when the node you're targeting doesn't support
Borsh.

## Reconnects

If the WebSocket drops mid-session, the client reconnects on its own. Calls
made *during* the gap raise; calls made *after* the reconnect succeed.
There is no opt-out — if you need to know about disruptions, listen for the
relevant connection notifications (see [Subscriptions](subscriptions.md)).

## Where to next

- [Calls](calls.md) — what to do once `is_connected` is `True`.
- [Subscriptions](subscriptions.md) — real-time notifications, including
  connection-state events.
