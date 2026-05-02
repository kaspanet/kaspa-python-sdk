---
search:
  boost: 3
---

# Wallet Files

A wallet file is a single encrypted file on disk. Only one is open at
a time per `Wallet` instance. Creating, opening, and closing a file is
covered in [Lifecycle](lifecycle.md); this page covers the
file-management surface that runs *around* the open file: listing
what's on disk, backing up, restoring, renaming, and rotating the
password.

## Surface

| Method | Returns | Purpose |
| --- | --- | --- |
| [`wallet_enumerate()`](../../reference/Classes/Wallet.md#kaspa.Wallet.wallet_enumerate) | `list[`[`WalletDescriptor`](../../reference/Classes/WalletDescriptor.md)`]` | List every wallet file in the store. |
| [`wallet_export(...)`](../../reference/Classes/Wallet.md#kaspa.Wallet.wallet_export) | `str` (hex) | Dump the encrypted payload as hex. |
| [`wallet_import(...)`](../../reference/Classes/Wallet.md#kaspa.Wallet.wallet_import) | `dict` | Materialise a previously exported payload as a new file. |
| [`wallet_change_secret(...)`](../../reference/Classes/Wallet.md#kaspa.Wallet.wallet_change_secret) | — | Re-encrypt the open file with a new password. |
| [`wallet_rename(...)`](../../reference/Classes/Wallet.md#kaspa.Wallet.wallet_rename) | — | Update the title (and/or filename — see warning below). |

All methods require [`start()`](../../reference/Classes/Wallet.md#kaspa.Wallet.start); none require a wRPC connection.
[`wallet_change_secret`](../../reference/Classes/Wallet.md#kaspa.Wallet.wallet_change_secret) and [`wallet_rename`](../../reference/Classes/Wallet.md#kaspa.Wallet.wallet_rename) operate on the currently
open file.

## Enumerate

```python
descriptors = await wallet.wallet_enumerate()
for d in descriptors:
    print(d.filename, d.title)
```

[`WalletDescriptor`](../../reference/Classes/WalletDescriptor.md) is a typed object — use attribute access. Available
before any wallet is opened, useful for a wallet picker UI.

## Export and import

```python
hex_payload = await wallet.wallet_export(
    wallet_secret="example-secret",
    include_transactions=True,
)
# ... transfer hex_payload to another machine, then:
imported = await wallet.wallet_import(
    wallet_secret="example-secret",
    wallet_data=hex_payload,
)
new_filename = imported["walletDescriptor"]["filename"]
await wallet.wallet_open(
    wallet_secret="example-secret",
    account_descriptors=True,
    filename=new_filename,
)
```

[`wallet_import`](../../reference/Classes/Wallet.md#kaspa.Wallet.wallet_import) is the only file-API method that returns a dict —
read the new filename via `imported["walletDescriptor"]["filename"]`.

The exported payload is borsh-serialized and remains encrypted with
`wallet_secret`; private key material never leaves memory in the
clear. [`wallet_import`](../../reference/Classes/Wallet.md#kaspa.Wallet.wallet_import) writes a new file in the store and returns its
descriptor — you still need to [`wallet_open`](../../reference/Classes/Wallet.md#kaspa.Wallet.wallet_open) it.

## Change secret

```python
await wallet.wallet_change_secret(
    old_wallet_secret="example-secret",
    new_wallet_secret="new-secret",
)
```

Re-encrypts the open file in place. The wallet stays open under the
new secret; subsequent `wallet_open` calls must use the new password.

## Rename

```python
await wallet.wallet_rename(
    wallet_secret="example-secret",
    title="renamed wallet",
)
```

!!! warning "Rename `title` only — `filename` is broken upstream"
    Passing `filename` triggers an upstream rusty-kaspa bug: the new path
    is resolved relative to the process cwd (not the wallet store folder)
    and the `.wallet` extension is not appended. The renamed file ends up
    at `./<filename>` and the in-memory store starts pointing at that bare
    path. Until fixed upstream, leave `filename=None`.

## Where to next

- [Lifecycle](lifecycle.md) — create, open, close, and the rest of
  the state machine.
- [Private Keys](private-keys.md) — populate the open wallet with key
  data.
- [Accounts](accounts.md) — derive accounts from stored key data.
