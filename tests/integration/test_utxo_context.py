"""
Integration tests for UTXO context functionality.

These tests require network access and connect to the Kaspa testnet.
"""

import pytest

from kaspa import NetworkId, UtxoContext, UtxoProcessor


class TestUtxoContext:
    """Tests for UtxoProcessor/UtxoContext with live RPC."""

    async def test_track_addresses_and_ranges(self, rpc_client, network_id, test_address):
        processor = UtxoProcessor(rpc_client, NetworkId(network_id))
        await processor.start()
        try:
            context = UtxoContext(processor)
            await context.track_addresses([test_address])

            assert isinstance(context.mature_length, int)

            empty = context.mature_range(0, 0)
            assert isinstance(empty, list)
            assert len(empty) == 0

            end = min(1, context.mature_length)
            entries = context.mature_range(0, end)
            assert isinstance(entries, list)
            assert len(entries) <= end

            _ = context.balance
            _ = context.balance_strings
        finally:
            await processor.stop()

    async def test_mature_range_invalid_range(self, rpc_client, network_id):
        processor = UtxoProcessor(rpc_client, NetworkId(network_id))
        await processor.start()
        try:
            context = UtxoContext(processor)
            with pytest.raises(Exception):
                context.mature_range(1, 0)
        finally:
            await processor.stop()

    async def test_track_addresses_invalid_address(self, rpc_client, network_id):
        processor = UtxoProcessor(rpc_client, NetworkId(network_id))
        await processor.start()
        try:
            context = UtxoContext(processor)
            with pytest.raises(Exception):
                await context.track_addresses(["not-a-valid-address"])
        finally:
            await processor.stop()

    async def test_context_invalid_id(self, rpc_client, network_id):
        processor = UtxoProcessor(rpc_client, NetworkId(network_id))
        await processor.start()
        try:
            with pytest.raises(Exception):
                UtxoContext(processor, "not-hex")
        finally:
            await processor.stop()
