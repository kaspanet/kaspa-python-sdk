"""Example showing wallet export, import, and reload."""

import argparse
import asyncio
from pathlib import Path

from kaspa import PrvKeyDataVariantKind, Resolver, Wallet
from kaspa.exceptions import WalletAlreadyExistsError

from shared import (
    FIXED_MNEMONIC_PHRASE,
    NETWORK_ID,
    WALLET_SECRET,
)

TITLE = "example wallet export import"
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

    # ------------------------------------------------------------------
    # Open existing wallet or create new
    # ------------------------------------------------------------------
    print()
    print("-" * 100)
    print("\tOpen existing wallet or create new")
    print("-" * 100)

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

        # Create first prv key data for wallet
        prv_key_id = await wallet.prv_key_data_create(
            wallet_secret=WALLET_SECRET,
            secret=FIXED_MNEMONIC_PHRASE,
            kind=PrvKeyDataVariantKind.Mnemonic,
            payment_secret=None,
            name="demo-key",
        )

        # Create first account
        account_descriptor = await wallet.accounts_create_bip32(
            wallet_secret=WALLET_SECRET,
            prv_key_data_id=prv_key_id,
            payment_secret=None,
            account_name="demo-acct",
            account_index=0,
        )
        print(f"Created BIP32 account: {account_descriptor}\n")
    except WalletAlreadyExistsError:
        # Open existing wallet
        print(f"Wallet with filename {FILENAME} already exists\n")
        opened = await wallet.wallet_open(WALLET_SECRET, True, FILENAME)
        print(f"Opened existing wallet {FILENAME}: {opened}\n")

    # ------------------------------------------------------------------
    # Export -> close -> delete original -> import as new -> open
    # ------------------------------------------------------------------
    print()
    print("-" * 100)
    print("\tExport, close, delete, import, reopen")
    print("-" * 100)

    exported = await wallet.wallet_export(WALLET_SECRET, True)
    print(f"Wallet exported ({len(exported)} hex chars): {exported}\n")

    await wallet.wallet_close()
    print("Wallet closed\n")

    # Delete on-disk wallet files so the import process
    # can create from the prior exported hex
    wallet_dir = Path.home() / ".kaspa"
    wallet_path = wallet_dir / f"{FILENAME}.wallet"
    wallet_path.unlink()
    print("Wallet deleted (to simulate fresh import)\n")

    # No transactions under this wallet, so, nothing to delete
    # transactions_path = wallet_dir / f"{FILENAME}.transactions"
    # transactions_path.rmdir()
    # print(f"Removed transactions dir at {wallet_path}")

    # `wallet_import` creates new wallet file
    imported = await wallet.wallet_import(WALLET_SECRET, exported)
    print(f"Wallet imported (from the prior exported hex): {imported}\n")
    imported_filename = imported["walletDescriptor"]["filename"]

    reopened = await wallet.wallet_open(WALLET_SECRET, True, imported_filename)
    print(f"Reopened imported wallet ({imported_filename}): {reopened}\n")

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
