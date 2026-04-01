import asyncio

from kaspa import Wallet

async def main():
    wallet = Wallet()
    print("--- wallet_create() ")
    print(await wallet.wallet_create("mySecret", "tmp2", True, "title2", None))

    print("--- wallet_close() ")
    print(await wallet.wallet_close())

    print("--- wallet_eumerate() ")
    print(await wallet.wallet_enumerate())

    print("--- wallet_open() ")
    print(await wallet.wallet_open("mySecret", "tmp2", True))

    print("--- wallet_reload() ")
    print(await wallet.wallet_reload(True))

    print("--- wallet_rename() ")
    print(await wallet.wallet_rename("mySecret", "title2renamed", "tmp2renamed"))

    print("--- wallet_change_secret() ")
    print(await wallet.wallet_change_secret("mySecret", "mySecretNew"))

    # print("--- wallet_export() ")
    # wallet_data_hex = await wallet.wallet_export("mySecretNew", True)
    # print(wallet_data_hex)

    # print("--- wallet_import() ")
    # print(await wallet.wallet_import("mySecretNew", wallet_data_hex))

    print("--- accounts_create_bip32")
    print(await wallet.accounts)

    print("--- accounts_enumerate()")
    print(await wallet.accounts_enumerate())


if __name__ == "__main__":
    asyncio.run(main())