import asyncio

from kaspa import (
    Mnemonic,
    PrvKeyDataVariantKind,
    Wallet
)

WALLET_SECRET = "walletSecret"

async def main():
    wallet = Wallet()
    print("--- wallet_create() ")
    print(await wallet.wallet_create(WALLET_SECRET, "tmp2", True, "title2", None))

    print("--- wallet_close() ")
    print(await wallet.wallet_close())

    print("--- wallet_eumerate() ")
    print(await wallet.wallet_enumerate())

    print("--- wallet_open() ")
    print(await wallet.wallet_open(WALLET_SECRET, True, "tmp2"))

    print("--- wallet_reload() ")
    print(await wallet.wallet_reload(True))

    # print("--- wallet_rename() ")
    # print(await wallet.wallet_rename("mySecret", "title2renamed", "tmp2renamed"))

    # print("--- wallet_change_secret() ")
    # print(await wallet.wallet_change_secret("mySecret", "mySecretNew"))

    # print("--- wallet_export() ")
    # wallet_data_hex = await wallet.wallet_export("mySecretNew", True)
    # print(wallet_data_hex)

    # print("--- wallet_import() ")
    # print(await wallet.wallet_import("mySecretNew", wallet_data_hex))

    print("--- prv_key_data_crete()")
    print(await wallet.prv_key_data_create(
        wallet_secret=WALLET_SECRET,
        name=None,
        payment_secret=None,
        secret=Mnemonic.random().phrase,
        kind=PrvKeyDataVariantKind.Mnemonic
    ))

    print("--- prv_key_data_enumerate()")
    prv_key_ids = await wallet.prv_key_data_enumerate()
    print(prv_key_ids)

    print("--- prv_key_data_get()")
    print(await wallet.prv_key_data_get(
        wallet_secret=WALLET_SECRET,
        prv_key_data_id=prv_key_ids[0].id
    ))

    print("--- accounts_create_bip32")
    print(await wallet.accounts_create_bip32(
        wallet_secret=WALLET_SECRET,
        account_name=None,
        account_index=None,
        prv_key_data_id=prv_key_ids[0].id,
        payment_secret=None
    ))

    print("--- accounts_enumerate()")
    print(await wallet.accounts_enumerate())


if __name__ == "__main__":
    asyncio.run(main())