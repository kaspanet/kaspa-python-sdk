---
search:
  boost: 3
---

# Addresses

A Kaspa address encodes a public key or script hash, the address
*version* (signature scheme or script type), and its network. The SDK
exposes them as [`Address`](../reference/Classes/Address.md)
instances.

## Anatomy

```
kaspatest:qrxf48dgrdrm70rsk2nqf9p5xj4d4myrwq8mn3wvxcq8…
^^^^^^^^^ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
prefix    bech32-encoded version + payload + checksum
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

[`NetworkType`](../reference/Enums/NetworkType.md) (no suffix) is enough for address derivation: testnet-10
and testnet-11 share the same `kaspatest:` prefix, so
`NetworkType.Testnet` produces the right address either way. Pass a
string or [`NetworkId`](../reference/Classes/NetworkId.md) only when an API needs to distinguish the two
testnets.

## Versions

| Version | Pays to | Used by |
| --- | --- | --- |
| `PubKey` | Schnorr public key | The SDK's default key derivation; every BIP32 address. |
| `PubKeyECDSA` | ECDSA public key | Keypair accounts created with `ecdsa=True`. |
| `ScriptHash` | A script hash (P2SH-style) | Multisig addresses; custom scripts. |

```python
addr = Address("kaspa:qz...")
print(addr.version)            # "PubKey" / "PubKeyECDSA" / "ScriptHash"
```

`addr.version` returns a string. The
[`AddressVersion`](../reference/Enums/AddressVersion.md) enum exists
if you'd rather pattern-match.

## Re-encoding for a different network

The `prefix` attribute is writable. Setting it re-encodes the bech32
string with the new prefix and checksum, but **does not change the
underlying public-key or script-hash bytes** — it's a display-time
operation:

```python
addr = Address("kaspa:qz...")
addr.prefix = "kaspatest"
print(addr.to_string())   # kaspatest:qz...
```

To get the genuine testnet address for a specific *key*, derive it
again under the testnet network:

```python
key.to_address(NetworkType.Testnet)
```

## Scripts and addresses

```python
from kaspa import (
    Address, NetworkType,
    address_from_script_public_key, pay_to_address_script,
)

# address → script (the lockup you put in a TransactionOutput)
spk = pay_to_address_script(Address("kaspa:qz..."))
print(spk.version, spk.script)

# script → address
addr = address_from_script_public_key(spk, NetworkType.Mainnet)
```

[`address_from_script_public_key`](../reference/Functions/address_from_script_public_key.md) needs a [`NetworkType`](../reference/Enums/NetworkType.md) because the
script doesn't carry a prefix — you have to tell the decoder which
network you're displaying for. See
[Transactions → Outputs](transactions/outputs.md).

## Multi-signature addresses

Build a multi-signature address with [`create_multisig_address`](../reference/Functions/create_multisig_address.md):

```python
from kaspa import create_multisig_address, NetworkType, PublicKey

pubkeys = [
    PublicKey("<33-byte compressed-pubkey hex>"),
    PublicKey("<33-byte compressed-pubkey hex>"),
    PublicKey("<33-byte compressed-pubkey hex>"),
]
multi = create_multisig_address(
    minimum_signatures=2,
    keys=pubkeys,
    network_type=NetworkType.Mainnet,
)
print(multi.to_string())
```

For the full multisig spend flow (address creation, multi-cosigner
signing, submission), see
[`examples/transactions/multisig.py`](https://github.com/kaspanet/kaspa-python-sdk/blob/main/examples/transactions/multisig.py).
