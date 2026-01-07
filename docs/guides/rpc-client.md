# RPC Client

This guide covers the Kaspa RPC Client - connecting to Kaspa nodes and basic RPC API use.

## Overview

The Kaspa Python SDK provides an asynchronous RPC client for communicating with Kaspa nodes via WebSocket. Features include:

- **Automatic node discovery (PNN)** via Resolver.
- **Connection management** with reconnection support.
- **Full RPC API** coverage, including event subscriptions for real-time updates.

## Quick Start

```python
import asyncio
from kaspa import RpcClient, Resolver

async def main():
    # Create client with resolver
    client = RpcClient(
        resolver=Resolver(),
        network_id="mainnet"
    )
    
    # Connect
    await client.connect()
    print(f"Connected to: {client.url}")
    
    # Make RPC calls
    info = await client.get_info()
    print(f"Server info: {info}")
    
    # Disconnect
    await client.disconnect()

asyncio.run(main())
```

## Connection Options

### Using a Resolver

The Resolver automatically finds available nodes:

```python
from kaspa import RpcClient, Resolver

# Default resolver (uses public infrastructure)
resolver = Resolver()

# Custom resolver URLs
resolver = Resolver(urls=["https://resolver1.kaspa.org"])

# With TLS configuration
resolver = Resolver(tls=True)

client = RpcClient(resolver=resolver, network_id="mainnet")
```

### Direct Connection

Connect directly to a known node:

```python
from kaspa import RpcClient

client = RpcClient(
    url="wss://node.kaspa.org:17110",
    network_id="mainnet",
    encoding="borsh"  # or "json"
)
```

### Connection Parameters

```python
await client.connect(
    block_async_connect=True,      # Wait for connection
    strategy="fallback",           # Connection strategy
    timeout_duration=30000,        # Timeout in ms
    retry_interval=1000,           # Retry interval in ms
)
```

## Client Properties

```python
# Check connection status
print(f"Connected: {client.is_connected}")

# Get current URL
print(f"URL: {client.url}")

# Get encoding
print(f"Encoding: {client.encoding}")

# Get node ID
print(f"Node ID: {client.node_id}")

# Get resolver
resolver = client.resolver
```

## RPC Methods

### Network Information

```python
# Get general info
info = await client.get_info()

# Get block count
count = await client.get_block_count()
print(f"Blocks: {count['blockCount']}, Headers: {count['headerCount']}")

# Get block DAG info
dag_info = await client.get_block_dag_info()
print(f"Network: {dag_info['networkName']}")
print(f"Block count: {dag_info['blockCount']}")

# Get coin supply
supply = await client.get_coin_supply()
print(f"Circulating: {supply['circulatingSompi']}")

# Get current network
network = await client.get_current_network()

# Get sync status
sync = await client.get_sync_status()
print(f"Synced: {sync['isSynced']}")
```

### Balance and UTXOs

```python
# Get balance for single address
balance = await client.get_balance_by_address({
    "address": "kaspa:qz..."
})
print(f"Balance: {balance['balance']} sompi")

# Get balances for multiple addresses
balances = await client.get_balances_by_addresses({
    "addresses": ["kaspa:qz...", "kaspa:qr..."]
})

# Get UTXOs
utxos = await client.get_utxos_by_addresses({
    "addresses": ["kaspa:qz..."]
})
for entry in utxos.get("entries", []):
    print(f"UTXO: {entry['outpoint']} = {entry['utxoEntry']['amount']}")
```

### Blocks

```python
# Get specific block
block = await client.get_block({
    "hash": "block-hash-hex",
    "includeTransactions": True
})

# Get multiple blocks
blocks = await client.get_blocks({
    "lowHash": "starting-hash",
    "includeBlocks": True,
    "includeTransactions": False
})

# Get block template (for mining)
template = await client.get_block_template({
    "payAddress": "kaspa:mining-address...",
    "extraData": []
})
```

### Transactions

```python
# Submit transaction
from kaspa import Transaction

result = await client.submit_transaction({
    "transaction": tx.serialize_to_dict(),
    "allowOrphan": False
})
print(f"Transaction ID: {result['transactionId']}")

# Get mempool entries
mempool = await client.get_mempool_entries({
    "includeOrphanPool": False,
    "filterTransactionPool": True
})

# Get mempool entry by transaction ID
entry = await client.get_mempool_entry({
    "transactionId": "tx-id...",
    "includeOrphanPool": False,
    "filterTransactionPool": True
})
```

### Fees

```python
# Get fee estimate
fee = await client.get_fee_estimate()
print(f"Priority fee: {fee['estimate']['priorityBucket']}")

# Experimental fee estimate with more detail
fee_exp = await client.get_fee_estimate_experimental({
    "verbose": True
})
```

### Peer Management

```python
# Get connected peers
peers = await client.get_connected_peer_info()

# Get peer addresses
addresses = await client.get_peer_addresses()

# Add peer
await client.add_peer({
    "peerAddress": "192.168.1.1:16111",
    "isPermanent": False
})

# Ban/unban peer
await client.ban({"ip": "192.168.1.1"})
await client.unban({"ip": "192.168.1.1"})
```

### System

```python
# Ping node
pong = await client.ping()

# Get server info
server_info = await client.get_server_info()

# Get system info
system_info = await client.get_system_info()

# Get metrics
metrics = await client.get_metrics({
    "processMetrics": True,
    "connectionMetrics": True,
    "bandwidthMetrics": True,
    "consensusMetrics": True,
    "storageMetrics": False,
    "customMetrics": False
})
```

## Event Subscriptions

Subscribe to real-time events:

### UTXO Changes

```python
from kaspa import Address

# Define callback
def on_utxo_change(event):
    print(f"UTXO change: {event}")

# Add listener
client.add_event_listener("utxos-changed", on_utxo_change)

# Subscribe to addresses
await client.subscribe_utxos_changed([
    Address("kaspa:qz...")
])

# Later: unsubscribe
await client.unsubscribe_utxos_changed([
    Address("kaspa:qz...")
])
```

### Block Events

```python
def on_block_added(event):
    print(f"New block: {event['block']['header']['hash']}")

client.add_event_listener("block-added", on_block_added)
await client.subscribe_block_added()
```

### Virtual Chain Changes

```python
def on_chain_change(event):
    print(f"Chain updated: {event}")

client.add_event_listener("virtual-chain-changed", on_chain_change)
await client.subscribe_virtual_chain_changed(
    include_accepted_transaction_ids=True
)
```

### DAA Score Changes

```python
def on_daa_change(event):
    print(f"DAA score: {event['virtualDaaScore']}")

client.add_event_listener("virtual-daa-score-changed", on_daa_change)
await client.subscribe_virtual_daa_score_changed()
```

### Available Events

| Event | Subscription Method |
|-------|---------------------|
| `utxos-changed` | `subscribe_utxos_changed()` |
| `block-added` | `subscribe_block_added()` |
| `virtual-chain-changed` | `subscribe_virtual_chain_changed()` |
| `virtual-daa-score-changed` | `subscribe_virtual_daa_score_changed()` |
| `sink-blue-score-changed` | `subscribe_sink_blue_score_changed()` |
| `finality-conflict` | `subscribe_finality_conflict()` |
| `finality-conflict-resolved` | `subscribe_finality_conflict_resolved()` |
| `new-block-template` | `subscribe_new_block_template()` |
| `pruning-point-utxo-set-override` | `subscribe_pruning_point_utxo_set_override()` |

### Managing Listeners

```python
# Add listener with extra args
client.add_event_listener("block-added", callback, extra_arg)

# Remove specific listener
client.remove_event_listener("block-added", callback)

# Remove all listeners for an event
client.remove_event_listener("block-added")

# Remove all listeners
client.remove_all_event_listeners()
```

## Complete Example: Wallet Monitor

```python
import asyncio
from kaspa import RpcClient, Resolver, Address, sompi_to_kaspa

class WalletMonitor:
    def __init__(self, addresses):
        self.addresses = [Address(a) for a in addresses]
        self.client = RpcClient(
            resolver=Resolver(),
            network_id="mainnet"
        )
    
    async def start(self):
        await self.client.connect()
        
        # Set up event handler
        self.client.add_event_listener(
            "utxos-changed",
            self.on_utxo_change
        )
        
        # Subscribe to address changes
        await self.client.subscribe_utxos_changed(self.addresses)
        
        # Get initial balances
        await self.check_balances()
        
        print("Monitoring... Press Ctrl+C to stop")
        
        # Keep running
        while True:
            await asyncio.sleep(1)
    
    async def check_balances(self):
        for addr in self.addresses:
            result = await self.client.get_balance_by_address({
                "address": addr.to_string()
            })
            balance = sompi_to_kaspa(result.get("balance", 0))
            print(f"{addr.to_string()[:20]}...: {balance} KAS")
    
    def on_utxo_change(self, event):
        print(f"UTXO change detected!")
        for added in event.get("added", []):
            amount = sompi_to_kaspa(added["utxoEntry"]["amount"])
            print(f"  + {amount} KAS")
        for removed in event.get("removed", []):
            amount = sompi_to_kaspa(removed["utxoEntry"]["amount"])
            print(f"  - {amount} KAS")
    
    async def stop(self):
        await self.client.disconnect()

async def main():
    addresses = ["kaspa:qz..."]
    monitor = WalletMonitor(addresses)
    
    try:
        await monitor.start()
    except KeyboardInterrupt:
        await monitor.stop()

asyncio.run(main())
```