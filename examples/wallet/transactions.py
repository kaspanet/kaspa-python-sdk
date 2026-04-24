"""Send, transfer, estimate, and inspect transactions for a funded wallet.

Requires testnet-10 funds. On the first run the script pauses at the receive
address and polls `accounts_get_utxos` until funds appear; on later runs the
existing UTXO set is picked up immediately.
"""

import argparse
import asyncio

from kaspa import (
    Fees,
    NetworkId,
    Resolver,
    TransactionKind,
    Wallet,
    WalletEventType,
)

from shared import NETWORK_ID, WALLET_SECRET, open_or_create_wallet

FILENAME = "wallet_transactions_demo"


def _on_balance(event):
    print(f"  [balance event]: {event.get('type', '<unknown>')}")


async def main(rpc_url: str | None):
    # ------------------------------------------------------------------
    # 1. Construct, start, connect (RPC is required for everything below).
    # ------------------------------------------------------------------
    if rpc_url:
        wallet = Wallet(network_id=NETWORK_ID, url=rpc_url)
    else:
        wallet = Wallet(network_id=NETWORK_ID, resolver=Resolver())

    await wallet.start()
    print("Wallet started")

    await wallet.connect(url=rpc_url, strategy="fallback", timeout_duration=5000)
    print("RPC connected")

    wallet.add_event_listener(WalletEventType.Balance, _on_balance)

    # ------------------------------------------------------------------
    # 2. Idempotent bootstrap — see `shared.py`.
    # ------------------------------------------------------------------
    account_id = await open_or_create_wallet(
        wallet, FILENAME, title="transactions demo"
    )
    print(f"account_id = {account_id}")
    print()

    await wallet.accounts_activate([account_id])
    print(f"Activated account {account_id}")
    print()

    # ------------------------------------------------------------------
    # 3. Pick up (or derive) a receive address and wait until it is funded.
    # ------------------------------------------------------------------
    account = await wallet.accounts_get(account_id)
    receive_address = account.receive_address
    print(f"receive_address = {receive_address}")
    print()

    print(">>> Waiting for testnet-10 funds at the receive address above...")
    while True:
        utxos = await wallet.accounts_get_utxos(account_id=account_id)
        if utxos:
            print(f">>> Funds detected: {len(utxos)} UTXO(s)")
            print()
            break
        await asyncio.sleep(5)

    # ------------------------------------------------------------------
    # 4. Fee-rate helpers.
    # ------------------------------------------------------------------
    fee_estimate = await wallet.fee_rate_estimate()
    print(f"Fee rate estimate: {fee_estimate}")
    print()

    await wallet.fee_rate_poller_enable(5)
    print("Fee-rate poller enabled (5s interval)")

    await wallet.fee_rate_poller_disable()
    print("Fee-rate poller disabled")
    print()

    # ------------------------------------------------------------------
    # 5. UTXOs, estimate, send (change-only: no destination), self-transfer.
    # ------------------------------------------------------------------
    utxos_for_account = await wallet.accounts_get_utxos(account_id=account_id)
    print(f"UTXOs for account: {utxos_for_account}")
    print()

    estimate = await wallet.accounts_estimate(
        account_id=account_id,
        priority_fee_sompi=Fees(0, None),
        fee_rate=None,
        payload=None,
        destination=None,
    )
    print(f"Change-only estimate (fee=0): {estimate}")
    print()

    send_result = await wallet.accounts_send(
        wallet_secret=WALLET_SECRET,
        account_id=account_id,
        priority_fee_sompi=Fees(0, None),
        payment_secret=None,
        fee_rate=None,
        payload=None,
        destination=None,
    )
    print(f"Change-only send: {send_result}")
    print()

    transfer_result = await wallet.accounts_transfer(
        wallet_secret=WALLET_SECRET,
        source_account_id=account_id,
        destination_account_id=account_id,
        transfer_amount_sompi=1,
        payment_secret=None,
        fee_rate=None,
        priority_fee_sompi=None,
    )
    print(f"Self-transfer (1 sompi): {transfer_result}")
    print()

    # ------------------------------------------------------------------
    # 6. Transaction history. The replace_* calls need a real tx id; we
    #    use a placeholder so the code path is still exercised.
    # ------------------------------------------------------------------
    try:
        incoming = await wallet.transactions_data_get(
            account_id=account_id,
            network_id=NetworkId(NETWORK_ID),
            start=0,
            end=10,
            filter=[TransactionKind.Incoming],
        )
        print(f"Incoming transactions: {incoming}")
    except Exception as exc:
        print(f"{type(exc).__name__}: {exc}")
    print()

    placeholder_tx_id = "0" * 64

    try:
        await wallet.transactions_replace_note(
            account_id=account_id,
            network_id=NetworkId(NETWORK_ID),
            transaction_id=placeholder_tx_id,
            note="demo note",
        )
        print("Replaced note on placeholder tx")
    except Exception as exc:
        print(f"{type(exc).__name__}: {exc}")
    print()

    try:
        await wallet.transactions_replace_metadata(
            account_id=account_id,
            network_id=NetworkId(NETWORK_ID),
            transaction_id=placeholder_tx_id,
            metadata="demo metadata",
        )
        print("Replaced metadata on placeholder tx")
    except Exception as exc:
        print(f"{type(exc).__name__}: {exc}")
    print()

    # ------------------------------------------------------------------
    # 7. Wind down.
    # ------------------------------------------------------------------
    wallet.remove_event_listener(WalletEventType.Balance)

    await wallet.wallet_close()
    print("Wallet closed")

    await wallet.disconnect()
    print("RPC disconnected")

    await wallet.stop()
    print("Wallet stopped")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--rpc-url", default=None, help="RPC URL (defaults to Resolver)")
    args = parser.parse_args()
    asyncio.run(main(args.rpc_url))
