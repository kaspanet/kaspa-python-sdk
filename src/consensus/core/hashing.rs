use kaspa_consensus_client as cctx;
use kaspa_consensus_core::hashing::covenant_id::covenant_id;
use kaspa_consensus_core::hashing::wasm::SighashType;
use kaspa_consensus_core::tx as ctx;
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::gen_stub_pyclass_enum;
use std::str::FromStr;

use crate::{
    consensus::client::{outpoint::PyTransactionOutpoint, output::PyTransactionOutput},
    crypto::hashes::PyHash,
};

crate::wrap_unit_enum_for_py!(
    /// Kaspa signature hash types for transaction signing.
    PySighashType, "SighashType", SighashType, {
    All,
    None,
    Single,
    AllAnyOneCanPay,
    NoneAnyOneCanPay,
    SingleAnyOneCanPay,
});

impl FromStr for PySighashType {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all" => Ok(PySighashType::All),
            "none" => Ok(PySighashType::None),
            "single" => Ok(PySighashType::Single),
            "allanyonecanpay" => Ok(PySighashType::AllAnyOneCanPay),
            "noneanyonecanpay" => Ok(PySighashType::NoneAnyOneCanPay),
            "singleanyonecanpay" => Ok(PySighashType::SingleAnyOneCanPay),
            _ => Err(PyException::new_err(
                "Unsupported string value for SighashType",
            )),
        }
    }
}

impl<'py> FromPyObject<'_, 'py> for PySighashType {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(s) = obj.extract::<String>() {
            PySighashType::from_str(&s).map_err(|err| PyException::new_err(err.to_string()))
        } else if let Ok(t) = obj.cast::<PySighashType>() {
            Ok(t.borrow().clone())
        } else {
            Err(PyException::new_err("Expected type `str` or `SighashType`"))
        }
    }
}

#[pyfunction]
#[pyo3(name = "covenant_id")]
pub fn py_covenant_id(
    outpoint: PyTransactionOutpoint,
    auth_outputs: Vec<PyTransactionOutput>,
) -> PyHash {
    let outpoint: cctx::TransactionOutpoint = outpoint.into();
    let auth_outputs = auth_outputs
        .into_iter()
        .map(|py_output| ctx::TransactionOutput::from(&cctx::TransactionOutput::from(py_output)))
        .collect::<Vec<ctx::TransactionOutput>>();
    let indexed: Vec<(u32, &ctx::TransactionOutput)> = auth_outputs
        .iter()
        .enumerate()
        .map(|(i, o)| (i as u32, o))
        .collect();
    covenant_id(outpoint.into(), indexed.into_iter()).into()
}
