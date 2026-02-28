use crate::callback::PyCallback;
use crate::consensus::core::network::PyNetworkId;
use crate::rpc::wrpc::client::PyRpcClient;
use ahash::AHashMap;
use futures::*;
use kaspa_wallet_core::events::EventKind;
use kaspa_wallet_core::rpc::{DynRpcApi, Rpc};
use kaspa_wallet_core::utxo::{
    UtxoProcessor, set_coinbase_transaction_maturity_period_daa,
    set_user_transaction_maturity_period_daa,
};
use pyo3::{
    exceptions::PyException,
    prelude::*,
    types::{PyDict, PyTuple},
};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyclass_enum, gen_stub_pymethods};
use serde::{Deserialize, Serialize};
use std::{
    str::FromStr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use workflow_core::channel::DuplexChannel;
use workflow_log::*;

/// Event types for `UtxoProcessor` listeners.
#[gen_stub_pyclass_enum]
#[pyclass(name = "UtxoProcessorEvent", skip_from_py_object, eq)]
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PyUtxoProcessorEvent {
    All,
    Connect,
    Disconnect,
    UtxoIndexNotEnabled,
    SyncState,
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
    Error,
}

impl<'py> FromPyObject<'_, 'py> for PyUtxoProcessorEvent {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
        if let Ok(s) = obj.extract::<String>() {
            serde_json::from_value::<PyUtxoProcessorEvent>(serde_json::Value::String(s))
                .map_err(|err| PyException::new_err(err.to_string()))
        } else if let Ok(t) = obj.cast::<PyUtxoProcessorEvent>() {
            Ok(t.borrow().clone())
        } else {
            Err(PyException::new_err(
                "Expected type `str` or `UtxoProcessorEvent`",
            ))
        }
    }
}

impl From<PyUtxoProcessorEvent> for EventKind {
    fn from(value: PyUtxoProcessorEvent) -> Self {
        match value {
            PyUtxoProcessorEvent::All => EventKind::All,
            PyUtxoProcessorEvent::Connect => EventKind::Connect,
            PyUtxoProcessorEvent::Disconnect => EventKind::Disconnect,
            PyUtxoProcessorEvent::UtxoIndexNotEnabled => EventKind::UtxoIndexNotEnabled,
            PyUtxoProcessorEvent::SyncState => EventKind::SyncState,
            PyUtxoProcessorEvent::ServerStatus => EventKind::ServerStatus,
            PyUtxoProcessorEvent::UtxoProcStart => EventKind::UtxoProcStart,
            PyUtxoProcessorEvent::UtxoProcStop => EventKind::UtxoProcStop,
            PyUtxoProcessorEvent::UtxoProcError => EventKind::UtxoProcError,
            PyUtxoProcessorEvent::DaaScoreChange => EventKind::DaaScoreChange,
            PyUtxoProcessorEvent::Pending => EventKind::Pending,
            PyUtxoProcessorEvent::Reorg => EventKind::Reorg,
            PyUtxoProcessorEvent::Stasis => EventKind::Stasis,
            PyUtxoProcessorEvent::Maturity => EventKind::Maturity,
            PyUtxoProcessorEvent::Discovery => EventKind::Discovery,
            PyUtxoProcessorEvent::Balance => EventKind::Balance,
            PyUtxoProcessorEvent::Error => EventKind::Error,
        }
    }
}

/// UTXO processor coordinating address tracking and UTXO updates.
#[gen_stub_pyclass]
#[pyclass(name = "UtxoProcessor")]
#[derive(Clone)]
pub struct PyUtxoProcessor {
    processor: UtxoProcessor,
    rpc: PyRpcClient,
    callbacks: Arc<Mutex<AHashMap<EventKind, Vec<PyCallback>>>>,
    notification_task: Arc<AtomicBool>,
    notification_ctl: DuplexChannel,
}

impl PyUtxoProcessor {
    pub fn inner(&self) -> &UtxoProcessor {
        &self.processor
    }

    fn normalize_event_payload(
        py: Python,
        event_type: EventKind,
        event: &Bound<PyDict>,
    ) -> PyResult<()> {
        // WASM side uses `to_js_value()` which always emits `data` for some events
        // (e.g. TransactionRecordNotification), but unit variants may omit it.
        if event.get_item("data")?.is_none() {
            event.set_item("data", py.None())?;
            return Ok(());
        }

        // Align to WASM `Events::to_js_value()` which flattens transaction record events
        // to `{ type, data: TransactionRecord }` (not `{ type, data: { record } }`).
        match event_type {
            EventKind::Pending
            | EventKind::Reorg
            | EventKind::Stasis
            | EventKind::Maturity
            | EventKind::Discovery => {
                if let Some(data_any) = event.get_item("data")?
                    && let Ok(data_dict) = data_any.cast::<PyDict>()
                    && let Some(record) = data_dict.get_item("record")?
                {
                    event.set_item("data", record)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn notification_callbacks(&self, event: EventKind) -> Option<Vec<PyCallback>> {
        let notification_callbacks = self.callbacks.lock().unwrap();
        let all = notification_callbacks.get(&EventKind::All).cloned();
        let target = notification_callbacks.get(&event).cloned();
        match (all, target) {
            (Some(mut vec_all), Some(vec_target)) => {
                vec_all.extend(vec_target);
                Some(vec_all)
            }
            (Some(vec_all), None) => Some(vec_all),
            (None, Some(vec_target)) => Some(vec_target),
            (None, None) => None,
        }
    }

    fn start_notification_task(&self, py: Python) -> PyResult<bool> {
        if self
            .notification_task
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Ok(false);
        }

        let ctl_receiver = self.notification_ctl.request.receiver.clone();
        let ctl_sender = self.notification_ctl.response.sender.clone();
        let channel = self.processor.multiplexer().channel();
        let this = self.clone();

        let fut = async move {
            let mut shutdown_requested = false;
            loop {
                if shutdown_requested && channel.receiver.is_empty() {
                    break;
                }

                select_biased! {
                    _ = ctl_receiver.recv().fuse() => {
                        shutdown_requested = true;
                    }
                    msg = channel.receiver.recv().fuse() => {
                        match msg {
                            Ok(notification) => {
                                let event_type = EventKind::from(notification.as_ref());
                                if let Some(handlers) = this.notification_callbacks(event_type) {
                                    for handler in handlers.into_iter() {
                                        if let Err(err) = Python::attach(|py| -> PyResult<()> {
                                            let event_any = match serde_pyobject::to_pyobject(py, notification.as_ref()) {
                                                Ok(obj) => obj,
                                                Err(err) => {
                                                    log_error!("UtxoProcessor: failed to serialize event `{}`: {}", event_type, err);
                                                    return Ok(());
                                                }
                                            };

                                            let event = match event_any.cast::<PyDict>() {
                                                Ok(dict) => dict,
                                                Err(err) => {
                                                    log_error!(
                                                        "UtxoProcessor: serialized event `{}` is not a dict: {}",
                                                        event_type,
                                                        err
                                                    );
                                                    return Ok(());
                                                }
                                            };

                                            if let Err(err) = Self::normalize_event_payload(py, event_type, event) {
                                                log_error!(
                                                    "UtxoProcessor: failed to normalize event payload for `{}`: {}",
                                                    event_type,
                                                    err
                                                );
                                            }

                                            if let Err(err) = handler.execute(py, (*event).clone()) {
                                                log_error!(
                                                    "UtxoProcessor: error while executing event listener for `{}`: {}",
                                                    event_type,
                                                    err
                                                );
                                            }

                                            Ok(())
                                        }) {
                                            log_error!(
                                                "UtxoProcessor: error while building event payload for `{}`: {}",
                                                event_type,
                                                err
                                            );
                                        }
                                    }
                                }
                            }
                            Err(err) => {
                                log_error!("UtxoProcessor: error while receiving multiplexer event: {err}");
                                break;
                            }
                        }
                    }
                }
            }

            channel.close();
            this.notification_task.store(false, Ordering::SeqCst);
            ctl_sender.send(()).await.ok();
            Python::attach(|_| Ok(()))
        };

        if let Err(err) = pyo3_async_runtimes::tokio::future_into_py(py, fut) {
            self.notification_task.store(false, Ordering::SeqCst);
            return Err(err);
        }

        Ok(true)
    }

    async fn stop_notification_task(
        &self,
    ) -> std::result::Result<(), workflow_core::channel::ChannelError<()>> {
        if self.notification_task.load(Ordering::SeqCst) {
            self.notification_ctl.signal(()).await?;
            self.notification_task.store(false, Ordering::SeqCst);
        }
        Ok(())
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

        Ok(Self {
            processor,
            rpc,
            callbacks: Arc::new(Mutex::new(Default::default())),
            notification_task: Arc::new(AtomicBool::new(false)),
            notification_ctl: DuplexChannel::oneshot(),
        })
    }

    /// Start UTXO processing (async).
    #[gen_stub(override_return_type(type_repr = "None"))]
    fn start<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let processor = self.processor.clone();
        let slf = self.clone();
        let notification_task_started = self.start_notification_task(py)?;
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            if let Err(err) = processor.start().await {
                if notification_task_started {
                    slf.stop_notification_task().await.ok();
                }
                return Err(PyException::new_err(err.to_string()));
            }
            Ok(())
        })
    }

    /// Stop UTXO processing (async).
    #[gen_stub(override_return_type(type_repr = "None"))]
    fn stop<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let processor = self.processor.clone();
        let slf = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let stop_result = processor.stop().await;
            let notification_stop_result = slf.stop_notification_task().await;

            if let Err(err) = stop_result {
                return Err(PyException::new_err(err.to_string()));
            }

            notification_stop_result.map_err(|err| PyException::new_err(err.to_string()))?;
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

    /// Register a callback for UtxoProcessor events.
    ///
    /// Args:
    ///     event_or_callback: Event target as string (kebab-case), `UtxoProcessorEvent` variant, a list of those, "*" / "all", or a callback (listen to all events).
    ///     callback: Function to call when event occurs (required when event_or_callback is an event target).
    ///     *args: Additional arguments to pass to callback.
    ///     **kwargs: Additional keyword arguments to pass to callback.
    ///
    /// Returns:
    ///     None
    ///
    /// Notes:
    ///     Callback will be invoked as: callback(*args, event, **kwargs)
    ///     Where event is a dict like: {"type": str, "data": ...}
    #[pyo3(signature = (event_or_callback, callback=None, *args, **kwargs))]
    fn add_event_listener(
        &self,
        py: Python,
        event_or_callback: Bound<'_, PyAny>,
        callback: Option<Py<PyAny>>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        let (targets, callback) = match callback {
            Some(callback) => (parse_event_targets(event_or_callback)?, callback),
            None => {
                if event_or_callback.is_callable() {
                    (
                        vec![EventKind::All],
                        event_or_callback.extract::<Py<PyAny>>()?,
                    )
                } else {
                    return Err(PyException::new_err(
                        "Expected `str | UtxoProcessorEvent | Sequence[str | UtxoProcessorEvent]` for event_or_callback and `callback` to be callable",
                    ));
                }
            }
        };

        let args = args.into_pyobject(py)?.extract::<Py<PyTuple>>()?;
        let kwargs = match kwargs {
            Some(kw) => kw.into_pyobject(py)?.extract::<Py<PyDict>>()?,
            None => PyDict::new(py).into(),
        };

        let py_callback = PyCallback::new(callback, args, kwargs);

        let mut callbacks = self.callbacks.lock().unwrap();
        for target in targets {
            callbacks
                .entry(target)
                .or_default()
                .push(py_callback.clone());
        }
        Ok(())
    }

    /// Remove an event listener.
    ///
    /// Args:
    ///     event_or_callback: Event target as string (kebab-case), `UtxoProcessorEvent` variant, a list of those, "*" / "all", or a callback (remove from all events).
    ///     callback: Specific callback to remove, or None to remove all callbacks for the event target(s).
    ///
    /// Returns:
    ///     None
    #[pyo3(signature = (event_or_callback, callback=None))]
    fn remove_event_listener(
        &self,
        event_or_callback: Bound<'_, PyAny>,
        callback: Option<Py<PyAny>>,
    ) -> PyResult<()> {
        let mut callbacks = self.callbacks.lock().unwrap();

        if callback.is_none() && event_or_callback.is_callable() {
            let callback = event_or_callback.extract::<Py<PyAny>>()?;
            for handlers in callbacks.values_mut() {
                handlers.retain(|entry| !entry.callback_ptr_eq(&callback));
            }
            return Ok(());
        }

        let targets = parse_event_targets(event_or_callback)?;

        match callback {
            Some(callback) => {
                for target in targets {
                    if let Some(handlers) = callbacks.get_mut(&target) {
                        handlers.retain(|entry| !entry.callback_ptr_eq(&callback));
                    }
                }
            }
            None => {
                for target in targets {
                    callbacks.remove(&target);
                }
            }
        }

        Ok(())
    }

    /// Remove all registered event listeners.
    ///
    /// Returns:
    ///     None
    fn remove_all_event_listeners(&self) -> PyResult<()> {
        self.callbacks.lock().unwrap().clear();
        Ok(())
    }
}

fn parse_event_targets(value: Bound<'_, PyAny>) -> PyResult<Vec<EventKind>> {
    // Strings are iterable in Python. Ensure string-like targets are validated
    // as a single target first, so invalid values like "" do not silently no-op.
    if value.extract::<String>().is_ok() || value.cast::<PyUtxoProcessorEvent>().is_ok() {
        return parse_event_target_item(&value).map(|event| vec![event]);
    }

    let iter = value.try_iter().map_err(|_| {
        PyException::new_err("event target must be str, UtxoProcessorEvent, or a sequence of those")
    })?;

    iter.map(|item| {
        let item = item?;
        parse_event_target_item(&item).map_err(|_| {
            PyException::new_err(
                "event target must be str, UtxoProcessorEvent, or a sequence of those",
            )
        })
    })
    .collect()
}

fn parse_event_target_item(value: &Bound<'_, PyAny>) -> PyResult<EventKind> {
    if let Ok(event) = value.extract::<PyUtxoProcessorEvent>() {
        return Ok(event.into());
    }

    if let Ok(s) = value.extract::<String>() {
        return parse_event_kind(&s);
    }

    Err(PyException::new_err(
        "event target must be str, UtxoProcessorEvent, or a sequence of those",
    ))
}

fn parse_event_kind(s: &str) -> PyResult<EventKind> {
    if s == "all" {
        return Ok(EventKind::All);
    }
    EventKind::from_str(s).map_err(|err| PyException::new_err(err.to_string()))
}
