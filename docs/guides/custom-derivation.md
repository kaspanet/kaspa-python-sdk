# Custom derivation paths

The default
[`PrivateKeyGenerator`](../learn/wallet-sdk/derivation.md#privatekeygenerator)
walks the BIP-44 path `m/44'/111111'/<account_index>'/<chain>/<index>`.
When you need something off-spec — a custom path for migrating from
another wallet, a one-off subkey, a non-BIP-44 layout — derive directly
from the `XPrv`.

## Recipe

```python
from kaspa import DerivationPath, Mnemonic, NetworkType, XPrv

m = Mnemonic("<24 words>")
xprv = XPrv(m.to_seed())

# Walk an arbitrary path
custom = xprv.derive_path("m/9999'/7'/0/42")
print(custom.private_key.to_string())
print(custom.private_key.to_address(NetworkType.Mainnet).to_string())

# Build paths programmatically
path = DerivationPath("m/44'/111111'/0'")
path.push(0)            # → m/44'/111111'/0'/0
path.push(0)            # → m/44'/111111'/0'/0/0
leaf = xprv.derive_path(path)

# Step by step (no path string needed)
account_xprv = xprv.derive_child(0, hardened=True)
chain_xprv   = account_xprv.derive_child(0)
addr_xprv    = chain_xprv.derive_child(0)
```

## Notes

- **Hardened path components** end in `'`. Pass `hardened=True` to
  `derive_child` for the same effect.
- **Re-using `DerivationPath`.** It's mutable — `push`, `parent`,
  `to_string`, `length`, `is_empty`. Convenient when walking a chain
  in a loop.
- **The result of `derive_path` is itself an `XPrv`**, so you can keep
  deriving below it.
- **Watch-only with a custom path.** Take `XPub` from any node in the
  tree and derive *below it* with `XPub.derive_path(...)` (unhardened
  components only — hardened derivation requires the private side).
- **Address encoding.** `derive_path` gives you a key, not an address.
  Call `.to_address(NetworkType.X)` to encode for the right network.

## When *not* to do this

- For everyday HD wallets, use
  [`PrivateKeyGenerator`](../learn/wallet-sdk/derivation.md#privatekeygenerator)
  — it produces the addresses the rest of the ecosystem expects.
- For multisig, use the cosigner-aware path (`is_multisig=True`,
  `cosigner_index=...`) — see
  [Multi-signature transactions](multisig.md).
- A custom path means *you own the recovery story*. Document the path
  alongside the mnemonic; "I derived from `m/9999'/7'/0/42`" is not
  recoverable without that note.
