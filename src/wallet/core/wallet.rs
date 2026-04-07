use super::error::Error;
use crate::{
    address::PyAddress,
    callback::PyCallback,
    consensus::core::network::PyNetworkId,
    error::IntoPyResult,
    rpc::{
        encoding::PyEncoding,
        wrpc::{client::PyRpcClient, resolver::PyResolver},
    },
    types::PyBinary,
    wallet::core::{
        account::{descriptor::PyAccountDescriptor, kind::PyAccountKind},
        api::message::{PyAccountsDiscoveryKind, PyCommitRevealAddressKind, PyNewAddressKind},
        events::PyWalletEventType,
        storage::{
            interface::PyWalletDescriptor,
            keydata::{PyPrvKeyDataInfo, PyPrvKeyDataVariantKind},
            transaction::PyTransactionKind,
        },
        tx::{fees::PyFees, generator::PyGeneratorSummary, payment::PyPaymentOutput},
    },
};
use ahash::AHashMap;
use futures::{FutureExt, select};
use kaspa_addresses::Address;
use kaspa_hashes::Hash;
use kaspa_utils::hex::{FromHex, ToHex};
use kaspa_wallet_core::{
    api::{
        AccountsCommitRevealManualRequest, AccountsCommitRevealRequest, AccountsDiscoveryRequest,
        AccountsEstimateRequest, AccountsGetRequest, AccountsGetUtxosRequest,
        AccountsImportRequest, AccountsSendRequest, AccountsTransferRequest,
        AddressBookEnumerateRequest, BatchRequest, FeeRateEstimateRequest,
        FeeRatePollerDisableRequest, FeeRatePollerEnableRequest, FlushRequest, GetStatusRequest,
        PrvKeyDataRemoveRequest, RetainContextRequest, TransactionsDataGetRequest,
        TransactionsReplaceMetadataRequest, TransactionsReplaceNoteRequest, WalletApi,
        WalletExportRequest, WalletImportRequest,
    },
    error::Error as NativeError,
    events::{EventKind, Events},
    prelude::{AccountId, EncryptionKind},
    result::Result,
    rpc::{DynRpcApi, Rpc},
    storage::{Hint, PrvKeyDataId, PrvKeyDataInfo, TransactionKind},
    tx::{PaymentDestination, PaymentOutput, PaymentOutputs},
    wallet::{
        self as native, AccountCreateArgs, AccountCreateArgsBip32, PrvKeyDataArgs,
        PrvKeyDataCreateArgs, WalletCreateArgs,
    },
};
use kaspa_wallet_keys::secret::Secret;
use kaspa_wrpc_client::prelude::NetworkId;
use pyo3::{
    exceptions::PyException,
    prelude::*,
    types::{PyDict, PyTuple},
};
use pyo3_stub_gen::derive::*;
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
        #[gen_stub(override_type(type_repr = "None | NetworkId | str"))] network_id: Option<
            PyNetworkId,
        >,
        #[gen_stub(override_type(type_repr = "None | Encoding | str"))] encoding: Option<PyEncoding>,
        url: Option<String>,
        resolver: Option<PyResolver>,
    ) -> PyResult<Self> {
        let store = native::Wallet::local_store().into_py_result()?;

        let rpc = PyRpcClient::ctor(resolver, url, encoding, network_id.clone())?;
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

    #[gen_stub(override_return_type(type_repr = "bool"))]
    #[pyo3(signature = (name=None))]
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

    #[gen_stub(override_return_type(type_repr = "None"))]
    pub fn start<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.start_notification_task(py, self.wallet().multiplexer())
            .into_py_result()?;

        let slf = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            slf.wallet().start().await.into_py_result()?;
            Ok(())
        })
    }

    #[gen_stub(override_return_type(type_repr = "None"))]
    pub fn stop<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let slf = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            slf.stop_notification_task().await.into_py_result()?;

            slf.wallet().stop().await.into_py_result()?;
            Ok(())
        })
    }

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
        self.inner.rpc.connect(
            py,
            block_async_connect,
            strategy,
            url,
            timeout_duration,
            retry_interval,
        )
    }

    #[gen_stub(override_return_type(type_repr = "None"))]
    pub fn disconnect<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner.rpc.disconnect(py)
    }

    #[pyo3(signature = (event, callback, *args, **kwargs))]
    fn add_event_listener(
        &self,
        py: Python,
        #[gen_stub(override_type(type_repr = "WalletEventType | str"))] event: PyWalletEventType,
        #[gen_stub(override_type(type_repr = "typing.Callable[..., None]"))] callback: Py<PyAny>,
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
        #[gen_stub(override_type(type_repr = "WalletEventType | str"))] event: PyWalletEventType,
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

    pub fn set_network_id(
        &self,
        #[gen_stub(override_type(type_repr = "NetworkId | str"))] network_id: PyNetworkId,
    ) -> Result<(), Error> {
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

// Wallet API wallet_ functions
#[gen_stub_pymethods]
#[pymethods]
impl PyWallet {
    #[gen_stub(override_return_type(type_repr = "list[WalletDescriptor]"))]
    pub fn wallet_enumerate<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::PyAny>> {
        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let descriptors: Vec<PyWalletDescriptor> = wallet
                .wallet_enumerate()
                .await
                .into_py_result()?
                .iter()
                .map(PyWalletDescriptor::from)
                .collect();
            Ok(descriptors)
        })
    }

    #[gen_stub(override_return_type(type_repr = "dict"))]
    #[pyo3(signature = (wallet_secret, filename=None, overwrite_wallet_storage=None, title=None, user_hint=None))]
    pub fn wallet_create<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        filename: Option<String>,
        overwrite_wallet_storage: Option<bool>,
        title: Option<String>,
        user_hint: Option<String>,
    ) -> PyResult<Bound<'py, pyo3::PyAny>> {
        let args = WalletCreateArgs::new(
            title,
            filename,
            EncryptionKind::default(),
            user_hint.map(Hint::from),
            overwrite_wallet_storage.unwrap_or(false),
        );

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .wallet_create(wallet_secret.into(), args)
                .await
                .into_py_result()?;

            Python::attach(|py| Ok(serde_pyobject::to_pyobject(py, &resp)?.unbind()))
        })
    }

    #[gen_stub(override_return_type(type_repr = "dict"))]
    #[pyo3(signature = (wallet_secret, account_descriptors, filename=None))]
    pub fn wallet_open<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        account_descriptors: bool,
        filename: Option<String>,
    ) -> PyResult<Bound<'py, pyo3::PyAny>> {
        // let args = WalletOpenArgs { account_descriptors, legacy_accounts: false };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .wallet_open(wallet_secret.into(), filename, account_descriptors, false)
                .await
                .into_py_result()?;

            Python::attach(|py| Ok(serde_pyobject::to_pyobject(py, &resp)?.unbind()))
        })
    }

    #[gen_stub(override_return_type(type_repr = "None"))]
    pub fn wallet_close<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::PyAny>> {
        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet.wallet_close().await.into_py_result()?;

            Ok(())
        })
    }

    #[gen_stub(override_return_type(type_repr = "None"))]
    pub fn wallet_reload<'py>(
        &self,
        py: Python<'py>,
        reactivate: bool,
    ) -> PyResult<Bound<'py, pyo3::PyAny>> {
        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet.wallet_reload(reactivate).await.into_py_result()?;

            Ok(())
        })
    }

    #[gen_stub(override_return_type(type_repr = "None"))]
    #[pyo3(signature = (wallet_secret, title=None, filename=None))]
    pub fn wallet_rename<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        title: Option<String>,
        filename: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet
                .wallet_rename(title.as_deref(), filename.as_deref(), wallet_secret.into())
                .await
                .into_py_result()?;
            Ok(())
        })
    }

    #[gen_stub(override_return_type(type_repr = "None"))]
    pub fn wallet_change_secret<'py>(
        &self,
        py: Python<'py>,
        old_wallet_secret: String,
        new_wallet_secret: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet
                .wallet_change_secret(old_wallet_secret.into(), new_wallet_secret.into())
                .await
                .into_py_result()?;
            Ok(())
        })
    }

    #[gen_stub(override_return_type(type_repr = "bytes"))]
    pub fn wallet_export<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        include_transactions: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        let req = WalletExportRequest {
            wallet_secret: wallet_secret.into(),
            include_transactions,
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet.wallet_export_call(req).await.into_py_result()?;

            Ok(resp.wallet_data)
        })
    }

    #[gen_stub(override_return_type(type_repr = "dict"))]
    pub fn wallet_import<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        #[gen_stub(override_type(type_repr = "str | bytes | list[int]"))] wallet_data: PyBinary,
    ) -> PyResult<Bound<'py, PyAny>> {
        let req = WalletImportRequest {
            wallet_secret: wallet_secret.into(),
            wallet_data: wallet_data.into(),
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet.wallet_import_call(req).await.into_py_result()?;

            Python::attach(|py| Ok(serde_pyobject::to_pyobject(py, &resp)?.unbind()))
        })
    }
}

// Wallet API prv_key_data_ functions
#[gen_stub_pymethods]
#[pymethods]
impl PyWallet {
    #[gen_stub(override_return_type(type_repr = "list[PrvKeyDataInfo]"))]
    pub fn prv_key_data_enumerate<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .prv_key_data_enumerate()
                .await
                .into_py_result()?
                .into_iter()
                .map(|v| PyPrvKeyDataInfo::from(v.as_ref()))
                .collect::<Vec<PyPrvKeyDataInfo>>();

            Ok(resp)
        })
    }

    #[gen_stub(override_return_type(type_repr = "str"))]
    #[pyo3(signature = (wallet_secret, secret, kind, payment_secret=None, name=None))]
    pub fn prv_key_data_create<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        secret: String,
        #[gen_stub(override_type(type_repr = "PrvKeyDataVariantKind | str"))]
        kind: PyPrvKeyDataVariantKind,
        payment_secret: Option<String>,
        name: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let args = PrvKeyDataCreateArgs::new(
            name,
            payment_secret.map(Secret::from),
            secret.into(),
            kind.into(),
        );

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .prv_key_data_create(wallet_secret.into(), args)
                .await
                .into_py_result()?;
            Ok(resp.to_hex())
        })
    }

    #[gen_stub(override_return_type(type_repr = "None"))]
    pub fn prv_key_data_remove<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        prv_key_data_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = PrvKeyDataRemoveRequest {
            wallet_secret: wallet_secret.into(),
            prv_key_data_id: PrvKeyDataId::from_hex(&prv_key_data_id)
                .map_err(|err| PyException::new_err(err.to_string()))?,
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet
                .prv_key_data_remove_call(request)
                .await
                .into_py_result()?;

            Ok(())
        })
    }

    #[gen_stub(override_return_type(type_repr = "PrvKeyDataInfo"))]
    pub fn prv_key_data_get<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        prv_key_data_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let prv_key_data_id = PrvKeyDataId::from_hex(&prv_key_data_id)
            .map_err(|err| PyException::new_err(err.to_string()))?;

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .prv_key_data_get(prv_key_data_id, wallet_secret.into())
                .await
                .into_py_result()?;

            Ok(PyPrvKeyDataInfo::from(PrvKeyDataInfo::from(&resp)))
        })
    }
}

// Wallet API account_ functions
#[gen_stub_pymethods]
#[pymethods]
impl PyWallet {
    #[gen_stub(override_return_type(type_repr = "list[AccountDescriptor]"))]
    pub fn accounts_enumerate<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let accounts = wallet
                .accounts_enumerate()
                .await
                .into_py_result()?
                .into_iter()
                .map(PyAccountDescriptor::from)
                .collect::<Vec<PyAccountDescriptor>>();
            Ok(accounts)
        })
    }

    #[gen_stub(override_return_type(type_repr = "AccountDescriptor"))]
    #[pyo3(signature = (wallet_secret, prv_key_data_id, payment_secret=None, account_name=None, account_index=None))]
    pub fn accounts_create_bip32<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        prv_key_data_id: String,
        payment_secret: Option<String>,
        account_name: Option<String>,
        account_index: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let args = AccountCreateArgs::Bip32 {
            prv_key_data_args: PrvKeyDataArgs {
                prv_key_data_id: PrvKeyDataId::from_hex(&prv_key_data_id)
                    .map_err(|err| PyException::new_err(err.to_string()))?,
                payment_secret: payment_secret.map(Secret::from),
            },
            account_args: AccountCreateArgsBip32 {
                account_name,
                account_index,
            },
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .accounts_create(wallet_secret.into(), args)
                .await
                .into_py_result()?;
            Ok(PyAccountDescriptor::from(resp))
        })
    }

    #[gen_stub(override_return_type(type_repr = "AccountDescriptor"))]
    #[pyo3(signature = (wallet_secret, prv_key_data_id, ecdsa, account_name=None))]
    pub fn accounts_create_keypair<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        prv_key_data_id: String,
        ecdsa: bool,
        account_name: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let args = AccountCreateArgs::Keypair {
            prv_key_data_id: PrvKeyDataId::from_hex(&prv_key_data_id)
                .map_err(|err| PyException::new_err(err.to_string()))?,
            account_name,
            ecdsa,
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .accounts_create(wallet_secret.into(), args)
                .await
                .into_py_result()?;
            Ok(PyAccountDescriptor::from(resp))
        })
    }

    #[gen_stub(override_return_type(type_repr = "AccountDescriptor"))]
    #[pyo3(signature = (wallet_secret, prv_key_data_id, payment_secret=None, account_name=None, account_index=None))]
    pub fn accounts_import_bip32<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        prv_key_data_id: String,
        payment_secret: Option<String>,
        account_name: Option<String>,
        account_index: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let account_create_args = AccountCreateArgs::Bip32 {
            prv_key_data_args: PrvKeyDataArgs {
                prv_key_data_id: PrvKeyDataId::from_hex(&prv_key_data_id)
                    .map_err(|err| PyException::new_err(err.to_string()))?,
                payment_secret: payment_secret.map(Secret::from),
            },
            account_args: AccountCreateArgsBip32 {
                account_name,
                account_index,
            },
        };

        let request = AccountsImportRequest {
            wallet_secret: wallet_secret.into(),
            account_create_args,
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .accounts_import_call(request)
                .await
                .into_py_result()?;
            Ok(PyAccountDescriptor::from(resp.account_descriptor))
        })
    }

    #[gen_stub(override_return_type(type_repr = "AccountDescriptor"))]
    #[pyo3(signature = (wallet_secret, prv_key_data_id, ecdsa, account_name=None))]
    pub fn accounts_import_keypair<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        prv_key_data_id: String,
        ecdsa: bool,
        account_name: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let account_create_args = AccountCreateArgs::Keypair {
            prv_key_data_id: PrvKeyDataId::from_hex(&prv_key_data_id)
                .map_err(|err| PyException::new_err(err.to_string()))?,
            account_name,
            ecdsa,
        };

        let request = AccountsImportRequest {
            wallet_secret: wallet_secret.into(),
            account_create_args,
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .accounts_import_call(request)
                .await
                .into_py_result()?;
            Ok(PyAccountDescriptor::from(resp.account_descriptor))
        })
    }

    #[gen_stub(override_return_type(type_repr = "None"))]
    #[pyo3(signature = (wallet_secret, account_id, name=None))]
    pub fn accounts_rename<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        account_id: String,
        name: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let account_id = AccountId::from_hex(&account_id)
            .map_err(|err| PyException::new_err(err.to_string()))?;

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet
                .accounts_rename(account_id, name, wallet_secret.into())
                .await
                .into_py_result()?;

            Ok(())
        })
    }

    #[gen_stub(override_return_type(type_repr = "int"))]
    #[pyo3(signature = (discovery_kind, address_scan_extent, account_scan_extent, bip39_mnemonic, bip39_passphrase=None))]
    pub fn accounts_discovery<'py>(
        &self,
        py: Python<'py>,
        #[gen_stub(override_type(type_repr = "AccountsDiscoveryKind | str"))]
        discovery_kind: PyAccountsDiscoveryKind,
        address_scan_extent: u32,
        account_scan_extent: u32,
        bip39_mnemonic: String,
        bip39_passphrase: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = AccountsDiscoveryRequest {
            discovery_kind: discovery_kind.into(),
            address_scan_extent,
            account_scan_extent,
            bip39_passphrase: bip39_passphrase.map(Secret::from),
            bip39_mnemonic: bip39_mnemonic.into(),
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .accounts_discovery_call(request)
                .await
                .into_py_result()?;

            Ok(resp.last_account_index_found)
        })
    }

    #[gen_stub(override_return_type(type_repr = "AccountDescriptor"))]
    #[pyo3(signature = (wallet_secret, account_kind, payment_secret=None, mnemonic_phrase=None))]
    pub fn accounts_ensure_default<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        account_kind: PyAccountKind,
        payment_secret: Option<String>,
        mnemonic_phrase: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .accounts_ensure_default(
                    wallet_secret.into(),
                    payment_secret.map(Secret::from),
                    account_kind.into(),
                    mnemonic_phrase.map(Secret::from),
                )
                .await
                .into_py_result()?;

            Ok(PyAccountDescriptor::from(resp))
        })
    }

    #[gen_stub(override_return_type(type_repr = "None"))]
    #[pyo3(signature = (account_ids=None))]
    pub fn accounts_activate<'py>(
        &self,
        py: Python<'py>,
        account_ids: Option<Vec<String>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let account_ids = account_ids
            .map(|ids| {
                ids.iter()
                    .map(|id| {
                        AccountId::from_hex(id).map_err(|err| PyException::new_err(err.to_string()))
                    })
                    .collect::<PyResult<Vec<AccountId>>>()
            })
            .transpose()?;

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet
                .accounts_activate(account_ids)
                .await
                .into_py_result()?;

            Ok(())
        })
    }

    #[gen_stub(override_return_type(type_repr = "None"))]
    #[pyo3(signature = (account_ids=None))]
    pub fn accounts_deactivate<'py>(
        &self,
        py: Python<'py>,
        account_ids: Option<Vec<String>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let account_ids = account_ids
            .map(|ids| {
                ids.iter()
                    .map(|id| {
                        AccountId::from_hex(id).map_err(|err| PyException::new_err(err.to_string()))
                    })
                    .collect::<PyResult<Vec<AccountId>>>()
            })
            .transpose()?;

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet
                .accounts_deactivate(account_ids)
                .await
                .into_py_result()?;

            Ok(())
        })
    }

    #[gen_stub(override_return_type(type_repr = "None"))]
    pub fn accounts_get<'py>(
        &self,
        py: Python<'py>,
        account_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = AccountsGetRequest {
            account_id: AccountId::from_hex(&account_id)
                .map_err(|err| PyException::new_err(err.to_string()))?,
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet.accounts_get_call(request).await.into_py_result()?;

            Ok(())
        })
    }

    #[gen_stub(override_return_type(type_repr = "Address"))]
    pub fn accounts_create_new_address<'py>(
        &self,
        py: Python<'py>,
        account_id: String,
        #[gen_stub(override_type(type_repr = "NewAddressKind | str"))]
        address_kind: PyNewAddressKind,
    ) -> PyResult<Bound<'py, PyAny>> {
        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .accounts_create_new_address(
                    AccountId::from_hex(&account_id)
                        .map_err(|err| PyException::new_err(err.to_string()))?,
                    address_kind.into(),
                )
                .await
                .into_py_result()?;

            Ok(PyAddress::from(resp.address))
        })
    }

    #[gen_stub(override_return_type(type_repr = "GeneratorSummary"))]
    #[pyo3(signature = (account_id, priority_fee_sompi, fee_rate=None, payload=None, destination=None))]
    pub fn accounts_estimate<'py>(
        &self,
        py: Python<'py>,
        account_id: String,
        #[gen_stub(override_type(type_repr = "Fees | dict"))] priority_fee_sompi: PyFees,
        fee_rate: Option<f64>,
        #[gen_stub(override_type(type_repr = "None | str | bytes | list[int]"))] payload: Option<
            PyBinary,
        >,
        destination: Option<Vec<PyPaymentOutput>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let destination = match destination {
            Some(outputs) => {
                let outputs = outputs
                    .into_iter()
                    .map(PaymentOutput::from)
                    .collect::<Vec<PaymentOutput>>();
                PaymentDestination::PaymentOutputs(PaymentOutputs { outputs })
            }
            None => PaymentDestination::Change,
        };

        let request = AccountsEstimateRequest {
            account_id: AccountId::from_hex(&account_id)
                .map_err(|err| PyException::new_err(err.to_string()))?,
            destination,
            fee_rate,
            priority_fee_sompi: priority_fee_sompi.into(),
            payload: payload.map(|p| p.data),
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .accounts_estimate_call(request)
                .await
                .into_py_result()?;
            Ok(PyGeneratorSummary::from(resp.generator_summary))
        })
    }

    #[gen_stub(override_return_type(type_repr = "GeneratorSummary"))]
    #[pyo3(signature = (wallet_secret, account_id, priority_fee_sompi, payment_secret=None, fee_rate=None, payload=None, destination=None))]
    pub fn accounts_send<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        account_id: String,
        #[gen_stub(override_type(type_repr = "Fees | dict"))] priority_fee_sompi: PyFees,
        payment_secret: Option<String>,
        fee_rate: Option<f64>,
        #[gen_stub(override_type(type_repr = "None | str | bytes | list[int]"))] payload: Option<
            PyBinary,
        >,
        destination: Option<Vec<PyPaymentOutput>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let destination = match destination {
            Some(outputs) => {
                let outputs = outputs
                    .into_iter()
                    .map(PaymentOutput::from)
                    .collect::<Vec<PaymentOutput>>();
                PaymentDestination::PaymentOutputs(PaymentOutputs { outputs })
            }
            None => PaymentDestination::Change,
        };

        let request = AccountsSendRequest {
            wallet_secret: wallet_secret.into(),
            payment_secret: payment_secret.map(Secret::from),
            account_id: AccountId::from_hex(&account_id)
                .map_err(|err| PyException::new_err(err.to_string()))?,
            destination,
            fee_rate,
            priority_fee_sompi: priority_fee_sompi.into(),
            payload: payload.map(|p| p.data),
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet.accounts_send(request).await.into_py_result()?;
            Ok(PyGeneratorSummary::from(resp))
        })
    }

    #[gen_stub(override_return_type(type_repr = "dict"))]
    #[pyo3(signature = (account_id, addresses=None, min_amount_sompi=None))]
    pub fn accounts_get_utxos<'py>(
        &self,
        py: Python<'py>,
        account_id: String,
        #[gen_stub(override_type(type_repr = "None | typing.Sequence[Address | str]"))]
        addresses: Option<Vec<PyAddress>>,
        min_amount_sompi: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = AccountsGetUtxosRequest {
            account_id: AccountId::from_hex(&account_id)
                .map_err(|err| PyException::new_err(err.to_string()))?,
            addresses: addresses.map(|addrs| addrs.into_iter().map(Address::from).collect()),
            min_amount_sompi,
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .accounts_get_utxos_call(request)
                .await
                .into_py_result()?;

            Python::attach(|py| Ok(serde_pyobject::to_pyobject(py, &resp.utxos)?.unbind()))
        })
    }

    #[gen_stub(override_return_type(type_repr = "dict"))]
    #[pyo3(signature = (wallet_secret, source_account_id, destination_account_id, transfer_amount_sompi, payment_secret=None, fee_rate=None, priority_fee_sompi=None))]
    pub fn accounts_transfer<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        source_account_id: String,
        destination_account_id: String,
        transfer_amount_sompi: u64,
        payment_secret: Option<String>,
        fee_rate: Option<f64>,
        #[gen_stub(override_type(type_repr = "None | Fees | dict"))] priority_fee_sompi: Option<
            PyFees,
        >,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = AccountsTransferRequest {
            source_account_id: AccountId::from_hex(&source_account_id)
                .map_err(|err| PyException::new_err(err.to_string()))?,
            destination_account_id: AccountId::from_hex(&destination_account_id)
                .map_err(|err| PyException::new_err(err.to_string()))?,
            wallet_secret: wallet_secret.into(),
            payment_secret: payment_secret.map(Secret::from),
            transfer_amount_sompi,
            fee_rate,
            priority_fee_sompi: priority_fee_sompi.map(PyFees::into),
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .accounts_transfer_call(request)
                .await
                .into_py_result()?;

            Python::attach(|py| {
                // let dict = PyDict::new(py);
                // dict.set_item(
                //     "generator_summary",
                //     PyGeneratorSummary::from(resp.generator_summary),
                // );
                // dict.set_item("transaction_ids", resp.transaction_ids);
                // Ok(dict)
                Ok(serde_pyobject::to_pyobject(py, &resp)?.unbind())
            })
        })
    }

    #[gen_stub(override_return_type(type_repr = "list[str]"))]
    #[pyo3(signature = (wallet_secret, account_id, address_type, address_index, script_sig, commit_amount_sompi, reveal_fee_sompi, payment_secret=None, fee_rate=None, payload=None))]
    pub fn accounts_commit_reveal<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        account_id: String,
        #[gen_stub(override_type(type_repr = "CommitRevealAddressKind | str"))]
        address_type: PyCommitRevealAddressKind,
        address_index: u32,
        #[gen_stub(override_type(type_repr = "str | bytes | list[int]"))] script_sig: PyBinary,
        commit_amount_sompi: u64,
        reveal_fee_sompi: u64,
        payment_secret: Option<String>,
        fee_rate: Option<f64>,
        #[gen_stub(override_type(type_repr = "None | str | bytes | list[int]"))] payload: Option<
            PyBinary,
        >,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = AccountsCommitRevealRequest {
            account_id: AccountId::from_hex(&account_id)
                .map_err(|err| PyException::new_err(err.to_string()))?,
            address_type: address_type.into(),
            address_index,
            script_sig: script_sig.data,
            wallet_secret: wallet_secret.into(),
            commit_amount_sompi,
            payment_secret: payment_secret.map(Secret::from),
            fee_rate,
            reveal_fee_sompi,
            payload: payload.map(|p| p.data),
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .accounts_commit_reveal_call(request)
                .await
                .into_py_result()?;

            Python::attach(
                |py| Ok(serde_pyobject::to_pyobject(py, &resp.transaction_ids)?.unbind()),
            )
        })
    }

    #[gen_stub(override_return_type(type_repr = "list[str]"))]
    #[pyo3(signature = (wallet_secret, account_id, script_sig, reveal_fee_sompi, payment_secret=None, fee_rate=None, payload=None, start_destination=None, end_destination=None))]
    pub fn accounts_commit_reveal_manual<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        account_id: String,
        #[gen_stub(override_type(type_repr = "str | bytes | list[int]"))] script_sig: PyBinary,
        reveal_fee_sompi: u64,
        payment_secret: Option<String>,
        fee_rate: Option<f64>,
        #[gen_stub(override_type(type_repr = "None | str | bytes | list[int]"))] payload: Option<
            PyBinary,
        >,
        start_destination: Option<Vec<PyPaymentOutput>>,
        end_destination: Option<Vec<PyPaymentOutput>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let start_destination = match start_destination {
            Some(outputs) => {
                let outputs = outputs
                    .into_iter()
                    .map(PaymentOutput::from)
                    .collect::<Vec<PaymentOutput>>();
                PaymentDestination::PaymentOutputs(PaymentOutputs { outputs })
            }
            None => PaymentDestination::Change,
        };

        let end_destination = match end_destination {
            Some(outputs) => {
                let outputs = outputs
                    .into_iter()
                    .map(PaymentOutput::from)
                    .collect::<Vec<PaymentOutput>>();
                PaymentDestination::PaymentOutputs(PaymentOutputs { outputs })
            }
            None => PaymentDestination::Change,
        };

        let request = AccountsCommitRevealManualRequest {
            account_id: AccountId::from_hex(&account_id)
                .map_err(|err| PyException::new_err(err.to_string()))?,
            script_sig: script_sig.data,
            start_destination,
            end_destination,
            wallet_secret: wallet_secret.into(),
            payment_secret: payment_secret.map(Secret::from),
            fee_rate,
            reveal_fee_sompi,
            payload: payload.map(|p| p.data),
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .accounts_commit_reveal_manual_call(request)
                .await
                .into_py_result()?;

            Python::attach(
                |py| Ok(serde_pyobject::to_pyobject(py, &resp.transaction_ids)?.unbind()),
            )
        })
    }
}

// Wallet API batch, flush, retain_context, get_status functions
#[gen_stub_pymethods]
#[pymethods]
impl PyWallet {
    #[gen_stub(override_return_type(type_repr = "None"))]
    pub fn batch<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet.batch_call(BatchRequest {}).await.into_py_result()?;
            Ok(())
        })
    }

    #[gen_stub(override_return_type(type_repr = "None"))]
    pub fn flush<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = FlushRequest {
            wallet_secret: wallet_secret.into(),
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet.flush_call(request).await.into_py_result()?;
            Ok(())
        })
    }

    #[gen_stub(override_return_type(type_repr = "None"))]
    #[pyo3(signature = (name, data=None))]
    pub fn retain_context<'py>(
        &self,
        py: Python<'py>,
        name: String,
        #[gen_stub(override_type(type_repr = "None | str | bytes | list[int]"))] data: Option<
            PyBinary,
        >,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = RetainContextRequest {
            name,
            data: data.map(|d| d.data),
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet.retain_context_call(request).await.into_py_result()?;
            Ok(())
        })
    }

    #[gen_stub(override_return_type(type_repr = "dict"))]
    #[pyo3(signature = (name=None))]
    pub fn get_status<'py>(
        &self,
        py: Python<'py>,
        name: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = GetStatusRequest { name };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet.get_status_call(request).await.into_py_result()?;

            Python::attach(|py| Ok(serde_pyobject::to_pyobject(py, &resp)?.unbind()))
        })
    }

    #[gen_stub(override_return_type(type_repr = "None"))]
    pub fn address_book_enumerate<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet
                .address_book_enumerate_call(AddressBookEnumerateRequest {})
                .await
                .into_py_result()?;
            Ok(())
        })
    }
}

// Wallet API transactions_ functions
#[gen_stub_pymethods]
#[pymethods]
impl PyWallet {
    #[gen_stub(override_return_type(type_repr = "dict"))]
    #[pyo3(signature = (account_id, network_id, start, end, filter=None))]
    pub fn transactions_data_get<'py>(
        &self,
        py: Python<'py>,
        account_id: String,
        #[gen_stub(override_type(type_repr = "NetworkId | str"))] network_id: PyNetworkId,
        start: u64,
        end: u64,
        #[gen_stub(override_type(type_repr = "None | typing.Sequence[TransactionKind | str]"))]
        filter: Option<Vec<PyTransactionKind>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = TransactionsDataGetRequest {
            account_id: AccountId::from_hex(&account_id)
                .map_err(|err| PyException::new_err(err.to_string()))?,
            network_id: network_id.into(),
            filter: filter.map(|kinds| {
                kinds
                    .into_iter()
                    .map(TransactionKind::from)
                    .collect::<Vec<TransactionKind>>()
            }),
            start,
            end,
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .transactions_data_get_call(request)
                .await
                .into_py_result()?;

            Python::attach(|py| Ok(serde_pyobject::to_pyobject(py, &resp)?.unbind()))
        })
    }

    #[gen_stub(override_return_type(type_repr = "None"))]
    #[pyo3(signature = (account_id, network_id, transaction_id, note=None))]
    pub fn transactions_replace_note<'py>(
        &self,
        py: Python<'py>,
        account_id: String,
        #[gen_stub(override_type(type_repr = "NetworkId | str"))] network_id: PyNetworkId,
        transaction_id: String,
        note: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = TransactionsReplaceNoteRequest {
            account_id: AccountId::from_hex(&account_id)
                .map_err(|err| PyException::new_err(err.to_string()))?,
            network_id: network_id.into(),
            transaction_id: Hash::from_hex(&transaction_id)
                .map_err(|err| PyException::new_err(err.to_string()))?,
            note,
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet
                .transactions_replace_note_call(request)
                .await
                .into_py_result()?;
            Ok(())
        })
    }

    #[gen_stub(override_return_type(type_repr = "None"))]
    #[pyo3(signature = (account_id, network_id, transaction_id, metadata=None))]
    pub fn transactions_replace_metadata<'py>(
        &self,
        py: Python<'py>,
        account_id: String,
        #[gen_stub(override_type(type_repr = "NetworkId | str"))] network_id: PyNetworkId,
        transaction_id: String,
        metadata: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = TransactionsReplaceMetadataRequest {
            account_id: AccountId::from_hex(&account_id)
                .map_err(|err| PyException::new_err(err.to_string()))?,
            network_id: network_id.into(),
            transaction_id: Hash::from_hex(&transaction_id)
                .map_err(|err| PyException::new_err(err.to_string()))?,
            metadata,
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet
                .transactions_replace_metadata_call(request)
                .await
                .into_py_result()?;
            Ok(())
        })
    }
}

// Wallet API fee_rate_ functions
#[gen_stub_pymethods]
#[pymethods]
impl PyWallet {
    #[gen_stub(override_return_type(type_repr = "dict"))]
    pub fn fee_rate_estimate<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .fee_rate_estimate_call(FeeRateEstimateRequest {})
                .await
                .into_py_result()?;

            Python::attach(|py| Ok(serde_pyobject::to_pyobject(py, &resp)?.unbind()))
        })
    }

    #[gen_stub(override_return_type(type_repr = "None"))]
    pub fn fee_rate_poller_enable<'py>(
        &self,
        py: Python<'py>,
        interval_seconds: u64,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = FeeRatePollerEnableRequest { interval_seconds };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet
                .fee_rate_poller_enable_call(request)
                .await
                .into_py_result()?;
            Ok(())
        })
    }

    #[gen_stub(override_return_type(type_repr = "None"))]
    pub fn fee_rate_poller_disable<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet
                .fee_rate_poller_disable_call(FeeRatePollerDisableRequest {})
                .await
                .into_py_result()?;
            Ok(())
        })
    }
}
