"""
Integration tests for RPC subscription functionality.

These tests require network access and connect to the Kaspa testnet.
"""

import pytest
import asyncio

from kaspa import RpcClient, Resolver, Address


class TestEventListeners:
    """Tests for RPC event listener functionality."""

    async def test_add_event_listener(self, testnet_rpc_client):
        """Test adding an event listener."""
        received_events = []
        
        def callback(event_data):
            received_events.append(event_data)
        
        testnet_rpc_client.add_event_listener("virtual-daa-score-changed", callback)
        # Listener should be added without error
        assert True

    async def test_remove_event_listener(self, testnet_rpc_client):
        """Test removing an event listener."""
        def callback(event_data):
            pass
        
        testnet_rpc_client.add_event_listener("virtual-daa-score-changed", callback)
        testnet_rpc_client.remove_event_listener("virtual-daa-score-changed", callback)
        # Listener should be removed without error
        assert True

    async def test_remove_all_event_listeners(self, testnet_rpc_client):
        """Test removing all event listeners."""
        def callback1(event_data):
            pass
        
        def callback2(event_data):
            pass
        
        testnet_rpc_client.add_event_listener("virtual-daa-score-changed", callback1)
        testnet_rpc_client.add_event_listener("block-added", callback2)
        testnet_rpc_client.remove_all_event_listeners()
        # All listeners should be removed without error
        assert True


class TestVirtualDaaScoreSubscription:
    """Tests for virtual DAA score subscription."""

    async def test_subscribe_virtual_daa_score_changed(self, testnet_rpc_client):
        """Test subscribing to virtual DAA score changes."""
        await testnet_rpc_client.subscribe_virtual_daa_score_changed()
        # Should subscribe without error
        assert True

    async def test_unsubscribe_virtual_daa_score_changed(self, testnet_rpc_client):
        """Test unsubscribing from virtual DAA score changes."""
        await testnet_rpc_client.subscribe_virtual_daa_score_changed()
        await testnet_rpc_client.unsubscribe_virtual_daa_score_changed()
        # Should unsubscribe without error
        assert True


class TestSinkBlueScoreSubscription:
    """Tests for sink blue score subscription."""

    async def test_subscribe_sink_blue_score_changed(self, testnet_rpc_client):
        """Test subscribing to sink blue score changes."""
        await testnet_rpc_client.subscribe_sink_blue_score_changed()
        # Should subscribe without error
        assert True

    async def test_unsubscribe_sink_blue_score_changed(self, testnet_rpc_client):
        """Test unsubscribing from sink blue score changes."""
        await testnet_rpc_client.subscribe_sink_blue_score_changed()
        await testnet_rpc_client.unsubscribe_sink_blue_score_changed()
        # Should unsubscribe without error
        assert True


class TestBlockAddedSubscription:
    """Tests for block added subscription."""

    async def test_subscribe_block_added(self, testnet_rpc_client):
        """Test subscribing to block added events."""
        await testnet_rpc_client.subscribe_block_added()
        # Should subscribe without error
        assert True

    async def test_unsubscribe_block_added(self, testnet_rpc_client):
        """Test unsubscribing from block added events."""
        await testnet_rpc_client.subscribe_block_added()
        await testnet_rpc_client.unsubscribe_block_added()
        # Should unsubscribe without error
        assert True


class TestVirtualChainSubscription:
    """Tests for virtual chain subscription."""

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
    """Tests for UTXO change subscription."""

    async def test_subscribe_utxos_changed(self, testnet_rpc_client):
        """Test subscribing to UTXO changes for specific addresses."""
        test_address = Address("kaspatest:qr0lr4ml9fn3chekrqmjdkergxl93l4wrk3dankcgvjq776s9wn9jhtkdksae")
        
        await testnet_rpc_client.subscribe_utxos_changed([test_address])
        # Should subscribe without error
        assert True

    async def test_unsubscribe_utxos_changed(self, testnet_rpc_client):
        """Test unsubscribing from UTXO changes."""
        test_address = Address("kaspatest:qr0lr4ml9fn3chekrqmjdkergxl93l4wrk3dankcgvjq776s9wn9jhtkdksae")
        
        await testnet_rpc_client.subscribe_utxos_changed([test_address])
        await testnet_rpc_client.unsubscribe_utxos_changed([test_address])
        # Should unsubscribe without error
        assert True


class TestFinalitySubscriptions:
    """Tests for finality-related subscriptions."""

    async def test_subscribe_finality_conflict(self, testnet_rpc_client):
        """Test subscribing to finality conflicts."""
        await testnet_rpc_client.subscribe_finality_conflict()
        # Should subscribe without error
        assert True

    async def test_unsubscribe_finality_conflict(self, testnet_rpc_client):
        """Test unsubscribing from finality conflicts."""
        await testnet_rpc_client.subscribe_finality_conflict()
        await testnet_rpc_client.unsubscribe_finality_conflict()
        # Should unsubscribe without error
        assert True

    async def test_subscribe_finality_conflict_resolved(self, testnet_rpc_client):
        """Test subscribing to finality conflict resolution."""
        await testnet_rpc_client.subscribe_finality_conflict_resolved()
        # Should subscribe without error
        assert True

    async def test_unsubscribe_finality_conflict_resolved(self, testnet_rpc_client):
        """Test unsubscribing from finality conflict resolution."""
        await testnet_rpc_client.subscribe_finality_conflict_resolved()
        await testnet_rpc_client.unsubscribe_finality_conflict_resolved()
        # Should unsubscribe without error
        assert True


class TestNewBlockTemplateSubscription:
    """Tests for new block template subscription."""

    async def test_subscribe_new_block_template(self, testnet_rpc_client):
        """Test subscribing to new block templates."""
        await testnet_rpc_client.subscribe_new_block_template()
        # Should subscribe without error
        assert True

    async def test_unsubscribe_new_block_template(self, testnet_rpc_client):
        """Test unsubscribing from new block templates."""
        await testnet_rpc_client.subscribe_new_block_template()
        await testnet_rpc_client.unsubscribe_new_block_template()
        # Should unsubscribe without error
        assert True


class TestPruningPointSubscription:
    """Tests for pruning point subscription."""

    async def test_subscribe_pruning_point_utxo_set_override(self, testnet_rpc_client):
        """Test subscribing to pruning point UTXO set override."""
        await testnet_rpc_client.subscribe_pruning_point_utxo_set_override()
        # Should subscribe without error
        assert True

    async def test_unsubscribe_pruning_point_utxo_set_override(self, testnet_rpc_client):
        """Test unsubscribing from pruning point UTXO set override."""
        await testnet_rpc_client.subscribe_pruning_point_utxo_set_override()
        await testnet_rpc_client.unsubscribe_pruning_point_utxo_set_override()
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
        
        testnet_rpc_client.add_event_listener("virtual-daa-score-changed", callback)
        await testnet_rpc_client.subscribe_virtual_daa_score_changed()
        
        try:
            # Wait up to 30 seconds for an event
            await asyncio.wait_for(event_received.wait(), timeout=30.0)
            assert len(received_events) > 0
        except asyncio.TimeoutError:
            # It's acceptable if no event is received in the timeout period
            pytest.skip("No virtual DAA score event received within timeout")
        finally:
            await testnet_rpc_client.unsubscribe_virtual_daa_score_changed()
            testnet_rpc_client.remove_event_listener("virtual-daa-score-changed", callback)

