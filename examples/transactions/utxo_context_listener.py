import asyncio

from kaspa import (
    NetworkId,
    Resolver,
    RpcClient,
    UtxoContext,
    UtxoProcessor,
)

TEST_ADDRESS = "kaspatest:qr0lr4ml9fn3chekrqmjdkergxl93l4wrk3dankcgvjq776s9wn9jhtkdksae"


def format_event(event: dict) -> str:
    event_type = event.get("type")
    data = event.get("data")

    if event_type in ("pending", "maturity", "reorg", "stasis", "discovery") and isinstance(
        data, dict
    ):
        tx_id = data.get("id")
        return f"{event_type}: tx_id={tx_id}"

    return f"{event_type}: {data}"


async def main():
    client = RpcClient(resolver=Resolver(), network_id="testnet-10")
    await client.connect()
    print(f"Client is connected: {client.is_connected}")

    processor = UtxoProcessor(client, NetworkId("testnet-10"))

    loop = asyncio.get_running_loop()
    got_start = asyncio.Event()

    def on_event(event: dict):
        print(format_event(event))

        # Listener callbacks may run on a background thread.
        # Use thread-safe asyncio bridging for any async signaling.
        if event.get("type") == "utxo-proc-start":
            loop.call_soon_threadsafe(got_start.set)

    processor.add_event_listener(
        [
            "utxo-proc-start",
            "utxo-proc-stop",
            "pending",
            "maturity",
            "reorg",
            "stasis",
            "discovery",
            "balance",
            "utxo-proc-error",
            "error",
        ],
        on_event,
    )

    await processor.start()
    await got_start.wait()

    context = UtxoContext(processor)
    await context.track_addresses([TEST_ADDRESS])

    print("Tracking addresses; waiting for events...")
    await asyncio.sleep(60.0)

    processor.remove_event_listener(on_event)
    await processor.stop()
    await client.disconnect()


if __name__ == "__main__":
    asyncio.run(main())

