"""
Integration tests for RPC subscription functionality.

These tests require network access and connect to the Kaspa testnet.
"""

import pytest
import asyncio

from kaspa import RpcClient, Resolver, Address


# Simple subscriptions that take no arguments
SIMPLE_SUBSCRIPTIONS = [
    ("virtual_daa_score_changed", "subscribe_virtual_daa_score_changed",
     "unsubscribe_virtual_daa_score_changed"),
    ("sink_blue_score_changed", "subscribe_sink_blue_score_changed",
     "unsubscribe_sink_blue_score_changed"),
    ("block_added", "subscribe_block_added", "unsubscribe_block_added"),
    ("finality_conflict", "subscribe_finality_conflict",
     "unsubscribe_finality_conflict"),
    ("finality_conflict_resolved", "subscribe_finality_conflict_resolved",
     "unsubscribe_finality_conflict_resolved"),
    ("new_block_template", "subscribe_new_block_template",
     "unsubscribe_new_block_template"),
    ("pruning_point_utxo_set_override", "subscribe_pruning_point_utxo_set_override",
     "unsubscribe_pruning_point_utxo_set_override"),
]


class TestEventListeners:
    """Tests for RPC event listener functionality."""

    async def test_add_event_listener(self, testnet_rpc_client):
        """Test adding an event listener."""
        received_events = []

        def callback(event_data):
            received_events.append(event_data)

        testnet_rpc_client.add_event_listener(
            "virtual-daa-score-changed", callback)
        # Listener should be added without error
        assert True

    async def test_remove_event_listener(self, testnet_rpc_client):
        """Test removing an event listener."""
        def callback(event_data):
            pass

        testnet_rpc_client.add_event_listener(
            "virtual-daa-score-changed", callback)
        testnet_rpc_client.remove_event_listener(
            "virtual-daa-score-changed", callback)
        # Listener should be removed without error
        assert True

    async def test_remove_all_event_listeners(self, testnet_rpc_client):
        """Test removing all event listeners."""
        def callback1(event_data):
            pass

        def callback2(event_data):
            pass

        testnet_rpc_client.add_event_listener(
            "virtual-daa-score-changed", callback1)
        testnet_rpc_client.add_event_listener("block-added", callback2)
        testnet_rpc_client.remove_all_event_listeners()
        # All listeners should be removed without error
        assert True


class TestSimpleSubscriptions:
    """Tests for simple subscribe/unsubscribe operations that take no arguments."""

    @pytest.mark.parametrize("name,subscribe_method,unsubscribe_method", SIMPLE_SUBSCRIPTIONS)
    async def test_subscribe(self, testnet_rpc_client, name, subscribe_method, unsubscribe_method):
        """Test subscribing to various events."""
        await getattr(testnet_rpc_client, subscribe_method)()
        # Should subscribe without error
        assert True

    @pytest.mark.parametrize("name,subscribe_method,unsubscribe_method", SIMPLE_SUBSCRIPTIONS)
    async def test_subscribe_and_unsubscribe(self, testnet_rpc_client, name, subscribe_method, unsubscribe_method):
        """Test subscribing and then unsubscribing from various events."""
        await getattr(testnet_rpc_client, subscribe_method)()
        await getattr(testnet_rpc_client, unsubscribe_method)()
        # Should unsubscribe without error
        assert True


class TestVirtualChainSubscription:
    """Tests for virtual chain subscription (requires parameters)."""

    async def test_subscribe_virtual_chain_changed(self, testnet_rpc_client):
        """Test subscribing to virtual chain changes."""
        await testnet_rpc_client.subscribe_virtual_chain_changed(
            include_accepted_transaction_ids=False
        )
        # Should subscribe without error
        assert True

    async def test_unsubscribe_virtual_chain_changed(self, testnet_rpc_client):
        """Test unsubscribing from virtual chain changes."""
        await testnet_rpc_client.subscribe_virtual_chain_changed(
            include_accepted_transaction_ids=False
        )
        await testnet_rpc_client.unsubscribe_virtual_chain_changed(
            include_accepted_transaction_ids=False
        )
        # Should unsubscribe without error
        assert True


class TestUtxoSubscription:
    """Tests for UTXO change subscription (requires address parameter)."""

    async def test_subscribe_utxos_changed(self, testnet_rpc_client):
        """Test subscribing to UTXO changes for specific addresses."""
        test_address = Address(
            "kaspatest:qr0lr4ml9fn3chekrqmjdkergxl93l4wrk3dankcgvjq776s9wn9jhtkdksae")

        await testnet_rpc_client.subscribe_utxos_changed([test_address])
        # Should subscribe without error
        assert True

    async def test_unsubscribe_utxos_changed(self, testnet_rpc_client):
        """Test unsubscribing from UTXO changes."""
        test_address = Address(
            "kaspatest:qr0lr4ml9fn3chekrqmjdkergxl93l4wrk3dankcgvjq776s9wn9jhtkdksae")

        await testnet_rpc_client.subscribe_utxos_changed([test_address])
        await testnet_rpc_client.unsubscribe_utxos_changed([test_address])
        # Should unsubscribe without error
        assert True


class TestEventReceiving:
    """Tests for actually receiving events (may take time)."""

    async def test_receive_virtual_daa_score_event(self, testnet_rpc_client):
        """Test receiving a virtual DAA score change event."""
        received_events = []
        event_received = asyncio.Event()

        def callback(event_data):
            received_events.append(event_data)
            event_received.set()

        testnet_rpc_client.add_event_listener(
            "virtual-daa-score-changed", callback)
        await testnet_rpc_client.subscribe_virtual_daa_score_changed()

        await asyncio.wait_for(event_received.wait(), timeout=30.0)
        assert len(received_events) > 0
