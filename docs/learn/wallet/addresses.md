# Addresses

A BIP32 account derives [`Address`](../../reference/Classes/Address.md) instances. The wallet records two
indices per account — one for receive, one for change. 
[`accounts_create_new_address`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_create_new_address) advances them. Keypair accounts have a
single fixed address; calling [`accounts_create_new_address`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_create_new_address) on one
raises.

For the lower-level [`Address`](../../reference/Classes/Address.md) primitive (parsing, encoding, script conversion),
see [Fundamentals → Addresses](../addresses.md).

## Read the current addresses

[`AccountDescriptor`](../../reference/Classes/AccountDescriptor.md) already carries the most recent of each:

```python
acct = await wallet.accounts_get(account_id=acct.account_id)
print(acct.receive_address)        # for the next-to-receive index
print(acct.change_address)         # for the next-to-spend-from-as-change index
print(acct.receive_address_index)  # int, BIP32 only
print(acct.change_address_index)   # int, BIP32 only
```

[`get_addresses()`](../../reference/Classes/AccountDescriptor.md) returns every derived address on the account — the
right choice for re-subscribing UTXO notifications across all of them.

## Derive the next address

[`accounts_create_new_address`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_create_new_address) takes a [`NewAddressKind`](../../reference/Enums/NewAddressKind.md) and returns the next derived address:

```python
from kaspa import NewAddressKind

next_recv = await wallet.accounts_create_new_address(
    account_id=acct.account_id,
    kind=NewAddressKind.Receive,
)
next_change = await wallet.accounts_create_new_address(
    account_id=acct.account_id,
    kind=NewAddressKind.Change,
)
```

The index used is the descriptor's `receive_address_index` or
`change_address_index` *before* the call; afterwards the descriptor's
counter advances by one. Newly derived addresses register
automatically with the account's [`UtxoContext`](../../reference/Classes/UtxoContext.md), so funds sent to them
appear in the next sync.

## Receive vs. change

- **Receive** addresses are what you hand out to 3rd parties.
- **Change** addresses are where the wallet returns leftover funds.

To sweep a UTXO set and leave leftover on a *fresh* change address,
advance the change index first — see [Sweep Funds](sweep.md).

## Address discovery on import

[`accounts_import_bip32`](../../reference/Classes/Wallet.md#kaspa.Wallet.accounts_import_bip32) walks the receive and change chains for
addresses that have ever held a UTXO and bumps the indices
accordingly. That's what lets a restored wallet "remember" addresses
it previously handed out. Without it, `next_recv` would silently
re-issue an already-used address.

## Where to next

- [Send Transaction](send-transaction.md) — sending to an address.
- [Events](events.md) — events that fire when a derived address
  receives funds.
