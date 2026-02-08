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

    async def test_receive_utxo_proc_start_target_filter(self, testnet_rpc_client):
        processor = UtxoProcessor(testnet_rpc_client, NetworkId("testnet-10"))

        loop = asyncio.get_running_loop()
        got_start = asyncio.Event()
        received_types = []

        def callback(event):
            received_types.append(event.get("type"))
            if event.get("type") == "utxo-proc-start":
                loop.call_soon_threadsafe(got_start.set)

        processor.add_event_listener("utxo-proc-start", callback)

        await processor.start()
        try:
            await asyncio.wait_for(got_start.wait(), timeout=30.0)
        finally:
            await processor.stop()

        assert "utxo-proc-start" in received_types
        assert "utxo-proc-stop" not in received_types

    async def test_callback_exception_does_not_break_dispatch(self, testnet_rpc_client):
        processor = UtxoProcessor(testnet_rpc_client, NetworkId("testnet-10"))

        loop = asyncio.get_running_loop()
        got_start = asyncio.Event()

        def bad_callback(event):
            if event.get("type") == "utxo-proc-start":
                raise RuntimeError("boom")

        def good_callback(event):
            if event.get("type") == "utxo-proc-start":
                loop.call_soon_threadsafe(got_start.set)

        processor.add_event_listener("utxo-proc-start", bad_callback)
        processor.add_event_listener("utxo-proc-start", good_callback)

        await processor.start()
        try:
            await asyncio.wait_for(got_start.wait(), timeout=30.0)
        finally:
            await processor.stop()
