# Mnemonics

This guide covers working with BIP-39 mnemonic seed phrases in the Kaspa Python SDK.

!!! danger "Security Warning"
    **Handle Private Keys Securely**

    **These examples do not use proper private key/mnemonic/seed handling.** This is omitted here for brevity.

    Never store your private keys in plain text, or directly in source code. Store securely offline. Anyone with access to this phrase has full control over your funds.

## Overview

Mnemonic phrases (also called seed phrases or recovery phrases) are human-readable representations of cryptographic seeds. The Kaspa SDK supports BIP-39 compatible mnemonics.

## Generating a New Mnemonic

```python
from kaspa import Mnemonic

# Generate a random 24-word mnemonic (default)
mnemonic = Mnemonic.random()
print(f"Your seed phrase: {mnemonic.phrase}")

# Generate with specific word count
mnemonic_12 = Mnemonic.random(word_count=12)  # 12 words
mnemonic_24 = Mnemonic.random(word_count=24)  # 24 words (recommended)
```

## Restoring from a Mnemonic

```python
from kaspa import Mnemonic

phrase = "abandon abandon abandon ... about"

# Validate before use
if Mnemonic.validate(phrase):
    mnemonic = Mnemonic(phrase)
    print("Mnemonic restored successfully")
else:
    print("Invalid mnemonic phrase!")
```

## Validation

```python
from kaspa import Mnemonic, Language

phrase = "word1 word2 word3 ..."

# Basic validation
is_valid = Mnemonic.validate(phrase)
print(f"Valid: {is_valid}")

# With specific language
is_valid_english = Mnemonic.validate(phrase, Language.English)
```

## Converting to Seed

```python
from kaspa import Mnemonic, XPrv

mnemonic = Mnemonic.random()

# Convert to seed (without passphrase)
seed = mnemonic.to_seed()

# Convert with optional passphrase for extra security
seed_with_passphrase = mnemonic.to_seed("my-secret-passphrase")

# Use seed to create extended private key
xprv = XPrv(seed)
```

!!! info "Passphrase"
    The passphrase (sometimes called "25th word") provides additional security. The same mnemonic with different passphrases produces different seeds.

## Working with Entropy

Access the underlying entropy:

```python
from kaspa import Mnemonic

mnemonic = Mnemonic.random()

# Get entropy as hex string
entropy = mnemonic.entropy
print(f"Entropy: {entropy}")

# Set new entropy (advanced use)
mnemonic.entropy = "new-entropy-hex"
```

## Language Support

```python
from kaspa import Mnemonic, Language

# Currently supported languages
mnemonic = Mnemonic.random()  # Uses English by default

# Specify language explicitly
mnemonic = Mnemonic(phrase, Language.English)
```

## Wallet Creation Example

```python
from kaspa import (
    Mnemonic, XPrv, PrivateKeyGenerator,
    NetworkType
)

# Step 1: Generate mnemonic
mnemonic = Mnemonic.random()
print(f"Seed phrase: {mnemonic.phrase}")

# Step 2: Convert to seed
seed = mnemonic.to_seed()

# Step 3: Create extended private key
xprv = XPrv(seed)

# Step 4: Create key generator
key_gen = PrivateKeyGenerator(
    xprv=xprv,
    is_multisig=False,
    account_index=0
)

# Step 5: Derive addresses
for i in range(3):
    private_key = key_gen.receive_key(i)
    address = private_key.to_address(NetworkType.Mainnet)
    print(f"Address {i}: {address.to_string()}")
```
