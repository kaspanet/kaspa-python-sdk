# Addresses

A Kaspa address encodes a public key or a script hash, the address
*version* (which signature scheme or script type it pays to), and the
network it belongs to. The SDK exposes them as `Address` instances.

## Anatomy

```
kaspa:  qz0s9f5p7d3e2c4x8n1b6m9k0j2h4g5f3d7a8s9w0e1r2t3y4u5i6o7p8
^^^^^   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
prefix  bech32-encoded version + payload + checksum
```

| Component | Source |
| --- | --- |
| Prefix | The network (see [Networks](networks.md)). |
| Version | One of `PubKey` (Schnorr), `PubKeyECDSA`, or `ScriptHash`. |
| Payload | The hash / public key bytes. |

## Construct or parse

```python
from kaspa import Address

addr = Address("kaspa:qz...")
print(addr.prefix, addr.version, addr.to_string())
```

`Address(string)` raises on malformed input. Use `Address.validate(s)`
to check first without raising:

```python
if Address.validate(s):
    addr = Address(s)
```

## From a key

Schnorr (the default):

```python
from kaspa import NetworkType, PrivateKey

key = PrivateKey("<64-char hex>")
addr = key.to_address(NetworkType.Mainnet)
```

ECDSA:

```python
ecdsa_addr = key.to_address_ecdsa(NetworkType.Mainnet)
```

From a public key:

```python
from kaspa import PublicKey

pub = PublicKey("02a1b2c3...")
addr = pub.to_address(NetworkType.Mainnet)
```

## Versions

| Version | Pays to | Used by |
| --- | --- | --- |
| `PubKey` | Schnorr public key | The SDK's default key derivation; every BIP32 address. |
| `PubKeyECDSA` | ECDSA public key | Keypair accounts created with `ecdsa=True`. |
| `ScriptHash` | A script hash (P2SH-style) | Multisig addresses; custom scripts. |

```python
addr = Address("kaspa:qz...")
print(addr.version)
```

## Network prefixes

| Prefix | Network |
| --- | --- |
| `kaspa:` | mainnet |
| `kaspatest:` | testnet-10, testnet-11 |
| `kaspadev:` | devnet |
| `kaspasim:` | simnet |

To re-encode an address for a different network — for example, to
display the testnet equivalent of a known mainnet address during
testing — set the prefix:

```python
addr = Address("kaspa:qz...")
addr.prefix = "kaspatest"
print(addr.to_string())   # kaspatest:qz...
```

This *does not* re-derive the address from a key; it just rewrites the
prefix. For programmatic conversion of an actual key to a different
network's address, derive again with the right `NetworkType`.

## Scripts and addresses

```python
from kaspa import (
    Address, NetworkType, ScriptPublicKey,
    address_from_script_public_key, pay_to_address_script,
)

# script → address
spk = ScriptPublicKey(0, "20a1b2c3...")
addr = address_from_script_public_key(spk, NetworkType.Mainnet)

# address → script
spk = pay_to_address_script(Address("kaspa:qz..."))
print(spk.script)
```

`pay_to_address_script` is the lockup script you put in a
`TransactionOutput` to pay to that address. See [Transactions → Outputs](transactions/outputs.md).

## Multi-signature addresses

```python
from kaspa import create_multisig_address, NetworkType, PublicKey

pubkeys = [PublicKey("02key1..."), PublicKey("02key2..."), PublicKey("02key3...")]
multi = create_multisig_address(
    minimum_signatures=2,
    keys=pubkeys,
    network_type=NetworkType.Mainnet,
)
print(multi.to_string())
```

For the full multisig spend flow (creating the address, signing with
multiple cosigners, submitting), see the
[Multi-signature transactions](../guides/multisig.md) recipe.

## Where to next

- [Networks](networks.md) — what each prefix means.
- [Transactions](transactions/index.md) — using addresses inside transaction
  outputs.
- [Wallet SDK → Derivation](wallet-sdk/derivation.md) — deriving many
  addresses from one key.
