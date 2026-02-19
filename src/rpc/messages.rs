use crate::{address::PyAddress, consensus::client::transaction::PyTransaction, rpc::model::*};
use kaspa_addresses::Address;
use kaspa_rpc_core::{
    RpcRawBlock, RpcTransaction, RpcTransactionInput, RpcTransactionOutput, message::*,
};
use paste::paste;
use pyo3::{
    exceptions::{PyDeprecationWarning, PyException, PyKeyError},
    ffi::c_str,
    prelude::*,
    types::{PyDict, PyList},
};
use serde_pyobject::from_pyobject;

// Macro to implement `TryFrom<Bound<'_, PyDict>>` for multiple request wrapper types.
//
// For each name (e.g., `GetBlockCount`), generates an impl for `PyGetBlockCountRequest`
// that deserializes from a Python dict using serde_pyobject.
macro_rules! impl_try_from_pydict {
    ([$($name:ident),* $(,)?]) => {
        paste! {
            $(
                impl TryFrom<Bound<'_, PyDict>> for [<Py $name Request>] {
                    type Error = PyErr;
                    fn try_from(dict: Bound<'_, PyDict>) -> PyResult<Self> {
                        Ok(from_pyobject(dict)?)
                    }
                }
            )*
        }
    };
}

impl_try_from_pydict!([
    // Optional request parameter (no args required)
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
    // Required request parameter (args needed)
    AddPeer,
    Ban,
    EstimateNetworkHashesPerSecond,
    // GetBalanceByAddress,
    // GetBalancesByAddresses,
    GetBlock,
    GetBlocks,
    GetBlockTemplate,
    GetCurrentBlockColor,
    GetDaaScoreTimestampEstimate,
    GetFeeEstimateExperimental,
    GetHeaders,
    GetMempoolEntries,
    // GetMempoolEntriesByAddresses,
    GetMempoolEntry,
    GetSubnetwork,
    // GetUtxosByAddresses,
    GetUtxoReturnAddress,
    GetVirtualChainFromBlock,
    GetVirtualChainFromBlockV2,
    ResolveFinalityConflict,
    // SubmitBlock,
    // SubmitTransaction,
    // SubmitTransactionReplacement,
    Unban,
]);

macro_rules! try_from_args {
    ($name:ident : $to_type:ty, $body:block) => {
        impl TryFrom<Bound<'_, PyDict>> for $to_type {
            type Error = PyErr;
            fn try_from($name: Bound<'_, PyDict>) -> PyResult<Self> {
                $body
            }
        }
    };
}

try_from_args! ( dict : PyGetBalanceByAddressRequest, {
    let address_value = dict.get_item("address")?
        .ok_or_else(|| PyKeyError::new_err("Key `address` not present"))?;

    let address = if let Ok(address) = address_value.extract::<PyAddress>() {
        address
    } else if let Ok(s) = address_value.extract::<String>() {
        PyAddress::try_from(s)
            .map_err(|err| PyException::new_err(format!("{}", err)))?
    } else {
        return Err(PyException::new_err("Addresses must be either an Address instance or a string"));
    };

    let inner = GetBalanceByAddressRequest { address: address.into() };

    Ok(PyGetBalanceByAddressRequest(inner))
});

try_from_args! ( dict : PyGetBalancesByAddressesRequest, {
    let items = dict.get_item("addresses")?
        .ok_or_else(|| PyKeyError::new_err("Key `addresses` not present"))?;

    let list = items.cast::<PyList>()
        .map_err(|_| PyException::new_err("`addresses` should be a list"))?;

    let addresses = list.iter().map(|item| {
        if let Ok(address) = item.extract::<PyAddress>() {
            Ok(address)
        } else if let Ok(s) = item.extract::<String>() {
            let address = PyAddress::try_from(s)
                .map_err(|err| PyException::new_err(format!("{}", err)))?;
            Ok(address)
        } else {
            Err(PyException::new_err("Addresses must be either an Address instance or an address as a string"))
        }
    }).collect::<PyResult<Vec<PyAddress>>>()?;

    let addresses: Vec<Address> = addresses.into_iter().map(PyAddress::into).collect();
    let inner = GetBalancesByAddressesRequest { addresses };
    Ok(PyGetBalancesByAddressesRequest(inner))
});

try_from_args! ( dict : PyGetMempoolEntriesByAddressesRequest, {
    let items = dict.get_item("addresses")?
        .ok_or_else(|| PyKeyError::new_err("Key `addresses` not present"))?;

    let list = items.cast::<PyList>()
        .map_err(|_| PyException::new_err("`addresses` should be a list"))?;

    let addresses = list.iter().map(|item| {
        if let Ok(address) = item.extract::<PyAddress>() {
            Ok(address)
        } else if let Ok(s) = item.extract::<String>() {
            let address = PyAddress::try_from(s)
                .map_err(|err| PyException::new_err(format!("{}", err)))?;
            Ok(address)
        } else {
            Err(PyException::new_err("Addresses must be either an Address instance or an address as a string"))
        }
    }).collect::<PyResult<Vec<PyAddress>>>()?;
    let addresses: Vec<Address> = addresses.into_iter().map(PyAddress::into).collect();

    let include_orphan_pool = dict.get_item("includeOrphanPool")?
        .ok_or_else(|| PyKeyError::new_err("Key `include_orphan_pool` not present"))?
        .extract::<bool>()?;

    let filter_transaction_pool = dict.get_item("filterTransactionPool")?
        .ok_or_else(|| PyKeyError::new_err("Key `filter_transaction_pool` not present"))?
        .extract::<bool>()?;

    let inner = GetMempoolEntriesByAddressesRequest { addresses, include_orphan_pool, filter_transaction_pool };
    Ok(PyGetMempoolEntriesByAddressesRequest(inner))
});

try_from_args! ( dict : PyGetUtxosByAddressesRequest, {
    let items = dict.get_item("addresses")?
        .ok_or_else(|| PyKeyError::new_err("Key `addresses` not present"))?;
    let list = items.cast::<PyList>()
        .map_err(|_| PyException::new_err("`addresses` should be a list"))?;

    let addresses = list.iter().map(|item| {
        if let Ok(address) = item.extract::<PyAddress>() {
            Ok(address)
        } else if let Ok(s) = item.extract::<String>() {
            let address = PyAddress::try_from(s)
                .map_err(|err| PyException::new_err(format!("{}", err)))?;
            Ok(address)
        } else {
            Err(PyException::new_err("Addresses must be either an Address instance or an address as a string"))
        }
    }).collect::<PyResult<Vec<PyAddress>>>()?;
    let addresses: Vec<Address> = addresses.into_iter().map(PyAddress::into).collect();

    let inner = GetUtxosByAddressesRequest { addresses };
    Ok(PyGetUtxosByAddressesRequest(inner))
});

try_from_args! ( dict : PySubmitBlockRequest, {
    let d = dict.as_any();

    let block = d.get_item("block")?;
    let header = serde_pyobject::from_pyobject(block.get_item("header")?)?;
    let transactions = serde_pyobject::from_pyobject(block.get_item("transactions")?)?;
    let allow_non_daa_blocks = d.get_item("allowNonDaaBlocks")?.extract::<bool>()?;

    let block = RpcRawBlock { header, transactions };

    let inner = SubmitBlockRequest { block, allow_non_daa_blocks };
    Ok(PySubmitBlockRequest(inner))
});

try_from_args! ( dict : PySubmitTransactionRequest, {
    let transaction: PyTransaction = dict.get_item("transaction")?
        .ok_or_else(|| PyKeyError::new_err("Key `transaction` not present"))?
        .extract()?;
    let inner = transaction.inner().inner();

    // Deprecate allow_orphan in favor of allowOrphan for case consistency
    // Deprecation warning added September 2025, version 1.0.1.post1
    let py = dict.py();
    if dict.get_item("allow_orphan")?.is_some() {
        PyErr::warn(
            py,
            &py.get_type::<PyDeprecationWarning>(),
            c_str!("`allow_orphan` will be deprecated in favor of `allowOrphan` for case consistency. Please switch."),
            0
        )?;
    }

    let allow_orphan: bool = if let Some(item) = dict.get_item("allowOrphan")? {
        item.extract()?
    } else if let Some(item) = dict.get_item("allow_orphan")? {
        item.extract()?
    } else {
        return Err(PyKeyError::new_err("Key `allowOrphan` not present"));
    };

    let inputs: Vec<RpcTransactionInput> =
        inner.inputs.clone().into_iter().map(|input| input.into()).collect::<Vec<RpcTransactionInput>>();
    let outputs: Vec<RpcTransactionOutput> =
        inner.outputs.clone().into_iter().map(|output| output.into()).collect::<Vec<RpcTransactionOutput>>();

    let rpc_transaction = RpcTransaction {
        version: inner.version,
        inputs,
        outputs,
        lock_time: inner.lock_time,
        subnetwork_id: inner.subnetwork_id,
        gas: inner.gas,
        payload: inner.payload.clone(),
        mass: inner.mass,
        verbose_data: None,
    };

    let inner = SubmitTransactionRequest { transaction: rpc_transaction, allow_orphan };
    Ok(PySubmitTransactionRequest(inner))
});

try_from_args! ( dict : PySubmitTransactionReplacementRequest, {
    let transaction: PyTransaction = dict.get_item("transaction")?
        .ok_or_else(|| PyKeyError::new_err("Key `transactions` not present"))?
        .extract()?;

    let inner = SubmitTransactionReplacementRequest { transaction: transaction.inner().into() };
    Ok(PySubmitTransactionReplacementRequest(inner))
});
