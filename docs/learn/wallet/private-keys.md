# Private Keys

A *private key data* (PKD) entry is the encrypted secret that backs one or
more accounts. A wallet file holds zero or more PKDs; each account
references exactly one by `PrvKeyDataId`.

## Variants

`PrvKeyDataVariantKind` selects the format of `secret` passed to
`prv_key_data_create`:

| Variant | `secret` format | Typical source |
| --- | --- | --- |
| `Mnemonic` | BIP-39 phrase (12 or 24 words) | New wallets, `Mnemonic.random(...)` |
| `Bip39Seed` | Hex-encoded BIP-39 seed | Pre-derived seeds from another tool |
| `ExtendedPrivateKey` | xprv string | Migrating an existing HD wallet |
| `SecretKey` | 64-char hex secp256k1 key | Single-key (keypair) accounts |

## Surface

| Method | Purpose |
| --- | --- |
| `prv_key_data_create(...)` | Encrypt and store a new PKD; returns its `PrvKeyDataId`. |
| `prv_key_data_enumerate()` | List `PrvKeyDataInfo` for every stored PKD. |
| `prv_key_data_get(secret, id)` | Fetch metadata for a single PKD. |

The wallet must be open. The actual secret never leaves the wallet — only
its metadata is returned.

## Create

```python
from kaspa import PrvKeyDataVariantKind

prv_key_id = await wallet.prv_key_data_create(
    wallet_secret="example-secret",
    secret="<your 24-word mnemonic>",
    kind=PrvKeyDataVariantKind.Mnemonic,
    payment_secret=None,   # optional second factor
    name="demo-key",
)
```

`payment_secret`, when set, layers a second password on top of
`wallet_secret`. Every operation that decrypts this PKD (account creation,
signing, export) must supply it. Use `None` for single-password wallets.

## Enumerate & inspect

```python
for info in await wallet.prv_key_data_enumerate():
    print(info.id, info.name, info.is_encrypted)
```

`PrvKeyDataInfo` exposes:

- `id: PrvKeyDataId` — stable identifier for account creation.
- `name: str | None` — the label set at creation time.
- `is_encrypted: bool` — `True` if a `payment_secret` is required.

`prv_key_data_get(wallet_secret, id)` returns the same metadata for one
entry, raising if the id is unknown.

## Using a PKD

`PrvKeyDataId` is the link between a PKD and the accounts derived from it:

```python
descriptor = await wallet.accounts_create_bip32(
    wallet_secret="example-secret",
    prv_key_data_id=prv_key_id,
    account_index=0,
)
```

A single PKD can back many accounts — common for BIP32 wallets where
multiple account indices share one mnemonic. See [Accounts](accounts.md)
for the account-creation surface.

## Where to next

- [Accounts](accounts.md) — derive BIP32 accounts from a PKD.
- [Keypair Accounts](keypair.md) — single-key accounts from `SecretKey`
  PKDs.
- [Wallet Recovery](../../guides/wallet-recovery.md) — BIP-44 scan for
  accounts already used under a mnemonic.
