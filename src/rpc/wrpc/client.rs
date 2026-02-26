use crate::address::PyAddress;
use crate::callback::PyCallback;
use crate::consensus::core::network::{PyNetworkId, PyNetworkType};
use crate::rpc::encoding::PyEncoding;
use crate::rpc::model::*;
use crate::rpc::notification::PyNotification;
use crate::rpc::wrpc::resolver::PyResolver;
use ahash::AHashMap;
use futures::*;
use kaspa_notify::listener::ListenerId;
use kaspa_notify::notification::Notification;
use kaspa_notify::scope::{
    BlockAddedScope, FinalityConflictResolvedScope, FinalityConflictScope, NewBlockTemplateScope,
    PruningPointUtxoSetOverrideScope, Scope, SinkBlueScoreChangedScope, UtxosChangedScope,
    VirtualChainChangedScope, VirtualDaaScoreChangedScope,
};
use kaspa_notify::{connection::ChannelType, events::EventType};
use kaspa_rpc_core::api::rpc::RpcApi;
use kaspa_rpc_core::model::*;
use kaspa_rpc_core::notify::connection::ChannelConnection;
use kaspa_wrpc_client::{
    KaspaRpcClient, client::ConnectOptions, error::Error, prelude::*, result::Result,
};
use paste::paste;
use pyo3::{
    exceptions::PyException,
    prelude::*,
    types::{PyDict, PyTuple},
};
use pyo3_stub_gen::derive::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use workflow_core::channel::{Channel, DuplexChannel};
use workflow_log::*;
use workflow_rpc::{client::Ctl, encoding::Encoding};

/// Notification event types for RPC client subscriptions.
///
/// Use with `RpcClient.subscribe()` and `RpcClient.unsubscribe()` to manage
/// event subscriptions for real-time updates from a Kaspa node.
///
/// Variants:
///     - All: Subscribe to all available notification events at once.
///     - BlockAdded: Triggered when a new block is added to the DAG.
///     - VirtualChainChanged: Triggered when the virtual (selected parent) chain changes.
///     - FinalityConflict: Triggered when a finality conflict is detected.
///     - FinalityConflictResolved: Triggered when a finality conflict is resolved.
///     - UtxosChanged: Triggered when UTXOs for subscribed addresses change.
///     - SinkBlueScoreChanged: Triggered when the sink block's blue score changes.
///     - VirtualDaaScoreChanged: Triggered when the virtual DAA score changes.
///     - PruningPointUtxoSetOverride: Triggered when the pruning point UTXO set is overridden.
///     - NewBlockTemplate: Triggered when a new block template is available for mining.
///     - Connect: Triggered when the RPC client connects to a node.
///     - Disconnect: Triggered when the RPC client disconnects from a node.
#[gen_stub_pyclass_enum]
#[pyclass(name = "NotificationEvent", skip_from_py_object, eq)]
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PyNotificationEvent {
    All,

    // Event Types
    BlockAdded,
    VirtualChainChanged,
    FinalityConflict,
    FinalityConflictResolved,
    UtxosChanged,
    SinkBlueScoreChanged,
    VirtualDaaScoreChanged,
    PruningPointUtxoSetOverride,
    NewBlockTemplate,

    // RPC Control
    Connect,
    Disconnect,
}

impl<'py> FromPyObject<'_, 'py> for PyNotificationEvent {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
        if let Ok(s) = obj.extract::<String>() {
            serde_json::from_value::<PyNotificationEvent>(serde_json::Value::String(s))
                .map_err(|err| PyException::new_err(err.to_string()))
        } else if let Ok(t) = obj.cast::<PyNotificationEvent>() {
            Ok(t.borrow().clone())
        } else {
            Err(PyException::new_err(
                "Expected type `str` or `NotificationEvent`",
            ))
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum NotificationEvent {
    All,
    Notification(EventType),
    RpcCtl(Ctl),
}

impl FromStr for NotificationEvent {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        if s == "all" {
            Ok(NotificationEvent::All)
        } else if let Ok(ctl) = Ctl::from_str(s) {
            Ok(NotificationEvent::RpcCtl(ctl))
        } else if let Ok(event) = EventType::from_str(s) {
            Ok(NotificationEvent::Notification(event))
        } else {
            Err(Error::custom(format!(
                "Invalid notification event type: `{}`",
                s
            )))
        }
    }
}

impl From<PyNotificationEvent> for NotificationEvent {
    fn from(value: PyNotificationEvent) -> Self {
        match value {
            PyNotificationEvent::All => NotificationEvent::All,

            // Event Types
            PyNotificationEvent::BlockAdded => {
                NotificationEvent::Notification(EventType::BlockAdded)
            }
            PyNotificationEvent::VirtualChainChanged => {
                NotificationEvent::Notification(EventType::VirtualChainChanged)
            }
            PyNotificationEvent::FinalityConflict => {
                NotificationEvent::Notification(EventType::FinalityConflict)
            }
            PyNotificationEvent::FinalityConflictResolved => {
                NotificationEvent::Notification(EventType::FinalityConflictResolved)
            }
            PyNotificationEvent::UtxosChanged => {
                NotificationEvent::Notification(EventType::UtxosChanged)
            }
            PyNotificationEvent::SinkBlueScoreChanged => {
                NotificationEvent::Notification(EventType::SinkBlueScoreChanged)
            }
            PyNotificationEvent::VirtualDaaScoreChanged => {
                NotificationEvent::Notification(EventType::VirtualDaaScoreChanged)
            }
            PyNotificationEvent::PruningPointUtxoSetOverride => {
                NotificationEvent::Notification(EventType::PruningPointUtxoSetOverride)
            }
            PyNotificationEvent::NewBlockTemplate => {
                NotificationEvent::Notification(EventType::NewBlockTemplate)
            }

            // RPC Control
            PyNotificationEvent::Connect => NotificationEvent::RpcCtl(Ctl::Connect),
            PyNotificationEvent::Disconnect => NotificationEvent::RpcCtl(Ctl::Disconnect),
        }
    }
}

pub struct Inner {
    client: Arc<KaspaRpcClient>,
    resolver: Option<Resolver>,
    notification_task: Arc<AtomicBool>,
    notification_ctl: DuplexChannel,
    callbacks: Arc<Mutex<AHashMap<NotificationEvent, Vec<PyCallback>>>>,
    listener_id: Arc<Mutex<Option<ListenerId>>>,
    notification_channel: Channel<kaspa_rpc_core::Notification>,
}

impl Inner {
    fn notification_callbacks(&self, event: NotificationEvent) -> Option<Vec<PyCallback>> {
        let notification_callbacks = self.callbacks.lock().unwrap();
        let all = notification_callbacks.get(&NotificationEvent::All).cloned();
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
}

/// WebSocket RPC client for communicating with Kaspa nodes.
///
/// Provides methods for querying blockchain state, submitting transactions,
/// and subscribing to real-time notifications. Supports both Borsh and JSON
/// encodings.
#[gen_stub_pyclass]
#[pyclass(name = "RpcClient")]
#[derive(Clone)]
pub struct PyRpcClient(Arc<Inner>);

impl PyRpcClient {
    pub fn new(
        resolver: Option<Resolver>,
        url: Option<String>,
        encoding: Option<PyEncoding>,
        network_id: Option<NetworkId>,
    ) -> PyResult<Self> {
        let encoding = encoding.unwrap_or(PyEncoding::Borsh);
        let url = url
            .map(|url| {
                if let Some(network_id) = network_id {
                    Self::parse_url(&url, encoding.clone().into(), network_id)
                } else {
                    Ok(url.to_string())
                }
            })
            .transpose()?;

        let client = Arc::new(
            KaspaRpcClient::new(
                encoding.into(),
                url.as_deref(),
                resolver.clone(),
                network_id,
                None,
            )
            .map_err(|err| PyException::new_err(err.to_string()))?,
        );

        let rpc_client = PyRpcClient(Arc::new(Inner {
            client,
            resolver,
            notification_task: Arc::new(AtomicBool::new(false)),
            notification_ctl: DuplexChannel::oneshot(),
            callbacks: Arc::new(Default::default()),
            listener_id: Arc::new(Mutex::new(None)),
            notification_channel: Channel::unbounded(),
        }));

        Ok(rpc_client)
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyRpcClient {
    /// Create a new RPC client.
    ///
    /// Args:
    ///     resolver: Optional resolver for node discovery.
    ///     url: Optional direct node URL.
    ///     encoding: RPC encoding - either a string ("borsh" or "json") or an Encoding enum variant (default: "borsh").
    ///     network_id: Network identifier (default: "mainnet").
    ///
    /// Returns:
    ///     RpcClient: A new RpcClient instance.
    ///
    /// Raises:
    ///     Exception: If client creation fails.
    #[new]
    #[pyo3(signature = (resolver=None, url=None, encoding=None, network_id=None))]
    fn ctor(
        resolver: Option<PyResolver>,
        url: Option<String>,
        #[gen_stub(override_type(type_repr = "str | Encoding | None = Encoding.Borsh"))]
        encoding: Option<PyEncoding>,
        network_id: Option<PyNetworkId>,
    ) -> PyResult<PyRpcClient> {
        let network_id = match network_id {
            Some(id) => id,
            None => PyNetworkId::from_str("mainnet")?,
        };

        Self::new(
            resolver.map(|r| r.inner()),
            url,
            Some(encoding.unwrap_or(PyEncoding::Borsh)),
            Some(network_id.into()),
        )
    }

    /// The current WebSocket connection URL, or None if not connected.
    #[getter]
    fn get_url(&self) -> Option<String> {
        self.0.client.url()
    }

    /// The resolver used for node discovery, or None if not set.
    #[getter]
    fn get_resolver(&self) -> Option<PyResolver> {
        self.0.resolver.clone().map(PyResolver::new)
    }

    /// Set a new resolver for node discovery.
    ///
    /// Args:
    ///     resolver: The resolver to use.
    ///
    /// Raises:
    ///     Exception: If setting the resolver fails.
    fn set_resolver(&self, resolver: PyResolver) -> PyResult<()> {
        self.0
            .client
            .set_resolver(resolver.into())
            .map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(())
    }

    /// Set the network ID for the client.
    ///
    /// Args:
    ///     network_id: The network identifier.
    ///
    /// Raises:
    ///     Exception: If setting the network ID fails.
    fn set_network_id(&self, network_id: PyNetworkId) -> PyResult<()> {
        self.0
            .client
            .set_network_id(&network_id.into())
            .map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(())
    }

    /// Whether the client is currently connected to a node.
    #[getter]
    fn get_is_connected(&self) -> bool {
        self.0.client.is_connected()
    }

    /// The RPC encoding format ("borsh" or "json").
    #[getter]
    fn get_encoding(&self) -> String {
        self.0.client.encoding().to_string()
    }

    /// The unique identifier of the connected node, or None if not connected via resolver.
    #[getter]
    fn get_node_id(&self) -> Option<String> {
        self.0.client.node_descriptor().map(|node| node.uid.clone())
    }

    /// Connect to a Kaspa node (async).
    ///
    /// Args:
    ///     block_async_connect: Block until connected (default: True).
    ///     strategy: Connection strategy ("retry" or "fallback", default: "retry").
    ///     url: Optional URL to connect to (overrides resolver).
    ///     timeout_duration: Connection timeout in milliseconds.
    ///     retry_interval: Retry interval in milliseconds.
    ///
    /// Raises:
    ///     Exception: If connection fails.
    #[pyo3(signature = (block_async_connect=None, strategy=None, url=None, timeout_duration=None, retry_interval=None))]
    #[gen_stub(override_return_type(type_repr = "None"))]
    pub fn connect<'py>(
        &self,
        py: Python<'py>,
        block_async_connect: Option<bool>,
        strategy: Option<String>,
        url: Option<String>,
        timeout_duration: Option<u64>,
        retry_interval: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let block_async_connect = block_async_connect.unwrap_or(true);
        let strategy = match strategy {
            Some(strategy) => ConnectStrategy::from_str(&strategy)
                .map_err(|err| PyException::new_err(format!("{}", err)))
                .map_err(|err| PyException::new_err(err.to_string()))?,
            None => ConnectStrategy::Retry,
        };
        let connect_timeout: Option<Duration> = timeout_duration.map(Duration::from_millis);
        let retry_interval: Option<Duration> = retry_interval.map(Duration::from_millis);

        let options = ConnectOptions {
            block_async_connect,
            strategy,
            url,
            connect_timeout,
            retry_interval,
        };

        self.start_notification_task(py)
            .map_err(|err| PyException::new_err(err.to_string()))?;

        let client = self.0.client.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .connect(Some(options))
                .await
                .map_err(|e| PyException::new_err(e.to_string()))?;
            Ok(())
        })
    }

    /// Disconnect from the node (async).
    ///
    /// Raises:
    ///     Exception: If disconnection fails.
    #[gen_stub(override_return_type(type_repr = "None"))]
    fn disconnect<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .0
                .client
                .disconnect()
                .await
                .map_err(|err| PyException::new_err(err.to_string()))?;
            client
                .stop_notification_task()
                .await
                .map_err(|err| PyException::new_err(err.to_string()))?;
            Ok(())
        })
    }

    /// Start the RPC client (async).
    ///
    /// Raises:
    ///     Exception: If starting fails.
    #[gen_stub(override_return_type(type_repr = "None"))]
    fn start<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.start_notification_task(py)
            .map_err(|err| PyException::new_err(err.to_string()))?;
        let inner = self.0.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner
                .client
                .start()
                .await
                .map_err(|err| PyException::new_err(err.to_string()))?;
            Ok(())
        })
    }

    /// Stop background RPC services (automatically stopped when invoking RpcClient.disconnect).
    #[gen_stub(override_return_type(type_repr = "None"))]
    fn stop<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let slf = self.clone();
        let inner = self.0.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner
                .client
                .stop()
                .await
                .map_err(|err| PyException::new_err(err.to_string()))?;
            slf.stop_notification_task()
                .await
                .map_err(|err| PyException::new_err(err.to_string()))?;
            Ok(())
        })
    }

    /// Triggers a disconnection on the underlying WebSocket
    /// if the WebSocket is in connected state.
    /// This is intended for debug purposes only.
    /// Can be used to test application reconnection logic.
    pub fn trigger_abort(&self) {
        self.0.client.trigger_abort().ok();
    }

    /// Register a callback for RPC events.
    ///
    /// Args:
    ///     event: Event type as kebab string or NotificationEvent variant. See NotificationEvent for acceptable values.
    ///     callback: Function to call when event occurs.
    ///     *args: Additional arguments to pass to callback.
    ///     **kwargs: Additional keyword arguments to pass to callback.
    ///
    /// Raises:
    ///     Exception: If the event type is invalid.
    #[pyo3(signature = (event, callback, *args, **kwargs))]
    fn add_event_listener(
        &self,
        py: Python,
        event: PyNotificationEvent,
        #[gen_stub(override_type(type_repr = "typing.Callable[..., None]"))] callback: Py<PyAny>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        let event: NotificationEvent = event.into();

        let args = args.into_pyobject(py)?.extract::<Py<PyTuple>>()?;

        let kwargs = match kwargs {
            Some(kw) => kw.into_pyobject(py)?.extract::<Py<PyDict>>()?,
            None => PyDict::new(py).into(),
        };

        let py_callback = PyCallback::new(callback, args, kwargs);

        self.0
            .callbacks
            .lock()
            .unwrap()
            .entry(event)
            .or_default()
            .push(py_callback);
        Ok(())
    }

    /// Remove an event listener.
    ///
    /// Args:
    ///     event: Event type as kebab string or NotificationEvent variant. See NotificationEvent for acceptable values.
    ///     callback: Specific callback to remove, or None to remove all.
    ///
    /// Raises:
    ///     Exception: If the event type is invalid.
    #[gen_stub(override_return_type(type_repr = "None"))]
    #[pyo3(signature = (event, callback=None))]
    fn remove_event_listener(
        &self,
        event: PyNotificationEvent,
        #[gen_stub(override_type(type_repr = "None | typing.Callable[..., None]"))]
        callback: Option<Py<PyAny>>,
    ) -> PyResult<()> {
        let event: NotificationEvent = event.into();
        let mut callbacks = self.0.callbacks.lock().unwrap();

        match (&event, callback) {
            (NotificationEvent::All, None) => {
                // Remove all callbacks from "all" events
                callbacks.clear();
            }
            (NotificationEvent::All, Some(callback)) => {
                // Remove given callback from "all" events
                for callbacks in callbacks.values_mut() {
                    callbacks.retain(|entry| !entry.callback_ptr_eq(&callback));
                }
            }
            (_, None) => {
                // Remove all callbacks from given event
                callbacks.remove(&event);
            }
            (_, Some(callback)) => {
                // Remove given callback from given event
                if let Some(callbacks) = callbacks.get_mut(&event) {
                    callbacks.retain(|entry| !entry.callback_ptr_eq(&callback));
                }
            }
        }
        Ok(())
    }

    // fn clear_event_listener TODO?
    // This functionality already exists via clear_event_listener("all", callback)

    /// Get the default RPC port for a given encoding and network type.
    ///
    /// Args:
    ///     encoding: RPC encoding format ("borsh" or "json").
    ///     network: Network type (e.g., "mainnet", "testnet-10", "testnet-11").
    ///
    /// Returns:
    ///     int: The default port number for the specified configuration.
    #[staticmethod]
    fn default_port(encoding: PyEncoding, network: PyNetworkType) -> PyResult<u16> {
        let network_type = NetworkType::from(network);
        match encoding {
            PyEncoding::Borsh => Ok(network_type.default_borsh_rpc_port()),
            PyEncoding::SerdeJson => Ok(network_type.default_json_rpc_port()),
        }
    }

    /// Remove all registered event listeners.
    fn remove_all_event_listeners(&self) -> PyResult<()> {
        *self.0.callbacks.lock().unwrap() = Default::default();
        Ok(())
    }
}

impl PyRpcClient {
    pub fn parse_url(url: &str, encoding: Encoding, network_id: NetworkId) -> PyResult<String> {
        let url_ = KaspaRpcClient::parse_url(url.to_string(), encoding, network_id.into())
            .map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(url_)
    }
}

impl PyRpcClient {
    // fn new_with_rpc_client() TODO

    pub fn listener_id(&self) -> Option<ListenerId> {
        *self.0.listener_id.lock().unwrap()
    }

    #[allow(dead_code)]
    pub fn client(&self) -> &Arc<KaspaRpcClient> {
        &self.0.client
    }

    async fn stop_notification_task(&self) -> Result<()> {
        if self.0.notification_task.load(Ordering::SeqCst) {
            self.0.notification_ctl.signal(()).await?;
            self.0.notification_task.store(false, Ordering::SeqCst);
        }
        Ok(())
    }

    #[allow(clippy::result_large_err)]
    fn start_notification_task(&self, py: Python) -> Result<()> {
        if self.0.notification_task.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.0.notification_task.store(true, Ordering::SeqCst);

        let ctl_receiver = self.0.notification_ctl.request.receiver.clone();
        let ctl_sender = self.0.notification_ctl.response.sender.clone();
        let notification_receiver = self.0.notification_channel.receiver.clone();
        let ctl_multiplexer_channel = self
            .0
            .client
            .rpc_client()
            .ctl_multiplexer()
            .as_ref()
            .expect("Python RpcClient ctl_multiplexer is None")
            .channel();
        let this = self.clone();

        let _ = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            loop {
                select_biased! {
                    msg = ctl_multiplexer_channel.recv().fuse() => {
                        if let Ok(ctl) = msg {

                            match ctl {
                                Ctl::Connect => {
                                    let listener_id = this.0.client.register_new_listener(ChannelConnection::new(
                                        "kaspapy-wrpc-client-python",
                                        this.0.notification_channel.sender.clone(),
                                        ChannelType::Persistent,
                                    ));
                                    *this.0.listener_id.lock().unwrap() = Some(listener_id);
                                }
                                Ctl::Disconnect => {
                                    let listener_id = this.0.listener_id.lock().unwrap().take();
                                    if let Some(listener_id) = listener_id
                                        && let Err(err) = this.0.client.unregister_listener(listener_id).await {
                                            panic!("Error in unregister_listener: {:?}",err);
                                    }
                                }
                            }

                            let event = NotificationEvent::RpcCtl(ctl);
                            if let Some(handlers) = this.0.notification_callbacks(event) {
                                for handler in handlers.into_iter() {
                                    Python::attach(|py| {
                                        let event = PyDict::new(py);
                                        event.set_item("type", ctl.to_string()).unwrap();
                                        event.set_item("rpc", this.get_url()).unwrap();

                                        handler.execute(py, event).unwrap_or_else(|err| panic!("{}", err));
                                    });
                                }
                            }
                        }
                    },
                    msg = notification_receiver.recv().fuse() => {
                        if let Ok(notification) = &msg {
                            match &notification {
                                kaspa_rpc_core::Notification::UtxosChanged(utxos_changed_notification) => {
                                    let event_type = notification.event_type();
                                    let notification_event = NotificationEvent::Notification(event_type);
                                    if let Some(handlers) = this.0.notification_callbacks(notification_event) {
                                        let UtxosChangedNotification { added, removed } = utxos_changed_notification;

                                        for handler in handlers.into_iter() {
                                            Python::attach(|py| {
                                                let added = serde_pyobject::to_pyobject(py, added).unwrap();
                                                let removed = serde_pyobject::to_pyobject(py, removed).unwrap();

                                                let event = PyDict::new(py);
                                                event.set_item("type", event_type.to_string()).unwrap();
                                                event.set_item("added", &added).unwrap();
                                                event.set_item("removed", &removed).unwrap();

                                                handler.execute(py, event).unwrap_or_else(|err| panic!("{}", err));
                                            })
                                        }
                                    }
                                },
                                _ => {
                                    let event_type = notification.event_type();
                                    let notification_event = NotificationEvent::Notification(event_type);
                                    if let Some(handlers) = this.0.notification_callbacks(notification_event) {
                                        for handler in handlers.into_iter() {
                                            Python::attach(|py| {
                                                let event = PyDict::new(py);
                                                event.set_item("type", event_type.to_string()).unwrap();
                                                event.set_item("data", PyNotification::from(notification.clone()).to_pyobject(py).unwrap()).unwrap();

                                                handler.execute(py, event).unwrap_or_else(|err| panic!("{}", err));
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ = ctl_receiver.recv().fuse() => {
                        break;
                    },

                }
            }

            if let Some(listener_id) = this.listener_id() {
                this.0.listener_id.lock().unwrap().take();
                if let Err(err) = this.0.client.unregister_listener(listener_id).await {
                    log_error!("Error in unregister_listener: {:?}", err);
                }
            }

            ctl_sender.send(()).await.ok();

            Python::attach(|_| Ok(()))
        });

        Ok(())
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyRpcClient {
    /// Subscribe to UTXO changes for specific addresses (async).
    ///
    /// Args:
    ///     addresses: List of addresses to monitor.
    ///
    /// Raises:
    ///     Exception: If not connected or subscription fails.
    #[gen_stub(override_return_type(type_repr = "None"))]
    fn subscribe_utxos_changed<'py>(
        &self,
        py: Python<'py>,
        addresses: Vec<PyAddress>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if let Some(listener_id) = self.listener_id() {
            let client = self.0.client.clone();
            let addresses = addresses.iter().map(|a| a.0.clone()).collect();
            pyo3_async_runtimes::tokio::future_into_py(py, async move {
                client
                    .start_notify(
                        listener_id,
                        Scope::UtxosChanged(UtxosChangedScope { addresses }),
                    )
                    .await
                    .map_err(|err| PyException::new_err(err.to_string()))?;
                Ok(())
            })
        } else {
            Err(PyException::new_err("RPC subscribe on a closed connection"))
        }
    }

    /// Unsubscribe from UTXO changes for specific addresses (async).
    ///
    /// Args:
    ///     addresses: List of addresses to stop monitoring.
    ///
    /// Raises:
    ///     Exception: If not connected or unsubscription fails.
    #[gen_stub(override_return_type(type_repr = "None"))]
    fn unsubscribe_utxos_changed<'py>(
        &self,
        py: Python<'py>,
        addresses: Vec<PyAddress>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if let Some(listener_id) = self.listener_id() {
            let client = self.0.client.clone();
            let addresses = addresses.iter().map(|a| a.0.clone()).collect();
            pyo3_async_runtimes::tokio::future_into_py(py, async move {
                client
                    .stop_notify(
                        listener_id,
                        Scope::UtxosChanged(UtxosChangedScope { addresses }),
                    )
                    .await
                    .map_err(|err| PyException::new_err(err.to_string()))?;
                Ok(())
            })
        } else {
            Err(PyException::new_err(
                "RPC unsubscribe on a closed connection",
            ))
        }
    }

    /// Subscribe to virtual chain changes (async).
    ///
    /// Args:
    ///     include_accepted_transaction_ids: Include transaction IDs in notifications.
    ///
    /// Raises:
    ///     Exception: If not connected or subscription fails.
    #[gen_stub(override_return_type(type_repr = "None"))]
    fn subscribe_virtual_chain_changed<'py>(
        &self,
        py: Python<'py>,
        include_accepted_transaction_ids: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        if let Some(listener_id) = self.listener_id() {
            let client = self.0.client.clone();
            pyo3_async_runtimes::tokio::future_into_py(py, async move {
                client
                    .start_notify(
                        listener_id,
                        Scope::VirtualChainChanged(VirtualChainChangedScope {
                            include_accepted_transaction_ids,
                        }),
                    )
                    .await
                    .map_err(|err| PyException::new_err(err.to_string()))?;
                Ok(())
            })
        } else {
            Err(PyException::new_err("RPC subscribe on a closed connection"))
        }
    }

    /// Unsubscribe from virtual chain changes (async).
    ///
    /// Args:
    ///     include_accepted_transaction_ids: Must match the subscription parameter.
    ///
    /// Raises:
    ///     Exception: If not connected or unsubscription fails.
    #[gen_stub(override_return_type(type_repr = "None"))]
    fn unsubscribe_virtual_chain_changed<'py>(
        &self,
        py: Python<'py>,
        include_accepted_transaction_ids: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        if let Some(listener_id) = self.listener_id() {
            let client = self.0.client.clone();
            pyo3_async_runtimes::tokio::future_into_py(py, async move {
                client
                    .stop_notify(
                        listener_id,
                        Scope::VirtualChainChanged(VirtualChainChangedScope {
                            include_accepted_transaction_ids,
                        }),
                    )
                    .await
                    .map_err(|err| PyException::new_err(err.to_string()))?;
                Ok(())
            })
        } else {
            Err(PyException::new_err(
                "RPC unsubscribe on a closed connection",
            ))
        }
    }
}

// Macro to generate subscribe/unsubscribe method implementations for RPC notifications.
//
// For each scope name (e.g., `BlockAdded`), this generates:
// - `subscribe_block_added` - Python-callable async method to start notifications
// - `unsubscribe_block_added` - Python-callable async method to stop notifications
macro_rules! build_wrpc_python_subscriptions {
    ([$($scope:ident),* $(,)?]) => {
        paste! {
            #[gen_stub_pymethods]
            #[pymethods]
            impl PyRpcClient {
                $(
                    #[gen_stub(override_return_type(type_repr="None"))]
                    fn [<subscribe_ $scope:snake>]<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
                        if let Some(listener_id) = self.listener_id() {
                            let client = self.0.client.clone();
                            pyo3_async_runtimes::tokio::future_into_py(py, async move {
                                client.start_notify(listener_id, Scope::$scope([<$scope Scope>] {})).await
                                    .map_err(|err| PyException::new_err(err.to_string()))?;
                                Ok(())
                            })
                        } else {
                            Err(PyException::new_err("RPC subscribe on a closed connection"))
                        }
                    }

                    #[gen_stub(override_return_type(type_repr="None"))]
                    fn [<unsubscribe_ $scope:snake>]<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
                        if let Some(listener_id) = self.listener_id() {
                            let client = self.0.client.clone();
                            pyo3_async_runtimes::tokio::future_into_py(py, async move {
                                client.stop_notify(listener_id, Scope::$scope([<$scope Scope>] {})).await
                                    .map_err(|err| PyException::new_err(err.to_string()))?;
                                Ok(())
                            })
                        } else {
                            Err(PyException::new_err("RPC unsubscribe on a closed connection"))
                        }
                    }
                )*
            }
        }
    };
}

build_wrpc_python_subscriptions!([
    BlockAdded,
    FinalityConflict,
    FinalityConflictResolved,
    NewBlockTemplate,
    PruningPointUtxoSetOverride,
    SinkBlueScoreChanged,
    VirtualDaaScoreChanged,
]);

// Macro to generate RPC method implementations for RpcClient.
//
// For each type name (e.g., `GetBlockCount`), this generates:
// - A Python-callable async method `get_block_count`
// - That accepts an optional `PyDict` as request parameters
// - Calls the corresponding `get_block_count_call` method on the RPC client
// - Returns the response as a Python object
macro_rules! build_wrpc_python_interface {
    ([$($name:ident),* $(,)?]) => {
        paste! {
            #[gen_stub_pymethods]
            #[pymethods]
            impl PyRpcClient {
                $(
                    #[pyo3(signature = (request=None))]
                    fn [<$name:snake>]<'py>(
                        &self,
                        py: Python<'py>,
                        request: Option<Bound<'_, PyDict>>
                    ) -> PyResult<Bound<'py, PyAny>> {
                        let client = self.0.client.clone();

                        let request: [<Py $name Request>] = request
                            .unwrap_or_else(|| PyDict::new(py))
                            .try_into()?;

                        pyo3_async_runtimes::tokio::future_into_py(py, async move {
                            let response: [<$name Response>] = client
                                .[<$name:snake _call>](None, request.0)
                                .await
                                .map_err(|err| PyException::new_err(err.to_string()))?;

                            Python::attach(|py| {
                                Ok(serde_pyobject::to_pyobject(py, &response)?.unbind())
                            })
                        })
                    }
                )*
            }
        }
    };
}

build_wrpc_python_interface!([
    GetBlockCount,
    GetBlockDagInfo,
    GetCoinSupply,
    GetConnectedPeerInfo,
    GetInfo,
    GetPeerAddresses,
    GetMetrics,
    GetConnections,
    GetSink,
    GetSinkBlueScore,
    Ping,
    Shutdown,
    GetServerInfo,
    GetSyncStatus,
    GetFeeEstimate,
    GetCurrentNetwork,
    GetSystemInfo,
]);

// Macro to generate RPC method implementations that require request parameters.
//
// Similar to `build_wrpc_python_interface!`, but the `request` parameter is required
// (not optional), for RPC calls that need specific arguments.
macro_rules! build_wrpc_python_interface_with_args {
    ([$($name:ident),* $(,)?]) => {
        paste! {
            #[gen_stub_pymethods]
            #[pymethods]
            impl PyRpcClient {
                $(
                    fn [<$name:snake>]<'py>(
                        &self,
                        py: Python<'py>,
                        request: Bound<'_, PyDict>
                    ) -> PyResult<Bound<'py, PyAny>> {
                        let client = self.0.client.clone();

                        let request: [<Py $name Request>] = request.try_into()?;

                        pyo3_async_runtimes::tokio::future_into_py(py, async move {
                            let response: [<$name Response>] = client
                                .[<$name:snake _call>](None, request.0)
                                .await
                                .map_err(|err| PyException::new_err(err.to_string()))?;

                            Python::attach(|py| {
                                Ok(serde_pyobject::to_pyobject(py, &response)?.unbind())
                            })
                        })
                    }
                )*
            }
        }
    };
}

build_wrpc_python_interface_with_args!([
    AddPeer,
    Ban,
    EstimateNetworkHashesPerSecond,
    GetBalanceByAddress,
    GetBalancesByAddresses,
    GetBlock,
    GetBlocks,
    GetBlockTemplate,
    GetCurrentBlockColor,
    GetDaaScoreTimestampEstimate,
    GetFeeEstimateExperimental,
    GetHeaders,
    GetMempoolEntries,
    GetMempoolEntriesByAddresses,
    GetMempoolEntry,
    GetSubnetwork,
    GetUtxosByAddresses,
    GetUtxoReturnAddress,
    GetVirtualChainFromBlock,
    GetVirtualChainFromBlockV2,
    ResolveFinalityConflict,
    SubmitBlock,
    SubmitTransaction,
    SubmitTransactionReplacement,
    Unban,
]);
