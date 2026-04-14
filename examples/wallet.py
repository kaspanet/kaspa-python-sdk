import argparse
import asyncio

from kaspa import (
    AccountKind,
    AccountsDiscoveryKind,
    CommitRevealAddressKind,
    Fees,
    Mnemonic,
    NetworkId,
    NewAddressKind,
    PrvKeyDataVariantKind,
    Resolver,
    TransactionKind,
    Wallet,
    WalletEventType,
)

WALLET_SECRET = "walletSecret"
NEW_WALLET_SECRET = "walletSecretNew"
FILENAME = "wallet_demo"
NETWORK_ID = "testnet-10"


def _on_event(event):
    print(f"  [event callback]: {event.get('type', '<unknown>')}")


async def main(rpc_url: str | None):
    # ------------------------------------------------------------------
    # 1. Construct the Wallet
    # ------------------------------------------------------------------
    if rpc_url:
        wallet = Wallet(network_id=NETWORK_ID, url=rpc_url)
    else:
        wallet = Wallet(network_id=NETWORK_ID, resolver=Resolver())

    print("--- Wallet.rpc")
    print(wallet.rpc)
    print()

    print("--- Wallet.is_open")
    print(wallet.is_open)
    print()

    print("--- Wallet.is_synced")
    print(wallet.is_synced)
    print()

    print("--- Wallet.descriptor")
    print(wallet.descriptor)
    print()

    print("--- Wallet.exists(FILENAME)")
    print(await wallet.exists(FILENAME))
    print()

    # ------------------------------------------------------------------
    # 2. Start wallet runtime & add event listener
    # ------------------------------------------------------------------
    print("--- Wallet.start()")
    print(await wallet.start())
    print()

    print("--- Wallet.connect()")
    print(await wallet.connect(url=rpc_url, strategy="fallback", timeout_duration=5000))
    print()

    print("--- add_event_listener('all')")
    print(wallet.add_event_listener(WalletEventType.All, _on_event))
    print()

    print("--- set_network_id(NETWORK_ID)")
    print(wallet.set_network_id(NetworkId(NETWORK_ID)))
    print()

    # ------------------------------------------------------------------
    # 3. wallet_* fns: enumerate -> create -> open.
    # ------------------------------------------------------------------
    print("--- wallet_enumerate()")
    print(await wallet.wallet_enumerate())
    print()

    print("--- wallet_create()")
    print(await wallet.wallet_create(
        wallet_secret=WALLET_SECRET,
        filename=FILENAME,
        overwrite_wallet_storage=True,
        title="full walkthrough",
        user_hint="example",
    ))
    print()

    print("--- Wallet.is_open [after create]")
    print(wallet.is_open)
    print()

    print("--- Wallet.descriptor [after create]")
    print(wallet.descriptor)
    print()

    # ------------------------------------------------------------------
    # 4. prv_key_data_* fns: create an entry, then list/fetch by id.
    # ------------------------------------------------------------------
    print("--- prv_key_data_create(Mnemonic)")
    mnemonic_prv_key_id = await wallet.prv_key_data_create(
        wallet_secret=WALLET_SECRET,
        secret=Mnemonic.random().phrase,
        kind=PrvKeyDataVariantKind.Mnemonic,
        payment_secret=None,
        name="walkthrough-mnemonic-key",
    )
    print(mnemonic_prv_key_id)
    print()

    # accounts_create_keypair / accounts_import_keypair require a SecretKey
    # variant prv_key_data; mnemonic-backed entries fail upstream with
    # "Sectet key is required".
    print("--- prv_key_data_create(SecretKey)")
    secret_key_prv_key_id = await wallet.prv_key_data_create(
        wallet_secret=WALLET_SECRET,
        secret="a" * 32,
        kind=PrvKeyDataVariantKind.SecretKey,
        payment_secret=None,
        name="walkthrough-secret-key",
    )
    print(secret_key_prv_key_id)
    print()

    print("--- prv_key_data_enumerate()")
    prv_key_infos = await wallet.prv_key_data_enumerate()
    print(prv_key_infos)
    print()

    print("--- prv_key_data_get(mnemonic id)")
    print(await wallet.prv_key_data_get(
        wallet_secret=WALLET_SECRET, prv_key_data_id=mnemonic_prv_key_id
    ))
    print()

    print("--- prv_key_data_get(secret-key id)")
    print(await wallet.prv_key_data_get(
        wallet_secret=WALLET_SECRET, prv_key_data_id=secret_key_prv_key_id
    ))
    print()

    # ------------------------------------------------------------------
    # 5. accounts_* fns: create -> enumerate -> get -> rename -> activate.
    # ------------------------------------------------------------------
    print("--- accounts_create_bip32()")
    bip32_descriptor = await wallet.accounts_create_bip32(
        wallet_secret=WALLET_SECRET,
        prv_key_data_id=mnemonic_prv_key_id,
        payment_secret=None,
        account_name="bip32-acct",
        account_index=None,
    )
    print(bip32_descriptor)
    print()

    account_id = bip32_descriptor.account_id

    print("--- accounts_create_keypair()")
    print(await wallet.accounts_create_keypair(
        wallet_secret=WALLET_SECRET,
        prv_key_data_id=secret_key_prv_key_id,
        ecdsa=False,
        account_name="kp-acct",
    ))
    print()

    print("--- accounts_import_bip32()")
    print(await wallet.accounts_import_bip32(
        wallet_secret=WALLET_SECRET,
        prv_key_data_id=mnemonic_prv_key_id,
        payment_secret=None,
        account_name="imported-bip32",
        account_index=None,
    ))
    print()

    print("--- accounts_import_keypair()")
    try:
        print(await wallet.accounts_import_keypair(
            wallet_secret=WALLET_SECRET,
            prv_key_data_id=secret_key_prv_key_id,
            ecdsa=False,
            account_name="imported-kp",
        ))
    except Exception as exc:
        print(f"{type(exc).__name__}: {exc}")
    print()

    print("--- accounts_ensure_default(Bip32)")
    print(await wallet.accounts_ensure_default(
        wallet_secret=WALLET_SECRET,
        account_kind=AccountKind("bip32"),
        payment_secret=None,
        mnemonic_phrase=Mnemonic.random().phrase,
    ))
    print()

    print("--- accounts_enumerate()")
    print(await wallet.accounts_enumerate())
    print()

    print("--- accounts_get(account_id)")
    print(await wallet.accounts_get(account_id))
    print()

    print("--- accounts_rename()")
    print(await wallet.accounts_rename(
        wallet_secret=WALLET_SECRET,
        account_id=account_id,
        name="renamed-bip32",
    ))
    print()

    print("--- accounts_activate([account_id])")
    print(await wallet.accounts_activate([account_id]))
    print()

    # ------------------------------------------------------------------
    # 6. Address derivation.
    # ------------------------------------------------------------------
    print("--- accounts_create_new_address(Receive)")
    receive_address = await wallet.accounts_create_new_address(
        account_id, NewAddressKind.Receive
    )
    print(receive_address)
    print()

    print("--- accounts_create_new_address(Change)")
    print(await wallet.accounts_create_new_address(account_id, NewAddressKind.Change))
    print()

    # ------------------------------------------------------------------
    # 6b. Wait for testnet-10 funds at the receive address before
    #     proceeding to UTXO/spend paths.
    # ------------------------------------------------------------------
    print(f"\n>>> Send testnet-10 funds to: {receive_address}")
    print(">>> Faucet: https://faucet.kaspanet.io/")
    print(">>> Waiting for funds...")
    while True:
        utxos = await wallet.accounts_get_utxos(account_id=account_id)
        if utxos:
            print(f">>> Funds detected: {len(utxos)} UTXO(s)")
            print()
            break
        await asyncio.sleep(5)

    # ------------------------------------------------------------------
    # 7. accounts_discovery (offline-friendly: scans a fresh mnemonic).
    # ------------------------------------------------------------------
    print("--- accounts_discovery(Bip44)")
    print(await wallet.accounts_discovery(
        discovery_kind=AccountsDiscoveryKind.Bip44,
        address_scan_extent=10,
        account_scan_extent=1,
        bip39_mnemonic=Mnemonic.random().phrase,
        bip39_passphrase=None,
    ))
    print()

    # ------------------------------------------------------------------
    # 8. UTXO + spend paths. These need an RPC connection and (for sends)
    #    funded UTXOs; they will surface their own errors when offline.
    # ------------------------------------------------------------------
    print("--- accounts_get_utxos(account_id)")
    try:
        print(await wallet.accounts_get_utxos(account_id=account_id))
    except Exception as exc:
        print(f"{type(exc).__name__}: {exc}")
    print()

    print("--- accounts_estimate(change-only, fee=0)")
    try:
        print(await wallet.accounts_estimate(
            account_id=account_id,
            priority_fee_sompi=Fees(0, None),
            fee_rate=None,
            payload=None,
            destination=None,
        ))
    except Exception as exc:
        print(f"{type(exc).__name__}: {exc}")
    print()

    print("--- accounts_send(change-only)")
    try:
        print(await wallet.accounts_send(
            wallet_secret=WALLET_SECRET,
            account_id=account_id,
            priority_fee_sompi=Fees(0, None),
            payment_secret=None,
            fee_rate=None,
            payload=None,
            destination=None,
        ))
    except Exception as exc:
        print(f"{type(exc).__name__}: {exc}")
    print()

    print("--- accounts_transfer(self -> self)")
    try:
        print(await wallet.accounts_transfer(
            wallet_secret=WALLET_SECRET,
            source_account_id=account_id,
            destination_account_id=account_id,
            transfer_amount_sompi=1,
            payment_secret=None,
            fee_rate=None,
            priority_fee_sompi=None,
        ))
    except Exception as exc:
        print(f"{type(exc).__name__}: {exc}")
    print()

    print("--- accounts_commit_reveal()")
    try:
        print(await wallet.accounts_commit_reveal(
            wallet_secret=WALLET_SECRET,
            account_id=account_id,
            address_type=CommitRevealAddressKind.Receive,
            address_index=0,
            script_sig=b"\x00",
            commit_amount_sompi=1,
            reveal_fee_sompi=1,
            payment_secret=None,
            fee_rate=None,
            payload=None,
        ))
    except Exception as exc:
        print(f"{type(exc).__name__}: {exc}")
    print()

    print("--- accounts_commit_reveal_manual()")
    try:
        print(await wallet.accounts_commit_reveal_manual(
            wallet_secret=WALLET_SECRET,
            account_id=account_id,
            script_sig=b"\x00",
            reveal_fee_sompi=1,
            payment_secret=None,
            fee_rate=None,
            payload=None,
            start_destination=None,
            end_destination=None,
        ))
    except Exception as exc:
        print(f"{type(exc).__name__}: {exc}")
    print()

    # ------------------------------------------------------------------
    # 9. transactions_*: read history, then attempt note/metadata edits.
    #    The replace_* calls expect a real transaction id; we use a
    #    placeholder so they exercise the code path.
    # ------------------------------------------------------------------
    print("--- transactions_data_get()")
    try:
        print(await wallet.transactions_data_get(
            account_id=account_id,
            network_id=NetworkId(NETWORK_ID),
            start=0,
            end=10,
            filter=[TransactionKind.Incoming],
        ))
    except Exception as exc:
        print(f"{type(exc).__name__}: {exc}")
    print()

    placeholder_tx_id = "0" * 64

    print("--- transactions_replace_note()")
    try:
        print(await wallet.transactions_replace_note(
            account_id=account_id,
            network_id=NetworkId(NETWORK_ID),
            transaction_id=placeholder_tx_id,
            note="walkthrough note",
        ))
    except Exception as exc:
        print(f"{type(exc).__name__}: {exc}")
    print()

    print("--- transactions_replace_metadata()")
    try:
        print(await wallet.transactions_replace_metadata(
            account_id=account_id,
            network_id=NetworkId(NETWORK_ID),
            transaction_id=placeholder_tx_id,
            metadata="walkthrough metadata",
        ))
    except Exception as exc:
        print(f"{type(exc).__name__}: {exc}")
    print()

    # ------------------------------------------------------------------
    # 10. batch / flush / retain_context / get_status / address book.
    # ------------------------------------------------------------------
    print("--- batch()")
    print(await wallet.batch())
    print()

    print("--- flush()")
    print(await wallet.flush(WALLET_SECRET))
    print()

    print("--- retain_context()")
    print(await wallet.retain_context("walkthrough-context", b"payload-bytes"))
    print()

    print("--- get_status()")
    print(await wallet.get_status(None))
    print()

    print("--- address_book_enumerate()")
    print(await wallet.address_book_enumerate())
    print()

    # ------------------------------------------------------------------
    # 11. fee_rate_*: needs an RPC connection.
    # ------------------------------------------------------------------
    print("--- fee_rate_estimate()")
    try:
        print(await wallet.fee_rate_estimate())
    except Exception as exc:
        print(f"{type(exc).__name__}: {exc}")
    print()

    print("--- fee_rate_poller_enable(5)")
    try:
        print(await wallet.fee_rate_poller_enable(5))
    except Exception as exc:
        print(f"{type(exc).__name__}: {exc}")
    print()

    print("--- fee_rate_poller_disable()")
    try:
        print(await wallet.fee_rate_poller_disable())
    except Exception as exc:
        print(f"{type(exc).__name__}: {exc}")
    print()

    # ------------------------------------------------------------------
    # 12. Wind down accounts.
    # ------------------------------------------------------------------
    # accounts_deactivate hangs due to an upstream rk bug
    # skipped for now
    # print("--- accounts_deactivate([account_id])")
    # print(await wallet.accounts_deactivate([account_id]))
    # print()

    # ------------------------------------------------------------------
    # 13. Export, close, import, reopen, change secret, rename, reload.
    # ------------------------------------------------------------------
    print("--- wallet_export()")
    exported = await wallet.wallet_export(WALLET_SECRET, True)
    print(exported)
    print()

    print("--- wallet_close()")
    print(await wallet.wallet_close())
    print()

    print("--- wallet_import()")
    print(await wallet.wallet_import(WALLET_SECRET, exported))
    print()

    print("--- wallet_open()")
    print(await wallet.wallet_open(WALLET_SECRET, True, FILENAME))
    print()

    print("--- wallet_reload(True)")
    print(await wallet.wallet_reload(True))
    print()

    print("--- wallet_change_secret()")
    print(await wallet.wallet_change_secret(WALLET_SECRET, NEW_WALLET_SECRET))
    print()

    print("--- wallet_rename(title only)")
    print(await wallet.wallet_rename(NEW_WALLET_SECRET, "walkthrough renamed", None))
    print()

    print("--- wallet_close() [final]")
    print(await wallet.wallet_close())
    print()

    # ------------------------------------------------------------------
    # 14. Wind down listeners and runtime.
    # ------------------------------------------------------------------
    print("--- remove_event_listener('all')")
    print(wallet.remove_event_listener(WalletEventType.All))
    print()

    print("--- Wallet.stop()")
    print(await wallet.stop())
    print()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--rpc-url", default=None, help="RPC URL (defaults to Resolver)")
    args = parser.parse_args()
    asyncio.run(main(args.rpc_url))
