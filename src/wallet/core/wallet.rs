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
    api::*,
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
    /// Create a new Wallet instance.
    ///
    /// Constructs a wallet backed by the local file store and an internal
    /// wRPC client. The wallet is created in a closed state — call
    /// `wallet_open` (or `wallet_create`) to open or initialize a wallet file.
    ///
    /// Args:
    ///     network_id: The network to operate on. May be a NetworkId or string.
    ///     encoding: The wRPC encoding (Borsh or JSON). Defaults to Borsh.
    ///     url: Optional explicit wRPC server URL.
    ///     resolver: Optional Resolver to discover wRPC endpoints.
    ///
    /// Returns:
    ///     Wallet: A new Wallet instance.
    ///
    /// Raises:
    ///     Exception: If the underlying wallet store or RPC client cannot be initialized.
    #[new]
    #[pyo3(signature = (network_id=None, encoding=None, url=None, resolver=None))]
    pub fn new(
        // resident: bool, TODO
        #[gen_stub(override_type(type_repr = "None | NetworkId | str"))] network_id: Option<
            PyNetworkId,
        >,
        #[gen_stub(override_type(type_repr = "None | Encoding | str"))] encoding: Option<
            PyEncoding,
        >,
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

    /// The underlying wRPC client used by this wallet.
    #[getter]
    pub fn get_rpc(&self) -> PyRpcClient {
        self.inner.rpc.clone()
    }

    /// Whether a wallet file is currently open.
    #[getter]
    pub fn get_is_open(&self) -> bool {
        self.wallet().is_open()
    }

    /// Whether the wallet's UTXO state is synced with the network.
    #[getter]
    pub fn get_is_synced(&self) -> bool {
        self.wallet().is_synced()
    }

    /// The descriptor of the currently open wallet, or None if no wallet is open.
    #[getter]
    pub fn get_descriptor(&self) -> Option<PyWalletDescriptor> {
        self.wallet().descriptor().map(PyWalletDescriptor::from)
    }

    /// Check if a wallet file exists in the local store.
    ///
    /// Args:
    ///     name: Optional wallet filename to check. If None, checks the default wallet.
    ///
    /// Returns:
    ///     bool: True if the wallet file exists, False otherwise.
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

    /// Start the wallet runtime and event notification task.
    ///
    /// Spawns the background task that dispatches wallet events to any
    /// registered Python listeners. Must be called before opening a wallet.
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

    /// Stop the wallet runtime and event notification task.
    ///
    /// Tears down the background notification task and stops the wallet's
    /// internal services. Should be called before disposing of the Wallet.
    #[gen_stub(override_return_type(type_repr = "None"))]
    pub fn stop<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let slf = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            slf.stop_notification_task().await.into_py_result()?;

            slf.wallet().stop().await.into_py_result()?;
            Ok(())
        })
    }

    /// Connect the wallet's wRPC client to a Kaspa node.
    ///
    /// Args:
    ///     block_async_connect: If True, await connection completion before resolving.
    ///     strategy: Optional connection strategy ("retry" or "fallback").
    ///     url: Optional explicit wRPC server URL override.
    ///     timeout_duration: Optional connection timeout in milliseconds.
    ///     retry_interval: Optional retry interval in milliseconds.
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

    /// Disconnect the wallet's wRPC client from the Kaspa node.
    #[gen_stub(override_return_type(type_repr = "None"))]
    pub fn disconnect<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner.rpc.disconnect(py)
    }

    /// Register a Python callback for a wallet event.
    ///
    /// Args:
    ///     event: The WalletEventType (or string) to listen for. Use "all" to receive every event.
    ///     callback: A callable invoked with `(event, *args, **kwargs)` when the event fires.
    ///     *args: Positional arguments forwarded to the callback.
    ///     **kwargs: Keyword arguments forwarded to the callback.
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

    /// Remove previously registered event listener(s).
    ///
    /// Args:
    ///     event: The WalletEventType (or string) to remove listeners from.
    ///         Use "all" to operate across every event kind.
    ///     callback: Optional specific callback to remove. If None, removes all
    ///         callbacks for the given event.
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

    /// Set the network id used by the wallet runtime.
    ///
    /// Args:
    ///     network_id: The NetworkId (or string) to bind the wallet to.
    ///
    /// Raises:
    ///     Exception: If the wallet rejects the network change.
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
    /// Enumerate all wallet files in the local store.
    ///
    /// Returns:
    ///     list[WalletDescriptor]: Descriptors for each wallet file found.
    #[gen_stub(override_return_type(type_repr = "list[WalletDescriptor]"))]
    pub fn wallet_enumerate<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::PyAny>> {
        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .wallet_enumerate_call(WalletEnumerateRequest {})
                .await
                .into_py_result()?;
            let descriptors: Vec<PyWalletDescriptor> = resp
                .wallet_descriptors
                .iter()
                .map(PyWalletDescriptor::from)
                .collect();
            Ok(descriptors)
        })
    }

    /// Create a new wallet file.
    ///
    /// Args:
    ///     wallet_secret: Password used to encrypt the wallet.
    ///     filename: Optional filename for the new wallet. Defaults to the standard wallet name.
    ///     overwrite_wallet_storage: If True, overwrite an existing wallet at the same path.
    ///     title: Optional human-readable wallet title.
    ///     user_hint: Optional password hint stored alongside the wallet.
    ///
    /// Returns:
    ///     dict: The wallet creation response, including descriptor and storage info.
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
        let request = WalletCreateRequest {
            wallet_secret: wallet_secret.into(),
            wallet_args: WalletCreateArgs::new(
                title,
                filename,
                EncryptionKind::default(),
                user_hint.map(Hint::from),
                overwrite_wallet_storage.unwrap_or(false),
            ),
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet.wallet_create_call(request).await.into_py_result()?;

            Python::attach(|py| Ok(serde_pyobject::to_pyobject(py, &resp)?.unbind()))
        })
    }

    /// Open an existing wallet file.
    ///
    /// Args:
    ///     wallet_secret: Password used to decrypt the wallet.
    ///     account_descriptors: If True, include account descriptors in the response.
    ///     filename: Optional filename to open. Defaults to the standard wallet name.
    ///
    /// Returns:
    ///     dict: The wallet open response, optionally including account descriptors.
    #[gen_stub(override_return_type(type_repr = "dict"))]
    #[pyo3(signature = (wallet_secret, account_descriptors, filename=None))]
    pub fn wallet_open<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        account_descriptors: bool,
        filename: Option<String>,
    ) -> PyResult<Bound<'py, pyo3::PyAny>> {
        let request = WalletOpenRequest {
            wallet_secret: wallet_secret.into(),
            filename,
            account_descriptors,
            legacy_accounts: None,
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet.wallet_open_call(request).await.into_py_result()?;

            Python::attach(|py| Ok(serde_pyobject::to_pyobject(py, &resp)?.unbind()))
        })
    }

    /// Close the currently open wallet file.
    #[gen_stub(override_return_type(type_repr = "None"))]
    pub fn wallet_close<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::PyAny>> {
        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet
                .wallet_close_call(WalletCloseRequest {})
                .await
                .into_py_result()?;

            Ok(())
        })
    }

    /// Reload the wallet from disk.
    ///
    /// Args:
    ///     reactivate: If True, re-activate previously active accounts after reload.
    #[gen_stub(override_return_type(type_repr = "None"))]
    pub fn wallet_reload<'py>(
        &self,
        py: Python<'py>,
        reactivate: bool,
    ) -> PyResult<Bound<'py, pyo3::PyAny>> {
        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet
                .wallet_reload_call(WalletReloadRequest { reactivate })
                .await
                .into_py_result()?;

            Ok(())
        })
    }

    /// Rename the currently open wallet (title and/or on-disk filename).
    ///
    /// Args:
    ///     wallet_secret: Password for the open wallet.
    ///     title: New human-readable title, or None to leave unchanged.
    ///     filename: New on-disk filename, or None to leave unchanged.
    #[gen_stub(override_return_type(type_repr = "None"))]
    #[pyo3(signature = (wallet_secret, title=None, filename=None))]
    pub fn wallet_rename<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        title: Option<String>,
        filename: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = WalletRenameRequest {
            title,
            filename,
            wallet_secret: wallet_secret.into(),
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet.wallet_rename_call(request).await.into_py_result()?;
            Ok(())
        })
    }

    /// Change the password protecting the currently open wallet.
    ///
    /// Args:
    ///     old_wallet_secret: The current wallet password.
    ///     new_wallet_secret: The new password to set.
    #[gen_stub(override_return_type(type_repr = "None"))]
    pub fn wallet_change_secret<'py>(
        &self,
        py: Python<'py>,
        old_wallet_secret: String,
        new_wallet_secret: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = WalletChangeSecretRequest {
            old_wallet_secret: old_wallet_secret.into(),
            new_wallet_secret: new_wallet_secret.into(),
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet
                .wallet_change_secret_call(request)
                .await
                .into_py_result()?;
            Ok(())
        })
    }

    /// Export the wallet's encrypted data as raw bytes.
    ///
    /// Args:
    ///     wallet_secret: Password for the open wallet.
    ///     include_transactions: If True, include stored transaction history in the export.
    ///
    /// Returns:
    ///     bytes: The encrypted wallet payload, suitable for backup or transfer.
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

    /// Import a previously exported wallet payload.
    ///
    /// Args:
    ///     wallet_secret: Password used to decrypt the import payload.
    ///     wallet_data: The encrypted wallet bytes produced by `wallet_export`.
    ///
    /// Returns:
    ///     dict: The wallet import response, including the resulting descriptor.
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
    /// Enumerate all private key data entries stored in the open wallet.
    ///
    /// Returns:
    ///     list[PrvKeyDataInfo]: Metadata for every stored private key data entry.
    #[gen_stub(override_return_type(type_repr = "list[PrvKeyDataInfo]"))]
    pub fn prv_key_data_enumerate<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .prv_key_data_enumerate_call(PrvKeyDataEnumerateRequest {})
                .await
                .into_py_result()?
                .prv_key_data_list
                .into_iter()
                .map(|v| PyPrvKeyDataInfo::from(v.as_ref()))
                .collect::<Vec<PyPrvKeyDataInfo>>();

            Ok(resp)
        })
    }

    /// Create and store a new private key data entry.
    ///
    /// Args:
    ///     wallet_secret: Password for the open wallet.
    ///     secret: The secret material (mnemonic phrase, hex seed, or extended key).
    ///     kind: The variant kind of `secret` (Mnemonic, Bip39Seed, ExtendedPrivateKey, or SecretKey).
    ///     payment_secret: Optional additional secret used to encrypt the entry.
    ///     name: Optional human-readable name for the entry.
    ///
    /// Returns:
    ///     str: The hex-encoded id of the newly created private key data entry.
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
        let request = PrvKeyDataCreateRequest {
            wallet_secret: wallet_secret.into(),
            prv_key_data_args: PrvKeyDataCreateArgs::new(
                name,
                payment_secret.map(Secret::from),
                secret.into(),
                kind.into(),
            ),
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .prv_key_data_create_call(request)
                .await
                .into_py_result()?;
            Ok(resp.prv_key_data_id.to_hex())
        })
    }

    /// Remove a private key data entry from the wallet.
    ///
    /// Args:
    ///     wallet_secret: Password for the open wallet.
    ///     prv_key_data_id: Hex-encoded id of the entry to remove.
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

    /// Fetch metadata for a single private key data entry.
    ///
    /// Args:
    ///     wallet_secret: Password for the open wallet.
    ///     prv_key_data_id: Hex-encoded id of the entry to fetch.
    ///
    /// Returns:
    ///     PrvKeyDataInfo: Metadata for the entry.
    ///
    /// Raises:
    ///     Exception: If no entry exists with the given id.
    #[gen_stub(override_return_type(type_repr = "PrvKeyDataInfo"))]
    pub fn prv_key_data_get<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        prv_key_data_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = PrvKeyDataGetRequest {
            prv_key_data_id: PrvKeyDataId::from_hex(&prv_key_data_id)
                .map_err(|err| PyException::new_err(err.to_string()))?,
            wallet_secret: wallet_secret.into(),
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .prv_key_data_get_call(request)
                .await
                .into_py_result()?;

            let prv_key_data = resp
                .prv_key_data
                .ok_or_else(|| PyException::new_err("private key not found"))?;
            Ok(PyPrvKeyDataInfo::from(PrvKeyDataInfo::from(&prv_key_data)))
        })
    }
}

// Wallet API account_ functions
#[gen_stub_pymethods]
#[pymethods]
impl PyWallet {
    /// Enumerate all accounts in the open wallet.
    ///
    /// Returns:
    ///     list[AccountDescriptor]: Descriptors for every account in the wallet.
    #[gen_stub(override_return_type(type_repr = "list[AccountDescriptor]"))]
    pub fn accounts_enumerate<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let accounts = wallet
                .accounts_enumerate_call(AccountsEnumerateRequest {})
                .await
                .into_py_result()?
                .account_descriptors
                .into_iter()
                .map(PyAccountDescriptor::from)
                .collect::<Vec<PyAccountDescriptor>>();
            Ok(accounts)
        })
    }

    /// Create a new BIP32 (HD) account from existing private key data.
    ///
    /// Args:
    ///     wallet_secret: Password for the open wallet.
    ///     prv_key_data_id: Hex-encoded id of the private key data entry to derive from.
    ///     payment_secret: Optional payment secret if the private key data is encrypted with one.
    ///     account_name: Optional human-readable name for the account.
    ///     account_index: Optional explicit BIP32 account index. Defaults to the next available.
    ///
    /// Returns:
    ///     AccountDescriptor: Descriptor of the newly created account.
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
        let request = AccountsCreateRequest {
            wallet_secret: wallet_secret.into(),
            account_create_args: AccountCreateArgs::Bip32 {
                prv_key_data_args: PrvKeyDataArgs {
                    prv_key_data_id: PrvKeyDataId::from_hex(&prv_key_data_id)
                        .map_err(|err| PyException::new_err(err.to_string()))?,
                    payment_secret: payment_secret.map(Secret::from),
                },
                account_args: AccountCreateArgsBip32 {
                    account_name,
                    account_index,
                },
            },
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .accounts_create_call(request)
                .await
                .into_py_result()?;
            Ok(PyAccountDescriptor::from(resp.account_descriptor))
        })
    }

    /// Create a new keypair (single-key) account from existing private key data.
    ///
    /// Args:
    ///     wallet_secret: Password for the open wallet.
    ///     prv_key_data_id: Hex-encoded id of the private key data entry to use.
    ///     ecdsa: If True, derive an ECDSA address; otherwise derive a Schnorr address.
    ///     account_name: Optional human-readable name for the account.
    ///
    /// Returns:
    ///     AccountDescriptor: Descriptor of the newly created account.
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
        let request = AccountsCreateRequest {
            wallet_secret: wallet_secret.into(),
            account_create_args: AccountCreateArgs::Keypair {
                prv_key_data_id: PrvKeyDataId::from_hex(&prv_key_data_id)
                    .map_err(|err| PyException::new_err(err.to_string()))?,
                account_name,
                ecdsa,
            },
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .accounts_create_call(request)
                .await
                .into_py_result()?;
            Ok(PyAccountDescriptor::from(resp.account_descriptor))
        })
    }

    /// Import a BIP32 (HD) account from existing private key data.
    ///
    /// Like `accounts_create_bip32`, but uses the import code path which performs
    /// address discovery before adding the account.
    ///
    /// Args:
    ///     wallet_secret: Password for the open wallet.
    ///     prv_key_data_id: Hex-encoded id of the private key data entry to derive from.
    ///     payment_secret: Optional payment secret if the private key data is encrypted with one.
    ///     account_name: Optional human-readable name for the account.
    ///     account_index: Optional explicit BIP32 account index.
    ///
    /// Returns:
    ///     AccountDescriptor: Descriptor of the imported account.
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

    /// Import a keypair (single-key) account from existing private key data.
    ///
    /// Args:
    ///     wallet_secret: Password for the open wallet.
    ///     prv_key_data_id: Hex-encoded id of the private key data entry to use.
    ///     ecdsa: If True, derive an ECDSA address; otherwise derive a Schnorr address.
    ///     account_name: Optional human-readable name for the account.
    ///
    /// Returns:
    ///     AccountDescriptor: Descriptor of the imported account.
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

    /// Rename an account.
    ///
    /// Args:
    ///     wallet_secret: Password for the open wallet.
    ///     account_id: Hex-encoded id of the account to rename.
    ///     name: New account name, or None to clear the name.
    #[gen_stub(override_return_type(type_repr = "None"))]
    #[pyo3(signature = (wallet_secret, account_id, name=None))]
    pub fn accounts_rename<'py>(
        &self,
        py: Python<'py>,
        wallet_secret: String,
        account_id: String,
        name: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = AccountsRenameRequest {
            account_id: AccountId::from_hex(&account_id)
                .map_err(|err| PyException::new_err(err.to_string()))?,
            name,
            wallet_secret: wallet_secret.into(),
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet
                .accounts_rename_call(request)
                .await
                .into_py_result()?;

            Ok(())
        })
    }

    /// Scan a BIP39 mnemonic for previously used accounts and addresses.
    ///
    /// Performs a discovery sweep to determine the highest account index that
    /// has been used on-chain. Useful when restoring a wallet from a mnemonic.
    ///
    /// Args:
    ///     discovery_kind: The discovery scheme (AccountsDiscoveryKind or string).
    ///     address_scan_extent: How many consecutive unused addresses to scan before stopping.
    ///     account_scan_extent: How many consecutive unused accounts to scan before stopping.
    ///     bip39_mnemonic: The BIP39 mnemonic phrase to scan.
    ///     bip39_passphrase: Optional BIP39 passphrase.
    ///
    /// Returns:
    ///     int: The last account index found to be in use.
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

    /// Ensure a default account of the given kind exists, creating one if needed.
    ///
    /// Args:
    ///     wallet_secret: Password for the open wallet.
    ///     account_kind: The AccountKind of the default account to ensure.
    ///     payment_secret: Optional payment secret used when generating new key data.
    ///     mnemonic_phrase: Optional mnemonic phrase to seed the account.
    ///
    /// Returns:
    ///     AccountDescriptor: Descriptor of the existing or newly created default account.
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
        let request = AccountsEnsureDefaultRequest {
            wallet_secret: wallet_secret.into(),
            payment_secret: payment_secret.map(Secret::from),
            account_kind: account_kind.into(),
            mnemonic_phrase: mnemonic_phrase.map(Secret::from),
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .accounts_ensure_default_call(request)
                .await
                .into_py_result()?;

            Ok(PyAccountDescriptor::from(resp.account_descriptor))
        })
    }

    /// Activate one or more accounts so they begin tracking UTXOs.
    ///
    /// Args:
    ///     account_ids: Optional list of hex-encoded account ids. If None, activates all accounts.
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

        let request = AccountsActivateRequest { account_ids };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet
                .accounts_activate_call(request)
                .await
                .into_py_result()?;

            Ok(())
        })
    }

    /// Deactivate one or more accounts so they stop tracking UTXOs.
    ///
    /// Args:
    ///     account_ids: Optional list of hex-encoded account ids. If None, deactivates all accounts.
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

        let request = AccountsDeactivateRequest { account_ids };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet
                .accounts_deactivate_call(request)
                .await
                .into_py_result()?;

            Ok(())
        })
    }

    /// Verify that an account exists in the open wallet.
    ///
    /// Args:
    ///     account_id: Hex-encoded id of the account to look up.
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

    /// Generate a new receive or change address for an account.
    ///
    /// Args:
    ///     account_id: Hex-encoded id of the account.
    ///     address_kind: The NewAddressKind (Receive or Change) to derive.
    ///
    /// Returns:
    ///     Address: The newly derived address.
    #[gen_stub(override_return_type(type_repr = "Address"))]
    pub fn accounts_create_new_address<'py>(
        &self,
        py: Python<'py>,
        account_id: String,
        #[gen_stub(override_type(type_repr = "NewAddressKind | str"))]
        address_kind: PyNewAddressKind,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = AccountsCreateNewAddressRequest {
            account_id: AccountId::from_hex(&account_id)
                .map_err(|err| PyException::new_err(err.to_string()))?,
            kind: address_kind.into(),
        };

        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let resp = wallet
                .accounts_create_new_address_call(request)
                .await
                .into_py_result()?;

            Ok(PyAddress::from(resp.address))
        })
    }

    /// Estimate the fees and structure of a prospective send without submitting.
    ///
    /// Runs the transaction generator against the account's UTXOs to produce
    /// a summary of what a real send would look like.
    ///
    /// Args:
    ///     account_id: Hex-encoded id of the source account.
    ///     priority_fee_sompi: Priority fee specification (Fees object or dict).
    ///     fee_rate: Optional explicit fee rate (sompi per gram of mass).
    ///     payload: Optional binary payload to embed in the transaction.
    ///     destination: Optional list of PaymentOutputs. If None, sends to change.
    ///
    /// Returns:
    ///     GeneratorSummary: Summary of the estimated transaction(s).
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

    /// Send funds from an account, signing and submitting the resulting transactions.
    ///
    /// Args:
    ///     wallet_secret: Password for the open wallet.
    ///     account_id: Hex-encoded id of the source account.
    ///     priority_fee_sompi: Priority fee specification (Fees object or dict).
    ///     payment_secret: Optional payment secret if the source key data is encrypted with one.
    ///     fee_rate: Optional explicit fee rate (sompi per gram of mass).
    ///     payload: Optional binary payload to embed in the transaction.
    ///     destination: Optional list of PaymentOutputs. If None, sends to change.
    ///
    /// Returns:
    ///     GeneratorSummary: Summary of the submitted transaction(s).
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
            let resp = wallet.accounts_send_call(request).await.into_py_result()?;
            Ok(PyGeneratorSummary::from(resp.generator_summary))
        })
    }

    /// List UTXOs available to an account, optionally filtered.
    ///
    /// Args:
    ///     account_id: Hex-encoded id of the account.
    ///     addresses: Optional list of Address objects to restrict results to.
    ///     min_amount_sompi: Optional minimum UTXO value to include, in sompi.
    ///
    /// Returns:
    ///     dict: A serialized list of UTXO entries belonging to the account.
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

    /// Transfer funds between two accounts in the same wallet.
    ///
    /// Args:
    ///     wallet_secret: Password for the open wallet.
    ///     source_account_id: Hex-encoded id of the sending account.
    ///     destination_account_id: Hex-encoded id of the receiving account.
    ///     transfer_amount_sompi: Amount to transfer in sompi.
    ///     payment_secret: Optional payment secret if the source key data is encrypted with one.
    ///     fee_rate: Optional explicit fee rate (sompi per gram of mass).
    ///     priority_fee_sompi: Optional priority fee specification.
    ///
    /// Returns:
    ///     dict: The transfer response, including generator summary and transaction ids.
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

    /// Execute a commit-reveal transaction pair against an account-derived address.
    ///
    /// Submits the commit transaction (locking funds to a P2SH script derived from
    /// `script_sig`) followed by the reveal transaction (spending those funds back).
    ///
    /// Args:
    ///     wallet_secret: Password for the open wallet.
    ///     account_id: Hex-encoded id of the source account.
    ///     address_type: The CommitRevealAddressKind selecting which derivation chain to use.
    ///     address_index: Derivation index for the commit address.
    ///     script_sig: Raw script bytes used to construct the commit P2SH.
    ///     commit_amount_sompi: Amount to lock in the commit transaction.
    ///     reveal_fee_sompi: Fee paid by the reveal transaction.
    ///     payment_secret: Optional payment secret if the source key data is encrypted with one.
    ///     fee_rate: Optional explicit fee rate for the commit transaction.
    ///     payload: Optional binary payload to embed in the reveal transaction.
    ///
    /// Returns:
    ///     list[str]: Hex-encoded ids of the submitted commit and reveal transactions.
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

    /// Execute a commit-reveal transaction pair with explicit start and end destinations.
    ///
    /// Lower-level variant of `accounts_commit_reveal` that lets the caller
    /// specify both the commit (start) and reveal (end) destinations directly
    /// instead of deriving an address from the account.
    ///
    /// Args:
    ///     wallet_secret: Password for the open wallet.
    ///     account_id: Hex-encoded id of the source account.
    ///     script_sig: Raw script bytes used to construct the commit P2SH.
    ///     reveal_fee_sompi: Fee paid by the reveal transaction.
    ///     payment_secret: Optional payment secret if the source key data is encrypted with one.
    ///     fee_rate: Optional explicit fee rate for the commit transaction.
    ///     payload: Optional binary payload to embed in the reveal transaction.
    ///     start_destination: Optional outputs for the commit transaction. Defaults to change.
    ///     end_destination: Optional outputs for the reveal transaction. Defaults to change.
    ///
    /// Returns:
    ///     list[str]: Hex-encoded ids of the submitted commit and reveal transactions.
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
    /// Begin a batched storage transaction.
    ///
    /// Subsequent wallet operations are coalesced into a single storage write
    /// until `flush` is called.
    #[gen_stub(override_return_type(type_repr = "None"))]
    pub fn batch<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let wallet = self.wallet().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            wallet.batch_call(BatchRequest {}).await.into_py_result()?;
            Ok(())
        })
    }

    /// Flush any pending batched changes to storage.
    ///
    /// Args:
    ///     wallet_secret: Password for the open wallet.
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

    /// Persist arbitrary named context data alongside the wallet.
    ///
    /// Args:
    ///     name: A name identifying the context entry.
    ///     data: Optional binary payload to associate with `name`. If None, the entry is cleared.
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

    /// Fetch the wallet runtime status.
    ///
    /// Args:
    ///     name: Optional wallet name. If None, reports the status of the active wallet.
    ///
    /// Returns:
    ///     dict: Status information including connection state, sync state, and selected network.
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

    /// Enumerate entries in the wallet address book.
    ///
    /// Note: this is currently a no-op placeholder that returns nothing; the
    /// underlying API is reserved for a future address book implementation.
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
    /// Fetch a window of stored transaction history for an account.
    ///
    /// Args:
    ///     account_id: Hex-encoded id of the account.
    ///     network_id: The network the transactions belong to.
    ///     start: Start index (inclusive) into the transaction history.
    ///     end: End index (exclusive) into the transaction history.
    ///     filter: Optional list of TransactionKind values to filter by.
    ///
    /// Returns:
    ///     dict: The transaction data response, including the matching transactions.
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

    /// Replace the user-provided note attached to a stored transaction.
    ///
    /// Args:
    ///     account_id: Hex-encoded id of the account that owns the transaction.
    ///     network_id: The network the transaction belongs to.
    ///     transaction_id: Hex-encoded id of the transaction to update.
    ///     note: New note text, or None to clear the note.
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

    /// Replace the user-provided metadata attached to a stored transaction.
    ///
    /// Args:
    ///     account_id: Hex-encoded id of the account that owns the transaction.
    ///     network_id: The network the transaction belongs to.
    ///     transaction_id: Hex-encoded id of the transaction to update.
    ///     metadata: New metadata string, or None to clear it.
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
    /// Fetch the current fee rate estimate from the connected node.
    ///
    /// Returns:
    ///     dict: Fee rate estimates at low, normal, and priority levels.
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

    /// Enable a background poller that periodically refreshes fee rate estimates.
    ///
    /// Args:
    ///     interval_seconds: How often the poller should query the node, in seconds.
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

    /// Disable the background fee rate poller, if running.
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
