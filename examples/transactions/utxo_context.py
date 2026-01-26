import asyncio
from kaspa import (
    RpcClient,
    Resolver,
    UtxoProcessor,
    UtxoContext,
    NetworkId,
)

TEST_ADDRESS = "kaspatest:qr0lr4ml9fn3chekrqmjdkergxl93l4wrk3dankcgvjq776s9wn9jhtkdksae"


async def main():
    client = RpcClient(resolver=Resolver(), network_id="testnet-10")
    await client.connect()
    print(f"Client is connected: {client.is_connected}")

    processor = UtxoProcessor(client, NetworkId("testnet-10"))
    await processor.start()
    print(f"Processor active: {processor.is_active}")

    context = UtxoContext(processor)
    await context.track_addresses([TEST_ADDRESS])

    mature_length = context.mature_length
    end = min(5, mature_length)
    print(f"Mature length: {mature_length}")
    print(f"Mature range (0..{end}): {context.mature_range(from_=0, to=end)}")
    print(f"Balance: {context.balance}")
    print(f"Balance strings: {context.balance_strings}")

    await processor.stop()
    await client.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
