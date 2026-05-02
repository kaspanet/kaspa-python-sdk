---
search:
  boost: 3
---

# Derivation

Once you have an [`XPrv`](../../reference/Classes/XPrv.md) (see
[Key Management](key-management.md)), derivation produces every other
key the wallet uses. Kaspa follows BIP-44 with coin type `111111`:

```
m / 44' / 111111' / account' / chain / address_index
```

A trailing `'` denotes a *hardened* level — derived from the parent's
private key, so the corresponding `xpub` cannot derive its children.
Non-hardened (no `'`) levels can be derived from the `xpub` alone,
which is what makes watch-only wallets possible. `chain` is `0` for
receive addresses, `1` for change.

See the [Kaspa MDBook page on
derivation](https://kaspa-mdbook.aspectron.com/wallets/addresses.html)
for protocol-level details.

## `PrivateKeyGenerator`

For "give me address `i`" derivation, use
[`PrivateKeyGenerator`](../../reference/Classes/PrivateKeyGenerator.md).
It walks the full BIP-44 path for you:

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

`xprv` accepts either an [`XPrv`](../../reference/Classes/XPrv.md) instance or its xprv string form;
[`NetworkType`](../../reference/Enums/NetworkType.md) selects the address prefix.

## `PublicKeyGenerator` — watch-only

When you only need addresses (no signing),
[`PublicKeyGenerator`](../../reference/Classes/PublicKeyGenerator.md)
derives them from an [`xpub`](../../reference/Classes/XPub.md):

```python
from kaspa import PublicKeyGenerator, NetworkType

pub = PublicKeyGenerator.from_xpub("xpub...")

addr  = pub.receive_address(NetworkType.Mainnet, 0)
addrs = pub.receive_addresses(NetworkType.Mainnet, start=0, end=10)
keys  = pub.receive_pubkeys(start=0, end=5)
```

A public-only generator can be built from an existing [`XPrv`](../../reference/Classes/XPrv.md):

```python
pub = PublicKeyGenerator.from_master_xprv(
    xprv=xprv_string,
    is_multisig=False,
    account_index=0,
)
```

`change_addresses(...)` and the `*_as_strings` variants are also
available.

## Manual derivation

For non-standard paths or one-off derivations, drop down to the
extended-key APIs directly. Most users won't need this — reach for
[`PrivateKeyGenerator`](../../reference/Classes/PrivateKeyGenerator.md) first.

```python
from kaspa import DerivationPath, Mnemonic, XPrv

xprv = XPrv(Mnemonic.random().to_seed())

print(xprv.xprv, xprv.depth, xprv.chain_code)

# Public counterpart — useful for watch-only wallets.
xpub = xprv.to_xpub()

# Direct derivation
child         = xprv.derive_child(0)                     # non-hardened
hardened      = xprv.derive_child(0, hardened=True)
account_xprv  = xprv.derive_path("m/44'/111111'/0'")
leaf          = xprv.derive_path(DerivationPath("m/44'/111111'/0'/0/0"))
```

[`DerivationPath`](../../reference/Classes/DerivationPath.md) is
mutable — handy for walking a chain incrementally with `push`,
`parent`, `length`, etc. See
[`examples/derivation.py`](https://github.com/kaspanet/kaspa-python-sdk/blob/main/examples/derivation.py)
for runnable derivation snippets.

## Where to next

- [UTXO Processor](utxo-processor.md) — set up the live event pipeline
  for the addresses you derived.
- [Transaction Generator](tx-generator.md) — sign and submit using
  these keys.
- [Wallet → Accounts](../wallet/accounts.md) — the managed Wallet's
  account API uses these primitives internally.
