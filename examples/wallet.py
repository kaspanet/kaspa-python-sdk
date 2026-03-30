import asyncio

from kaspa import Wallet

async def main():
    wallet = Wallet()
    print(await wallet.wallet_create("mySecret", "tmp2", True, "title2", None))
    print(await wallet.wallet_enumerate())
    print(await wallet.wallet_open("mySecret", "tmp2", True))

if __name__ == "__main__":
    asyncio.run(main())