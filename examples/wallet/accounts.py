"""Manage BIP32 and keypair accounts in one wallet file.

A wallet file holds N accounts of any mix of variants, each backed by an
entry in the wallet's `prv_key_data` store. This example exercises both
account types in the same file:

* **BIP32 (HD-derived)** — the default. One mnemonic backs unlimited
  accounts, each at a different `account_index` on Kaspa's BIP44 path
  `m/44'/111111'/{account_index}'`. Receive addresses live under
  `.../0/i`, change addresses under `.../1/i`. Each account derives its
  addresses on demand by walking the HD tree. The address at a given
  index is deterministic (same mnemonic → same bytes), but
  `accounts_create_new_address` advances the account's stored index on
  every call — so the printed addresses shift forward each run.

* **Keypair (non-HD)** — one pre-existing secp256k1 private key wrapped
  as an account. One account, one address, no derivation path. Use when
  importing a raw key from an external source (paper wallet, another
  tool, etc.) into the wallet file.

Both types coexist: the final `accounts_enumerate` prints them
side-by-side in the same on-disk wallet.
"""

import argparse
import asyncio

from kaspa import (
    AccountKind,
    AccountsDiscoveryKind,
    NewAddressKind,
    PrvKeyDataVariantKind,
    Resolver,
    Wallet,
)
from kaspa.exceptions import WalletAccountAlreadyExistsError

from shared import (
    FIXED_MNEMONIC_PHRASE,
    NETWORK_ID,
    WALLET_SECRET,
    open_or_create_wallet,
)

FILENAME = "wallet_accounts_demo"
# 32 UTF-8 bytes used directly as a secp256k1 scalar (see wallet-core's
# SecretKey variant handling in prv_key_data_create). In real use this
# would be a key you generated or imported, not a literal placeholder.
FIXED_SECRET_KEY_BYTES = "a" * 32

# Kaspa's SLIP-44 coin type. Combined with BIP44's purpose (44'), every
# HD account lives at m/44'/111111'/{account_index}'.
KASPA_COIN_TYPE = 111111


def bip32_account_path(account_index: int) -> str:
    return f"m/44'/{KASPA_COIN_TYPE}'/{account_index}'"


def bip32_address_path(account_index: int, change: bool, address_index: int) -> str:
    chain = 1 if change else 0
    return f"{bip32_account_path(account_index)}/{chain}/{address_index}"


async def main(rpc_url: str | None):
    # ------------------------------------------------------------------
    # Construct and start
    # ------------------------------------------------------------------
    if rpc_url:
        wallet = Wallet(network_id=NETWORK_ID, url=rpc_url)
    else:
        wallet = Wallet(network_id=NETWORK_ID, resolver=Resolver())

    await wallet.start()
    print("Wallet started\n")

    account_id = await open_or_create_wallet(
        wallet, FILENAME, title="accounts and addresses demo"
    )
    print(f"account_id = {account_id}\n")

    # ==================================================================
    # BIP32 accounts (HD-derived from the wallet mnemonic)
    # ==================================================================
    print("Wallet accounts:")
    for account in await wallet.accounts_enumerate():
        print(f" - {account}")
    print()

    account = await wallet.accounts_get(account_id)
    print(f"Account details: {account}\n")
    # Default account sits at BIP44 index 0 on Kaspa's coin type.
    print(f"Derivation path (account 0): {bip32_account_path(0)}\n")

    await wallet.accounts_activate([account_id])
    print(f"Activated account {account_id}\n")

    # Derive new receive/change addresses from the default BIP32 account.
    # Receive addresses live under `.../0/i`, change under `.../1/i`; `i`
    # is the per-chain index the account advances on each call.
    new_receive = await wallet.accounts_create_new_address(account_id, NewAddressKind.Receive)
    print(f"New receive address: {new_receive} (under {bip32_account_path(0)}/0/i)\n")

    new_change = await wallet.accounts_create_new_address(account_id, NewAddressKind.Change)
    print(f"New change address: {new_change} (under {bip32_account_path(0)}/1/i)\n")

    # Create a *second* BIP32 account at account_index=1, reusing the
    # same prv_key_data_id as account 0 (same mnemonic). Different HD
    # subtree → completely different addresses from account 0.
    mnemonic_prv_id = next(
        info.id for info in await wallet.prv_key_data_enumerate() if info.name == "demo-key"
    )
    try:
        second_bip32 = await wallet.accounts_create_bip32(
            wallet_secret=WALLET_SECRET,
            prv_key_data_id=mnemonic_prv_id,
            payment_secret=None,
            account_name="demo-acct-1",
            account_index=1,
        )
        print(f"Created second BIP32 account (index=1): {second_bip32}\n")
        acct1_id = second_bip32.account_id
    except WalletAccountAlreadyExistsError:
        print("Second BIP32 account (index=1) already exists\n")
        # Find it by elimination — the BIP32 account that isn't our
        # default. Robust to renames (unlike a name-based lookup).
        bip32_kind = AccountKind("bip32")
        acct1_id = next(
            a.account_id
            for a in await wallet.accounts_enumerate()
            if a.kind == bip32_kind and a.account_id != account_id
        )
    print(f"Derivation path (account 1): {bip32_account_path(1)}\n")

    # Derive a receive address under account 1 to show the tree diverges
    # from account 0 despite sharing the same mnemonic. Same caveat as
    # above: the receive-chain index advances on every run.
    acct1_receive = await wallet.accounts_create_new_address(acct1_id, NewAddressKind.Receive)
    print(f"Account 1 receive address: {acct1_receive} (under {bip32_account_path(1)}/0/i)\n")

    # BIP44 discovery: given a mnemonic, derive accounts at
    # m/44'/111111'/{i}' for i in [0, account_scan_extent) and ask the
    # node whether any derived addresses have on-chain history. Returns
    # the highest account index that had activity. RPC is required for
    # the activity check, so we connect just for this call.
    await wallet.connect(url=rpc_url, strategy="fallback", timeout_duration=5000)
    print("RPC connected (for BIP44 discovery)\n")
    discovered = await wallet.accounts_discovery(
        discovery_kind=AccountsDiscoveryKind.Bip44,
        address_scan_extent=10,
        account_scan_extent=1,
        bip39_mnemonic=FIXED_MNEMONIC_PHRASE,
        bip39_passphrase=None,
    )
    print(f"BIP44 discovery result (last index with activity): {discovered}\n")
    await wallet.disconnect()
    print("RPC disconnected\n")

    # ==================================================================
    # Section 2 — Keypair account (single pre-existing key, non-HD)
    #
    # One secp256k1 private key → one account → one address. No HD tree,
    # no account_index. Use when importing a key from outside the wallet.
    # ==================================================================
    existing = next(
        (info for info in await wallet.prv_key_data_enumerate() if info.name == "demo-secret-key"),
        None,
    )
    if existing is None:
        secret_key_prv_id = await wallet.prv_key_data_create(
            wallet_secret=WALLET_SECRET,
            secret=FIXED_SECRET_KEY_BYTES,
            kind=PrvKeyDataVariantKind.SecretKey,
            payment_secret=None,
            name="demo-secret-key",
        )
    else:
        secret_key_prv_id = existing.id

    try:
        keypair_account = await wallet.accounts_create_keypair(
            wallet_secret=WALLET_SECRET,
            prv_key_data_id=secret_key_prv_id,
            ecdsa=False,
            account_name="kp-acct",
        )
        print(f"Created keypair account: {keypair_account}\n")
    except WalletAccountAlreadyExistsError:
        print("Keypair account already exists\n")

    # ==================================================================
    # Section 3 — Both types live side-by-side in the same wallet file
    # ==================================================================
    print("Private key data entries:")
    for info in await wallet.prv_key_data_enumerate():
        print(f" - {info}")
    print()

    print("Final account list (BIP32 and keypair side-by-side):")
    for account in await wallet.accounts_enumerate():
        print(f" - {account}")
    print()

    # ------------------------------------------------------------------
    # Wind down
    # ------------------------------------------------------------------
    await wallet.wallet_close()
    print("Wallet closed\n")

    await wallet.stop()
    print("Wallet stopped\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--rpc-url", default=None, help="RPC URL (defaults to Resolver)")
    args = parser.parse_args()
    asyncio.run(main(args.rpc_url))
