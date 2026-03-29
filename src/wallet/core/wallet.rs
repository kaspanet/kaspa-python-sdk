use super::error::Error;
use crate::{
    callback::PyCallback,
    consensus::core::network::PyNetworkId,
    error::IntoPyResult,
    rpc::{
        encoding::PyEncoding,
        wrpc::{client::PyRpcClient, resolver::PyResolver},
    },
    wallet::core::{events::PyWalletEventType, storage::interface::PyWalletDescriptor},
};
use ahash::AHashMap;
use futures::{FutureExt, select};
use kaspa_wallet_core::{
    api::WalletApi,
    error::Error as NativeError,
    events::{EventKind, Events},
    result::Result,
    rpc::{DynRpcApi, Rpc},
    wallet as native,
};
use kaspa_wrpc_client::prelude::NetworkId;
use pyo3::{
    prelude::*,
    types::{PyDict, PyTuple},
};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use workflow_core::prelude::{DuplexChannel, Multiplexer};

struct Inner {
    wallet: Arc<native::Wallet>,
    rpc: PyRpcClient,
    callbacks: Arc<Mutex<AHashMap<EventKind, Vec<PyCallback>>>>,
    task_running: AtomicBool,
    task_ctl: DuplexChannel,
}

impl Inner {
    fn callbacks(&self, event: EventKind) -> Option<Vec<PyCallback>> {
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
}

#[gen_stub_pyclass]
#[pyclass(name = "Wallet")]
#[derive(Clone)]
pub struct PyWallet {
    inner: Arc<Inner>,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyWallet {
    #[new]
    #[pyo3(signature = (network_id=None, encoding=None, url=None, resolver=None))]
    pub fn new(
        // resident: bool, TODO
        network_id: Option<PyNetworkId>,
        encoding: Option<PyEncoding>,
        url: Option<String>,
        resolver: Option<PyResolver>,
    ) -> PyResult<Self> {
        let store = native::Wallet::local_store().into_py_result()?;

        let rpc = PyRpcClient::ctor(
            resolver,
            url,
            encoding,
            network_id.clone(),
        )?;
        let rpc_api: Arc<DynRpcApi> = rpc.client().rpc_api().clone();
        let rpc_ctl = rpc.client().rpc_ctl().clone();
        let rpc_binding = Rpc::new(rpc_api, rpc_ctl);

        let wallet = Arc::new(
            native::Wallet::try_with_rpc(Some(rpc_binding), store, network_id.map(NetworkId::from))
                .into_py_result()?,
        );

        Ok(Self {
            inner: Arc::new(Inner {
                wallet,
                rpc,
                callbacks: Arc::new(Mutex::new(AHashMap::new())),
                task_running: AtomicBool::new(false),
                task_ctl: DuplexChannel::oneshot(),
            }),
        })
    }

    #[getter]
    pub fn get_rpc(&self) -> PyRpcClient {
        self.inner.rpc.clone()
    }

    #[getter]
    pub fn get_is_open(&self) -> bool {
        self.wallet().is_open()
    }

    #[getter]
    pub fn get_is_synced(&self) -> bool {
        self.wallet().is_synced()
    }

    #[getter]
    pub fn get_descriptor(&self) -> Option<PyWalletDescriptor> {
        self.wallet().descriptor().map(PyWalletDescriptor::from)
    }

    // TODO override return type to bool
    pub fn exists<'py>(
        &self,
        py: Python<'py>,
        name: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let slf = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let exists = slf
                .wallet()
                .exists(name.as_deref())
                .await
                .into_py_result()?;
            Ok(exists)
        })
    }

    // TODO override return type to none
    pub fn start<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.start_notification_task(py, self.wallet().multiplexer())
            .into_py_result()?;

        let slf = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            slf.wallet().start().await.into_py_result()?;
            Ok(())
        })
    }

    // TODO override return type to none
    pub fn stop<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let slf = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            slf.stop_notification_task().await.into_py_result()?;

            slf.wallet().stop().await.into_py_result()?;
            Ok(())
        })
    }

    // TODO override return type
    pub fn connect<'py>(
        &self,
        py: Python<'py>,
        block_async_connect: Option<bool>,
        strategy: Option<String>,
        url: Option<String>,
        timeout_duration: Option<u64>,
        retry_interval: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.inner.rpc.connect(
            py,
            block_async_connect,
            strategy,
            url,
            timeout_duration,
            retry_interval,
        )
    }

    pub fn disconnect<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner.rpc.disconnect(py)
    }

    #[pyo3(signature = (event, callback, *args, **kwargs))]
    fn add_event_listener(
        &self,
        py: Python,
        event: PyWalletEventType,
        #[gen_stub(override_type(type_repr = "typing.Callable[..., None"))] callback: Py<PyAny>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        let event: EventKind = event.into();

        let args = args.into_pyobject(py)?.extract::<Py<PyTuple>>()?;

        let kwargs = match kwargs {
            Some(kw) => kw.into_pyobject(py)?.extract::<Py<PyDict>>()?,
            None => PyDict::new(py).into(),
        };

        let py_callback = PyCallback::new(callback, args, kwargs);

        self.inner
            .callbacks
            .lock()
            .unwrap()
            .entry(event)
            .or_default()
            .push(py_callback);

        Ok(())
    }

    #[gen_stub(override_return_type(type_repr = "None"))]
    #[pyo3(signature = (event, callback=None))]
    fn remove_event_listener(
        &self,
        event: PyWalletEventType,
        #[gen_stub(override_type(type_repr = "None | typing.Callable[..., None]"))]
        callback: Option<Py<PyAny>>,
    ) -> PyResult<()> {
        let event: EventKind = event.into();
        let mut callbacks = self.inner.callbacks.lock().unwrap();

        match (&event, callback) {
            (EventKind::All, None) => {
                // Remove all callbacks from "all" events
                callbacks.clear();
            }
            (EventKind::All, Some(callback)) => {
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

    pub fn set_network_id(&self, network_id: PyNetworkId) -> Result<(), Error> {
        self.inner.wallet.set_network_id(&(network_id.into()))?;
        Ok(())
    }
}

impl PyWallet {
    pub fn wallet(&self) -> &Arc<native::Wallet> {
        &self.inner.wallet
    }

    pub fn start_notification_task(
        &self,
        py: Python,
        multiplexer: &Multiplexer<Box<Events>>,
    ) -> Result<()> {
        let inner = self.inner.clone();

        if inner.task_running.load(Ordering::SeqCst) {
            panic!("ReflectorClient task is already running");
        } else {
            inner.task_running.store(true, Ordering::SeqCst);
        }

        let ctl_receiver = inner.task_ctl.request.receiver.clone();
        let ctl_sender = inner.task_ctl.response.sender.clone();

        let channel = multiplexer.channel();

        let _ = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            loop {
                select! {
                    _ = ctl_receiver.recv().fuse() => {
                        break;
                    },
                    msg = channel.receiver.recv().fuse() => {
                        if let Ok(notification) = &msg {
                            let event_type = EventKind::from(notification.as_ref());
                            let callbacks = inner.callbacks(event_type);
                            if let Some(handlers) = callbacks {
                                for handler in handlers.into_iter() {
                                    Python::attach(|py| {
                                        let event = PyDict::new(py);
                                        event.set_item("type", event_type.to_string()).unwrap();
                                        event.set_item("data", serde_pyobject::to_pyobject(py, notification.as_ref()).unwrap()).unwrap();

                                        handler.execute(py, event).unwrap_or_else(|err| panic!("{}", err));
                                    });
                                }
                            }
                        }
                    }
                }
            }

            channel.close();
            ctl_sender.send(()).await.ok();
            Ok(())
        });

        Ok(())
    }

    pub async fn stop_notification_task(&self) -> Result<()> {
        let inner = &self.inner;
        if inner.task_running.load(Ordering::SeqCst) {
            inner.task_running.store(false, Ordering::SeqCst);
            inner
                .task_ctl
                .signal(())
                .await
                .map_err(|err| NativeError::custom(err.to_string()))?;
        }
        Ok(())
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyWallet {
    // TODO override return type to Vec<PyWalletDescriptor> or corresponding exception
    pub fn wallet_enumerate<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::PyAny>> {
        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let d: Vec<PyWalletDescriptor> = wallet
                .wallet_enumerate()
                .await
                .into_py_result()?
                .iter()
                .map(PyWalletDescriptor::from)
                .collect();
            Ok(d)
        })
    }
}
