## [Unreleased]

*Target: 1.1.0*

### Added
- Enum `PyAddressVersion` exposed to Python as `AddressVersion`
- Enum `PyNetworkType` exposed to Python as `NetworkType`
- Enum `PyEncoding` exposed to Python as `Encoding`
- Enum `PyNotificationEvent` exposed to Python as `NotificationEvent`
- Documentation site using [MkDocs](https://www.mkdocs.org/), [Material for MkDocs](https://squidfunk.github.io/mkdocs-material/), and [mike](https://github.com/jimporter/mike).
- Automatic generation of (most of) the stub (.pyi) file using `pyo3-stub-gen` crate and a binary. RPC TypedDicts (Request/Response structures, RPC types) are manually maintained in `kaspa_rpc.pyi` still.
- Unit and integration tests with [pytest]https://docs.pytest.org/en/stable/.
- Bumped rusty-kaspa dependency version to commit 1a2f98a.
- `GetVirtualChainFromBlockV2` RPC method.
- `to_dict()` method for `Transaction`, `TransactionInput`, `TransactionOutput`, `TransactionOutpoint`, `UtxoEntry`, `UtxoEntries`, and `UtxoEntryReference`.
- `from_dict()` method for `Transaction`, `TransactionInput`, `TransactionOutput`, `TransactionOutpoint`, and `UtxoEntry`.
- `UtxoProcessor` and `UtxoContext` bindings for UTXO tracking and mature range access.
- `UtxoProcessor` maturity setters (`set_coinbase_transaction_maturity_daa`, `set_user_transaction_maturity_daa`).

### Changed
- Moved Kaspa Python SDK out of Rusty-Kaspa (as a workspace member crate) to its own dedicated repository. The internals of this project have changed significantly as a result. However, all APIs exposed to Python remain unchanged. 
- All Python-exposed structs and enums are prefixed with `Py` (e.g. `PyAddress`) internally. The corresponding Python class name has not changed (prefix is dropped in Python).
- All Python-exposed functions are prefixed with `py_` (e.g. `py_sign_message`) internally. The corresponding Python function name has not changed (prefix is dropped in Python).
- All enum parameter types across all functions/methods can be passed as a string (for backwards compatibility) or enum variant. Prior, only a string was accepted. `Opcodes` is the exception to this.
- Standardize internal Rust method names for getters/setters to comply with pyo3 and pyo3-stub-gen. Prefix all with `get_` or `set_`. Remove unnecessary name overrides.
- All setters changed to use consistent `value` for parameter name.
- `PrivateKeyGenerator` constructor accepts `xprv` parameter as both a `str` or `XPrv` instance now.
- `PublicKeyGenerator.from_master_xprv()` accepts `xprv` parameter as both a `str` or `XPrv` instance now.
- Python 3.9 is no longer supported. Minimum supported version is now 3.10.

### Fixed

### Breaking Changes
- Python 3.9 is no longer supported. Minimum supported version is now 3.10.

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
