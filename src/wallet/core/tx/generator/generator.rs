use super::super::super::imports::*;
use super::pending::PendingTransaction;
use super::summary::PyGeneratorSummary;
use crate::consensus::core::network::PyNetworkId;
use crate::{
    consensus::client::utxo::PyUtxoEntryReference, wallet::core::tx::payment::PyPaymentOutput,
    wallet::core::utxo::context::PyUtxoContext,
};
use kaspa_consensus_client::UtxoEntryReference;
use kaspa_wallet_core::result::Result;
use kaspa_wallet_core::tx::{
    Fees, PaymentDestination, PaymentOutput, PaymentOutputs, generator as native,
};
use kaspa_wallet_core::utxo::UtxoContext;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use workflow_core::prelude::Abortable;

/// UTXO entries collection for flexible input handling.
///
/// This type is not intended to be instantiated directly from Python.
/// It serves as a helper type that allows Rust functions to accept a list
/// of UTXO entries in multiple convenient forms.
///
/// Accepts:
///     list[UtxoEntryReference]: A list of UtxoEntryReference objects.
///     list[dict]: A list of dicts with UtxoEntryReference-compatible keys.
#[gen_stub_pyclass]
#[pyclass(name = "UtxoEntries")]
pub struct PyUtxoEntries {
    pub entries: Vec<UtxoEntryReference>,
}

impl<'py> FromPyObject<'_, 'py> for PyUtxoEntries {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        // Must be list
        let list = obj.cast::<PyList>()?;

        let entries = list
            .iter()
            .map(|item| {
                if let Ok(entry) = item.extract::<PyUtxoEntryReference>() {
                    Ok(entry)
                } else if let Ok(entry) = item.cast::<PyDict>() {
                    PyUtxoEntryReference::try_from(entry)
                } else {
                    Err(PyException::new_err(
                        "All entries must be UtxoEntryReference instance or compatible dict",
                    ))
                }
            })
            .collect::<PyResult<Vec<PyUtxoEntryReference>>>()?;

        let inner = entries.into_iter().map(UtxoEntryReference::from).collect();
        Ok(PyUtxoEntries { entries: inner })
    }
}

/// Payment outputs collection for flexible input handling.
///
/// This type is not intended to be instantiated directly from Python.
/// It serves as a helper type that allows Rust functions to accept a list
/// of payment outputs in multiple convenient forms.
///
/// Accepts:
///     list[PaymentOutput]: A list of PaymentOutput objects.
///     list[dict]: A list of dicts with `address` and `amount` keys.
#[gen_stub_pyclass]
#[pyclass(name = "Outputs")]
pub struct PyOutputs {
    pub outputs: Vec<PaymentOutput>,
}

impl<'py> FromPyObject<'_, 'py> for PyOutputs {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        // Must be list
        let list = obj.cast::<PyList>()?;

        let outputs = list
            .iter()
            .map(|item| {
                if let Ok(output) = item.extract::<PyPaymentOutput>() {
                    Ok(output)
                } else if let Ok(output) = item.cast::<PyDict>() {
                    PyPaymentOutput::try_from(output)
                } else {
                    Err(PyException::new_err(
                        "All outputs must be PaymentOutput instance or compatible dict",
                    ))
                }
            })
            .collect::<PyResult<Vec<PyPaymentOutput>>>()?;

        // TODO move into closure above
        let outputs = outputs.into_iter().map(PaymentOutput::from).collect();

        Ok(PyOutputs { outputs })
    }
}

/// Transaction generator for building and signing transactions.
///
/// Handles UTXO selection, fee calculation, change outputs, and transaction
/// splitting for large transfers.
#[gen_stub_pyclass]
#[pyclass(name = "Generator")]
pub struct PyGenerator(Arc<native::Generator>);

#[gen_stub_pymethods]
#[pymethods]
impl PyGenerator {
    /// Create a new transaction generator.
    ///
    /// Args:
    ///     network_id: The network to build transactions for (required for UTXO entries).
    ///     entries: UtxoContext or list of UTXO entries to spend from.
    ///     change_address: Address to send change to.
    ///     outputs: Optional list of payment outputs.
    ///     payload: Optional transaction payload (OP_RETURN data).
    ///     fee_rate: Optional fee rate multiplier.
    ///     priority_fee: Additional fee in sompi.
    ///     priority_entries: UTXOs to use first.
    ///     sig_op_count: Signature operations per input (default: 1).
    ///     minimum_signatures: For multisig fee estimation.
    ///
    /// Returns:
    ///     Generator: A new Generator instance.
    ///
    /// Raises:
    ///     Exception: If generator creation fails.
    #[new]
    #[pyo3(signature = (entries, change_address, network_id=None, outputs=None, payload=None, fee_rate=None, priority_fee=None, priority_entries=None, sig_op_count=None, minimum_signatures=None))]
    pub fn ctor(
        #[gen_stub(override_type(type_repr = "UtxoEntries | UtxoContext"))] entries: Bound<
            '_,
            PyAny,
        >,
        change_address: PyAddress,
        network_id: Option<PyNetworkId>,
        outputs: Option<PyOutputs>,
        payload: Option<PyBinary>,
        fee_rate: Option<f64>,
        priority_fee: Option<u64>,
        priority_entries: Option<PyUtxoEntries>,
        sig_op_count: Option<u8>,
        minimum_signatures: Option<u16>,
    ) -> PyResult<Self> {
        let source = parse_generator_source(entries)?;
        let settings = GeneratorSettings::new(
            outputs,
            change_address.into(),
            fee_rate,
            priority_fee,
            source,
            priority_entries.map(|p| p.entries),
            sig_op_count,
            minimum_signatures,
            payload.map(Into::into),
            network_id.map(Into::into),
        );

        let settings = match settings.source {
            GeneratorSource::UtxoEntries(utxo_entries) => {
                let change_address = settings.change_address.ok_or_else(|| {
                    PyException::new_err(
                        "changeAddress is required for Generator constructor with UTXO entries",
                    )
                })?;

                let network_id = settings.network_id.ok_or_else(|| {
                    PyException::new_err(
                        "networkId is required for Generator constructor with UTXO entries",
                    )
                })?;

                native::GeneratorSettings::try_new_with_iterator(
                    network_id,
                    Box::new(utxo_entries.into_iter()),
                    settings.priority_utxo_entries,
                    change_address,
                    settings.sig_op_count,
                    settings.minimum_signatures,
                    settings.final_transaction_destination,
                    settings.fee_rate,
                    settings.final_priority_fee,
                    settings.payload,
                    settings.multiplexer,
                )
                .map_err(|err| PyException::new_err(err.to_string()))?
            }
            GeneratorSource::UtxoContext(utxo_context) => {
                let change_address = settings.change_address.ok_or_else(|| {
                    PyException::new_err(
                        "changeAddress is required for Generator constructor with UTXO entries",
                    )
                })?;

                native::GeneratorSettings::try_new_with_context(
                    utxo_context,
                    settings.priority_utxo_entries,
                    change_address,
                    settings.sig_op_count,
                    settings.minimum_signatures,
                    settings.final_transaction_destination,
                    settings.fee_rate,
                    settings.final_priority_fee,
                    settings.payload,
                    settings.multiplexer,
                )
                .map_err(|err| PyException::new_err(err.to_string()))?
            }
        };

        let abortable = Abortable::default();
        let generator = native::Generator::try_new(settings, None, Some(&abortable))
            .map_err(|err| PyException::new_err(err.to_string()))?;

        Ok(Self(Arc::new(generator)))
    }

    /// Estimate the transaction without generating.
    ///
    /// Returns:
    ///     GeneratorSummary: A summary with fee, transaction count, and other details.
    ///
    /// Raises:
    ///     Exception: If estimation fails.
    pub fn estimate(&self) -> PyResult<PyGeneratorSummary> {
        self.0
            .iter()
            .collect::<Result<Vec<_>>>()
            .map_err(|err| PyException::new_err(err.to_string()))?;
        Ok(self.0.summary().into())
    }

    /// Get the summary after generation.
    ///
    /// Returns:
    ///     GeneratorSummary: The generation summary with fees and transaction details.
    pub fn summary(&self) -> PyGeneratorSummary {
        self.0.summary().into()
    }
}

impl PyGenerator {
    pub fn iter(&self) -> impl Iterator<Item = Result<native::PendingTransaction>> {
        self.0.iter()
    }

    #[allow(dead_code)]
    pub fn stream(&self) -> impl Stream<Item = Result<native::PendingTransaction>> {
        self.0.stream()
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyGenerator {
    /// Return self as an iterator.
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<Py<Self>> {
        Ok(slf.into())
    }

    /// Get the next pending transaction, or None if complete.
    ///
    /// Returns:
    ///     PendingTransaction | None: The next transaction to sign and submit.
    ///
    /// Raises:
    ///     Exception: If transaction generation fails.
    fn __next__(slf: PyRefMut<Self>) -> PyResult<Option<PendingTransaction>> {
        match slf.0.iter().next() {
            Some(result) => match result {
                Ok(transaction) => Ok(Some(transaction.into())),
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!(
                    "{}",
                    e
                ))),
            },
            None => Ok(None),
        }
    }
}

fn parse_generator_source(entries: Bound<'_, PyAny>) -> PyResult<GeneratorSource> {
    if let Ok(context) = entries.extract::<PyUtxoContext>() {
        Ok(GeneratorSource::UtxoContext(context.into()))
    } else if let Ok(entries) = entries.extract::<PyUtxoEntries>() {
        Ok(GeneratorSource::UtxoEntries(entries.entries))
    } else {
        Err(PyException::new_err(
            "entries must be a UtxoContext or list of UtxoEntryReference-compatible items",
        ))
    }
}

#[allow(dead_code)]
enum GeneratorSource {
    UtxoEntries(Vec<UtxoEntryReference>),
    UtxoContext(UtxoContext),
    // Account(Account),
}

#[allow(dead_code)]
struct GeneratorSettings {
    pub network_id: Option<NetworkId>,
    pub source: GeneratorSource,
    pub priority_utxo_entries: Option<Vec<UtxoEntryReference>>,
    pub multiplexer: Option<Multiplexer<Box<Events>>>,
    pub final_transaction_destination: PaymentDestination,
    pub change_address: Option<Address>,
    pub fee_rate: Option<f64>,
    pub final_priority_fee: Fees,
    pub sig_op_count: u8,
    pub minimum_signatures: u16,
    pub payload: Option<Vec<u8>>,
}

impl GeneratorSettings {
    pub fn new(
        outputs: Option<PyOutputs>,
        change_address: Address,
        fee_rate: Option<f64>,
        priority_fee: Option<u64>,
        source: GeneratorSource,
        priority_entries: Option<Vec<UtxoEntryReference>>,
        sig_op_count: Option<u8>,
        minimum_signatures: Option<u16>,
        payload: Option<Vec<u8>>,
        network_id: Option<NetworkId>,
    ) -> GeneratorSettings {
        let final_transaction_destination = match outputs {
            Some(py_outputs) => PaymentOutputs {
                outputs: py_outputs.outputs,
            }
            .into(),
            None => PaymentDestination::Change,
        };

        let fee_rate =
            fee_rate.and_then(|v| (v.is_finite() && !v.is_nan() && v >= 1e-8).then_some(v));

        let final_priority_fee = match priority_fee {
            Some(fee) => fee.into(),
            None => Fees::None,
        };

        let sig_op_count = sig_op_count.unwrap_or(1);

        let minimum_signatures = minimum_signatures.unwrap_or(1);

        GeneratorSettings {
            network_id,
            source,
            priority_utxo_entries: priority_entries,
            multiplexer: None,
            final_transaction_destination,
            change_address: Some(change_address),
            fee_rate,
            final_priority_fee,
            sig_op_count,
            minimum_signatures,
            payload,
        }
    }
}
