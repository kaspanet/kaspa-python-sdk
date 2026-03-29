"""
TypedDict definitions for Kaspa RPC request/response messages & contained types.

This file is manually maintained and appended to python/kaspa/__init__.pyi via stub gen process.

Long term, attempts should be made to auto generate.
"""

from enum import Enum
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


class RpcVerboseData(TypedDict):
    """Represent Kaspa transaction input verbose data"""
    ...


class RpcTransactionInput(TypedDict):
    """A transaction input."""
    previousOutpoint: RpcOutpoint
    signatureScript: str
    sequence: int
    sigOpCount: int
    verboseData: RpcVerboseData | None


class RpcTransactionOutputVerboseData(TypedDict):
    """Verbose data for a transaction output."""
    scriptPublicKeyType: str
    scriptPublicKeyAddress: str


class RpcTransactionOutput(TypedDict):
    """A transaction output."""
    value: int
    scriptPublicKey: str
    verboseData: RpcTransactionOutputVerboseData | None


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
    verboseData: RpcTransactionVerboseData | None


class RpcTransactionVerboseData(TypedDict, total=False):
    """Verbose transaction data."""
    transactionId: str
    hash: str
    computeMass: int
    blockHash: str
    blockTime: int


class RpcBlockHeader(TypedDict):
    """A block header."""
    hash: str
    version: int
    parentsByLevel: list[list[str]]
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


class RpcRawHeader(TypedDict):
    """A raw block header without a cached hash."""
    version: int
    parentsByLevel: list[list[str]]
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


class RpcRawBlock(TypedDict):
    """
        Raw Rpc block type - without a cached header hash and without verbose data.
        Used for mining APIs (get_block_template & submit_block)
    """
    header: RpcRawHeader
    transactions: list[RpcTransaction]


class RpcMempoolEntry(TypedDict):
    """A mempool entry."""
    fee: int
    transaction: RpcTransaction
    is_orphan: bool


class RpcMempoolEntryByAddress(TypedDict):
    """Mempool entries for a specific address."""
    address: str
    sending: list[RpcMempoolEntry]
    receiving: list[RpcMempoolEntry]


class RpcAcceptedTransactionIds(TypedDict):
    """Accepted transaction IDs for a block."""
    acceptingBlockHash: str
    acceptedTransactionIds: list[str]


class ProcessMetrics(TypedDict):
    """Process metrics.
    
    Category: RPC/Types
    """
    residentSetSize: int
    virtualMemorySize: int
    coreNum: int
    cpuUsage: float
    fdNum: int
    diskIoReadBytes: int
    diskIoWriteBytes: int
    diskIoReadPerSec: float
    diskIoWritePerSec: float


class RpcStorageMetrics(TypedDict):
    """Storage metrics."""
    storageSizeBytes: int


class ConsensusMetrics(TypedDict):
    """Consensus metrics.
    
    Category: RPC/Types
    """
    nodeBlocksSubmittedCount: int
    nodeHeadersProcessedCount: int
    nodeDependenciesProcessedCount: int
    nodeBodiesProcessedCount: int
    nodeTransactionsProcessedCount: int
    nodeChainBlocksProcessedCount: int
    nodeMassProcessedCount: int
    nodeDatabaseBlocksCount: int
    nodeDatabaseHeadersCount: int
    networkMempoolSize: int
    networkTipHashesCount: int
    networkDifficulty: float
    networkPastMedianTime: int
    networkVirtualParentHashesCount: int
    networkVirtualDaaScore: int


class ConnectionMetrics(TypedDict):
    """Connection metrics.
    
    Category: RPC/Types
    """
    borshLiveConnections: int
    borshConnectionAttempts: int
    borshHandshakeFailures: int
    jsonLiveConnections: int
    jsonConnectionAttempts: int
    jsonHandshakeFailures: int
    activePeers: int


class RpcPeerInfo(TypedDict):
    """Peer information."""
    id: str
    address: RpcPeerAddress
    last_ping_duration: int
    is_outbound: bool
    time_offset: int
    user_agent: str
    advertised_protocol_version: int
    time_connected: int
    is_ibd_peer: bool


class BandwidthMetrics(TypedDict):
    """Bandwidth metrics for various protocols.
    
    Category: RPC/Types
    """
    borshBytesTx: int
    borshBytesRx: int
    jsonBytesTx: int
    jsonBytesRx: int
    p2pBytesTx: int
    p2pBytesRx: int
    grpcBytesTx: int
    grpcBytesRx: int


class RpcPeerAddress(TypedDict):
    """A peer address."""
    ip: str
    port: int


class RpcDataVerbosityLevel(Enum):
    """Verbosity level for GetVirtualChainFromBlockV2Request"""
    _None = 0,
    Low = 1,
    High = 2,
    Full = 3,


class RpcOptionalHeader(TypedDict):
    """Represents a block header with optional fields populated based on verbosity level.

    Fields are included based on the RpcDataVerbosityLevel specified in the request.
    Each attribute is only populated when the verbosity level meets or exceeds
    the required level for that field.

    Attributes:
        hash: The block hash. Level: None (always included).
        version: Block version number. Level: Low.
        parentsByLevel: Compressed parent block hashes by level. Level: High.
        hashMerkleRoot: Merkle root of block hashes. Level: High.
        acceptedIdMerkleRoot: Merkle root of accepted transaction IDs. Level: High.
        utxoCommitment: UTXO commitment hash. Level: Full.
        timestamp: Block timestamp in milliseconds. Level: Low.
        bits: Difficulty target bits. Level: Low.
        nonce: Block nonce. Level: Low.
        daaScore: Difficulty adjustment algorithm score. Level: Low.
        blueWork: Cumulative blue work. Level: Low.
        blueScore: Blue score of the block. Level: Low.
        pruningPoint: Pruning point block hash. Level: Full.
    """
    hash: str | None
    version: int | None
    parentsByLevel: list[tuple[int, list[str]]] | None
    hashMerkleRoot: str | None
    acceptedIdMerkleRoot: str | None
    utxoCommitment: str | None
    timestamp: int | None
    bits: int | None
    nonce: int | None
    daaScore: int | None
    blueWork: str | None
    blueScore: int | None
    pruningPoint: str | None


class RpcOptionalTransactionOutpoint(TypedDict):
    """Represents a Kaspa transaction outpoint"""
    transactionId: str | None
    index: int | None


class RpcOptionalUtxoEntryVerboseData(TypedDict):
    """Represents verbose data for a UTXO entry with optional fields based on verbosity level.

    Attributes:
        scriptPublicKeyType: The type/class of the script public key. Level: Low.
        scriptPublicKeyAddress: The address derived from the script public key. Level: Low.
    """
    scriptPublicKeyType: str | None
    scriptPublicKeyAddress: str | None


class RpcOptionalUtxoEntry(TypedDict):
    """Represents a UTXO entry with optional fields based on verbosity level.

    Attributes:
        amount: The amount in sompi. Level: High.
        scriptPublicKey: The script public key. Level: High.
        blockDaaScore: The DAA score of the block containing this UTXO. Level: Full.
        isCoinbase: Whether this UTXO is from a coinbase transaction. Level: High.
        verboseData: Additional verbose data for this UTXO entry.
    """
    amount: int | None
    scriptPublicKey: str | None
    blockDaaScore: int | None
    isCoinbase: bool | None
    verboseData: RpcOptionalUtxoEntryVerboseData | None


class RpcOptionalTransactionInputVerboseData(TypedDict):
    """Represent Kaspa transaction input verbose data"""
    utxoEntry: RpcOptionalUtxoEntry | None


class RpcOptionalTransactionOutputVerboseData(TypedDict):
    """Represents verbose data for a transaction output with optional fields based on verbosity level.

    Attributes:
        scriptPublicKeyType: The type/class of the script public key. Level: Low.
        scriptPublicKeyAddress: The address derived from the script public key. Level: Low.
    """
    scriptPublicKeyType: str | None
    scriptPublicKeyAddress: str | None


class RpcOptionalTransactionOutput(TypedDict):
    """Represents a transaction output with optional fields based on verbosity level.

    Attributes:
        value: The output value in sompi. Level: Low.
        scriptPublicKey: The script public key for this output. Level: Low.
        verboseData: Additional verbose data for this output.
    """
    value: int | None
    scriptPublicKey: str | None
    verboseData: RpcOptionalTransactionOutputVerboseData | None


class RpcOptionalTransactionInput(TypedDict):
    """Represents a transaction input with optional fields based on verbosity level.

    Attributes:
        previousOutpoint: The outpoint being spent. Level: High.
        signatureScript: The signature script (hex encoded). Level: Low.
        sequence: The sequence number. Level: High.
        sigOpCount: The signature operation count. Level: High.
        verboseData: Additional verbose data for this input.
    """
    previousOutpoint: RpcOptionalTransactionOutpoint | None # TODO
    signatureScript: str | None
    sequence: int | None
    sigOpCount: int | None
    verboseData: RpcOptionalTransactionInputVerboseData | None # TODO


class RpcOptionalTransactionVerboseData(TypedDict):
    """Represents verbose data for a transaction with optional fields based on verbosity level.

    Attributes:
        transactionId: The transaction ID. Level: Low.
        hash: The transaction hash. Level: Low.
        computeMass: The computed mass of the transaction. Level: High.
        blockHash: The hash of the block containing this transaction. Level: Low.
        blockTime: The timestamp of the block containing this transaction. Level: Low.
    """
    transactionId: str | None
    hash: str | None
    computeMass: int | None
    blockHash: str | None
    blockTime: int | None


class RpcOptionalTransaction(TypedDict):
    """Represents a transaction with optional fields based on verbosity level.

    Attributes:
        version: The transaction version. Level: Full.
        inputs: List of transaction inputs.
        outputs: List of transaction outputs.
        lockTime: The lock time of the transaction. Level: Full.
        subnetworkId: The subnetwork ID. Level: Full.
        gas: The gas limit. Level: Full.
        payload: The transaction payload (hex encoded). Level: High.
        mass: The transaction mass. Level: High.
        verboseData: Additional verbose data for this transaction.
    """
    version: int | None
    inputs: list[RpcOptionalTransactionInput] | None
    outputs: list[RpcOptionalTransactionOutput] | None
    lockTime: int | None
    subnetworkId: str | None
    gas: int | None
    payload: list[int] | None
    mass: int | None
    verboseData: RpcOptionalTransactionVerboseData | None


class RpcChainBlockAcceptedTransactions(TypedDict):
    """Transaction acceptance data returned by GetVirtualChainFromBlockV2"""
    chainBlockHeader: RpcOptionalHeader
    acceptedTransactions: list[RpcOptionalTransaction]


# =============================================================================
# Request Types
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
    customMetrics: bool


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
    minConfirmationCount: int | None


class GetVirtualChainFromBlockV2Request(TypedDict):
    """Request for get_virtual_chain_from_block_v2."""
    startHash: str
    dataVerbosityLevel: RpcDataVerbosityLevel | None
    minConfirmationCount: int | None


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
    network: str
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
    serverTime: int
    processMetrics: ProcessMetrics
    connectionMetrics: ConnectionMetrics
    bandwidthMetrics: BandwidthMetrics
    consensusMetrics: ConsensusMetrics
    storageMetrics: RpcStorageMetrics
    customMetrics: dict | None


class ConnectionsProfileData(TypedDict):
    """Profile data for connection resource usage.
    
    Category: RPC/Types
    """
    cpuUsage: int
    memoryUsage: int


class GetConnectionsResponse(TypedDict):
    """Response from get_connections."""
    clients: int
    peers: int
    profileData: ConnectionsProfileData | None


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
    rpcApiVersion: int
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


class GetSystemInfoResponse(TypedDict):
    """Response from get_system_info."""
    version: str
    systemId: list[int] | None
    gitHash: list[int] | None
    cpuPhysicalCores: int
    totalMemory: int
    fdLimit: int
    proxySocketLimitPerCpuCore: int | None


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
    block: RpcRawBlock
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
    headers: list[RpcBlockHeader]


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
    returnAddress: str | None


class GetVirtualChainFromBlockResponse(TypedDict):
    """Response from get_virtual_chain_from_block."""
    removedChainBlockHashes: list[str]
    addedChainBlockHashes: list[str]
    acceptedTransactionIds: list[RpcAcceptedTransactionIds]


class GetVirtualChainFromBlockV2Response(TypedDict):
    """Response from get_virtual_chain_from_block_v2."""
    removedChainBlockHashes: list[str]
    addedChainBlockHashes: list[str]
    chainBlockAcceptedTransactions: list[RpcChainBlockAcceptedTransactions]


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

