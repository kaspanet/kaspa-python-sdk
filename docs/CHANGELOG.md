## [Unreleased]

### Added
- Class `Wallet` exposed to Python, providing the full rusty-kaspa wallet API: lifecycle (`start`, `stop`, `connect`, `disconnect`, `set_network_id`, `get_status`), wallet file operations (`wallet_enumerate`, `wallet_create`, `wallet_open`, `wallet_close`, `wallet_reload`, `wallet_rename`, `wallet_change_secret`, `wallet_export`, `wallet_import`), private key data (`prv_key_data_enumerate`, `prv_key_data_create`, `prv_key_data_remove`, `prv_key_data_get`), accounts (`accounts_enumerate`, `accounts_create_bip32`, `accounts_create_keypair`, `accounts_import_bip32`, `accounts_import_keypair`, `accounts_rename`, `accounts_discovery`, `accounts_ensure_default`, `accounts_activate`, `accounts_get`, `accounts_create_new_address`, `accounts_get_utxos`), spending (`accounts_estimate`, `accounts_send`, `accounts_transfer`, `accounts_commit_reveal`, `accounts_commit_reveal_manual`), transaction history (`transactions_data_get`, `transactions_replace_note`, `transactions_replace_metadata`), storage (`batch`, `flush`, `retain_context`, `address_book_enumerate`), fee rate (`fee_rate_estimate`, `fee_rate_poller_enable`, `fee_rate_poller_disable`), and event listeners (`add_event_listener`, `remove_event_listener`).
- Class `AccountDescriptor` exposed to Python. Contains account metadata including kind, ID, name, balance, and addresses, and account kind specific properties: `account_index`, `xpub_keys`, `ecdsa`, `receive_address_index`, and `change_address_index` (each `None` when the property is not applicable to the account kind).
- Class `AccountId` exposed to Python. Hex-encoded identifier for a wallet account.
- Class `PrvKeyDataId` exposed to Python. Hex-encoded identifier for a private key data entry. Constructible from a hex string and accepted interchangeably with `str` by `Wallet` methods.
- Class `WalletDescriptor` exposed to Python. Contains wallet metadata including title and filename.
- Class `PrvKeyDataInfo` exposed to Python. Holds private key data info including ID, name, and encryption status.
- Class `PaymentOutput` is now constructible via `PaymentOutput(address, amount)`, and accepted as either an instance or a `{"address": ..., "amount": ...}` dict wherever the bindings take a `PaymentOutput`.
- Class `Fees` exposed to Python. Specifies transaction fees as an absolute sompi amount with an optional fee source.
- Enum `WalletEventType` exposed to Python. Enumerates wallet event types such as connection, account, and transaction events.
- Enum `AccountsDiscoveryKind` exposed to Python. Specifies the account discovery method (e.g. Bip44).
- Enum `NewAddressKind` exposed to Python. Indicates whether a new address is for receiving or change.
- Enum `CommitRevealAddressKind` exposed to Python. Specifies the address type for commit-reveal operations.
- Enum `PrvKeyDataVariantKind` exposed to Python. Identifies the storage format of private key data (mnemonic, seed, etc.).
- Enum `TransactionKind` exposed to Python. Categorizes transactions by type (incoming, outgoing, change, etc.).
- Enum `FeeSource` exposed to Python. Indicates who pays the transaction fee: sender or receiver.
- Enum `AccountKind` exposed to Python. Represents the account type (legacy, bip32, multisig, keypair, etc.).
- Wallet-specific exception classes populated into the `kaspa.exceptions` submodule, covering the rusty-kaspa wallet error variants (e.g. `WalletInsufficientFundsError`, `WalletAccountNotFoundError`, `WalletNotSyncedError`, etc.).
- Examples under `examples/wallet/` demonstrating wallet usage
- Pytest options `--network-id` and `--rpc-url` for targeting integration tests at a specific network / node.
- Documentation site reorganised around [Diataxis](https://diataxis.fr): a sequenced **Learn** section (RPC, Wallet, Wallet SDK, Networks, Addresses, Transactions, Kaspa Concepts) covers the SDK topic by topic, with a focused **Guides** cookbook for cross-cutting recipes (mnemonic restore, message signing, wallet recovery, custom derivation, multisig).
- `docs/getting-started/security.md`: a single canonical page covering secret-handling rules. Other pages link to it instead of duplicating the warning.
- Learn → Transactions split into a dedicated section: Overview, Inputs, Outputs, Mass & Fees, Signing, Submission, Metadata Fields, Serialization. Each page covers one component of a transaction with light Kaspa-protocol context, replacing the single `learn/transactions.md` page.

### Changed
- `py_error_map!` macro extended to register wallet exception variants into the `kaspa.exceptions` submodule.
- Integration tests now default to `mainnet` (overridable via `--network-id` / `--rpc-url`).
- `build-dev` script builds with `--strip` for smaller artifacts.
- `pyproject.toml`: set `python-source = "python"` and moved the package stub tree under `python/kaspa/` (`kaspa.pyi` → `python/kaspa/__init__.pyi`).
- `Hash` accepts `str` in addition to `Hash` instances wherever it is used as an argument, and gained `to_hex()` and `__repr__` methods.
- Added `Balance` `__repr__` method.

### Fixed
- `AccountDescriptor.__repr__` now correctly renders optional fields.

## [1.1.0] - 2026-03-04

### Added
- Enum `PyAddressVersion` exposed to Python as `AddressVersion`
- Enum `PyNetworkType` exposed to Python as `NetworkType`
- Enum `PyEncoding` exposed to Python as `Encoding`
- Enum `PyNotificationEvent` exposed to Python as `NotificationEvent`
- Documentation site using [MkDocs](https://www.mkdocs.org/), [Material for MkDocs](https://squidfunk.github.io/mkdocs-material/), and [mike](https://github.com/jimporter/mike).
- Automatic generation of (most of) the stub (.pyi) file using `pyo3-stub-gen` crate and a binary. RPC TypedDicts (Request/Response structures, RPC types) are manually maintained in `kaspa_rpc.pyi` still.
- Unit and integration tests with [pytest]https://docs.pytest.org/en/stable/.
- `GetVirtualChainFromBlockV2` RPC method.
- `to_dict()` method for `Transaction`, `TransactionInput`, `TransactionOutput`, `TransactionOutpoint`, `UtxoEntry`, `UtxoEntries`, and `UtxoEntryReference`.
- `from_dict()` method for `Transaction`, `TransactionInput`, `TransactionOutput`, `TransactionOutpoint`, and `UtxoEntry`.
- Classes `UtxoProcessor` and `UtxoContext` bindings for UTXO tracking and mature range access.
- Enum `PyUtxoProcessorEvent` exposed to Python as `UtxoProcessorEvent`.
- Submodule `exceptions` where custom exceptions will be located. Currently empty given no custom exceptions exist (yet).
- `version` getter for `ScriptPublicKey`.
- Added to `GeneratorSummary`: `to_dict()` method, properties `network_id`, `mass`, and `stages`.

### Changed
- Bumped rusty-kaspa dependency version to commit e97070f.
- Moved Kaspa Python SDK out of Rusty-Kaspa (as a workspace member crate) to its own dedicated repository. The internals of this project have changed significantly as a result. However, all APIs exposed to Python remain unchanged. 
- All Python-exposed structs and enums are prefixed with `Py` (e.g. `PyAddress`) internally. The corresponding Python class name has not changed (prefix is dropped in Python).
- All Python-exposed functions are prefixed with `py_` (e.g. `py_sign_message`) internally. The corresponding Python function name has not changed (prefix is dropped in Python).
- All enum parameter types across all functions/methods can be passed as a string (for backwards compatibility) or enum variant. Prior, only a string was accepted. `Opcodes` is the exception to this.
- Standardize internal Rust method names for getters/setters to comply with pyo3 and pyo3-stub-gen. Prefix all with `get_` or `set_`. Remove unnecessary name overrides.
- All setters changed to use consistent `value` for parameter name.
- `PrivateKeyGenerator` constructor accepts `xprv` parameter as both a `str` or `XPrv` instance now.
- `PublicKeyGenerator.from_master_xprv()` accepts `xprv` parameter as both a `str` or `XPrv` instance now.
- `Generator`, `create_transactions`, and `estimate_transactions` now accept `UtxoContext` entries (network_id optional for context inputs).
- Python 3.9 is no longer supported. Minimum supported version is now 3.10.
- Fix ScriptBuilder `add_op`/`add_ops` functions. `add_op` incorrectly allowed mulitple ops to be passed. `add_ops` incorrectly allowed a single op to be passed.

### Fixed
- `kaspa.pyi`: add overloads for `UtxoProcessor.add_event_listener` / `remove_event_listener` (typing only).

### Breaking Changes
- Python 3.9 is no longer supported. Minimum supported version is now 3.10.
- `Generator`, `create_transactions`, and `estimate_transactions` reordered parameters to keep required arguments first (entries, change_address, network_id optional). Positional callers must update.

## [1.0.1.post2] - 2025-11-13
### Added
- Support for Python 3.14

### Changed
- Specify Python compatibility as >=3.9,<=3.14
- Upgraded crate pyo3 from 0.24.2 to 0.27.1.
- Upgraded crate pyo3-async-runtimes from 0.24 to 0.27.0
- Upgraded crate pyo3-log from 0.12.4 to 0.13.2
- Upgraded crate serde-pyobject from 0.6.2 to 0.8.0
- CI updates


## [1.0.1.post1] - 2025-09-27
### Added
- Added RPC method `submit_block`.
- RPC method `get_virtual_chain_from_block` support of `minConfirmationCount`.
- RPC method doc strings in .pyi with expected `request` dict structure (for calls that require a `request` dict).

### Changed
- RPC method `submit_transaction`'s `request` parameter now supports key `allowOrphan`. A deprecation warning will print when key `allow_orphan` is used. Support for `allow_orphan` will be removed in future version. This moves towards case consistency.
- KeyError is now raised when an expected key is not contained in a dictionary. Prior, a general Exception was raised.
