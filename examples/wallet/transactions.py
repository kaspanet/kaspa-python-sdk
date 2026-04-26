"""Example showing transactions via wallet.

Requires a live testnet-10 RPC node (passed via `--rpc-url`, or reachable
via the public `Resolver`) and testnet-10 funds sent to the printed
account_0 receive address.

Funds are sent between two BIP32 accounts derived from the same mnemonic.

- Wait at `account_0`'s receive address until incoming UTXOs are visible on
  testnet-10.
- Derive five receive addresses on `account_1` and send to all five in a
  single transaction (six outputs: 5 destinations + 1 change).
- Wait for `account_1`'s to receive prior tx. Derive a fresh receive address
  on `account_1`, then sweep every UTXO into that single address.
- Print the resulting transaction history and balances for both accounts.
"""

import argparse
import asyncio

from kaspa import (
    FeeSource,
    Fees,
    NetworkId,
    NewAddressKind,
    PaymentOutput,
    PrvKeyDataVariantKind,
    Resolver,
    TransactionKind,
    Wallet,
)
from kaspa.exceptions import WalletAlreadyExistsError

from shared import (
    FIXED_MNEMONIC_PHRASE,
    NETWORK_ID,
    WALLET_SECRET,
)

TITLE = "example wallet transactions"
FILENAME = "-".join(TITLE.split(" "))

# Per-destination amount for the multi-output send
PER_OUTPUT_SOMPI = 100_000_000  # 1 KAS
NUM_DESTINATIONS = 5


async def _wait_for_utxos(wallet, account_id, label, min_count=1):
    while True:
        utxos = await wallet.accounts_get_utxos(account_id=account_id)
        descriptor = await wallet.accounts_get(account_id)
        print(f"  utxos={len(utxos)} balance={descriptor.balance}")
        if len(utxos) >= min_count:
            print(f">>> {label}: {len(utxos)} UTXO(s) available\n")
            return utxos
        await asyncio.sleep(1)


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

    await wallet.connect(url=rpc_url, strategy="fallback", timeout_duration=5000)
    print("RPC connected\n")

    while not wallet.is_synced:
        await asyncio.sleep(0.5)
    print("Wallet synced\n")

    # ------------------------------------------------------------------
    # Open existing wallet or create new (with two BIP32 accounts)
    # ------------------------------------------------------------------
    print()
    print("-" * 100)
    print("\tOpen existing wallet or create new")
    print("-" * 100)

    try:
        created = await wallet.wallet_create(
            wallet_secret=WALLET_SECRET,
            filename=FILENAME,
            overwrite_wallet_storage=False,
            title=TITLE,
            user_hint="example",
        )
        print(f"Created new wallet file {FILENAME}: {created}\n")
    except WalletAlreadyExistsError:
        print(f"Wallet with filename {FILENAME} already exists\n")
        opened = await wallet.wallet_open(WALLET_SECRET, True, FILENAME)
        print(f"Opened existing wallet {FILENAME}: {opened}\n")

    accounts = await wallet.accounts_enumerate()

    account_0 = next((a for a in accounts if a.account_name == "demo-acct-0"), None)
    if account_0 is None:
        prv_key_id = await wallet.prv_key_data_create(
            wallet_secret=WALLET_SECRET,
            secret=FIXED_MNEMONIC_PHRASE,
            kind=PrvKeyDataVariantKind.Mnemonic,
            payment_secret=None,
            name="demo-key",
        )
        account_0 = await wallet.accounts_create_bip32(
            wallet_secret=WALLET_SECRET,
            prv_key_data_id=prv_key_id,
            payment_secret=None,
            account_name="demo-acct-0",
            account_index=0,
        )
        print(f"Created BIP32 account (index=0): {account_0}\n")
    else:
        prv_key_id = account_0.prv_key_data_ids[0]
        print(f"Using existing BIP32 account (index=0): {account_0}\n")

    account_1 = next((a for a in accounts if a.account_name == "demo-acct-1"), None)
    if account_1 is None:
        account_1 = await wallet.accounts_create_bip32(
            wallet_secret=WALLET_SECRET,
            prv_key_data_id=prv_key_id,
            payment_secret=None,
            account_name="demo-acct-1",
            account_index=1,
        )
        print(f"Created BIP32 account (index=1): {account_1}\n")
    else:
        print(f"Using existing BIP32 account (index=1): {account_1}\n")

    print(f"account_0.account_id = {account_0.account_id}")
    print(f"account_1.account_id = {account_1.account_id}\n")

    await wallet.accounts_activate([account_0.account_id, account_1.account_id])
    print(f"Activated accounts {account_0.account_id}, {account_1.account_id}\n")

    # ------------------------------------------------------------------
    # Derive 5 receive addresses on account_1 (multi-output destinations)
    # ------------------------------------------------------------------
    print()
    print("-" * 100)
    print("\tDerive 5 receive addresses on account_1")
    print("-" * 100)

    destination_addresses = []
    for i in range(NUM_DESTINATIONS):
        addr = await wallet.accounts_create_new_address(
            account_1.account_id, NewAddressKind.Receive
        )
        destination_addresses.append(addr)
        print(f" - destination {i}: {addr}")
    print()

    # ------------------------------------------------------------------
    # Wait for testnet-10 funds at account_0
    # ------------------------------------------------------------------
    print()
    print("-" * 100)
    print("\tWait for testnet-10 funds at account_0")
    print("-" * 100)

    account_0_descriptor = await wallet.accounts_get(account_0.account_id)
    receive_address = account_0_descriptor.receive_address
    print(f"account_0 receive_address = {receive_address}\n")

    print(">>> Send testnet-10 funds to the receive address above. Script will continue a few seconds after funds arrive.")
    await _wait_for_utxos(wallet, account_0.account_id, "account_0 funded")

    # ------------------------------------------------------------------
    # Fee-rate estimate
    # ------------------------------------------------------------------
    print()
    print("-" * 100)
    print("\tFee-rate estimate")
    print("-" * 100)

    fee_estimate = await wallet.fee_rate_estimate()
    print(f"Fee rate estimate: {fee_estimate}\n")

    # ------------------------------------------------------------------
    # Multi-output send: account_0 -> 5 receive addresses on account_1
    # (one transaction, 6 outputs: 5 destinations + 1 change)
    # ------------------------------------------------------------------
    print()
    print("-" * 100)
    print("\tMulti-output send (account_0 -> 5 destinations on account_1)")
    print("-" * 100)

    outputs = [PaymentOutput(addr, PER_OUTPUT_SOMPI) for addr in destination_addresses]
    print(f"Built {len(outputs)} PaymentOutput(s) of {PER_OUTPUT_SOMPI} sompi each\n")

    multi_estimate = await wallet.accounts_estimate(
        account_id=account_0.account_id,
        priority_fee_sompi=Fees(0, FeeSource.SenderPays),
        fee_rate=None,
        payload=None,
        destination=outputs,
    )
    print(f"Multi-output estimate: {multi_estimate}\n")

    multi_send = await wallet.accounts_send(
        wallet_secret=WALLET_SECRET,
        account_id=account_0.account_id,
        priority_fee_sompi=Fees(0, FeeSource.SenderPays),
        payment_secret=None,
        fee_rate=None,
        payload=None,
        destination=outputs,
    )
    print(f"Multi-output send: {multi_send}")
    print(f" - final_transaction_id: {multi_send.final_transaction_id}")
    print(f" - aggregate fees: {multi_send.fees} sompi")
    print(f" - final_amount: {multi_send.final_amount} sompi\n")

    # ------------------------------------------------------------------
    # Wait for account_1 to see all 5 incoming UTXOs
    # ------------------------------------------------------------------
    print()
    print("-" * 100)
    print("\tWait for account_1 UTXOs to settle")
    print("-" * 100)

    await _wait_for_utxos(
        wallet, account_1.account_id, "account_1 received", min_count=NUM_DESTINATIONS
    )

    # ------------------------------------------------------------------
    # Sweep account_1: consolidate every UTXO into one fresh address.
    # Fees(0, FeeSource.ReceiverPays) deducts the network fee from the
    # destination amount, leaving zero change.
    # ------------------------------------------------------------------
    print()
    print("-" * 100)
    print("\tSweep account_1 into a fresh receive address")
    print("-" * 100)

    sweep_address = await wallet.accounts_create_new_address(
        account_1.account_id, NewAddressKind.Receive
    )
    print(f"sweep destination: {sweep_address}\n")

    account_1_utxos = await wallet.accounts_get_utxos(account_id=account_1.account_id)
    sweep_total = sum(u["amount"] for u in account_1_utxos)
    print(f"account_1 total: {sweep_total} sompi across {len(account_1_utxos)} UTXO(s)\n")

    sweep_send = await wallet.accounts_send(
        wallet_secret=WALLET_SECRET,
        account_id=account_1.account_id,
        priority_fee_sompi=Fees(0, FeeSource.ReceiverPays),
        payment_secret=None,
        fee_rate=None,
        payload=None,
        destination=[PaymentOutput(sweep_address, sweep_total)],
    )
    print(f"Sweep send: {sweep_send}")
    print(f" - final_transaction_id: {sweep_send.final_transaction_id}")
    print(f" - aggregate fees: {sweep_send.fees} sompi")
    print(f" - final_amount: {sweep_send.final_amount} sompi\n")

    # ------------------------------------------------------------------
    # Transaction history (both accounts)
    # ------------------------------------------------------------------
    print()
    print("-" * 100)
    print("\tTransaction history")
    print("-" * 100)

    network_id = NetworkId(NETWORK_ID)

    for label, acct_id in (
        ("account_0", account_0.account_id),
        ("account_1", account_1.account_id),
    ):
        history = await wallet.transactions_data_get(
            account_id=acct_id,
            network_id=network_id,
            start=0,
            end=20,
            filter=[TransactionKind.Outgoing, TransactionKind.Incoming],
        )
        print(f"{label} history:")
        print(f" - {history}\n")

    sweep_tx_id = sweep_send.final_transaction_id
    await wallet.transactions_replace_note(
        account_id=account_1.account_id,
        network_id=network_id,
        transaction_id=sweep_tx_id,
        note="demo sweep",
    )
    print(f"Replaced note on sweep tx {sweep_tx_id}\n")

    await wallet.transactions_replace_metadata(
        account_id=account_1.account_id,
        network_id=network_id,
        transaction_id=sweep_tx_id,
        metadata="demo metadata",
    )
    print(f"Replaced metadata on sweep tx {sweep_tx_id}\n")

    # ------------------------------------------------------------------
    # Account / address balances
    #
    # Wait for the sweep output to mature so per-address counts reflect
    # the post-sweep state instead of pending/outgoing.
    # ------------------------------------------------------------------
    print()
    print("-" * 100)
    print("\tAccount / address balances")
    print("-" * 100)

    await _wait_for_utxos(
        wallet, account_1.account_id, "account_1 sweep output matured"
    )

    for label, acct in (("account_0", account_0), ("account_1", account_1)):
        descriptor = await wallet.accounts_get(acct.account_id)
        balance = descriptor.balance
        print(f"{label} ({acct.account_id})")
        if balance is not None:
            print(
                f" - balance: mature={balance.mature} pending={balance.pending} "
                f"outgoing={balance.outgoing} "
                f"(mature_utxos={balance.mature_utxo_count}, "
                f"pending_utxos={balance.pending_utxo_count})"
            )
        else:
            print(" - balance: None")

        addresses = descriptor.get_addresses() or []
        for addr in addresses:
            addr_utxos = await wallet.accounts_get_utxos(
                account_id=acct.account_id,
                addresses=[addr],
            )
            if not addr_utxos:
                continue
            total = sum(u["amount"] for u in addr_utxos)
            print(f"   - {addr}: {total} sompi across {len(addr_utxos)} UTXO(s)")
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

    await wallet.disconnect()
    print("RPC disconnected\n")

    await wallet.stop()
    print("Wallet stopped\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--rpc-url", default=None, help="RPC URL (defaults to Resolver)")
    args = parser.parse_args()
    asyncio.run(main(args.rpc_url))
