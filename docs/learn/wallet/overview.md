# Wallet

The [`Wallet`](../../reference/Classes/Wallet.md) class is the SDK's
high-level managed wallet. It layers encrypted on-disk storage,
multi-account management, an event bus, and built-in send / transfer /
sweep flows on top of the primitives in
[Wallet SDK](../wallet-sdk/overview.md).

Features:

- Persistent encrypted on-disk storage for keys and account metadata
- Multi-account management across BIP32 and keypair accounts
- Event bus for chain notifications (balance, maturity, reorg)
- Built-in send, transfer, and sweep flows
- Address derivation and discovery
- Transaction history tracking

## A wallet, end to end

```python
import asyncio
from kaspa import PrvKeyDataVariantKind, Resolver, Wallet

async def main():
    wallet = Wallet(network_id="testnet-10", resolver=Resolver())
    await wallet.start()
    await wallet.connect()

    await wallet.wallet_create(
        wallet_secret="example-secret",
        filename="demo",
        title="demo",
    )
    prv_key_id = await wallet.prv_key_data_create(
        wallet_secret="example-secret",
        secret="<your 24-word mnemonic>",
        kind=PrvKeyDataVariantKind.Mnemonic,
    )
    descriptor = await wallet.accounts_create_bip32(
        wallet_secret="example-secret",
        prv_key_data_id=prv_key_id,
        account_index=0,
    )
    await wallet.accounts_activate([descriptor.account_id])

    await wallet.wallet_close()
    await wallet.stop()

asyncio.run(main())
```

This script creates a wallet file on disk, derives a BIP32 account from a
mnemonic, and activates it. Re-running raises
`WalletAlreadyExistsError` unless you switch to `wallet_open` — see
[Lifecycle](lifecycle.md#open-a-wallet-file).

## How this section is laid out

- [Architecture](architecture.md) — `Wallet` / `UtxoProcessor` /
  `UtxoContext` and how notifications flow.
- [Lifecycle](lifecycle.md) — the full state machine: construct,
  start, connect, open, activate, close, stop.
- [Sync State](sync-state.md) — node IBD vs. processor readiness.
- [Wallet Files](wallet-files.md) — enumerate, export, import,
  rename, change secret.
- [Private Keys](private-keys.md), [Accounts](accounts.md),
  [Addresses](addresses.md), [Keypair Accounts](keypair.md) —
  populating the wallet.
- [Send Transaction](send-transaction.md), [Sweep Funds](sweep.md) —
  outgoing flows.
- [Transaction History](transaction-history.md) — events and history
  APIs.
