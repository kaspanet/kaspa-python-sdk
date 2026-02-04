import asyncio
from kaspa import (
    NetworkId,
    PrivateKey,
    Resolver,
    RpcClient,
    UtxoContext,
    UtxoProcessor,
    create_transactions,
    estimate_transactions,
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

    priority_fee = kaspa_to_sompi(0.0001)
    min_required = kaspa_to_sompi(0.2) + priority_fee
    if balance.mature <= min_required:
        print("Not enough funds to create transactions.")
        await processor.stop()
        await client.disconnect()
        return

    if context.mature_length == 0:
        print("No mature UTXOs for this address. Fund it first.")
        await processor.stop()
        await client.disconnect()
        return

    outputs = [{"address": source_address, "amount": kaspa_to_sompi(0.2)}]

    summary = estimate_transactions(
        entries=context,
        change_address=source_address,
        outputs=outputs,
        priority_fee=priority_fee,
    )
    print(summary)

    result = create_transactions(
        entries=context,
        change_address=source_address,
        outputs=outputs,
        priority_fee=priority_fee,
    )

    for pending_tx in result["transactions"]:
        pending_tx.sign([private_key])
        tx_id = await pending_tx.submit(client)
        print(f"Submitted tx: {tx_id}")

    print(result["summary"])

    await processor.stop()
    await client.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
