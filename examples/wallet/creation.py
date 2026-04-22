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

FILENAME = "wallet-creation-demo"
TITLE = "wallet creation demo"


async def main(rpc_url: str | None):
    # ------------------------------------------------------------------
    # Construct wallet
    # ------------------------------------------------------------------
    if rpc_url:
        wallet = Wallet(network_id=NETWORK_ID, url=rpc_url)
    else:
        wallet = Wallet(network_id=NETWORK_ID, resolver=Resolver())

    print("Initialized Wallet instance properties:")
    print(f" - rpc: {wallet.rpc}")
    print(f" - is_open: {wallet.is_open}")
    print(f" - descriptor: {wallet.descriptor}")

    # ------------------------------------------------------------------
    # Start wallet runtime
    # ------------------------------------------------------------------
    await wallet.start()
    print("Wallet started")

    wallets = await wallet.wallet_enumerate()
    print("Wallets on disk:")
    for w in wallets:
        print(f" - {w}")

    # ------------------------------------------------------------------
    # Create wallet (catch and fall back to open if it already exists)
    # ------------------------------------------------------------------
    try:
        created = await wallet.wallet_create(
            wallet_secret=WALLET_SECRET,
            filename=FILENAME,
            overwrite_wallet_storage=False,
            title=TITLE,
            user_hint="example",
        )
        print(f"Created new wallet file {FILENAME!r}: {created}")

        prv_key_id = await wallet.prv_key_data_create(
            wallet_secret=WALLET_SECRET,
            secret=FIXED_MNEMONIC_PHRASE,
            kind=PrvKeyDataVariantKind.Mnemonic,
            payment_secret=None,
            name="demo-key",
        )
        print(f"Created PrvKeyDataId: {prv_key_id}")

        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=WALLET_SECRET,
            prv_key_data_id=prv_key_id,
            payment_secret=None,
            account_name="demo-acct",
            account_index=0,
        )
        print(f"Created BIP32 account: {descriptor}")
    except WalletAlreadyExistsError:
        print(f"Wallet with filename `{FILENAME}` already exists")
        opened = await wallet.wallet_open(WALLET_SECRET, True, FILENAME)
        print(f"Opened existing wallet {FILENAME}: {opened}")

    print("Opened Wallet instance properties:")
    print(f" - is_open: {wallet.is_open}")
    print(f" - descriptor: {wallet.descriptor}")

    # ------------------------------------------------------------------
    # Close and reopen (just to show persistence)
    # ------------------------------------------------------------------
    await wallet.wallet_close()
    print("Closed Wallet instance properties:")
    print(f" - is_open: {wallet.is_open}")
    print(f" - descriptor: {wallet.descriptor}")

    reopened = await wallet.wallet_open(WALLET_SECRET, True, FILENAME)
    print(f"Reopened wallet: {reopened}")

    await wallet.wallet_close()
    print("Wallet closed")

    # ------------------------------------------------------------------
    # Stop wallet runtime
    # ------------------------------------------------------------------
    await wallet.stop()
    print("Wallet stopped")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--rpc-url", default=None, help="RPC URL (defaults to Resolver)")
    args = parser.parse_args()
    asyncio.run(main(args.rpc_url))
