pub mod error;

use crate::consensus::client::transaction::PyTransaction;
use kaspa_consensus_client::Transaction;
use kaspa_wallet_pskt::{
    pskt::{Inner, PSKT},
    role::*,
    wasm::pskt::State,
};
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::*;
use std::sync::{Arc, Mutex};

/// Partially Signed Kaspa Transaction
#[gen_stub_pyclass]
#[pyclass(name = "PSKT")]
#[derive(Clone)]
pub struct PyPSKT {
    state: Arc<Mutex<Option<State>>>,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyPSKT {
    #[new]
    pub fn new(payload: Bound<'_, PyAny>) -> PyResult<Self> {
        let payload = if let Ok(p) = payload.extract::<String>() {
            let inner =
                serde_json::from_str(&p).map_err(|err| PyException::new_err(err.to_string()))?;
            Ok(PyPSKT::from(State::NoOp(Some(inner))))
        } else if let Ok(py_tx) = payload.extract::<PyTransaction>() {
            let tx: Transaction = py_tx.into();
            let inner: Inner = tx
                .try_into()
                .map_err(|_| PyException::new_err("Transaction to Inner failed"))?;
            Ok(PyPSKT::from(State::NoOp(Some(inner))))
        } else if payload.is_none() {
            Ok(PyPSKT::from(State::Creator(PSKT::<Creator>::default())))
        } else {
            Err(PyException::new_err("Invalid payload"))
        }?;

        Ok(payload)
    }

    #[getter]
    pub fn get_role(&self) -> String {
        self.state().as_ref().unwrap().display().to_string()
    }

    #[getter]
    pub fn get_payload(&self) -> JsValue {
        let state = self.state();
        workflow_wasm::serde::to_value(state.as_ref().unwrap()).unwrap()
    }

    pub fn serialize(&self) -> String {
        let state = self.state();
        serde_json::to_string(state.as_ref().unwrap()).unwrap()
    }

    fn state(&self) -> MutexGuard<'_, Option<State>> {
        self.state.lock().unwrap()
    }

    fn take(&self) -> State {
        self.state.lock().unwrap().take().unwrap()
    }

    fn replace(&self, state: State) -> PyResult<PyPSKT> {
        self.state.lock().unwrap().replace(state);
        Ok(self.clone())
    }

    /// Change role to `CREATOR`
    pub fn creator(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::NoOp(inner) => match inner {
                None => State::Creator(PSKT::default()),
                Some(_) => Err(Error::CreateNotAllowed)?,
            },
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    /// Change role to `CONSTRUCTOR`
    pub fn to_constructor(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::NoOp(inner) => State::Constructor(inner.ok_or(Error::NotInitialized)?.into()),
            State::Creator(pskt) => State::Constructor(pskt.constructor()),
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    /// Change role to `UPDATER`
    pub fn to_updater(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::NoOp(inner) => State::Updater(inner.ok_or(Error::NotInitialized)?.into()),
            State::Constructor(constructor) => State::Updater(constructor.updater()),
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    /// Change role to `SIGNER`
    pub fn to_signer(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::NoOp(inner) => State::Signer(inner.ok_or(Error::NotInitialized)?.into()),
            State::Constructor(pskt) => State::Signer(pskt.signer()),
            State::Updater(pskt) => State::Signer(pskt.signer()),
            State::Combiner(pskt) => State::Signer(pskt.signer()),
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    /// Change role to `COMBINER`
    pub fn to_combiner(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::NoOp(inner) => State::Combiner(inner.ok_or(Error::NotInitialized)?.into()),
            State::Constructor(pskt) => State::Combiner(pskt.combiner()),
            State::Updater(pskt) => State::Combiner(pskt.combiner()),
            State::Signer(pskt) => State::Combiner(pskt.combiner()),
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    /// Change role to `FINALIZER`
    pub fn to_finalizer(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::NoOp(inner) => State::Finalizer(inner.ok_or(Error::NotInitialized)?.into()),
            State::Combiner(pskt) => State::Finalizer(pskt.finalizer()),
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    /// Change role to `EXTRACTOR`
    pub fn to_extractor(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::NoOp(inner) => State::Extractor(inner.ok_or(Error::NotInitialized)?.into()),
            State::Finalizer(pskt) => State::Extractor(pskt.extractor()?),
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    pub fn fallback_lock_time(&self, lock_time: u64) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::Creator(pskt) => State::Creator(pskt.fallback_lock_time(lock_time)),
            _ => Err(Error::expected_state("Creator"))?,
        };

        self.replace(state)
    }

    pub fn inputs_modifiable(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::Creator(pskt) => State::Creator(pskt.inputs_modifiable()),
            _ => Err(Error::expected_state("Creator"))?,
        };

        self.replace(state)
    }

    pub fn outputs_modifiable(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::Creator(pskt) => State::Creator(pskt.outputs_modifiable()),
            _ => Err(Error::expected_state("Creator"))?,
        };

        self.replace(state)
    }

    pub fn no_more_inputs(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::Constructor(pskt) => State::Constructor(pskt.no_more_inputs()),
            _ => Err(Error::expected_state("Constructor"))?,
        };

        self.replace(state)
    }

    pub fn no_more_outputs(&self) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::Constructor(pskt) => State::Constructor(pskt.no_more_outputs()),
            _ => Err(Error::expected_state("Constructor"))?,
        };

        self.replace(state)
    }

    pub fn input_and_redeem_scrtip(&self, input: &TransactionInputT, data: &JsValue) -> PyResult<PyPSKT> {
        let obj = js_sys::Object::from(data.clone());

        let input = TransactionInput::try_owned_from(input)?;
        let mut input: Input = input.try_into()?;
        let redeem_script = js_sys::Reflect::get(&obj, &"redeemScript".into())
            .expect("Missing redeemscript field")
            .as_string()
            .expect("redeemscript must be a string");
        input.redeem_script =
            Some(hex::decode(redeem_script).map_err(|e| Error::custom(format!("Redeem script is not a hex string: {}", e)))?);
        let state = match self.take() {
            State::Constructor(pskt) => State::Constructor(pskt.input(input)),
            _ => Err(Error::expected_state("Constructor"))?,
        };

        self.replace(state)
    }

    pub fn input(&self, input: &TransactionInputT) -> PyResult<PyPSKT> {
        let input = TransactionInput::try_owned_from(input)?;
        let state = match self.take() {
            State::Constructor(pskt) => State::Constructor(pskt.input(input.try_into()?)),
            _ => Err(Error::expected_state("Constructor"))?,
        };

        self.replace(state)
    }

    pub fn output(&self, output: &TransactionOutputT) -> PyResult<PyPSKT> {
        let output = TransactionOutput::try_owned_from(output)?;
        let state = match self.take() {
            State::Constructor(pskt) => State::Constructor(pskt.output(output.try_into()?)),
            _ => Err(Error::expected_state("Constructor"))?,
        };

        self.replace(state)
    }

    pub fn set_sequence(&self, n: u64, input_index: usize) -> PyResult<PyPSKT> {
        let state = match self.take() {
            State::Updater(pskt) => State::Updater(pskt.set_sequence(n, input_index)?),
            _ => Err(Error::expected_state("Updater"))?,
        };

        self.replace(state)
    }

    #[wasm_bindgen(js_name = calculateId)]
    pub fn calculate_id(&self) -> Result<TransactionId> {
        let state = self.state();
        match state.as_ref().unwrap() {
            State::Signer(pskt) => Ok(pskt.calculate_id()),
            _ => Err(Error::expected_state("Signer"))?,
        }
    }

    #[wasm_bindgen(js_name = calculateMass)]
    pub fn calculate_mass(&self, data: &JsValue) -> Result<u64> {
        let obj = js_sys::Object::from(data.clone());
        let network_id = js_sys::Reflect::get(&obj, &"networkId".into())
            .map_err(|_| Error::custom("networkId is missing"))?
            .as_string()
            .ok_or_else(|| Error::custom("networkId must be a string"))?;

        let network_id = NetworkType::from_str(&network_id).map_err(|e| Error::custom(format!("Invalid networkId: {}", e)))?;

        let cloned_pskt = self.clone();

        let extractor = {
            let finalizer = cloned_pskt.finalizer()?;

            let finalizer_state = finalizer.state().clone().unwrap();

            match finalizer_state {
                State::Finalizer(pskt) => {
                    for input in pskt.inputs.iter() {
                        if input.redeem_script.is_some() {
                            return Err(Error::custom("Mass calculation is not supported for inputs with redeem scripts"));
                        }
                    }
                    let pskt = pskt
                        .finalize_sync(|inner: &Inner| -> Result<Vec<Vec<u8>>> { Ok(vec![vec![0u8, 65]; inner.inputs.len()]) })
                        .map_err(|e| Error::custom(format!("Failed to finalize PSKT: {e}")))?;
                    pskt.extractor()?
                }
                _ => panic!("Finalizer state is not valid"),
            }
        };
        let tx = extractor
            .extract_tx_unchecked(&network_id.into())
            .map_err(|e| Error::custom(format!("Failed to extract transaction: {e}")))?;
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
