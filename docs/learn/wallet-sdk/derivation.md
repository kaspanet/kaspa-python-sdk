# Derivation

Once you have an [`XPrv`](../../reference/Classes/XPrv.md) (see
[Key Management](key-management.md)), derivation produces every other
key the wallet uses. Kaspa follows BIP-44 with coin type `111111`:

```
m / 44' / 111111' / account' / chain / address_index
```

`chain` is `0` for receive addresses, `1` for change. `account` and
`address_index` are unhardened relative to the account-level node.

See the [Kaspa MDBook page on
derivation](https://kaspa-mdbook.aspectron.com/wallets/addresses.html)
for the protocol-level details.

## Extended keys

```python
from kaspa import Mnemonic, XPrv

seed = Mnemonic.random().to_seed()
xprv = XPrv(seed)

print(xprv.xprv)            # serialized xprv string
print(xprv.private_key)     # the master secp256k1 secret
print(xprv.depth)           # 0 for the master
print(xprv.chain_code)      # 32 bytes
```

[`XPub`](../../reference/Classes/XPub.md) is the public counterpart —
useful for watch-only wallets:

```python
xpub = xprv.to_xpub()
print(xpub.xpub)
print(xpub.to_public_key())
```

## Deriving child keys directly

```python
from kaspa import DerivationPath

# By child number
child = xprv.derive_child(0)
hardened = xprv.derive_child(0, hardened=True)

# By path string
account_xprv = xprv.derive_path("m/44'/111111'/0'")

# By DerivationPath instance
path = DerivationPath("m/44'/111111'/0'/0/0")
leaf = xprv.derive_path(path)
```

[`DerivationPath`](../../reference/Classes/DerivationPath.md) is
mutable — handy for walking a chain incrementally:

```python
path = DerivationPath("m/44'/111111'/0'")
path.push(0)                # → m/44'/111111'/0'/0
path.push(0)                # → m/44'/111111'/0'/0/0
print(path.to_string(), path.length(), path.is_empty())
print(path.parent().to_string())
```

## `PrivateKeyGenerator`

For everyday "give me address `i`" derivation, use
[`PrivateKeyGenerator`](../../reference/Classes/PrivateKeyGenerator.md)
— it handles the full BIP-44 path for you:

```python
from kaspa import PrivateKeyGenerator, NetworkType

gen = PrivateKeyGenerator(
    xprv=xprv,                # accepts XPrv or its xprv-string form
    is_multisig=False,
    account_index=0,
)

for i in range(5):
    key = gen.receive_key(i)               # m/44'/111111'/0'/0/i
    addr = key.to_address(NetworkType.Mainnet)
    print(i, addr.to_string())

change = gen.change_key(0)                  # m/44'/111111'/0'/1/0
```

## `PublicKeyGenerator` (watch-only)

When you only need addresses — no signing —
[`PublicKeyGenerator`](../../reference/Classes/PublicKeyGenerator.md)
derives them from an `xpub` alone:

```python
from kaspa import PublicKeyGenerator, NetworkType

pub = PublicKeyGenerator.from_xpub("xpub...")

# A single address
addr = pub.receive_address(NetworkType.Mainnet, 0)

# A range
addrs = pub.receive_addresses(NetworkType.Mainnet, start=0, end=10)

# Public keys (not addresses)
pubkeys = pub.receive_pubkeys(start=0, end=5)
```

If you have an `XPrv` but want a public-key-only generator (e.g.
watch-only mode in the same process):

```python
pub = PublicKeyGenerator.from_master_xprv(
    xprv=xprv_string,
    is_multisig=False,
    account_index=0,
)
```

`PublicKeyGenerator` exposes `change_addresses(...)` and the
`*_as_strings` variants that skip the `Address` wrapper.

## Multi-signature derivation

Each cosigner has their own `cosigner_index`:

```python
gen0 = PrivateKeyGenerator(xprv=our_xprv, is_multisig=True,
                            account_index=0, cosigner_index=0)
gen1 = PrivateKeyGenerator(xprv=their_xprv, is_multisig=True,
                            account_index=0, cosigner_index=1)
```

For the full multisig wallet flow (creating the multisig address,
spending from it), see the
[Multi-signature transactions](../../guides/multisig.md) recipe.

## Account-kind tag

[`AccountKind`](../../reference/Classes/AccountKind.md) is the
metadata type the wallet uses to track which derivation rules apply.
Construct one explicitly only when calling the wallet's
account-creation methods:

```python
from kaspa import AccountKind

bip32 = AccountKind("bip32")
print(bip32.to_string())   # "bip32"
```

## Where to next

- [Transaction Generator](tx-generator.md) — sign and submit transactions
  with the keys you just derived.
- [Wallet → Accounts](../wallet/accounts.md) — the managed Wallet's
  higher-level account API uses these primitives internally.
- [Custom derivation paths](../../guides/custom-derivation.md) — recipe
  for non-standard paths.
