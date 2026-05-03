---
search:
  boost: 3
---

# Accounts

A wallet can hold multiple accounts of mixed kinds, each backed by one
[private key data entry](private-keys.md).

## Account kinds

| Kind | Backing private key data | Address derivation |
| --- | --- | --- |
| `bip32` | `Mnemonic` | HD path `m/44'/111111'/<account_index>'/<chain>/<index>` |
| `keypair` | `SecretKey` | One address per account (Schnorr or ECDSA) |
| `multisig`, `bip32watch`, `legacy` | — | Specialised; not covered here. |

## Surface

| Method | Purpose |
| --- | --- |
| [`accounts_enumerate()`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_enumerate) | List [`AccountDescriptor`](../../reference/Classes/AccountDescriptor.md) for every account. |
| [`accounts_get(id)`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_get) | Fetch a single descriptor. |
| [`accounts_create_bip32(...)`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_create_bip32) | Create a new HD account at a given `account_index`. |
| [`accounts_create_keypair(...)`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_create_keypair) | Create a single-key account. |
| [`accounts_import_bip32(...)`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_import_bip32) | Same as create_bip32, but runs an address-discovery scan first. |
| [`accounts_import_keypair(...)`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_import_keypair) | Same as create_keypair (discovery is a no-op for a single address). |
| [`accounts_rename(...)`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_rename) | Update an account's name. |
| [`accounts_activate([ids])`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_activate) | Begin UTXO tracking for the given accounts (or all). |
| [`accounts_ensure_default(...)`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_ensure_default) | Ensure a default `bip32` account exists, creating one if needed. |

[`accounts_enumerate`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_enumerate), [`accounts_get`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_get), and the create/import methods
return [`AccountDescriptor`](../../reference/Classes/AccountDescriptor.md) objects (with attribute access). For
deriving the next address on an existing account, see
[Addresses](addresses.md). For sending, see
[Send Transaction](send-transaction.md).

## BIP32 accounts

```python
prv_key_id = await wallet.prv_key_data_create(
    wallet_secret=wallet_secret,
    secret="<24-word mnemonic>",
    kind=PrvKeyDataVariantKind.Mnemonic,
    name="demo-mnemonic-key",
)

acct = await wallet.accounts_create_bip32(
    wallet_secret=wallet_secret,
    prv_key_data_id=prv_key_id,
    account_name="demo-acct-0",
    account_index=0,   # omit to use the next free index
)
```

A second account from the same mnemonic only changes `account_index`:

```python
acct1 = await wallet.accounts_create_bip32(
    wallet_secret=wallet_secret,
    prv_key_data_id=prv_key_id,
    account_index=1,
)
```

### Inspect

```python
for a in await wallet.accounts_enumerate():
    print(a.account_id, a.kind, a.account_name, a.balance)
    print(" receive:", a.receive_address)
    print(" change: ", a.change_address)
```

[`AccountDescriptor`](../../reference/Classes/AccountDescriptor.md)
exposes `kind`, `account_id`, `account_name`, `balance`,
`prv_key_data_ids`, `receive_address` / `change_address`, and (HD
only) `account_index`, `xpub_keys`, `ecdsa`, `receive_address_index`,
`change_address_index`. `get_addresses()` returns every derived
address.

### Import vs. create

[`accounts_import_bip32`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_import_bip32) runs an address-discovery scan before adding
the account, so previously-funded addresses are recognised as used.
Use it when restoring a known-used mnemonic; use
[`accounts_create_bip32`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_create_bip32) for fresh accounts.

## Keypair accounts

A keypair account holds one secp256k1 key and produces one address.
No derivation tree — no "next address", no `account_index`. Use one
when you have a single secret to manage alongside other accounts in
the same wallet, or when moving an existing standalone key into
managed storage.

A keypair account is backed by a `SecretKey`-variant
[private key data entry](private-keys.md):

```python
from kaspa import PrivateKey, PrvKeyDataVariantKind

# 64-char hex secp256k1 secret
secret_hex = PrivateKey(...).to_string()

secret_pkd = await wallet.prv_key_data_create(
    wallet_secret=wallet_secret,
    secret=secret_hex,
    kind=PrvKeyDataVariantKind.SecretKey,
    name="demo-secret-key",
)

kp = await wallet.accounts_create_keypair(
    wallet_secret=wallet_secret,
    prv_key_data_id=secret_pkd,
    ecdsa=False,            # False = Schnorr (default), True = ECDSA
    account_name="keypair-acct",
)
```

`ecdsa` selects the signature scheme. Schnorr is the modern default
and what almost every caller wants. `ecdsa=True` produces an ECDSA
address — only choose it when you need compatibility with a tool or
hardware that requires ECDSA addresses on Kaspa.

### Keypair descriptor shape

| Field | Value |
| --- | --- |
| `kind` | `"keypair"` |
| `account_id` | stable id |
| `receive_address` | the one address |
| `change_address` | the *same* address — no separate change chain |
| `account_index`, `xpub_keys`, `receive_address_index`, `change_address_index` | `None` |
| `ecdsa` | reflects the constructor flag |

[`accounts_create_new_address`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_create_new_address) raises on a keypair account.
[`accounts_import_keypair`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_import_keypair) is the same as [`accounts_create_keypair`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_create_keypair) —
the address-discovery scan is a no-op for a single address.

## Activate

Accounts must be activated before they emit balance events or accept
sends. [`accounts_activate`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_activate) requires a connected wRPC client and a synced
wallet — see [Sync State](sync-state.md).

```python
await wallet.accounts_activate([acct.account_id])
# or, activate every account:
await wallet.accounts_activate()
```

## Ensure-default

Use [`accounts_ensure_default`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_ensure_default) and the
[`AccountKind`](../../reference/Classes/AccountKind.md) helper:

```python
from kaspa import AccountKind

acct = await wallet.accounts_ensure_default(
    wallet_secret=wallet_secret,
    account_kind=AccountKind("bip32"),
    mnemonic_phrase=None,    # generate a fresh mnemonic if creating
)
```

Returns the default `bip32` account if there is one, otherwise creates
one (generating a fresh mnemonic when `mnemonic_phrase` is `None`).
Only `bip32` is supported; other kinds raise.

## Where to next

- [Addresses](addresses.md) — derive new receive / change addresses on an
  existing account.
- [Send Transaction](send-transaction.md) — outgoing flows.
- [Events](events.md) — `Balance`, `Pending`, and `Maturity` events.
