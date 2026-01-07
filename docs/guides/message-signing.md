# Message Signing

This guide covers signing and verifying arbitrary messages with the Kaspa Python SDK.

!!! danger "Security Warning"
    **Handle Private Keys Securely**

    **These examples do not use proper private key/mnemonic/seed handling.** This is omitted here for brevity.

    Never store your private keys in plain text, or directly in source code. Store securely offline. Anyone with access to this phrase has full control over your funds.

## Overview

Message signing allows you to:

- **Prove ownership** of an address without revealing your private key
- **Sign data** for off-chain verification
- **Authenticate** actions or statements

## Signing a Message

```python
from kaspa import sign_message, PrivateKey

# Your private key
private_key = PrivateKey("your-private-key-hex")

# Message to sign
message = "Hello, I own this address!"

# Sign the message
signature = sign_message(message, private_key)
print(f"Signature: {signature}")
```

### Deterministic Signing

By default, signatures use auxiliary randomness for additional security. For deterministic signatures:

```python
# Deterministic signature (same message + key = same signature)
signature = sign_message(message, private_key, no_aux_rand=True)
```

## Verifying a Signature

```python
from kaspa import verify_message, PublicKey

# The public key of the signer
public_key = PublicKey("02a1b2c3...")

# Or derive from private key
public_key = private_key.to_public_key()

# Verify the signature
message = "Hello, I own this address!"
signature = "signature-from-signing..."

is_valid = verify_message(message, signature, public_key)

if is_valid:
    print("Signature is valid!")
else:
    print("Invalid signature!")
```

## Complete Example

```python
from kaspa import (
    Mnemonic, XPrv, PrivateKeyGenerator,
    sign_message, verify_message, NetworkType
)

# Create a wallet
mnemonic = Mnemonic.random()
xprv = XPrv(mnemonic.to_seed())
key_gen = PrivateKeyGenerator(xprv, False, 0)

# Get a keypair
private_key = key_gen.receive_key(0)
public_key = private_key.to_public_key()
address = private_key.to_address(NetworkType.Mainnet)

print(f"Address: {address.to_string()}")

# Sign a message
message = f"I control address {address.to_string()} on 2024-01-15"
signature = sign_message(message, private_key)
print(f"Signature: {signature}")

# Verify the signature
is_valid = verify_message(message, signature, public_key)
print(f"Valid: {is_valid}")

# Try with wrong message
wrong_message = "I control a different address"
is_valid_wrong = verify_message(wrong_message, signature, public_key)
print(f"Wrong message valid: {is_valid_wrong}")  # False
```

## Use Cases

### Proving Address Ownership

```python
def prove_ownership(private_key, address, timestamp):
    """Generate a proof of address ownership."""
    message = f"I own {address.to_string()} at {timestamp}"
    signature = sign_message(message, private_key)
    return {
        "address": address.to_string(),
        "message": message,
        "signature": signature,
        "timestamp": timestamp
    }

def verify_ownership(proof, public_key):
    """Verify a proof of address ownership."""
    return verify_message(
        proof["message"],
        proof["signature"],
        public_key
    )
```

### Signing Structured Data

```python
import json
import hashlib

def sign_json_data(data, private_key):
    """Sign structured JSON data."""
    # Canonical JSON serialization
    canonical = json.dumps(data, sort_keys=True, separators=(',', ':'))
    
    # Sign the serialized data
    signature = sign_message(canonical, private_key)
    
    return {
        "data": data,
        "signature": signature
    }

def verify_json_data(signed_data, public_key):
    """Verify signed JSON data."""
    canonical = json.dumps(
        signed_data["data"],
        sort_keys=True,
        separators=(',', ':')
    )
    
    return verify_message(
        canonical,
        signed_data["signature"],
        public_key
    )
```

### Authentication Token

```python
import time

def create_auth_token(private_key, address, validity_seconds=300):
    """Create a time-limited authentication token."""
    expires = int(time.time()) + validity_seconds
    message = f"auth:{address.to_string()}:{expires}"
    signature = sign_message(message, private_key)
    
    return {
        "address": address.to_string(),
        "expires": expires,
        "signature": signature
    }

def verify_auth_token(token, public_key):
    """Verify an authentication token."""
    # Check expiration
    if int(time.time()) > token["expires"]:
        return False, "Token expired"
    
    # Verify signature
    message = f"auth:{token['address']}:{token['expires']}"
    is_valid = verify_message(message, token["signature"], public_key)
    
    if is_valid:
        return True, "Valid"
    else:
        return False, "Invalid signature"
```
