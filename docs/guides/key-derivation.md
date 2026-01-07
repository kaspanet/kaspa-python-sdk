# Key Derivation

This guide covers hierarchical deterministic (HD) wallet key derivation in the Kaspa Python SDK.

!!! danger "Security Warning"
    **Handle Private Keys Securely**

    **These examples do not use proper private key/mnemonic/seed handling.** This is omitted here for brevity.

    Never store your private keys in plain text, or directly in source code. Store securely offline. Anyone with access to this phrase has full control over your funds.

## Derivation Path

```
m / purpose' / coin_type' / account' / change / address_index
```

See [Kaspa MDBook's page on derivation](https://kaspa-mdbook.aspectron.com/wallets/addresses.html) for more information.

## Extended Keys

### Extended Private Key (XPrv)

```python
from kaspa import Mnemonic, XPrv

# Generate from mnemonic
mnemonic = Mnemonic.random()
seed = mnemonic.to_seed()
xprv = XPrv(seed)

# Access properties
print(f"XPrv: {xprv.xprv}")
print(f"Private key: {xprv.private_key}")
print(f"Depth: {xprv.depth}")
print(f"Chain code: {xprv.chain_code}")
```

### Extended Public Key (XPub)

```python
from kaspa import XPrv, XPub

# Derive XPub from XPrv
xprv = XPrv(seed)
xpub = xprv.to_xpub()

# Access properties
print(f"XPub: {xpub.xpub}")
print(f"Depth: {xpub.depth}")
print(f"Chain code: {xpub.chain_code}")

# Get public key
public_key = xpub.to_public_key()
```

## Manual Derivation

### Deriving Child Keys

```python
from kaspa import XPrv, DerivationPath

xprv = XPrv(seed)

# Derive by child number
child = xprv.derive_child(0)  # Non-hardened
hardened_child = xprv.derive_child(0, hardened=True)

# Derive by path string
account_key = xprv.derive_path("m/44'/111111'/0'")

# Derive using DerivationPath object
path = DerivationPath("m/44'/111111'/0'/0/0")
derived = xprv.derive_path(path)
```

### Working with Derivation Paths

```python
from kaspa import DerivationPath

# Create a path
path = DerivationPath("m/44'/111111'/0'/0/0")

# Check properties
print(f"Length: {path.length()}")
print(f"Is empty: {path.is_empty()}")
print(f"String: {path.to_string()}")

# Get parent path
parent = path.parent()
print(f"Parent: {parent.to_string()}")

# Extend path
path.push(1)  # Add non-hardened child
path.push(0, hardened=True)  # Add hardened child
```

## Key Generators

### Private Key Generator

```python
from kaspa import XPrv, PrivateKeyGenerator, NetworkType

xprv = XPrv(seed)

# Create generator for standard (non-multisig) wallet
key_gen = PrivateKeyGenerator(
    xprv=xprv,
    is_multisig=False,
    account_index=0
)

# Generate receive addresses
for i in range(5):
    private_key = key_gen.receive_key(i)
    address = private_key.to_address(NetworkType.Mainnet)
    print(f"Receive {i}: {address.to_string()}")

# Generate change addresses
change_key = key_gen.change_key(0)
change_address = change_key.to_address(NetworkType.Mainnet)
print(f"Change: {change_address.to_string()}")
```

### Public Key Generator

For watch-only wallets or when you only need addresses:

```python
from kaspa import PublicKeyGenerator, NetworkType

# Create from XPub string
pub_gen = PublicKeyGenerator.from_xpub("xpub...")

# Or create from master XPrv
pub_gen = PublicKeyGenerator.from_master_xprv(
    xprv=xprv_string,
    is_multisig=False,
    account_index=0
)

# Generate receive addresses
addresses = pub_gen.receive_addresses(
    network_type=NetworkType.Mainnet,
    start=0,
    end=10
)
for i, addr in enumerate(addresses):
    print(f"Address {i}: {addr.to_string()}")

# Get single address
single_addr = pub_gen.receive_address(NetworkType.Mainnet, 0)

# Get as strings directly
addr_strings = pub_gen.receive_addresses_as_strings(
    network_type=NetworkType.Mainnet,
    start=0,
    end=10
)

# Get public keys
pubkeys = pub_gen.receive_pubkeys(start=0, end=5)
pubkey_strings = pub_gen.receive_pubkeys_as_strings(start=0, end=5)
```

### Change Addresses

Key generators provide both receive and change key paths:

```python
from kaspa import PrivateKeyGenerator, PublicKeyGenerator, NetworkType

# Private key generator
priv_gen = PrivateKeyGenerator(xprv_string, False, 0)
receive_key = priv_gen.receive_key(0)   # m/.../0/0
change_key = priv_gen.change_key(0)     # m/.../1/0

# Public key generator
pub_gen = PublicKeyGenerator.from_xpub(xpub_string)
receive_addrs = pub_gen.receive_addresses(NetworkType.Mainnet, 0, 5)
change_addrs = pub_gen.change_addresses(NetworkType.Mainnet, 0, 5)
```

## Multi-Signature Wallets

```python
from kaspa import PrivateKeyGenerator, PublicKeyGenerator

# Each cosigner uses their own index
cosigner_0_gen = PrivateKeyGenerator(
    xprv=xprv_string,
    is_multisig=True,
    account_index=0,
    cosigner_index=0
)

cosigner_1_gen = PrivateKeyGenerator(
    xprv=other_xprv_string,
    is_multisig=True,
    account_index=0,
    cosigner_index=1
)
```

## Account Types

```python
from kaspa import AccountKind

# Create account kind from string
bip32 = AccountKind("bip32")
legacy = AccountKind("legacy")
multisig = AccountKind("multisig")

print(f"Account kind: {bip32.to_string()}")
```

## Complete Example: HD Wallet

```python
from kaspa import (
    Mnemonic, XPrv, PrivateKeyGenerator,
    PublicKeyGenerator, NetworkType
)

# Create wallet from mnemonic
mnemonic = Mnemonic.random()
seed = mnemonic.to_seed()
master_xprv = XPrv(seed)

# Derive account-level key
account_xprv = master_xprv.derive_path("m/44'/111111'/0'")
account_xpub = account_xprv.to_xpub()

# Export XPub for watch-only wallet
xpub_export = account_xpub.to_str("kpub")
print(f"Watch-only XPub: {xpub_export}")

# Full wallet with private keys
priv_gen = PrivateKeyGenerator(master_xprv, False, 0)

# Watch-only wallet
pub_gen = PublicKeyGenerator.from_xpub(xpub_export)

# Both generate the same addresses
for i in range(3):
    # From private key generator
    priv_addr = priv_gen.receive_key(i).to_address(NetworkType.Mainnet)
    
    # From public key generator
    pub_addr = pub_gen.receive_address(NetworkType.Mainnet, i)
    
    assert priv_addr.to_string() == pub_addr.to_string()
    print(f"Address {i}: {priv_addr.to_string()}")
```
