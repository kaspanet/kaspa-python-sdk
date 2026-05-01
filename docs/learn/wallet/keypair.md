# Keypair Accounts

A keypair account holds one secp256k1 key and produces one address.
It has no derivation tree — no "next address", no `account_index`.
Use one when you have a single secret to manage alongside other
accounts in the same wallet, or when moving an existing standalone
key into managed storage.

## Create a keypair account

A keypair account is backed by a `SecretKey`-variant
[private key data entry](private-keys.md):

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
    ecdsa=False,            # False = Schnorr, True = ECDSA
    account_name="keypair-acct",
)
```

`ecdsa` is required. `ecdsa=False` (Schnorr) is what most callers
want; `ecdsa=True` produces an ECDSA-style keypair account.

## What the descriptor looks like

Keypair `AccountDescriptor`s have:

| Field | Value |
| --- | --- |
| `kind` | `"keypair"` |
| `account_id` | stable id |
| `receive_address` | the one address |
| `change_address` | the *same* address — no separate change chain |
| `account_index`, `xpub_keys`, `receive_address_index`, `change_address_index`, `ecdsa` | `None` for the indices; `ecdsa` reflects the constructor flag |

`accounts_create_new_address` raises on a keypair account — there's
no next address to derive.

## When to use a keypair account

- You generated a key with the standalone
  [`PrivateKey`](../../reference/Classes/PrivateKey.md) API or
  imported one from another tool, and want it managed in a wallet
  file.
- You want a single-purpose hot wallet — one address, no rotation.
- You're testing, and a single deterministic address is easier to
  reason about than an HD chain.

For user-facing or rotation-sensitive wallets, use a
[BIP32 account](accounts.md) instead.

## Import vs. create

`accounts_import_keypair` is the variant for an existing key with
on-chain history. The address-discovery scan is a no-op (only one
address), so it's effectively the same as `accounts_create_keypair` —
pick whichever reads better at the call site.

## Where to next

- [Accounts](accounts.md) — BIP32 accounts.
- [Send Transaction](send-transaction.md) — sending from a keypair
  account works the same as from a BIP32 account.
- [Wallet SDK → Key Management](../wallet-sdk/key-management.md) —
  generating a `PrivateKey` outside the wallet first.
