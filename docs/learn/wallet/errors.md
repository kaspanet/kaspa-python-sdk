---
search:
  boost: 3
---

# Errors

The wallet's errors live in `kaspa.exceptions`. Most are
self-explanatory; a few are common enough that they're worth knowing
ahead of time. The full list is in
[`kaspa.exceptions`](../../reference/SUMMARY.md).

## Common errors

| Error | Triggered by | Fix |
| --- | --- | --- |
| [`WalletAlreadyExistsError`](../../reference/Exceptions/WalletAlreadyExistsError.md) | [`wallet_create`](../../reference/Classes/Wallet.md#kaspa.Wallet.wallet_create)`(filename=..., overwrite_wallet_storage=False)` when the file exists. | Catch and call [`wallet_open(...)`](../../reference/Classes/Wallet.md#kaspa.Wallet.wallet_open) instead, or pass `overwrite_wallet_storage=True` (clobbers). See [Lifecycle → create-or-open](lifecycle.md#create-or-open-pattern). |
| [`WalletNotOpenError`](../../reference/Exceptions/WalletNotOpenError.md) | Any `prv_key_data_*`, `accounts_*`, `transactions_*` call before [`wallet_create`](../../reference/Classes/Wallet.md#kaspa.Wallet.wallet_create) / [`wallet_open`](../../reference/Classes/Wallet.md#kaspa.Wallet.wallet_open). | Open the wallet first. |
| [`WalletNotConnectedError`](../../reference/Exceptions/WalletNotConnectedError.md) | [`accounts_activate(...)`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_activate) without a connected wRPC client. | `await wallet.`[`connect`](../../reference/Classes/Wallet.md#kaspa.Wallet.connect)`(...)` first. |
| [`WalletNotSyncedError`](../../reference/Exceptions/WalletNotSyncedError.md) | [`accounts_send`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_send), [`accounts_estimate`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_estimate), [`accounts_get_utxos`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_get_utxos), [`accounts_transfer`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_transfer) before the processor finishes its sync handshake. | Wait on [`wallet.is_synced`](../../reference/Classes/Wallet.md#kaspa.Wallet.is_synced) (poll or `SyncState` listener) — see [Sync State](sync-state.md). |
| [`WalletInsufficientFundsError`](../../reference/Exceptions/WalletInsufficientFundsError.md) | [`accounts_send`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_send) / [`accounts_transfer`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_transfer) when the mature balance can't cover `outputs + fees`. | Wait for pending UTXOs to mature (`Maturity` [event](events.md)), reduce amount, or lower priority fee. |
| `"Invalid prv key data kind, supported types are Mnemonic and SecretKey"` | [`prv_key_data_create`](../../reference/Classes/Wallet.md#kaspa.Wallet.prv_key_data_create)`(kind=`[`PrvKeyDataVariantKind`](../../reference/Enums/PrvKeyDataVariantKind.md)`.Bip39Seed)` or `kind=ExtendedPrivateKey`. | Use `Mnemonic` or `SecretKey`. The other two enum variants are not implemented upstream — see [Private Keys](private-keys.md#variants). |
| `UtxoIndexNotEnabled` ([event](events.md), not exception) | Connecting to a kaspad without the UTXO index. The processor refuses to proceed. | Point at a node that has UTXO indexing enabled. |
| `RuntimeError: cannot change network while connected` | [`wallet.set_network_id(...)`](../../reference/Classes/Wallet.md#kaspa.Wallet.set_network_id) after [`connect()`](../../reference/Classes/Wallet.md#kaspa.Wallet.connect). | `await wallet.`[`disconnect()`](../../reference/Classes/Wallet.md#kaspa.Wallet.disconnect) → [`set_network_id(...)`](../../reference/Classes/Wallet.md#kaspa.Wallet.set_network_id) → `await wallet.`[`connect()`](../../reference/Classes/Wallet.md#kaspa.Wallet.connect). |
| `RuntimeError: cannot derive address on a keypair account` | [`accounts_create_new_address(...)`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_create_new_address) on a keypair account. | Keypair accounts have one fixed address; read it from [`descriptor.receive_address`](../../reference/Classes/AccountDescriptor.md). See [Keypair accounts](accounts.md#keypair-accounts). |

## Where to next

- [Lifecycle](lifecycle.md) — preconditions and ordering rules.
- [Sync State](sync-state.md) — what the sync gate is and how to wait
  on it.
- [`kaspa.exceptions` reference](../../reference/SUMMARY.md) — the
  full set of named exceptions.
