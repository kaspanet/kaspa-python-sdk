# Wallet

The `Wallet` class is the SDK's high-level managed wallet. It layers
encrypted on-disk storage, multi-account management, an event bus, and
built-in send / transfer / sweep flows on top of the lower-level primitives
in [Wallet SDK](../wallet-sdk/index.md).

## When to reach for `Wallet`

| You want to... | Use |
| --- | --- |
| Persist secrets, manage multiple accounts, react to chain events | `Wallet` |
| Sign one transaction in a script, no on-disk state | The primitives in [Wallet SDK](../wallet-sdk/index.md) |
| Embed wallet behaviour in your own app | `Wallet` — the listener model and account API are designed for this |

If you only need a one-shot signer, the primitives are simpler. If you need
the "open file → manage keys → track UTXOs → send" loop, `Wallet` saves
you from re-implementing it.

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

This script creates a wallet file, derives a BIP32 account from a
mnemonic, and activates it. Re-running fails with
`WalletAlreadyExistsError` unless you switch to `wallet_open` — see
[Open](open.md).

## How this section is laid out

- [Architecture](architecture.md) — `Wallet` / `UtxoProcessor` /
  `UtxoContext` and how notifications flow through them.
- [Lifecycle](lifecycle.md) — the state machine and ordering rules for the
  whole class.
- [Initialize](initialize.md), [Start](start.md), [Open](open.md) — each
  phase of bringing a wallet up.
- [Private Keys](private-keys.md), [Accounts](accounts.md),
  [Addresses](addresses.md), [Keypair Accounts](keypair.md) — populating
  the wallet.
- [Send Transaction](send-transaction.md), [Sweep Funds](sweep.md) —
  outgoing flows.
- [Transaction History](transaction-history.md) — the event surface and
  history APIs.
