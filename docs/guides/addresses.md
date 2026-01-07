# Addresses

This guide covers working with Kaspa addresses in the Python SDK.

## Overview

Kaspa addresses are encoded representations of public keys or script hashes. They include:

- A **network prefix** (`kaspa:`, `kaspatest:`, `kaspadev:`, `kaspasim:`)
- A **version** indicating the address type (PubKey, PubKeyECDSA, ScriptHash)
- An **encoded payload** derived from the public key or script

## Creating Addresses

### From a String

```python
from kaspa import Address

# Parse an address string
address = Address("kaspa:qz0s9f5p7d3e2c4x8n1b6m9k0j2h4g5f3d7a8s9w0e1r2t3y4u5i6o7p8")

# Get address components
print(f"Prefix: {address.prefix}")
print(f"Version: {address.version}")
print(f"Full: {address.to_string()}")
```

### From a Private Key

```python
from kaspa import PrivateKey, NetworkType

# Create from private key hex
private_key = PrivateKey("your-private-key-hex")

# Derive Schnorr address (default)
address = private_key.to_address(NetworkType.Mainnet)
print(f"Schnorr address: {address.to_string()}")

# Derive ECDSA address
ecdsa_address = private_key.to_address_ecdsa(NetworkType.Mainnet)
print(f"ECDSA address: {ecdsa_address.to_string()}")
```

### From a Public Key

```python
from kaspa import PublicKey, NetworkType

# Create from public key hex
public_key = PublicKey("02a1b2c3d4e5f6...")

# Derive address
address = public_key.to_address(NetworkType.Mainnet)
print(f"Address: {address.to_string()}")
```

## Validating Addresses

```python
from kaspa import Address

address_str = "kaspa:qz..."

# Static validation (returns bool)
if Address.validate(address_str):
    address = Address(address_str)
    print(f"Valid address: {address.to_string()}")
else:
    print("Invalid address!")
```

## Address Types

Kaspa supports several address versions:

| Version | Description |
|---------|-------------|
| PubKey | Schnorr signature |
| PubKeyECDSA | ECDSA signature |
| ScriptHash | Pay-to-Script-Hash |

```python
from kaspa import Address

address = Address("kaspa:qz...")

# Check the version
version = address.version
print(f"Address version: {version}")
```

## Network Prefixes

Addresses include a prefix indicating the network:

| Prefix | Network | Use |
|--------|---------|-----|
| `kaspa:` | Mainnet | Production |
| `kaspatest:` | Testnet | Testing |
| `kaspadev:` | Devnet | Development |
| `kaspasim:` | Simnet | Simulation |

### Changing the Prefix

```python
from kaspa import Address

# Create mainnet address
address = Address("kaspa:qz...")

# Change to testnet
address.prefix = "kaspatest"
print(f"Testnet address: {address.to_string()}")
```

!!! warning
    Changing the prefix doesn't change the underlying key. It only changes the encoding. Sending to the wrong network prefix could result in lost funds.

## Address from Script

Create an address from a script public key:

```python
from kaspa import ScriptPublicKey, address_from_script_public_key, NetworkType

# Create script public key
script_pubkey = ScriptPublicKey(0, "20a1b2c3...")

# Convert to address
address = address_from_script_public_key(script_pubkey, NetworkType.Mainnet)
print(f"Address: {address.to_string()}")
```

## Script from Address

Get the script public key for an address:

```python
from kaspa import Address, pay_to_address_script

address = Address("kaspa:qz...")

# Get the locking script
script_pubkey = pay_to_address_script(address)
print(f"Script: {script_pubkey.script}")
```

## Multi-Signature Addresses

Create a multi-signature address:

```python
from kaspa import create_multisig_address, PublicKey, NetworkType

# Gather public keys from all participants
pubkeys = [
    PublicKey("02key1..."),
    PublicKey("02key2..."),
    PublicKey("02key3..."),
]

# Create 2-of-3 multisig address
multisig_address = create_multisig_address(
    minimum_signatures=2,
    keys=pubkeys,
    network_type=NetworkType.Mainnet
)

print(f"Multisig address: {multisig_address.to_string()}")
```
