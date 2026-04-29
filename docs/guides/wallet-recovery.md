# Recover a wallet (BIP-44 scan)

You have a 24-word mnemonic and you want to restore the accounts it
backs. `accounts_discovery` scans BIP-44 account and address ranges
against a connected node, returns the highest-used `account_index`, and
gets you a list of indices to import.

This is a how-to. For background on what these primitives mean, see
[Wallet → Accounts](../learn/wallet/accounts.md) and
[Wallet → Private Keys](../learn/wallet/private-keys.md).

## Recipe

```python
import asyncio
from kaspa import (
    AccountsDiscoveryKind, PrvKeyDataVariantKind, Resolver, Wallet,
)

MNEMONIC = "<24 words>"
SECRET = "<wallet password>"

async def main():
    wallet = Wallet(network_id="mainnet", resolver=Resolver())
    await wallet.start()
    await wallet.connect()

    last_used = await wallet.accounts_discovery(
        discovery_kind=AccountsDiscoveryKind.Bip44,
        address_scan_extent=20,    # consecutive empty addresses before stopping
        account_scan_extent=5,     # consecutive empty accounts before stopping
        bip39_mnemonic=MNEMONIC,
        bip39_passphrase=None,
    )
    print(f"highest used account_index: {last_used}")

    # Open (or create) a wallet file to hold the imports
    await wallet.wallet_create(wallet_secret=SECRET, filename="restored")

    pkd_id = await wallet.prv_key_data_create(
        wallet_secret=SECRET,
        secret=MNEMONIC,
        kind=PrvKeyDataVariantKind.Mnemonic,
    )

    descriptors = []
    for i in range(0, last_used + 1):
        d = await wallet.accounts_import_bip32(
            wallet_secret=SECRET,
            prv_key_data_id=pkd_id,
            account_index=i,
        )
        descriptors.append(d)

    while not wallet.is_synced:
        await asyncio.sleep(0.5)

    await wallet.accounts_activate([d.account_id for d in descriptors])

    await wallet.wallet_close()
    await wallet.disconnect()
    await wallet.stop()

asyncio.run(main())
```

## Tuning the extents

- **`address_scan_extent`** — how far past the last used receive address
  to look before declaring an account empty. BIP-44's recommended value
  is 20.
- **`account_scan_extent`** — how many consecutive empty accounts to
  tolerate before stopping. Most wallets stop at 5; raise it if the
  user is known to have skipped account indices.

If the discovery returns `-1`, no on-chain history exists under that
mnemonic — treat it as a fresh wallet (use `accounts_create_bip32`
instead of import).

## Why `_import_*` and not `_create_*`

`accounts_import_bip32` runs an address-discovery scan as part of the
import — addresses already in use are recognised and the receive /
change indices advance accordingly. The `_create_*` variants don't, so
the next address derived would silently re-issue an already-used one.
For a recovery flow, always import.

## Notes

- `accounts_discovery` does not require a wallet file to be open; it
  *does* require a connected wRPC client.
- For testnets, set `network_id="testnet-10"` (or `testnet-11`) on both
  the discovery and the wallet open.
- A passphrase-protected mnemonic must pass the same passphrase to
  `bip39_passphrase=...` and to `prv_key_data_create(...)`. Mismatched
  passphrases derive unrelated wallets.
