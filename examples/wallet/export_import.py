"""Export, import and reload, a wallet.
"""

import argparse
import asyncio
from pathlib import Path

from kaspa import Resolver, Wallet

from shared import NETWORK_ID, WALLET_SECRET, open_or_create_wallet

TITLE = "wallet export import demo"
FILENAME = "-".join(TITLE.split(" "))


async def main(rpc_url: str | None):
    # ------------------------------------------------------------------
    # Construct and start wallet
    # ------------------------------------------------------------------
    if rpc_url:
        wallet = Wallet(network_id=NETWORK_ID, url=rpc_url)
    else:
        wallet = Wallet(network_id=NETWORK_ID, resolver=Resolver())

    await wallet.start()
    print("Wallet started")

    # ------------------------------------------------------------------
    # Open existing wallet file or create new
    # ------------------------------------------------------------------
    await open_or_create_wallet(wallet, FILENAME, title=TITLE)

    # ------------------------------------------------------------------
    # Export -> close -> import -> open
    # ------------------------------------------------------------------
    exported = await wallet.wallet_export(WALLET_SECRET, True)
    print(f"Wallet exported: {exported[:32]}... ({len(exported)} hex chars)")

    await wallet.wallet_close()
    print("Wallet closed")

    # Delete on-disk wallet files so the import process
    # can create from the prior exported hex
    wallet_dir = Path.home() / ".kaspa"
    wallet_path = wallet_dir / f"{FILENAME}.wallet"
    wallet_path.unlink()
    print("Wallet deleted (to simulate import from scratch)")

    # No transactions under this wallet, so, nothing to delete
    # transactions_path = wallet_dir / f"{FILENAME}.transactions"
    # transactions_path.rmdir()
    # print(f"Removed transactions dir at {wallet_path}")

    # `wallet_import` writes a fresh file derived from the payload's
    imported = await wallet.wallet_import(WALLET_SECRET, exported)
    print(f"Wallet imported (from the prior exported hex): {imported}")
    imported_filename = imported["walletDescriptor"]["filename"]

    reopened = await wallet.wallet_open(WALLET_SECRET, True, imported_filename)
    print(f"Reopened imported wallet ({imported_filename}): {reopened}")

    # ------------------------------------------------------------------
    # Wind down.
    # ------------------------------------------------------------------
    await wallet.wallet_close()
    print("Wallet closed")

    await wallet.stop()
    print("Wallet stopped")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--rpc-url", default=None, help="RPC URL (defaults to Resolver)")
    args = parser.parse_args()
    asyncio.run(main(args.rpc_url))
