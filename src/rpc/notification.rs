use kaspa_rpc_core::api::notifications::Notification;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::gen_stub_pyclass;
use serde_pyobject::to_pyobject;

/// RPC notification wrapper for event callbacks.
///
/// This type is not intended to be instantiated directly from Python.
/// It wraps notifications received from the Kaspa node when subscribed
/// to events via RpcClient. The notification data is automatically
/// converted to a Python dict when passed to callback handlers.
///
/// Notification types:
///     - BlockAdded: A new block was added to the DAG.
///     - FinalityConflict: A finality conflict was detected.
///     - FinalityConflictResolved: A finality conflict was resolved.
///     - NewBlockTemplate: A new block template is available.
///     - PruningPointUtxoSetOverride: The pruning point UTXO set changed.
///     - UtxosChanged: UTXOs in the subscribed addresses changed.
///     - VirtualDaaScoreChanged: The virtual DAA score changed.
///     - SinkBlueScoreChanged: The sink blue score changed.
///     - VirtualChainChanged: The virtual chain changed.
#[gen_stub_pyclass]
#[pyclass(name = "Notification")]
pub struct PyNotification(pub Notification);

impl PyNotification {
    pub fn to_pyobject(&self, py: Python) -> PyResult<Py<PyAny>> {
        let bound_obj = match &self.0 {
            Notification::BlockAdded(v) => to_pyobject(py, &v),
            Notification::FinalityConflict(v) => to_pyobject(py, &v),
            Notification::FinalityConflictResolved(v) => to_pyobject(py, &v),
            Notification::NewBlockTemplate(v) => to_pyobject(py, &v),
            Notification::PruningPointUtxoSetOverride(v) => to_pyobject(py, &v),
            Notification::UtxosChanged(v) => to_pyobject(py, &v),
            Notification::VirtualDaaScoreChanged(v) => to_pyobject(py, &v),
            Notification::SinkBlueScoreChanged(v) => to_pyobject(py, &v),
            Notification::VirtualChainChanged(v) => to_pyobject(py, &v),
        }?;

        Ok(bound_obj.unbind())
    }
}

impl From<Notification> for PyNotification {
    fn from(value: Notification) -> Self {
        PyNotification(value)
    }
}
