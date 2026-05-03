use kaspa_wallet_core::storage::TransactionKind;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;
use std::str::FromStr;

crate::wrap_unit_enum_for_py!(
    /// Transaction Kind
    PyTransactionKind, "TransactionKind", TransactionKind, {
        Reorg,
        Stasis,
        Batch,
        Change,
        Incoming,
        Outgoing,
        External,
        TransferIncoming,
        TransferOutgoing
    }
);

impl FromStr for PyTransactionKind {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = match s.to_lowercase().as_str() {
            "reorg" => PyTransactionKind::Reorg,
            "stasis" => PyTransactionKind::Stasis,
            "batch" => PyTransactionKind::Batch,
            "change" => PyTransactionKind::Change,
            "incoming" => PyTransactionKind::Incoming,
            "outgoing" => PyTransactionKind::Outgoing,
            "external" => PyTransactionKind::External,
            "transferincoming" | "transfer_incoming" => PyTransactionKind::TransferIncoming,
            "transferoutgoing" | "transfer_outgoing" => PyTransactionKind::TransferOutgoing,
            _ => Err(PyException::new_err(
                "Unsupported string value for `TransactionKind`",
            ))?,
        };

        Ok(v)
    }
}

impl<'py> FromPyObject<'_, 'py> for PyTransactionKind {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(s) = obj.extract::<String>() {
            PyTransactionKind::from_str(&s).map_err(|err| PyException::new_err(err.to_string()))
        } else if let Ok(t) = obj.cast::<PyTransactionKind>() {
            Ok(t.borrow().clone())
        } else {
            Err(PyException::new_err(
                "Expected type `str` or `TransactionKind`",
            ))
        }
    }
}
