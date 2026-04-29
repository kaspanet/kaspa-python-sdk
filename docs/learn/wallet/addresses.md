# Addresses

A BIP32 account derives addresses lazily. The wallet records two indices
per account — one for receive, one for change — and `accounts_create_new_address`
advances them. Keypair accounts have a single, fixed address and reject this
call.

## Read the current addresses

`AccountDescriptor` already carries the most recent of each:

```python
acct = await wallet.accounts_get(acct_id)
print(acct.receive_address)        # for the next-to-receive index
print(acct.change_address)         # for the next-to-spend-from-as-change index
print(acct.receive_address_index)  # int, BIP32 only
print(acct.change_address_index)   # int, BIP32 only
```

`get_addresses()` returns *every* derived address on the account, which
is what you want for re-subscribing UTXO notifications across all of them.

## Derive the next address

```python
from kaspa import NewAddressKind

next_recv = await wallet.accounts_create_new_address(
    acct.account_id, NewAddressKind.Receive,
)
next_change = await wallet.accounts_create_new_address(
    acct.account_id, NewAddressKind.Change,
)
```

The index used is the descriptor's `receive_address_index` or
`change_address_index` *before* the call; afterwards the descriptor's
counter advances by one. Each newly derived address is automatically
registered with the account's `UtxoContext`, so funds sent to it will
appear in the next sync.

## Receive vs. change

- **Receive** addresses are what you give out. Generate one any time you
  want a new public-facing address — for billing, for separating
  customers, for a hot/cold split.
- **Change** addresses are where the wallet returns leftover funds when
  spending. The `Generator` (used internally by `accounts_send`) picks the
  current change address automatically; you usually don't need to advance
  the change index manually.

If you're sweeping a UTXO set and want the leftover to stay in the same
account but on a *fresh* change address, advance the change index first —
see [Sweep Funds](sweep.md).

## Address discovery on import

When you `accounts_import_bip32` (rather than `accounts_create_bip32`),
the wallet walks the receive and change chains looking for addresses that
have ever held a UTXO and bumps the indices accordingly. This is what
makes a restored wallet "remember" addresses it had previously handed out.
Without it, `next_recv` would silently re-issue an already-used address.

## Where to next

- [Send Transaction](send-transaction.md) — sending to an address.
- [Transaction History](transaction-history.md) — events that fire when a
  derived address receives funds.
- [Wallet Recovery](../../guides/wallet-recovery.md) — scanning a mnemonic
  to find used account indices before importing.
