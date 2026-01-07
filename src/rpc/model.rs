use kaspa_rpc_core::message::*;
use paste::paste;
use serde::{Deserialize, Serialize};

// Macro to define Python request wrapper types.
//
// For each name (e.g., `GetBlockCount`), generates:
// ```ignore
// #[derive(Deserialize, Serialize)]
// pub struct PyGetBlockCountRequest(pub GetBlockCountRequest);
// ```
macro_rules! define_py_request_types {
    ([$($name:ident),* $(,)?]) => {
        paste! {
            $(
                #[derive(Deserialize, Serialize)]
                pub struct [<Py $name Request>](pub [<$name Request>]);
            )*
        }
    };
}

define_py_request_types!([
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
