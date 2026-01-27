use crate::consensus::core::network::PyNetworkId;
use crate::rpc::wrpc::client::PyRpcClient;
use kaspa_wallet_core::rpc::{DynRpcApi, Rpc};
use kaspa_wallet_core::utxo::{
    UtxoProcessor, set_coinbase_transaction_maturity_period_daa,
    set_user_transaction_maturity_period_daa,
};
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use std::sync::Arc;

/// UTXO processor coordinating address tracking and UTXO updates.
#[gen_stub_pyclass]
#[pyclass(name = "UtxoProcessor")]
#[derive(Clone)]
pub struct PyUtxoProcessor {
    processor: UtxoProcessor,
    rpc: PyRpcClient,
}

impl PyUtxoProcessor {
    pub fn inner(&self) -> &UtxoProcessor {
        &self.processor
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyUtxoProcessor {
    /// Create a new UtxoProcessor.
    ///
    /// Args:
    ///     rpc: The RPC client to use for network communication.
    ///     network_id: Network identifier for UTXO processing.
    #[new]
    pub fn ctor(rpc: PyRpcClient, network_id: PyNetworkId) -> PyResult<Self> {
        let rpc_api: Arc<DynRpcApi> = rpc.client().clone();
        let rpc_ctl = rpc.client().rpc_ctl().clone();
        let rpc_binding = Rpc::new(rpc_api, rpc_ctl);

        let processor = UtxoProcessor::new(Some(rpc_binding), Some(network_id.into()), None, None);

        Ok(Self { processor, rpc })
    }

    /// Start UTXO processing (async).
    fn start<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let processor = self.processor.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            processor
                .start()
                .await
                .map_err(|err| PyException::new_err(err.to_string()))?;
            Ok(())
        })
    }

    /// Stop UTXO processing (async).
    fn stop<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let processor = self.processor.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            processor
                .stop()
                .await
                .map_err(|err| PyException::new_err(err.to_string()))?;
            Ok(())
        })
    }

    /// The associated RPC client.
    #[getter]
    pub fn get_rpc(&self) -> PyRpcClient {
        self.rpc.clone()
    }

    /// The network id used by the processor (if set).
    #[getter]
    pub fn get_network_id(&self) -> Option<PyNetworkId> {
        self.processor.network_id().ok().map(PyNetworkId::from)
    }

    /// Set the network id for the processor.
    pub fn set_network_id(&self, network_id: PyNetworkId) {
        self.processor.set_network_id(&network_id.into());
    }

    /// Set the coinbase transaction maturity period DAA for a network.
    #[staticmethod]
    pub fn set_coinbase_transaction_maturity_daa(network_id: PyNetworkId, value: u64) {
        let network_id = network_id.into();
        set_coinbase_transaction_maturity_period_daa(&network_id, value);
    }

    /// Set the user transaction maturity period DAA for a network.
    #[staticmethod]
    pub fn set_user_transaction_maturity_daa(network_id: PyNetworkId, value: u64) {
        let network_id = network_id.into();
        set_user_transaction_maturity_period_daa(&network_id, value);
    }

    /// Whether the processor is connected and running.
    #[getter]
    pub fn get_is_active(&self) -> bool {
        self.processor
            .try_rpc_ctl()
            .map(|ctl| ctl.is_connected())
            .unwrap_or(false)
            && self.processor.is_connected()
            && self.processor.is_running()
    }
}
