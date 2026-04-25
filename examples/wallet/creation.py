"""Create (or open) a deterministic wallet and demonstrate persistence.

Reruns reuse the wallet file written on the first run, so `wallet.exists()`
flips to True and `wallet_open` replaces `wallet_create`. The fixed mnemonic
means the BIP32 account derives the same addresses every time.
"""

import argparse
import asyncio

from kaspa import PrvKeyDataVariantKind, Resolver, Wallet
from kaspa.exceptions import WalletAlreadyExistsError

from shared import FIXED_MNEMONIC_PHRASE, NETWORK_ID, WALLET_SECRET

TITLE = "example wallet creation"
FILENAME = "-".join(TITLE.split(" "))


async def main(rpc_url: str | None):
    # ------------------------------------------------------------------
    # Construct and start wallet
    # ------------------------------------------------------------------
    print()
    print("-" * 100)
    print("\tConstruct and start wallet")
    print("-" * 100)

    if rpc_url:
        wallet = Wallet(network_id=NETWORK_ID, url=rpc_url)
    else:
        wallet = Wallet(network_id=NETWORK_ID, resolver=Resolver())

    await wallet.start()
    print("Wallet started\n")

    print("Newly initialized wallet instance's properties:")
    print(f" - rpc: {wallet.rpc}")
    print(f" - is_open: {wallet.is_open}")
    print(f" - descriptor: {wallet.descriptor}")
    print()

    # ------------------------------------------------------------------
    # Open existing wallet or create new
    # ------------------------------------------------------------------
    print()
    print("-" * 100)
    print("\tOpen existing wallet or create new")
    print("-" * 100)

    # Enumerate wallets on disk
    wallets = await wallet.wallet_enumerate()
    print("Wallets on disk:")
    for w in wallets:
        print(f" - {w}")
    print()

    try:
        # Create wallet
        created = await wallet.wallet_create(
            wallet_secret=WALLET_SECRET,
            filename=FILENAME,
            overwrite_wallet_storage=False,
            title=TITLE,
            user_hint="example",
        )
        print(f"Created new wallet file {FILENAME}: {created}\n")

        # Create mnemonic type prv key data for wallet
        prv_key_id = await wallet.prv_key_data_create(
            wallet_secret=WALLET_SECRET,
            secret=FIXED_MNEMONIC_PHRASE,
            kind=PrvKeyDataVariantKind.Mnemonic,
            payment_secret=None,
            name="demo-key",
        )
        print(f"Created PrvKeyDataId: {prv_key_id}\n")

        # Create first account
        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=WALLET_SECRET,
            prv_key_data_id=prv_key_id,
            payment_secret=None,
            account_name="demo-acct",
            account_index=0,
        )
        print(f"Created BIP32 account: {descriptor}\n")
    except WalletAlreadyExistsError:
        # Open existing wallet
        print(f"Wallet with filename {FILENAME} already exists\n")
        opened = await wallet.wallet_open(WALLET_SECRET, True, FILENAME)
        print(f"Opened existing wallet {FILENAME}: {opened}\n")

    print("Opened wallet instance's properties:")
    print(f" - rpc: {wallet.rpc}")
    print(f" - is_open: {wallet.is_open}")
    print(f" - descriptor: {wallet.descriptor}")
    print()

    # ------------------------------------------------------------------
    # Close and reopen (to show persistence)
    # ------------------------------------------------------------------
    print()
    print("-" * 100)
    print("\tClose and reopen (to show persistence)")
    print("-" * 100)

    await wallet.wallet_close()
    print("Closed Wallet instance properties:")
    print(f" - rpc: {wallet.rpc}")
    print(f" - is_open: {wallet.is_open}")
    print(f" - descriptor: {wallet.descriptor}")
    print()

    reopened = await wallet.wallet_open(WALLET_SECRET, True, FILENAME)
    print(f"Reopened wallet: {reopened}\n")

    # ------------------------------------------------------------------
    # Wind down
    # ------------------------------------------------------------------
    print()
    print("-" * 100)
    print("\tWind down")
    print("-" * 100)

    await wallet.wallet_close()
    print("Wallet closed\n")

    await wallet.stop()
    print("Wallet stopped\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--rpc-url", default=None, help="RPC URL (defaults to Resolver)")
    args = parser.parse_args()
    asyncio.run(main(args.rpc_url))
