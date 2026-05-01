# Accounts

A wallet holds N accounts of mixed kinds, each backed by exactly one
[private key data entry](private-keys.md). The two everyday kinds:

- **BIP32** — HD-derived; one mnemonic backs many accounts at
  different `account_index`es.
- **Keypair** — a single secp256k1 key, one address. See
  [Keypair Accounts](keypair.md).

## Account kinds

| Kind | Backing private key data | Address derivation |
| --- | --- | --- |
| `bip32` | `Mnemonic`, `Bip39Seed`, or `ExtendedPrivateKey` | HD path `m/44'/111111'/<account_index>'/<chain>/<index>` |
| `keypair` | `SecretKey` | One address per account (Schnorr or ECDSA) |
| `multisig`, `bip32watch`, `legacy` | — | Specialised; not covered here. |

This page covers BIP32. Keypair accounts have their own page.

## Surface

| Method | Purpose |
| --- | --- |
| `accounts_enumerate()` | List `AccountDescriptor` for every account. |
| `accounts_get(id)` | Fetch a single descriptor. |
| `accounts_create_bip32(...)` | Create a new HD account at a given `account_index`. |
| `accounts_import_bip32(...)` | Same, but runs an address-discovery scan first. |
| `accounts_rename(...)` | Update an account's name. |
| `accounts_activate([ids])` | Begin UTXO tracking for the given accounts (or all). |
| `accounts_ensure_default(...)` | Idempotently ensure a default `bip32` account exists. |

For deriving the next address on an existing account, see
[Addresses](addresses.md). For sending, see
[Send Transaction](send-transaction.md).

## Create a BIP32 account

```python
prv_key_id = await wallet.prv_key_data_create(
    wallet_secret=secret,
    secret="<24-word mnemonic>",
    kind=PrvKeyDataVariantKind.Mnemonic,
    name="demo-mnemonic-key",
)

acct = await wallet.accounts_create_bip32(
    wallet_secret=secret,
    prv_key_data_id=prv_key_id,
    account_name="demo-acct-0",
    account_index=0,   # omit to use the next free index
)
```

A second account from the same mnemonic only changes `account_index`:

```python
acct1 = await wallet.accounts_create_bip32(
    wallet_secret=secret, prv_key_data_id=prv_key_id, account_index=1,
)
```

## Inspect

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

## Activate

Accounts must be activated before they emit balance events or accept
sends. Activation requires a connected wRPC client *and* a synced
wallet — see [Sync State](sync-state.md).

```python
await wallet.accounts_activate([acct.account_id])
# or, activate every account:
await wallet.accounts_activate()
```

## Ensure-default

```python
from kaspa import AccountKind

acct = await wallet.accounts_ensure_default(
    wallet_secret=secret,
    account_kind=AccountKind("bip32"),
    mnemonic_phrase=None,    # generate a fresh mnemonic if creating
)
```

Returns the default `bip32` account if there is one, otherwise creates
one (generating a fresh mnemonic when `mnemonic_phrase` is `None`).
Only `bip32` is supported; other kinds raise.

## Import vs. create

`accounts_import_bip32` is the recovery-flow variant: it runs an
address-discovery scan before adding the account, so previously-funded
addresses are recognised as used. Use it when restoring a known-used
mnemonic; use `accounts_create_bip32` for fresh accounts.

To scan a mnemonic *before* picking an index, see
[Wallet Recovery](../../guides/wallet-recovery.md).

## Where to next

- [Addresses](addresses.md) — derive new receive / change addresses on an
  existing account.
- [Keypair Accounts](keypair.md) — single-key accounts.
- [Send Transaction](send-transaction.md) — outgoing flows.
- [Transaction History](transaction-history.md) — `Balance`, `Pending`,
  and `Maturity` events.
