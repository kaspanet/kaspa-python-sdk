"""
TypedDict definitions for Kaspa RPC request and response types.

These types describe the dictionary structures used when calling RPC methods
on the RpcClient. Request types are passed as the `request` parameter, and
response types describe what the async methods return.

Example:
    >>> client = RpcClient(...)
    >>> await client.connect()
    >>> response: GetBlockDagInfoResponse = await client.get_block_dag_info()
    >>> print(response["blockCount"])
"""

from typing import TypedDict


# =============================================================================
# Shared / Nested Types
# =============================================================================

class RpcOutpoint(TypedDict):
    """A transaction outpoint (reference to a specific output)."""
    transactionId: str
    index: int


class RpcScriptPublicKey(TypedDict):
    """A script public key."""
    version: int
    script: str


class RpcUtxoEntry(TypedDict):
    """A UTXO entry."""
    amount: int
    scriptPublicKey: RpcScriptPublicKey
    blockDaaScore: int
    isCoinbase: bool


class RpcUtxosByAddressesEntry(TypedDict):
    """A UTXO entry associated with an address."""
    address: str
    outpoint: RpcOutpoint
    utxoEntry: RpcUtxoEntry


class RpcBalancesByAddressesEntry(TypedDict):
    """Balance information for a specific address."""
    address: str
    balance: int


class RpcFeeRateBucket(TypedDict):
    """A fee rate bucket for fee estimation."""
    feerate: float
    estimatedSeconds: float


class RpcFeeEstimate(TypedDict):
    """Fee estimation with priority, normal, and low buckets."""
    priorityBucket: RpcFeeRateBucket
    normalBuckets: list[RpcFeeRateBucket]
    lowBuckets: list[RpcFeeRateBucket]


class RpcTransactionInput(TypedDict):
    """A transaction input."""
    previousOutpoint: RpcOutpoint
    signatureScript: str
    sequence: int
    sigOpCount: int


class RpcTransactionOutput(TypedDict):
    """A transaction output."""
    value: int
    scriptPublicKey: RpcScriptPublicKey


class RpcTransaction(TypedDict):
    """A transaction."""
    version: int
    inputs: list[RpcTransactionInput]
    outputs: list[RpcTransactionOutput]
    lockTime: int
    subnetworkId: str
    gas: int
    payload: str
    mass: int
    verboseData: "RpcTransactionVerboseData | None"


class RpcTransactionVerboseData(TypedDict, total=False):
    """Verbose transaction data."""
    transactionId: str
    hash: str
    computeMass: int
    blockHash: str
    blockTime: int


class RpcBlockHeader(TypedDict):
    """A block header."""
    version: int
    parents: list["RpcBlockLevelParents"]
    hashMerkleRoot: str
    acceptedIdMerkleRoot: str
    utxoCommitment: str
    timestamp: int
    bits: int
    nonce: int
    daaScore: int
    blueWork: str
    blueScore: int
    pruningPoint: str


class RpcBlockLevelParents(TypedDict):
    """Parents at a specific block level."""
    parentHashes: list[str]


class RpcBlockVerboseData(TypedDict, total=False):
    """Verbose block data."""
    hash: str
    difficulty: float
    selectedParentHash: str
    transactionIds: list[str]
    isHeaderOnly: bool
    blueScore: int
    childrenHashes: list[str]
    mergeSetBluesHashes: list[str]
    mergeSetRedsHashes: list[str]
    isChainBlock: bool


class RpcBlock(TypedDict):
    """A block."""
    header: RpcBlockHeader
    transactions: list[RpcTransaction]
    verboseData: RpcBlockVerboseData | None


class RpcMempoolEntry(TypedDict):
    """A mempool entry."""
    fee: int
    transaction: RpcTransaction
    isOrphan: bool


class RpcMempoolEntryByAddress(TypedDict):
    """Mempool entries for a specific address."""
    address: str
    sending: list[RpcMempoolEntry]
    receiving: list[RpcMempoolEntry]


class RpcAcceptedTransactionIds(TypedDict):
    """Accepted transaction IDs for a block."""
    acceptingBlockHash: str
    acceptedTransactionIds: list[str]


class RpcProcessMetrics(TypedDict):
    """Process metrics."""
    residentSetSize: int
    virtualMemorySize: int
    cpuUsagePercentage: float
    cpuCoresNum: int


class RpcStorageMetrics(TypedDict):
    """Storage metrics."""
    storageSizeBytes: int


class RpcConsensusMetrics(TypedDict):
    """Consensus metrics."""
    blocksSubmittedCount: int
    headerCounts: int
    depCounts: int
    bodyCounts: int
    txsCounts: int
    chainBlockCounts: int
    massCounts: int
    virtualParentHashesCount: int
    virtualDaaScore: int


class RpcConnectionMetrics(TypedDict):
    """Connection metrics."""
    borshLiveConnections: int
    borshConnectionAttempts: int
    borshHandshakeFailures: int
    jsonLiveConnections: int
    jsonConnectionAttempts: int
    jsonHandshakeFailures: int


class RpcPeerInfo(TypedDict):
    """Peer information."""
    id: str
    address: str
    lastPingDuration: int
    isOutbound: bool
    timeOffset: int
    userAgent: str
    advertisedProtocolVersion: int
    timeConnected: int
    isIbdPeer: bool


class RpcPeerAddress(TypedDict):
    """A peer address."""
    ip: str
    port: int


# =============================================================================
# Request Types (optional parameters for RPC methods)
# =============================================================================

class GetBlockCountRequest(TypedDict, total=False):
    """Request for get_block_count."""
    pass


class GetBlockDagInfoRequest(TypedDict, total=False):
    """Request for get_block_dag_info."""
    pass


class GetCoinSupplyRequest(TypedDict, total=False):
    """Request for get_coin_supply."""
    pass


class GetConnectedPeerInfoRequest(TypedDict, total=False):
    """Request for get_connected_peer_info."""
    pass


class GetInfoRequest(TypedDict, total=False):
    """Request for get_info."""
    pass


class GetPeerAddressesRequest(TypedDict, total=False):
    """Request for get_peer_addresses."""
    pass


class GetMetricsRequest(TypedDict, total=False):
    """Request for get_metrics."""
    processMetrics: bool
    connectionMetrics: bool
    bandwidthMetrics: bool
    consensusMetrics: bool
    storageMetrics: bool


class GetConnectionsRequest(TypedDict, total=False):
    """Request for get_connections."""
    includeProfileData: bool


class GetSinkRequest(TypedDict, total=False):
    """Request for get_sink."""
    pass


class GetSinkBlueScoreRequest(TypedDict, total=False):
    """Request for get_sink_blue_score."""
    pass


class PingRequest(TypedDict, total=False):
    """Request for ping."""
    pass


class ShutdownRequest(TypedDict, total=False):
    """Request for shutdown."""
    pass


class GetServerInfoRequest(TypedDict, total=False):
    """Request for get_server_info."""
    pass


class GetSyncStatusRequest(TypedDict, total=False):
    """Request for get_sync_status."""
    pass


class GetFeeEstimateRequest(TypedDict, total=False):
    """Request for get_fee_estimate."""
    pass


class GetCurrentNetworkRequest(TypedDict, total=False):
    """Request for get_current_network."""
    pass


class GetSystemInfoRequest(TypedDict, total=False):
    """Request for get_system_info."""
    pass


class AddPeerRequest(TypedDict):
    """Request for add_peer."""
    peerAddress: str
    isPermanent: bool


class BanRequest(TypedDict):
    """Request for ban."""
    ip: str


class UnbanRequest(TypedDict):
    """Request for unban."""
    ip: str


class EstimateNetworkHashesPerSecondRequest(TypedDict):
    """Request for estimate_network_hashes_per_second."""
    windowSize: int
    startHash: str | None


class GetBalanceByAddressRequest(TypedDict):
    """Request for get_balance_by_address."""
    address: str


class GetBalancesByAddressesRequest(TypedDict):
    """Request for get_balances_by_addresses."""
    addresses: list[str]


class GetBlockRequest(TypedDict):
    """Request for get_block."""
    hash: str
    includeTransactions: bool


class GetBlocksRequest(TypedDict):
    """Request for get_blocks."""
    lowHash: str | None
    includeBlocks: bool
    includeTransactions: bool


class GetBlockTemplateRequest(TypedDict):
    """Request for get_block_template."""
    payAddress: str
    extraData: str


class GetCurrentBlockColorRequest(TypedDict):
    """Request for get_current_block_color."""
    hash: str


class GetDaaScoreTimestampEstimateRequest(TypedDict):
    """Request for get_daa_score_timestamp_estimate."""
    daaScores: list[int]


class GetFeeEstimateExperimentalRequest(TypedDict, total=False):
    """Request for get_fee_estimate_experimental."""
    verbose: bool


class GetHeadersRequest(TypedDict):
    """Request for get_headers."""
    startHash: str
    limit: int
    isAscending: bool


class GetMempoolEntriesRequest(TypedDict):
    """Request for get_mempool_entries."""
    includeOrphanPool: bool
    filterTransactionPool: bool


class GetMempoolEntriesByAddressesRequest(TypedDict):
    """Request for get_mempool_entries_by_addresses."""
    addresses: list[str]
    includeOrphanPool: bool
    filterTransactionPool: bool


class GetMempoolEntryRequest(TypedDict):
    """Request for get_mempool_entry."""
    transactionId: str
    includeOrphanPool: bool
    filterTransactionPool: bool


class GetSubnetworkRequest(TypedDict):
    """Request for get_subnetwork."""
    subnetworkId: str


class GetUtxosByAddressesRequest(TypedDict):
    """Request for get_utxos_by_addresses."""
    addresses: list[str]


class GetUtxoReturnAddressRequest(TypedDict):
    """Request for get_utxo_return_address."""
    transactionId: str


class GetVirtualChainFromBlockRequest(TypedDict):
    """Request for get_virtual_chain_from_block."""
    startHash: str
    includeAcceptedTransactionIds: bool


class GetVirtualChainFromBlockV2Request(TypedDict):
    """Request for get_virtual_chain_from_block_v2."""
    startHash: str
    includeAcceptedTransactionIds: bool


class ResolveFinalityConflictRequest(TypedDict):
    """Request for resolve_finality_conflict."""
    finalityBlockHash: str


class SubmitBlockRequest(TypedDict):
    """Request for submit_block."""
    block: RpcBlock
    allowNonDaaBlocks: bool


class SubmitTransactionRequest(TypedDict):
    """Request for submit_transaction."""
    transaction: RpcTransaction
    allowOrphan: bool


class SubmitTransactionReplacementRequest(TypedDict):
    """Request for submit_transaction_replacement."""
    transaction: RpcTransaction


# =============================================================================
# Response Types (returned from RPC methods)
# =============================================================================

class GetBlockCountResponse(TypedDict):
    """Response from get_block_count."""
    blockCount: int
    headerCount: int


class GetBlockDagInfoResponse(TypedDict):
    """Response from get_block_dag_info."""
    networkName: str
    blockCount: int
    headerCount: int
    tipHashes: list[str]
    difficulty: float
    pastMedianTime: int
    virtualParentHashes: list[str]
    pruningPointHash: str
    virtualDaaScore: int
    sink: str


class GetCoinSupplyResponse(TypedDict):
    """Response from get_coin_supply."""
    maxSompi: int
    circulatingSompi: int


class GetConnectedPeerInfoResponse(TypedDict):
    """Response from get_connected_peer_info."""
    peerInfo: list[RpcPeerInfo]


class GetInfoResponse(TypedDict):
    """Response from get_info."""
    p2pId: str
    mempoolSize: int
    serverVersion: str
    isUtxoIndexed: bool
    isSynced: bool
    hasNotifyCommand: bool
    hasMessageId: bool


class GetPeerAddressesResponse(TypedDict):
    """Response from get_peer_addresses."""
    knownAddresses: list[RpcPeerAddress]
    bannedAddresses: list[RpcPeerAddress]


class GetMetricsResponse(TypedDict, total=False):
    """Response from get_metrics."""
    processMetrics: RpcProcessMetrics
    connectionMetrics: RpcConnectionMetrics
    bandwidthMetrics: dict
    consensusMetrics: RpcConsensusMetrics
    storageMetrics: RpcStorageMetrics
    serverTime: int


class GetConnectionsResponse(TypedDict):
    """Response from get_connections."""
    clientsCount: int
    peersCount: int
    profileData: dict | None


class GetSinkResponse(TypedDict):
    """Response from get_sink."""
    sink: str


class GetSinkBlueScoreResponse(TypedDict):
    """Response from get_sink_blue_score."""
    blueScore: int


class PingResponse(TypedDict):
    """Response from ping."""
    pass


class ShutdownResponse(TypedDict):
    """Response from shutdown."""
    pass


class GetServerInfoResponse(TypedDict):
    """Response from get_server_info."""
    rpcApiVersion: list[int]
    rpcApiRevision: int
    serverVersion: str
    networkId: str
    hasUtxoIndex: bool
    isSynced: bool
    virtualDaaScore: int


class GetSyncStatusResponse(TypedDict):
    """Response from get_sync_status."""
    isSynced: bool


class GetFeeEstimateResponse(TypedDict):
    """Response from get_fee_estimate."""
    estimate: RpcFeeEstimate


class GetCurrentNetworkResponse(TypedDict):
    """Response from get_current_network."""
    network: str
    suffix: int | None


class GetSystemInfoResponse(TypedDict):
    """Response from get_system_info."""
    version: str
    systemId: str | None
    gitHash: str | None
    totalMemory: int
    coreNum: int


class AddPeerResponse(TypedDict):
    """Response from add_peer."""
    pass


class BanResponse(TypedDict):
    """Response from ban."""
    pass


class UnbanResponse(TypedDict):
    """Response from unban."""
    pass


class EstimateNetworkHashesPerSecondResponse(TypedDict):
    """Response from estimate_network_hashes_per_second."""
    networkHashesPerSecond: int


class GetBalanceByAddressResponse(TypedDict):
    """Response from get_balance_by_address."""
    balance: int


class GetBalancesByAddressesResponse(TypedDict):
    """Response from get_balances_by_addresses."""
    entries: list[RpcBalancesByAddressesEntry]


class GetBlockResponse(TypedDict):
    """Response from get_block."""
    block: RpcBlock


class GetBlocksResponse(TypedDict):
    """Response from get_blocks."""
    blockHashes: list[str]
    blocks: list[RpcBlock]


class GetBlockTemplateResponse(TypedDict):
    """Response from get_block_template."""
    block: RpcBlock
    isSynced: bool


class GetCurrentBlockColorResponse(TypedDict):
    """Response from get_current_block_color."""
    blue: bool


class GetDaaScoreTimestampEstimateResponse(TypedDict):
    """Response from get_daa_score_timestamp_estimate."""
    timestamps: list[int]


class GetFeeEstimateExperimentalResponse(TypedDict):
    """Response from get_fee_estimate_experimental."""
    estimate: RpcFeeEstimate
    verbose: dict | None


class GetHeadersResponse(TypedDict):
    """Response from get_headers."""
    headers: list[str]


class GetMempoolEntriesResponse(TypedDict):
    """Response from get_mempool_entries."""
    mempoolEntries: list[RpcMempoolEntry]


class GetMempoolEntriesByAddressesResponse(TypedDict):
    """Response from get_mempool_entries_by_addresses."""
    entries: list[RpcMempoolEntryByAddress]


class GetMempoolEntryResponse(TypedDict):
    """Response from get_mempool_entry."""
    mempoolEntry: RpcMempoolEntry


class GetSubnetworkResponse(TypedDict):
    """Response from get_subnetwork."""
    gasLimit: int


class GetUtxosByAddressesResponse(TypedDict):
    """Response from get_utxos_by_addresses."""
    entries: list[RpcUtxosByAddressesEntry]


class GetUtxoReturnAddressResponse(TypedDict):
    """Response from get_utxo_return_address."""
    address: str | None


class GetVirtualChainFromBlockResponse(TypedDict):
    """Response from get_virtual_chain_from_block."""
    removedChainBlockHashes: list[str]
    addedChainBlockHashes: list[str]
    acceptedTransactionIds: list[RpcAcceptedTransactionIds]


class GetVirtualChainFromBlockV2Response(TypedDict):
    """Response from get_virtual_chain_from_block_v2."""
    removedChainBlockHashes: list[str]
    addedChainBlockHashes: list[str]
    acceptedTransactionIds: list[RpcAcceptedTransactionIds]


class ResolveFinalityConflictResponse(TypedDict):
    """Response from resolve_finality_conflict."""
    pass


class SubmitBlockResponse(TypedDict):
    """Response from submit_block."""
    report: str


class SubmitTransactionResponse(TypedDict):
    """Response from submit_transaction."""
    transactionId: str


class SubmitTransactionReplacementResponse(TypedDict):
    """Response from submit_transaction_replacement."""
    transactionId: str
    replacedTransaction: RpcTransaction

