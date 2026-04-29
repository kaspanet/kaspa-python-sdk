# Keypair Accounts

A keypair account holds one secp256k1 key and produces one address. It has
no derivation tree — there's no "next address" and no `account_index`.
Use these when you have a single secret you want to manage alongside other
accounts in the same wallet, or when you're moving an existing standalone
key into managed storage.

## Create a keypair account

A keypair account is backed by a `SecretKey`-variant PKD:

```python
from kaspa import PrivateKey, PrvKeyDataVariantKind

# 64-char hex secp256k1 secret
secret_hex = PrivateKey(...).to_string()

secret_pkd = await wallet.prv_key_data_create(
    wallet_secret=secret,
    secret=secret_hex,
    kind=PrvKeyDataVariantKind.SecretKey,
    name="demo-secret-key",
)

kp = await wallet.accounts_create_keypair(
    wallet_secret=secret,
    prv_key_data_id=secret_pkd,
    ecdsa=False,            # False = Schnorr (default), True = ECDSA
    account_name="keypair-acct",
)
```

`ecdsa=True` is for ECDSA-style keypair accounts; the default Schnorr
variant is what most callers want.

## What the descriptor looks like

Keypair `AccountDescriptor`s have:

| Field | Value |
| --- | --- |
| `kind` | `"keypair"` |
| `account_id` | stable id |
| `receive_address` | the one address |
| `change_address` | the *same* address — there is no separate change chain |
| `account_index`, `xpub_keys`, `receive_address_index`, `change_address_index`, `ecdsa` | `None` for the indices; `ecdsa` reflects the constructor flag |

`accounts_create_new_address` raises on a keypair account — there is no
next address to derive.

## When to use a keypair account

- You generated a key with the standalone `PrivateKey` API or imported one
  from another tool, and want it managed inside a wallet file.
- You want a single-purpose hot wallet — one address, no rotation.
- You're testing and a single deterministic address is easier to reason
  about than an HD chain.

For everyday wallets — anything user-facing or anything where address
rotation matters — use a [BIP32 account](accounts.md) instead.

## Import vs. create

`accounts_import_keypair` is the variant for an existing key that may
already have on-chain history. The address-discovery scan path is a no-op
(there's only one address), so it's effectively the same as
`accounts_create_keypair` — use whichever reads better at the call site.

## Where to next

- [Accounts](accounts.md) — BIP32 accounts.
- [Send Transaction](send-transaction.md) — sending from a keypair account
  works the same as from a BIP32 account.
