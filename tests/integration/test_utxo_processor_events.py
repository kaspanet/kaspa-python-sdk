"""
Integration tests for UtxoProcessor event listeners.

These tests require network access and connect to the Kaspa testnet.
"""

import asyncio

from kaspa import NetworkId, UtxoProcessor


class TestUtxoProcessorEventListeners:
    async def test_receive_utxo_proc_start_stop(self, testnet_rpc_client):
        processor = UtxoProcessor(testnet_rpc_client, NetworkId("testnet-10"))

        loop = asyncio.get_running_loop()
        got_start = asyncio.Event()
        got_stop = asyncio.Event()
        received_types = []

        def callback(event):
            received_types.append(event.get("type"))
            t = event.get("type")
            if t == "utxo-proc-start":
                loop.call_soon_threadsafe(got_start.set)
            elif t == "utxo-proc-stop":
                loop.call_soon_threadsafe(got_stop.set)

        processor.add_event_listener(callback)

        await processor.start()
        try:
            await asyncio.wait_for(got_start.wait(), timeout=30.0)
        finally:
            await processor.stop()

        await asyncio.wait_for(got_stop.wait(), timeout=30.0)
        assert "utxo-proc-start" in received_types
        assert "utxo-proc-stop" in received_types

