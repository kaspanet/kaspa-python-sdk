"""
Integration tests for RPC client functionality.

These tests require network access and connect to the Kaspa testnet.
"""

import pytest
import asyncio

from kaspa import (
    RpcClient,
    Resolver,
    Address,
    Encoding,
)


class TestResolver:
    """Tests for Resolver class."""

    async def test_create_resolver_default(self):
        """Test creating a Resolver with default URLs."""
        resolver = Resolver()
        assert resolver is not None

    async def test_create_resolver_custom_urls(self):
        """Test creating a Resolver with custom URLs."""
        custom_urls = ["wss://resolver.kaspa.stream"]
        resolver = Resolver(urls=custom_urls)
        assert resolver is not None

    async def test_resolver_urls(self):
        """Test getting resolver URLs."""
        resolver = Resolver()
        urls = resolver.urls()
        assert urls is not None
        assert isinstance(urls, list)

    async def test_resolver_get_url_with_string_encoding(self):
        """Test getting a node URL from resolver with string encoding."""
        resolver = Resolver()
        url = await resolver.get_url("borsh", "testnet-10")
        assert url is not None
        assert isinstance(url, str)
        assert url.startswith("wss://") or url.startswith("ws://")

    async def test_resolver_get_url_with_enum_encoding(self):
        """Test getting a node URL from resolver with Encoding enum."""
        resolver = Resolver()
        url = await resolver.get_url(Encoding.Borsh, "testnet-10")
        assert url is not None
        assert isinstance(url, str)
        assert url.startswith("wss://") or url.startswith("ws://")

    async def test_resolver_get_node_with_string_encoding(self):
        """Test getting node info from resolver with string encoding."""
        resolver = Resolver()
        node = await resolver.get_node("borsh", "testnet-10")
        assert node is not None
        assert isinstance(node, dict)

    async def test_resolver_get_node_with_enum_encoding(self):
        """Test getting node info from resolver with Encoding enum."""
        resolver = Resolver()
        node = await resolver.get_node(Encoding.Borsh, "testnet-10")
        assert node is not None
        assert isinstance(node, dict)


class TestRpcClientConnection:
    """Tests for RPC client connection functionality."""

    async def test_create_rpc_client_with_resolver(self):
        """Test creating an RPC client with a resolver."""
        client = RpcClient(resolver=Resolver(), network_id="testnet-10")
        assert client is not None

    async def test_create_rpc_client_with_string_encoding(self):
        """Test creating an RPC client with string encoding."""
        client = RpcClient(resolver=Resolver(), network_id="testnet-10", encoding="borsh")
        assert client is not None

    async def test_create_rpc_client_with_enum_encoding(self):
        """Test creating an RPC client with Encoding enum."""
        client = RpcClient(resolver=Resolver(), network_id="testnet-10", encoding=Encoding.Borsh)
        assert client is not None

    async def test_create_rpc_client_with_json_string_encoding(self):
        """Test creating an RPC client with 'json' string encoding."""
        client = RpcClient(resolver=Resolver(), network_id="testnet-10", encoding="json")
        assert client is not None

    async def test_create_rpc_client_with_json_enum_encoding(self):
        """Test creating an RPC client with Encoding.SerdeJson enum."""
        client = RpcClient(resolver=Resolver(), network_id="testnet-10", encoding=Encoding.SerdeJson)
        assert client is not None

    async def test_rpc_client_connect_disconnect(self):
        """Test connecting and disconnecting from RPC."""
        client = RpcClient(resolver=Resolver(), network_id="testnet-10")
        
        await client.connect()
        assert client.is_connected is True
        
        await client.disconnect()
        assert client.is_connected is False

    async def test_rpc_client_encoding(self, testnet_rpc_client):
        """Test getting RPC encoding."""
        encoding = testnet_rpc_client.encoding
        assert encoding is not None
        assert isinstance(encoding, str)

    async def test_rpc_client_url(self, testnet_rpc_client):
        """Test getting RPC client URL."""
        url = testnet_rpc_client.url
        assert url is not None
        assert isinstance(url, str)

    async def test_rpc_client_node_id(self, testnet_rpc_client):
        """Test getting RPC node ID."""
        node_id = testnet_rpc_client.node_id
        assert node_id is not None


class TestRpcClientCalls:
    """Tests for RPC calls."""

    # Basic calls
    async def test_get_info(self, testnet_rpc_client):
        """Test get_info RPC call."""
        result = await testnet_rpc_client.get_info()
        assert result is not None
        assert isinstance(result, dict)

    async def test_get_server_info(self, testnet_rpc_client):
        """Test get_server_info RPC call."""
        result = await testnet_rpc_client.get_server_info()
        assert result is not None
        assert isinstance(result, dict)

    async def test_get_block_count(self, testnet_rpc_client):
        """Test get_block_count RPC call."""
        result = await testnet_rpc_client.get_block_count()
        assert result is not None
        assert isinstance(result, dict)
        assert "blockCount" in result or "headerCount" in result

    async def test_get_block_dag_info(self, testnet_rpc_client):
        """Test get_block_dag_info RPC call."""
        result = await testnet_rpc_client.get_block_dag_info()
        assert result is not None
        assert isinstance(result, dict)

    async def test_get_coin_supply(self, testnet_rpc_client):
        """Test get_coin_supply RPC call."""
        result = await testnet_rpc_client.get_coin_supply()
        assert result is not None
        assert isinstance(result, dict)

    async def test_get_sink(self, testnet_rpc_client):
        """Test get_sink RPC call."""
        result = await testnet_rpc_client.get_sink()
        assert result is not None
        assert isinstance(result, dict)

    async def test_get_sink_blue_score(self, testnet_rpc_client):
        """Test get_sink_blue_score RPC call."""
        result = await testnet_rpc_client.get_sink_blue_score()
        assert result is not None
        assert isinstance(result, dict)

    async def test_get_sync_status(self, testnet_rpc_client):
        """Test get_sync_status RPC call."""
        result = await testnet_rpc_client.get_sync_status()
        assert result is not None
        assert isinstance(result, dict)

    async def test_get_current_network(self, testnet_rpc_client):
        """Test get_current_network RPC call."""
        result = await testnet_rpc_client.get_current_network()
        assert result is not None
        assert isinstance(result, dict)

    async def test_get_fee_estimate(self, testnet_rpc_client):
        """Test get_fee_estimate RPC call."""
        result = await testnet_rpc_client.get_fee_estimate()
        assert result is not None
        assert isinstance(result, dict)
        assert "estimate" in result

    async def test_ping(self, testnet_rpc_client):
        """Test ping RPC call."""
        result = await testnet_rpc_client.ping()
        assert result is not None
        assert isinstance(result, dict)

    # Balance and UTXO queries
    async def test_get_balance_by_address(self, testnet_rpc_client):
        """Test get_balance_by_address RPC call."""
        test_address = "kaspatest:qr0lr4ml9fn3chekrqmjdkergxl93l4wrk3dankcgvjq776s9wn9jhtkdksae"
        
        result = await testnet_rpc_client.get_balance_by_address({
            "address": test_address
        })
        assert result is not None
        assert isinstance(result, dict)

    async def test_get_balances_by_addresses(self, testnet_rpc_client):
        """Test get_balances_by_addresses RPC call."""
        test_address = "kaspatest:qr0lr4ml9fn3chekrqmjdkergxl93l4wrk3dankcgvjq776s9wn9jhtkdksae"
        
        result = await testnet_rpc_client.get_balances_by_addresses({
            "addresses": [test_address]
        })
        assert result is not None
        assert isinstance(result, dict)

    async def test_get_utxos_by_addresses(self, testnet_rpc_client):
        """Test get_utxos_by_addresses RPC call."""
        test_address = "kaspatest:qr0lr4ml9fn3chekrqmjdkergxl93l4wrk3dankcgvjq776s9wn9jhtkdksae"
        
        result = await testnet_rpc_client.get_utxos_by_addresses({
            "addresses": [test_address]
        })
        assert result is not None
        assert isinstance(result, dict)

    # Network information queries
    async def test_get_connected_peer_info(self, testnet_rpc_client):
        """Test get_connected_peer_info RPC call."""
        result = await testnet_rpc_client.get_connected_peer_info()
        assert result is not None
        assert isinstance(result, dict)

    async def test_get_peer_addresses(self, testnet_rpc_client):
        """Test get_peer_addresses RPC call."""
        result = await testnet_rpc_client.get_peer_addresses()
        assert result is not None
        assert isinstance(result, dict)

    async def test_estimate_network_hashes_per_second(self, testnet_rpc_client):
        """Test estimate_network_hashes_per_second RPC call."""
        result = await testnet_rpc_client.estimate_network_hashes_per_second({
            "windowSize": 1000
        })
        assert result is not None
        assert isinstance(result, dict)

    # Mempool queries
    async def test_get_mempool_entries(self, testnet_rpc_client):
        """Test get_mempool_entries RPC call."""
        result = await testnet_rpc_client.get_mempool_entries({
            "includeOrphanPool": True,
            "filterTransactionPool": False
        })
        assert result is not None
        assert isinstance(result, dict)

    async def test_get_mempool_entries_by_addresses(self, testnet_rpc_client):
        """Test get_mempool_entries_by_addresses RPC call."""
        test_address = "kaspatest:qr0lr4ml9fn3chekrqmjdkergxl93l4wrk3dankcgvjq776s9wn9jhtkdksae"
        
        result = await testnet_rpc_client.get_mempool_entries_by_addresses({
            "addresses": [test_address],
            "includeOrphanPool": True,
            "filterTransactionPool": False
        })
        assert result is not None
        assert isinstance(result, dict)

