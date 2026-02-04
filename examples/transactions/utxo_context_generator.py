import asyncio
from kaspa import (
    Generator,
    NetworkId,
    PrivateKey,
    Resolver,
    RpcClient,
    UtxoContext,
    UtxoProcessor,
    kaspa_to_sompi,
)

TESTNET_ID = "testnet-10"
PRIVATE_KEY = "389840d7696e89c38856a066175e8e92697f0cf182b854c883237a50acaf1f69"


async def main():
    private_key = PrivateKey(PRIVATE_KEY)
    source_address = private_key.to_keypair().to_address("testnet")

    client = RpcClient(resolver=Resolver(), network_id=TESTNET_ID)
    await client.connect()

    server_info = await client.get_server_info()
    if not server_info.get("isSynced"):
        print("Node is not synced yet.")
        await client.disconnect()
        return

    processor = UtxoProcessor(client, NetworkId(TESTNET_ID))
    await processor.start()

    context = UtxoContext(processor)
    await context.track_addresses([source_address])

    balance = context.balance
    if balance is None:
        print("Balance is not available yet.")
        await processor.stop()
        await client.disconnect()
        return

    min_required = kaspa_to_sompi(0.2) + 1_000
    if balance.mature <= min_required:
        print("Not enough funds to send transaction.")
        await processor.stop()
        await client.disconnect()
        return

    if context.mature_length == 0:
        print("No mature UTXOs for this address. Fund it first.")
        await processor.stop()
        await client.disconnect()
        return

    print(f"Pending before: {len(context.pending())}")

    generator = Generator(
        entries=context,
        change_address=source_address,
        outputs=[{"address": source_address, "amount": kaspa_to_sompi(0.2)}],
        priority_fee=kaspa_to_sompi(0.0001),
    )

    for pending_tx in generator:
        pending_tx.sign([private_key])
        tx_id = await pending_tx.submit(client)
        print(f"Submitted tx: {tx_id}")

    print(generator.summary().transactions)
    print(f"Pending after: {len(context.pending())}")

    await processor.stop()
    await client.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
