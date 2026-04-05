use kaspa_wallet_core::events::EventKind;
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::*;
use std::str::FromStr;

crate::wrap_unit_enum_for_py!(
    /// Kaspa wallet event types
    PyWalletEventType, "WalletEventType", EventKind, {
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

impl FromStr for PyWalletEventType {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all" => Ok(PyWalletEventType::All),
            "connect" => Ok(PyWalletEventType::Connect),
            "disconnect" => Ok(PyWalletEventType::Disconnect),
            "utxo-index-not-enabled" => Ok(PyWalletEventType::UtxoIndexNotEnabled),
            "sync-state" => Ok(PyWalletEventType::SyncState),
            "wallet-list" => Ok(PyWalletEventType::WalletList),
            "wallet-start" => Ok(PyWalletEventType::WalletStart),
            "wallet-hint" => Ok(PyWalletEventType::WalletHint),
            "wallet-open" => Ok(PyWalletEventType::WalletOpen),
            "wallet-create" => Ok(PyWalletEventType::WalletCreate),
            "wallet-reload" => Ok(PyWalletEventType::WalletReload),
            "wallet-error" => Ok(PyWalletEventType::WalletError),
            "wallet-close" => Ok(PyWalletEventType::WalletClose),
            "prv-key-data-create" => Ok(PyWalletEventType::PrvKeyDataCreate),
            "account-activation" => Ok(PyWalletEventType::AccountActivation),
            "account-deactivation" => Ok(PyWalletEventType::AccountDeactivation),
            "account-selection" => Ok(PyWalletEventType::AccountSelection),
            "account-create" => Ok(PyWalletEventType::AccountCreate),
            "account-update" => Ok(PyWalletEventType::AccountUpdate),
            "server-status" => Ok(PyWalletEventType::ServerStatus),
            "utxo-proc-start" => Ok(PyWalletEventType::UtxoProcStart),
            "utxo-proc-stop" => Ok(PyWalletEventType::UtxoProcStop),
            "utxo-proc-error" => Ok(PyWalletEventType::UtxoProcError),
            "daa-score-change" => Ok(PyWalletEventType::DaaScoreChange),
            "pending" => Ok(PyWalletEventType::Pending),
            "reorg" => Ok(PyWalletEventType::Reorg),
            "stasis" => Ok(PyWalletEventType::Stasis),
            "maturity" => Ok(PyWalletEventType::Maturity),
            "discovery" => Ok(PyWalletEventType::Discovery),
            "balance" => Ok(PyWalletEventType::Balance),
            "metrics" => Ok(PyWalletEventType::Metrics),
            "feerate" => Ok(PyWalletEventType::FeeRate),
            "error" => Ok(PyWalletEventType::Error),
            _ => Err(PyException::new_err(
                "Unsupported string value for EventKind",
            )),
        }
    }
}

impl<'py> FromPyObject<'_, 'py> for PyWalletEventType {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, PyErr> {
        if let Ok(s) = obj.extract::<String>() {
            PyWalletEventType::from_str(&s).map_err(|err| PyException::new_err(err.to_string()))
        } else if let Ok(t) = obj.cast::<PyWalletEventType>() {
            Ok(t.borrow().clone())
        } else {
            Err(PyException::new_err("Expected type `str` or `EventKind`"))
        }
    }
}
