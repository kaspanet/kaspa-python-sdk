use crate::address::PyAddress;
use crate::consensus::client::utxo::PyUtxoEntryReference;
use crate::crypto::hashes::PyHash;
use crate::wallet::core::utxo::balance::{PyBalance, PyBalanceStrings};
use crate::wallet::core::utxo::processor::PyUtxoProcessor;
use futures::stream::StreamExt;
use kaspa_addresses::Address;
use kaspa_hashes::Hash;
use kaspa_wallet_core::utxo::balance::BalanceStrings;
use kaspa_wallet_core::utxo::{UtxoContext, UtxoContextBinding, UtxoContextId, UtxoStream};
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use std::str::FromStr;

/// UTXO context for tracking addresses and balances.
#[gen_stub_pyclass]
#[pyclass(name = "UtxoContext")]
#[derive(Clone)]
pub struct PyUtxoContext(UtxoContext);

impl PyUtxoContext {
    pub fn inner(&self) -> &UtxoContext {
        &self.0
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyUtxoContext {
    /// Create a new UtxoContext.
    ///
    /// Args:
    ///     processor: The UtxoProcessor to bind to.
    ///     id: Optional 32-byte hex id (string) or Hash.
    #[new]
    #[pyo3(signature = (processor, id=None))]
    pub fn ctor(processor: PyUtxoProcessor, id: Option<Bound<'_, PyAny>>) -> PyResult<Self> {
        let binding = if let Some(value) = id {
            if let Ok(hash) = value.extract::<PyHash>() {
                UtxoContextBinding::Id(UtxoContextId::new(hash.into()))
            } else if let Ok(hex) = value.extract::<String>() {
                let hash =
                    Hash::from_str(&hex).map_err(|err| PyException::new_err(err.to_string()))?;
                UtxoContextBinding::Id(UtxoContextId::new(hash))
            } else {
                return Err(PyException::new_err(
                    "id must be a 32-byte hex string or Hash",
                ));
            }
        } else {
            UtxoContextBinding::default()
        };

        let inner = UtxoContext::new(processor.inner(), binding);
        Ok(Self(inner))
    }

    /// Track and scan a list of addresses (async).
    ///
    /// Args:
    ///     addresses: List of Address objects or address strings.
    ///     current_daa_score: Optional current DAA score for scan context.
    #[pyo3(signature = (addresses, current_daa_score=None))]
    fn track_addresses<'py>(
        &self,
        py: Python<'py>,
        #[gen_stub(override_type(type_repr = "Sequence[Address] | Sequence[str]"))]
        addresses: Bound<'_, PyAny>,
        current_daa_score: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let addresses = parse_addresses(addresses)?;
        let context = self.0.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            context
                .scan_and_register_addresses(addresses, current_daa_score)
                .await
                .map_err(|err| PyException::new_err(err.to_string()))?;
            Ok(())
        })
    }

    /// Unregister a list of addresses (async).
    fn unregister_addresses<'py>(
        &self,
        py: Python<'py>,
        #[gen_stub(override_type(type_repr = "Sequence[Address] | Sequence[str]"))]
        addresses: Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let addresses = parse_addresses(addresses)?;
        let context = self.0.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            context
                .unregister_addresses(addresses)
                .await
                .map_err(|err| PyException::new_err(err.to_string()))?;
            Ok(())
        })
    }

    /// Clear all tracked addresses and UTXOs (async).
    fn clear<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let context = self.0.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            context
                .clear()
                .await
                .map_err(|err| PyException::new_err(err.to_string()))?;
            Ok(())
        })
    }

    /// Whether the underlying processor is connected and running.
    #[getter]
    fn get_is_active(&self) -> bool {
        let processor = self.0.processor();
        processor
            .try_rpc_ctl()
            .map(|ctl| ctl.is_connected())
            .unwrap_or(false)
            && processor.is_connected()
            && processor.is_running()
    }

    /// Number of mature UTXO entries.
    #[getter]
    fn get_mature_length(&self) -> usize {
        self.0.mature_utxo_size()
    }

    /// Return a range of mature UTXO entries.
    fn mature_range(&self, mut from_: usize, mut to: usize) -> PyResult<Vec<PyUtxoEntryReference>> {
        let total = self.0.mature_utxo_size();
        if from_ > to {
            return Err(PyException::new_err("'from_' must be <= 'to'"));
        }
        if from_ > total {
            from_ = total;
        }
        if to > total {
            to = total;
        }
        if from_ == to {
            return Ok(vec![]);
        }
        let entries = futures::executor::block_on(
            UtxoStream::new(&self.0)
                .skip(from_)
                .take(to - from_)
                .collect::<Vec<_>>(),
        );
        Ok(entries
            .into_iter()
            .map(PyUtxoEntryReference::from)
            .collect())
    }

    /// Current balance for this context (if available).
    #[getter]
    fn get_balance(&self) -> Option<PyBalance> {
        self.0.balance().map(PyBalance::from)
    }

    /// Current balance formatted as strings (if available).
    #[getter]
    fn get_balance_strings(&self) -> PyResult<Option<PyBalanceStrings>> {
        let network_id = self.0.processor().network_id().ok();
        let balance = self.0.balance();
        if let (Some(network_id), Some(balance)) = (network_id, balance) {
            let balance_strings: BalanceStrings =
                balance.to_balance_strings(&network_id.network_type, None);
            Ok(Some(balance_strings.into()))
        } else {
            Ok(None)
        }
    }
}

impl From<PyUtxoContext> for UtxoContext {
    fn from(value: PyUtxoContext) -> Self {
        value.0
    }
}

fn parse_addresses(value: Bound<'_, PyAny>) -> PyResult<Vec<Address>> {
    value
        .try_iter()
        .map_err(|_| PyException::new_err("addresses must be an iterable of Address or str"))?
        .map(|item| {
            let item = item?;
            if let Ok(address) = item.extract::<PyAddress>() {
                Ok(Address::from(address))
            } else if let Ok(address) = item.extract::<String>() {
                PyAddress::try_from(address).map(Address::from)
            } else {
                Err(PyException::new_err(
                    "addresses must be an iterable of Address or str",
                ))
            }
        })
        .collect()
}
