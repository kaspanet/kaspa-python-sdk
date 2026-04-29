# Open

A wallet file is a single encrypted file on disk. Only one is open at a
time per `Wallet` instance. This page covers the file-management surface:
create, open, enumerate, export/import, change-secret, rename.

## Surface

| Method | Purpose |
| --- | --- |
| `wallet_enumerate()` | List every wallet file in the store. |
| `wallet_create(...)` | Create a new encrypted file and open it. |
| `wallet_open(...)` | Decrypt and open an existing file. |
| `wallet_close()` | Release the open file; secrets leave memory. |
| `wallet_export(...)` | Dump the encrypted payload as hex. |
| `wallet_import(...)` | Materialise a previously exported payload as a new file. |
| `wallet_change_secret(...)` | Re-encrypt the open file with a new password. |
| `wallet_rename(...)` | Update the title (and/or filename — see warning below). |

All methods require `start()`; none require a wRPC connection.

## Create

```python
created = await wallet.wallet_create(
    wallet_secret="example-secret",
    filename="demo",
    overwrite_wallet_storage=False,
    title="demo",
    user_hint="example",
)
```

- `filename` is the on-disk basename; omit for the SDK default.
- `overwrite_wallet_storage=False` raises `WalletAlreadyExistsError` if
  the file exists; pass `True` to clobber.
- `user_hint` is stored alongside the file as a recoverable password hint.

A freshly created wallet has no private key data and no accounts — see
[Private Keys](private-keys.md) and [Accounts](accounts.md).

## Open

```python
opened = await wallet.wallet_open(
    wallet_secret="example-secret",
    account_descriptors=True,   # include account list in the response
    filename="demo",
)
```

`account_descriptors=True` returns the account list in the response dict
so you can pick which to activate without a follow-up
`accounts_enumerate()`.

## Create-or-open pattern

`wallet_create` raises `WalletAlreadyExistsError` when the file exists.
The canonical idempotent boot is:

```python
from kaspa.exceptions import WalletAlreadyExistsError

try:
    await wallet.wallet_create(
        wallet_secret=secret, filename="demo", overwrite_wallet_storage=False,
    )
except WalletAlreadyExistsError:
    await wallet.wallet_open(secret, True, "demo")
```

## Enumerate

```python
descriptors = await wallet.wallet_enumerate()
for d in descriptors:
    print(d.filename, d.title)
```

Returns a `list[WalletDescriptor]`. Available before any wallet is opened —
useful for showing the user a wallet picker.

## Export & import

```python
hex_payload = await wallet.wallet_export(
    wallet_secret="example-secret",
    include_transactions=True,
)
# ...transfer hex_payload to another machine, then:
imported = await wallet.wallet_import(
    wallet_secret="example-secret",
    wallet_data=hex_payload,
)
new_filename = imported["walletDescriptor"]["filename"]
await wallet.wallet_open("example-secret", True, new_filename)
```

The exported payload is borsh-serialized and remains encrypted with
`wallet_secret`; private key material never leaves memory in the clear.
`wallet_import` writes a new file in the store and returns its descriptor —
you still need to `wallet_open` it.

## Change secret

```python
await wallet.wallet_change_secret(
    old_wallet_secret="example-secret",
    new_wallet_secret="new-secret",
)
```

Re-encrypts the open file in place. The wallet stays open under the new
secret; future `wallet_open` calls must use the new password.

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

- [Private Keys](private-keys.md) — the next step after creating a wallet.
- [Accounts](accounts.md) — derive accounts from stored key data.
- [Lifecycle](lifecycle.md) — how these calls fit into start/stop.
