use kaspa_wallet_core::events::EventKind;
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::*;
use std::str::FromStr;

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

impl FromStr for PyEventKind {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all" => Ok(PyEventKind::All),
            "connect" => Ok(PyEventKind::Connect),
            "disconnect" => Ok(PyEventKind::Disconnect),
            "utxoindexnotenabled" => Ok(PyEventKind::UtxoIndexNotEnabled),
            "syncstate" => Ok(PyEventKind::SyncState),
            "walletlist" => Ok(PyEventKind::WalletList),
            "walletstart" => Ok(PyEventKind::WalletStart),
            "wallethint" => Ok(PyEventKind::WalletHint),
            "walletopen" => Ok(PyEventKind::WalletOpen),
            "walletcreate" => Ok(PyEventKind::WalletCreate),
            "walletreload" => Ok(PyEventKind::WalletReload),
            "walleterror" => Ok(PyEventKind::WalletError),
            "walletclose" => Ok(PyEventKind::WalletClose),
            "prvkeydatacreate" => Ok(PyEventKind::PrvKeyDataCreate),
            "accountactivation" => Ok(PyEventKind::AccountActivation),
            "accountdeactivation" => Ok(PyEventKind::AccountDeactivation),
            "accountselection" => Ok(PyEventKind::AccountSelection),
            "accountcreate" => Ok(PyEventKind::AccountCreate),
            "accountupdate" => Ok(PyEventKind::AccountUpdate),
            "serverstatus" => Ok(PyEventKind::ServerStatus),
            "utxoprocstart" => Ok(PyEventKind::UtxoProcStart),
            "utxoprocstop" => Ok(PyEventKind::UtxoProcStop),
            "utxoprocerror" => Ok(PyEventKind::UtxoProcError),
            "daascorechange" => Ok(PyEventKind::DaaScoreChange),
            "pending" => Ok(PyEventKind::Pending),
            "reorg" => Ok(PyEventKind::Reorg),
            "stasis" => Ok(PyEventKind::Stasis),
            "maturity" => Ok(PyEventKind::Maturity),
            "discovery" => Ok(PyEventKind::Discovery),
            "balance" => Ok(PyEventKind::Balance),
            "metrics" => Ok(PyEventKind::Metrics),
            "feerate" => Ok(PyEventKind::FeeRate),
            "error" => Ok(PyEventKind::Error),
            _ => Err(PyException::new_err(
                "Unsupported string value for EventKind",
            )),
        }
    }
}

impl<'py> FromPyObject<'_, 'py> for PyEventKind {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, PyErr> {
        if let Ok(s) = obj.extract::<String>() {
            PyEventKind::from_str(&s).map_err(|err| PyException::new_err(err.to_string()))
        } else if let Ok(t) = obj.cast::<PyEventKind>() {
            Ok(t.borrow().clone())
        } else {
            Err(PyException::new_err("Expected type `str` or `EventKind`"))
        }
    }
}
