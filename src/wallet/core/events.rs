use kaspa_wallet_core::events::EventKind;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

crate::wrap_unit_enum_for_py!(
    /// Kaspa wallet event types
    PyEventKind, "EventKind", EventKind, {
    All,
    Connect,
    Disconnect,
    UtxoIndexNotEnabled,
    SyncState,
    WalletList,
    WalletStart,
    WalletHint,
    WalletOpen,
    WalletCreate,
    WalletReload,
    WalletError,
    WalletClose,
    PrvKeyDataCreate,
    AccountActivation,
    AccountDeactivation,
    AccountSelection,
    AccountCreate,
    AccountUpdate,
    ServerStatus,
    UtxoProcStart,
    UtxoProcStop,
    UtxoProcError,
    DaaScoreChange,
    Pending,
    Reorg,
    Stasis,
    Maturity,
    Discovery,
    Balance,
    Metrics,
    FeeRate,
    Error,
});
