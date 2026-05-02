# Wallet

The [`Wallet`](../../reference/Classes/Wallet.md) class is the SDK's
high-level managed wallet. It leverages primitives in
[Wallet SDK](../wallet-sdk/overview.md) (and other rusty-kaspa components) to provide the following features:

- Persistent encrypted on-disk storage for keys and account metadata
- Multi-account management across BIP32 and keypair accounts
- Event bus for chain notifications (balance, maturity, reorg)
- Built-in send, transfer, and sweep flows
- Address derivation and discovery
- Transaction history tracking

## Runnable example

Generates a fresh testnet mnemonic, opens a new wallet file, derives a
BIP32 account, waits for sync, prints the receive address, and tears
down cleanly. No external funds required to run.

```python
import asyncio
from kaspa import (
    Mnemonic,
    PrvKeyDataVariantKind,
    Resolver,
    Wallet,
)

async def main():
    wallet = Wallet(network_id="testnet-10", resolver=Resolver())
    await wallet.start()
    await wallet.connect(strategy="fallback", timeout_duration=5_000)

    while not wallet.is_synced:
        await asyncio.sleep(0.5)

    await wallet.wallet_create(
        wallet_secret="example-secret",
        filename="demo",
        title="demo",
    )

    mnemonic = Mnemonic.random()
    prv_key_id = await wallet.prv_key_data_create(
        wallet_secret="example-secret",
        secret=mnemonic.phrase,
        kind=PrvKeyDataVariantKind.Mnemonic,
    )
    account = await wallet.accounts_create_bip32(
        wallet_secret="example-secret",
        prv_key_data_id=prv_key_id,
        account_index=0,
    )
    await wallet.accounts_activate([account.account_id])

    print("receive address:", account.receive_address)

    await wallet.wallet_close()
    await wallet.disconnect()
    await wallet.stop()

asyncio.run(main())
```

Re-running raises [`WalletAlreadyExistsError`](../../reference/Exceptions/WalletAlreadyExistsError.md) on [`wallet_create`](../../reference/Classes/Wallet.md#kaspa.Wallet.wallet_create) —
switch to [`wallet_open`](../../reference/Classes/Wallet.md#kaspa.Wallet.wallet_open), or pass `overwrite_wallet_storage=True`. See
[Lifecycle](lifecycle.md).

For a full send / sweep / history flow, see
[`examples/wallet/transactions.py`](https://github.com/kaspanet/kaspa-python-sdk/blob/main/examples/wallet/transactions.py).

## How this section is laid out

- [Architecture](architecture.md) — [`Wallet`](../../reference/Classes/Wallet.md) / [`UtxoProcessor`](../../reference/Classes/UtxoProcessor.md) /
  [`UtxoContext`](../../reference/Classes/UtxoContext.md) and how notifications flow.
- [Lifecycle](lifecycle.md) — the state machine and ordering rules.
- [Wallet Files](wallet-files.md) — enumerate, export, import,
  rename, change secret.
- [Private Keys](private-keys.md) and [Accounts](accounts.md) —
  populating the wallet (BIP32 and keypair).
- [Addresses](addresses.md) — derive and inspect addresses.
- [Send Transaction](send-transaction.md), [Sweep Funds](sweep.md) —
  outgoing flows, including the fee model and UTXO maturity.
- [Sync State](sync-state.md) — node IBD vs. processor readiness.
- [Events](events.md) — live event subscriptions.
- [Transaction History](transaction-history.md) — stored records and
  annotation APIs.
- [Errors](errors.md) — common exceptions and their fixes.
