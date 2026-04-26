"""Example showing both BIP32 and keypair account types in one wallet.

A wallet can hold N accounts any type, each backed by an
entry in the wallet's `prv_key_data` store.

This example shows creation of both account types in the same file:

- BIP32 (HD-derived): One mnemonic backs unlimited accounts,
each at a different `account_index` on Kaspa's BIP44 path
m/44'/111111'/{account_index}'

- Keypair (non-HD): One secp256k1 private key wrapped
  as an account. One account, one address, no derivation path.

Both types coexist ins ame wallet in this example
"""

import argparse
import asyncio

from kaspa import (
    NewAddressKind,
    PrivateKey,
    PrvKeyDataVariantKind,
    Resolver,
    Wallet,
)
from kaspa.exceptions import WalletAlreadyExistsError

from shared import (
    FIXED_MNEMONIC_PHRASE,
    NETWORK_ID,
    WALLET_SECRET,
)

TITLE = "example wallet accounts"
FILENAME = "-".join(TITLE.split(" "))

PRIVATE_KEY = PrivateKey("389840d7696e89c38856a066175e8e92697f0cf182b854c883237a50acaf1f69")


def bip32_account_path(account_index: int) -> str:
    return f"m/44'/111111'/{account_index}'"


def bip32_address_path(account_index: int, address_type: str, address_index: int) -> str:
    if address_type == "receive":
        chain = 0
    elif address_type == "change":
        chain = 1
    else:
        raise ValueError(f"address_type must be 'receive' or 'change', got {address_type!r}")
    return f"{bip32_account_path(account_index)}/{chain}/{address_index}"


async def main(rpc_url: str | None):
    # ------------------------------------------------------------------
    # Construct and start
    # ------------------------------------------------------------------
    print()
    print("-" * 100)
    print("\tInitialize & Create/Open Wallet")
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
    try:
        # Create wallet
        created = await wallet.wallet_create(
            wallet_secret=WALLET_SECRET,
            filename=FILENAME,
            overwrite_wallet_storage=False,
            title=TITLE,
            user_hint="example"
        )
        print(f"Created new wallet file {FILENAME}: {created}\n")
    except WalletAlreadyExistsError:
        # Open existing wallet
        print(f"Wallet with filename {FILENAME} already exists\n")
        opened = await wallet.wallet_open(WALLET_SECRET, True, FILENAME)
        print(f"Opened existing wallet {FILENAME}: {opened}\n")

    # Enumerate and print all accounts
    all_accounts = await wallet.accounts_enumerate()
    print("Wallet's existing accounts:")
    for account in all_accounts:
        print(f" - {account}")
    print()

    # ------------------------------------------------------------------
    # BIP32 accounts (HD-derived from the wallet mnemonic)
    # ------------------------------------------------------------------
    print()
    print("-" * 100)
    print("\tBIP32 accounts (HD-derived from the wallet mnemonic)")
    print("-" * 100)

    # Try to find existing demo account at index 0 first. If it exists, we can
    # pull the mnemonic prv key id from the account descriptor
    account_0 = next((a for a in all_accounts if a.account_name == "demo-acct-0"), None)
    if account_0 is None:
        # New wallet - need to create the mnemonic prv key, then the account
        mnemonic_prv_key_id = await wallet.prv_key_data_create(
            wallet_secret=WALLET_SECRET,
            secret=FIXED_MNEMONIC_PHRASE,
            kind=PrvKeyDataVariantKind.Mnemonic,
            payment_secret=None,
            name="demo-mnemonic-key",
        )
        account_0 = await wallet.accounts_create_bip32(
            wallet_secret=WALLET_SECRET,
            prv_key_data_id=mnemonic_prv_key_id,
            payment_secret=None,
            account_name="demo-acct-0",
            account_index=0,
        )
        print(f"Created BIP32 account (index=0): {account_0}\n")
    else:
        mnemonic_prv_key_id = account_0.prv_key_data_ids[0]
        print(f"Using existing BIP32 account (index=0): {account_0}\n")

    print(f"Account {account_0.account_index}")
    print(f" - derivation path: {bip32_account_path(account_0.account_index)}")
    print(f" - xpub_keys: {account_0.xpub_keys}")
    print(f" - existing address counts: receive={account_0.receive_address_index}, change={account_0.change_address_index}\n")

    # Derive new receive addresses for the BIP32 account_0
    print(f"Account {account_0.account_index} - deriving RECEIVE addresses:")
    recv_start = account_0.receive_address_index
    for i in range(5):
        addr_idx = recv_start + i
        rec_addr = await wallet.accounts_create_new_address(account_0.account_id, NewAddressKind.Receive)
        print(f" - path {bip32_address_path(account_0.account_index, 'receive', addr_idx)} - {rec_addr}")
    print()

    # Derive new change addresses for the BIP32 account_0
    print(f"Account {account_0.account_index} - deriving CHANGE addresses:")
    change_start = account_0.change_address_index
    for i in range(5):
        addr_idx = change_start + i
        change_addr = await wallet.accounts_create_new_address(account_0.account_id, NewAddressKind.Change)
        print(f" - path {bip32_address_path(account_0.account_index, 'change', addr_idx)} - {change_addr}")
    print()

    # Account at index 1 — same mnemonic, just a different account_index.
    account_1 = next((a for a in all_accounts if a.account_name == "demo-acct-1"), None)
    if account_1 is None:
        account_1 = await wallet.accounts_create_bip32(
            wallet_secret=WALLET_SECRET,
            prv_key_data_id=mnemonic_prv_key_id,
            payment_secret=None,
            account_name="demo-acct-1",
            account_index=1,
        )
        print(f"Created BIP32 account (index=1): {account_1}\n")
    else:
        print(f"Using existing BIP32 account (index=1): {account_1}\n")

    print(f"Account {account_1.account_index}")
    print(f" - derivation path: {bip32_account_path(account_1.account_index)}")
    print(f" - xpub_keys: {account_1.xpub_keys}")
    print(f" - existing address counts: receive={account_1.receive_address_index}, change={account_1.change_address_index}\n")

    # Derive new receive addresses for the BIP32 account_1
    print(f"Account {account_1.account_index} - deriving RECEIVE addresses:")
    recv_start = account_1.receive_address_index
    for i in range(5):
        addr_idx = recv_start + i
        rec_addr = await wallet.accounts_create_new_address(account_1.account_id, NewAddressKind.Receive)
        print(f" - path {bip32_address_path(account_1.account_index, 'receive', addr_idx)} - {rec_addr}")
    print()

    # Derive new change addresses for the BIP32 account_1
    print(f"Account {account_1.account_index} - deriving CHANGE addresses:")
    change_start = account_1.change_address_index
    for i in range(5):
        addr_idx = change_start + i
        change_addr = await wallet.accounts_create_new_address(account_1.account_id, NewAddressKind.Change)
        print(f" - path {bip32_address_path(account_1.account_index, 'change', addr_idx)} - {change_addr}")
    print()

    # ------------------------------------------------------------------
    # Keypair account (single key/account/address, non-HD)
    # ------------------------------------------------------------------
    print()
    print("-" * 100)
    print("\tKeypair account (single key/account/address, non-HD)")
    print("-" * 100)

    keypair_account = next((a for a in all_accounts if a.account_name == "keypair-acct"), None)
    if keypair_account is None:
        secret_key_prv_key_id = await wallet.prv_key_data_create(
            wallet_secret=WALLET_SECRET,
            secret=PRIVATE_KEY.to_string(),
            kind=PrvKeyDataVariantKind.SecretKey,
            payment_secret=None,
            name="demo-secret-key",
        )
        keypair_account = await wallet.accounts_create_keypair(
            wallet_secret=WALLET_SECRET,
            prv_key_data_id=secret_key_prv_key_id,
            ecdsa=False,
            account_name="keypair-acct",
        )
        print(f"Created keypair account: {keypair_account}\n")
    else:
        print(f"Using existing keypair account: {keypair_account}\n")

    # Keypair accounts are not HD-derived: account_index/xpub_keys/derivation
    # counts are all None. ecdsa is True/False depending on creation choice.
    print(f" - account_index: {keypair_account.account_index}")
    print(f" - xpub_keys: {keypair_account.xpub_keys}")
    print(f" - ecdsa: {keypair_account.ecdsa}")
    print(f" - receive_address_index: {keypair_account.receive_address_index}")
    print(f" - change_address_index: {keypair_account.change_address_index}\n")

    # ------------------------------------------------------------------
    # Both types live side-by-side in the same wallet file
    # ------------------------------------------------------------------
    print()
    print("-" * 100)
    print("\tBoth BIP32 and Keypair types live side-by-side in the same wallet file")
    print("-" * 100)

    print("Private key data entries:")
    for info in await wallet.prv_key_data_enumerate():
        print(f" - {info}")
    print()

    print("Final account list (BIP32 and keypair side-by-side):")
    for account in await wallet.accounts_enumerate():
        print(f" - {account}")
        print(f"     account_index={account.account_index} ecdsa={account.ecdsa} "
              f"derivation=(receive={account.receive_address_index}, change={account.change_address_index})")
    print()

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
