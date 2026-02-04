pub mod error;

use crate::consensus::client::input::PyTransactionInput;
use crate::consensus::client::output::PyTransactionOutput;
use crate::consensus::client::transaction::PyTransaction;
use crate::consensus::core::network::PyNetworkId;
use crate::consensus::core::tx::TransactionId;
use error::Error;
use kaspa_consensus_client::{Transaction, TransactionInput, TransactionOutput};
use kaspa_consensus_core::network::NetworkType;
use kaspa_wallet_pskt::pskt::Input;
use kaspa_wallet_pskt::wasm::error::Error as WasmError;
use kaspa_wallet_pskt::{
    error::Error as NativeError,
    pskt::{Inner, PSKT},
    role::*,
    wasm::pskt::State,
};
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::*;
use std::sync::{Arc, Mutex, MutexGuard};

/// Partially Signed Kaspa Transaction
#[gen_stub_pyclass]
#[pyclass(name = "PSKT")]
#[derive(Clone)]
pub struct PyPSKT {
    state: Arc<Mutex<Option<State>>>,
}

impl PyPSKT {
    fn take(&self) -> State {
        self.state.lock().unwrap().take().unwrap()
    }

    fn replace(&self, state: State) -> PyResult<PyPSKT> {
        self.state.lock().unwrap().replace(state);
        Ok(self.clone())
    }

    fn state(&self) -> MutexGuard<'_, Option<State>> {
        self.state.lock().unwrap()
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyPSKT {
    #[new]
    #[pyo3(signature = (payload=None))]
    pub fn new(payload: Option<Bound<'_, PyAny>>) -> PyResult<Self> {
        let pskt = match payload {
            None => PyPSKT::from(State::Creator(PSKT::<Creator>::default())),
            Some(p) => {
                if let Ok(s) = p.extract::<String>() {
                    let inner: State = serde_json::from_str(&s)
                        .map_err(|err| Error::from(WasmError::Ctor(err.to_string())))?;
                    PyPSKT::from(inner)
                } else if let Ok(py_tx) = p.extract::<PyTransaction>() {
                    let tx: Transaction = py_tx.into();
                    let inner: Inner = tx
                        .try_into()
                        .map_err(|err: NativeError| PyException::new_err(err.to_string()))?;
                    PyPSKT::from(State::NoOp(Some(inner)))
                } else {
                    return Err(Error::from(WasmError::InvalidPayload).into());
                }
            }
        };

        Ok(pskt)
    }

    #[getter]
    pub fn get_role(&self) -> String {
        self.state().as_ref().unwrap().display().to_string()
    }

    #[getter]
    pub fn get_payload(&self) -> PyResult<String> {
        let state = self.state();
        serde_json::to_string(state.as_ref().unwrap())
            .map_err(|err| PyException::new_err(err.to_string()))
        // workflow_wasm::serde::to_value(state.as_ref().unwrap()).unwrap()
    }

    pub fn serialize(&self) -> String {
        let state = self.state();
        serde_json::to_string(state.as_ref().unwrap()).unwrap()
    }

    /// Change role to `CREATOR`
    pub fn creator(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::NoOp(inner) => match inner {
                None => State::Creator(PSKT::default()),
                Some(_) => Err(Error::from(WasmError::CreateNotAllowed))?,
            },
            state => Err(Error::from(WasmError::state(state)))?,
        };

        self.replace(state)
    }

    /// Change role to `CONSTRUCTOR`
    pub fn to_constructor(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::NoOp(inner) => {
                State::Constructor(inner.ok_or(Error::from(WasmError::NotInitialized))?.into())
            }
            State::Creator(pskt) => State::Constructor(pskt.constructor()),
            state => Err(Error::from(WasmError::state(state)))?,
        };

        self.replace(state)
    }

    /// Change role to `UPDATER`
    pub fn to_updater(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::NoOp(inner) => {
                State::Updater(inner.ok_or(Error::from(WasmError::NotInitialized))?.into())
            }
            State::Constructor(constructor) => State::Updater(constructor.updater()),
            state => Err(Error::from(WasmError::state(state)))?,
        };

        self.replace(state)
    }

    /// Change role to `SIGNER`
    pub fn to_signer(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::NoOp(inner) => {
                State::Signer(inner.ok_or(Error::from(WasmError::NotInitialized))?.into())
            }
            State::Constructor(pskt) => State::Signer(pskt.signer()),
            State::Updater(pskt) => State::Signer(pskt.signer()),
            State::Combiner(pskt) => State::Signer(pskt.signer()),
            state => Err(Error::from(WasmError::state(state)))?,
        };

        self.replace(state)
    }

    /// Change role to `COMBINER`
    pub fn to_combiner(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::NoOp(inner) => {
                State::Combiner(inner.ok_or(Error::from(WasmError::NotInitialized))?.into())
            }
            State::Constructor(pskt) => State::Combiner(pskt.combiner()),
            State::Updater(pskt) => State::Combiner(pskt.combiner()),
            State::Signer(pskt) => State::Combiner(pskt.combiner()),
            state => Err(Error::from(WasmError::state(state)))?,
        };

        self.replace(state)
    }

    /// Change role to `FINALIZER`
    pub fn to_finalizer(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::NoOp(inner) => {
                State::Finalizer(inner.ok_or(Error::from(WasmError::NotInitialized))?.into())
            }
            State::Combiner(pskt) => State::Finalizer(pskt.finalizer()),
            state => Err(Error::from(WasmError::state(state)))?,
        };

        self.replace(state)
    }

    /// Change role to `EXTRACTOR`
    pub fn to_extractor(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::NoOp(inner) => {
                State::Extractor(inner.ok_or(Error::from(WasmError::NotInitialized))?.into())
            }
            State::Finalizer(pskt) => State::Extractor(
                pskt.extractor()
                    .map_err(WasmError::from)
                    .map_err(Error::from)?,
            ),
            state => Err(Error::from(WasmError::state(state)))?,
        };

        self.replace(state)
    }

    pub fn fallback_lock_time(&self, lock_time: u64) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::Creator(pskt) => State::Creator(pskt.fallback_lock_time(lock_time)),
            _ => Err(Error::from(WasmError::expected_state("Creator")))?,
        };

        self.replace(state)
    }

    pub fn inputs_modifiable(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::Creator(pskt) => State::Creator(pskt.inputs_modifiable()),
            _ => Err(Error::from(WasmError::expected_state("Creator")))?,
        };

        self.replace(state)
    }

    pub fn outputs_modifiable(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::Creator(pskt) => State::Creator(pskt.outputs_modifiable()),
            _ => Err(Error::from(WasmError::expected_state("Creator")))?,
        };

        self.replace(state)
    }

    pub fn no_more_inputs(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::Constructor(pskt) => State::Constructor(pskt.no_more_inputs()),
            _ => Err(Error::from(WasmError::expected_state("Constructor")))?,
        };

        self.replace(state)
    }

    pub fn no_more_outputs(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::Constructor(pskt) => State::Constructor(pskt.no_more_outputs()),
            _ => Err(Error::from(WasmError::expected_state("Constructor")))?,
        };

        self.replace(state)
    }

    pub fn input_and_redeem_script(
        &self,
        input: PyTransactionInput,
        data: String,
    ) -> PyResult<PyPSKT> {
        let input = TransactionInput::from(input);
        let mut input: Input = input
            .try_into()
            .map_err(|err| Error::from(WasmError::from(err)))?;
        input.redeem_script = Some(hex::decode(data).map_err(|e| {
            Error::from(WasmError::custom(format!(
                "Redeem script is not a hex string: {}",
                e
            )))
        })?);
        let state = match self.take() {
            State::Constructor(pskt) => State::Constructor(pskt.input(input)),
            _ => Err(Error::from(WasmError::expected_state("Constructor")))?,
        };

        self.replace(state)
    }

    pub fn input(&self, input: PyTransactionInput) -> PyResult<PyPSKT> {
        let input = TransactionInput::from(input);
        let state = match self.take() {
            State::Constructor(pskt) => State::Constructor(
                pskt.input(
                    input
                        .try_into()
                        .map_err(|err| Error::from(WasmError::from(err)))?,
                ),
            ),
            _ => Err(Error::from(WasmError::expected_state("Constructor")))?,
        };

        self.replace(state)
    }

    pub fn output(&self, output: PyTransactionOutput) -> PyResult<PyPSKT> {
        let output = TransactionOutput::from(output);
        let state = match self.take() {
            State::Constructor(pskt) => State::Constructor(
                pskt.output(
                    output
                        .try_into()
                        .map_err(|err| Error::from(WasmError::from(err)))?,
                ),
            ),
            _ => Err(Error::from(WasmError::expected_state("Constructor")))?,
        };

        self.replace(state)
    }

    pub fn set_sequence(&self, n: u64, input_index: usize) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::Updater(pskt) => State::Updater(
                pskt.set_sequence(n, input_index)
                    .map_err(|err| Error::from(WasmError::from(err)))?,
            ),
            _ => Err(Error::from(WasmError::expected_state("Updater")))?,
        };

        self.replace(state)
    }

    pub fn calculate_id(&self) -> PyResult<TransactionId> {
        let state = self.state();
        match state.as_ref().unwrap() {
            State::Signer(pskt) => Ok(pskt.calculate_id().into()),
            _ => Err(Error::from(WasmError::expected_state("Signer")))?,
        }
    }

    pub fn calculate_mass(&self, data: PyNetworkId) -> PyResult<u64> {
        let network_type = data.get_network_type();

        let cloned_pskt = self.clone();

        let extractor = {
            let finalizer = cloned_pskt.to_finalizer()?;

            let finalizer_state = finalizer.state().clone().unwrap();

            match finalizer_state {
                State::Finalizer(pskt) => {
                    for input in pskt.inputs.iter() {
                        if input.redeem_script.is_some() {
                            return Err(Error::from(WasmError::custom(
                                "Mass calculation is not supported for inputs with redeem scripts",
                            ))
                            .into());
                        }
                    }
                    let pskt = pskt
                        .finalize_sync(|inner: &Inner| -> PyResult<Vec<Vec<u8>>> {
                            Ok(vec![vec![0u8, 65]; inner.inputs.len()])
                        })
                        .map_err(|e| {
                            Error::from(WasmError::custom(format!("Failed to finalize PSKT: {e}")))
                        })?;
                    pskt.extractor()
                        .map_err(|err| Error::from(WasmError::TxNotFinalized(err)))?
                }
                _ => panic!("Finalizer state is not valid"),
            }
        };
        let tx = extractor
            .extract_tx_unchecked(&NetworkType::from(network_type).into())
            .map_err(|e| {
                Error::from(WasmError::custom(format!(
                    "Failed to extract transaction: {e}"
                )))
            })?;
        Ok(tx.tx.mass())
    }
}

impl From<State> for PyPSKT {
    fn from(value: State) -> Self {
        PyPSKT {
            state: Arc::new(Mutex::new(Some(value))),
        }
    }
}
