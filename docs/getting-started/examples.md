# Examples

A handful of brief examples showing core features of the Kaspa Python SDK.

!!! danger "Security Warning"
    **Handle Private Keys Securely**

    **These examples do not use proper private key/mnemonic/seed handling.** This is omitted here for brevity.

    Never store your private keys in plain text, or directly in source code. Store securely offline. Anyone with access to this phrase has full control over your funds.

## Creating a Wallet

```python
from kaspa import Mnemonic, XPrv, PrivateKeyGenerator, NetworkType

# Generate a new 24-word mnemonic
mnemonic = Mnemonic.random()
print(f"Your seed phrase: {mnemonic.phrase}")

# IMPORTANT: Store this phrase securely!
# Anyone with this phrase can access your funds.

# Convert mnemonic to seed
seed = mnemonic.to_seed()

# Create extended private key from seed
xprv = XPrv(seed)

# Create a key generator for deriving addresses
key_gen = PrivateKeyGenerator(xprv, False, 0)
```

## Generating Addresses

With the key generator, you can derive addresses:

```python
# ... continuation of example above

# Get the first receive address
private_key = key_gen.receive_key(0)
address = private_key.to_address(NetworkType.Mainnet)
print(f"Your address: {address.to_string()}")

# Generate multiple addresses
for i in range(5):
    pk = key_gen.receive_key(i)
    addr = pk.to_address(NetworkType.Mainnet)
    print(f"Address {i}: {addr.to_string()}")
```

## Working with Existing Wallets

To restore a wallet from an existing seed phrase:

```python
from kaspa import Mnemonic, XPrv, PrivateKeyGenerator, NetworkType

# Your existing seed phrase
phrase = "word1 word2 word3 ... word24"

# Validate and create mnemonic
if Mnemonic.validate(phrase):
    mnemonic = Mnemonic(phrase)
    seed = mnemonic.to_seed()
    xprv = XPrv(seed)
    key_gen = PrivateKeyGenerator(xprv, False, 0)
    
    # Derive your first address
    address = key_gen.receive_key(0).to_address(NetworkType.Mainnet)
    print(f"Restored address: {address.to_string()}")

    # Derive additional addresses as needed...
else:
    print("Invalid seed phrase!")
```

## Interacting with the Kaspa Network via RPC

```python
import asyncio
from kaspa import RpcClient, Resolver, NetworkId

async def main():
    # Create a resolver to use available PNN nodes
    resolver = Resolver()
    
    # Create RPC client
    client = RpcClient(
        resolver=resolver,
        network_id=NetworkId("mainnet")
    )
    
    # Connect to the network
    await client.connect()
    print(f"Connected to: {client.url}")
    
    # Get network info
    info = await client.get_info()
    print(f"Network: {info}")
    
    # Always disconnect when done
    await client.disconnect()

asyncio.run(main())
```

## Checking Balances

Query the balance of an address:

```python
import asyncio
from kaspa import RpcClient, Resolver, Address

async def check_balance(address_str: str):
    client = RpcClient(resolver=Resolver(), network_id="mainnet")
    await client.connect()
    
    try:
        result = await client.get_balance_by_address({
            "address": address_str
        })
        
        # Balance is in sompi (1 KAS = 100,000,000 sompi)
        balance_sompi = result.get("balance", 0)
        balance_kas = balance_sompi / 100_000_000
        print(f"Balance: {balance_kas} KAS")
        
    finally:
        await client.disconnect()

asyncio.run(check_balance("kaspa:qz..."))
```

## Building a Transaction

```python
import asyncio
from kaspa import (
    RpcClient, Resolver, Generator, PaymentOutput,
    Address, PrivateKey, NetworkId
)

async def send_transaction():
    client = RpcClient(resolver=Resolver(), network_id="mainnet")
    await client.connect()
    
    try:
        # Your private key (keep secret!)
        private_key = PrivateKey("your-private-key-hex")
        sender_address = private_key.to_address("mainnet")
        
        # Get UTXOs for your address
        utxos_response = await client.get_utxos_by_addresses({
            "addresses": [sender_address.to_string()]
        })
        
        # Create payment output
        recipient = Address("kaspa:recipient-address...")
        amount = 100_000_000  # 1 KAS in sompi
        payment = PaymentOutput(recipient, amount)
        
        # Create transaction generator
        generator = Generator(
            network_id=NetworkId("mainnet"),
            entries=utxos_response["entries"],
            change_address=sender_address,
            outputs=[payment],
        )
        
        # Generate and sign transactions
        for pending_tx in generator:
            # Sign the transaction
            pending_tx.sign([private_key])
            
            # Submit to network
            tx_id = await pending_tx.submit(client)
            print(f"Transaction submitted: {tx_id}")
            
    finally:
        await client.disconnect()

asyncio.run(send_transaction())
```
