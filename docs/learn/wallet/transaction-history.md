---
search:
  boost: 3
---

# Transaction History

The wallet stores a record of every transaction that touched each
account, with optional user-supplied note and metadata strings.

For the live event stream (`Balance`, `Pending`, `Maturity`, etc.),
see [Events](events.md).

## Fetching a window

[`transactions_data_get`](../../reference/Classes/Wallet.md#kaspa.Wallet.transactions_data_get) returns a paged window of records:

```python
data = await wallet.transactions_data_get(
    account_id=account.account_id,
    network_id="testnet-10",
    start=0,
    end=20,
)
```

- `account_id` — the account whose history you want.
- `network_id` — history is stored per network; pass the same
  identifier the wallet is connected to.
- `start` / `end` — half-open paging window into the stored history
  (newest first).
- `filter` *(optional)* — list of
  [`TransactionKind`](../../reference/Enums/TransactionKind.md) values
  (or kebab strings) to keep; everything else is dropped.

Returns a dict containing the matching `TransactionRecord`s. Each
record carries `id`, `unixtimeMsec`, `value`, `binding`,
`blockDaaScore`, `network`, `data` (the per-kind transaction body),
and the optional `note` / `metadata` strings.

## Annotating records

Notes and metadata are free-form strings stored alongside the record
in the wallet file, not on chain. Update them with
[`transactions_replace_note`](../../reference/Classes/Wallet.md#kaspa.Wallet.transactions_replace_note) and
[`transactions_replace_metadata`](../../reference/Classes/Wallet.md#kaspa.Wallet.transactions_replace_metadata):

```python
await wallet.transactions_replace_note(
    account_id=account.account_id,
    network_id="testnet-10",
    transaction_id=tx_id,
    note="rent",
)
await wallet.transactions_replace_metadata(
    account_id=account.account_id,
    network_id="testnet-10",
    transaction_id=tx_id,
    metadata='{"tag": "ops"}',
)
```

Pass `None` (or omit) to clear the existing value. If you want
structured metadata, encode it yourself (JSON, msgpack, etc.) — the
wallet stores opaque strings.

## Stored records vs. live events

The history APIs read what's already on disk; they don't reach out to
the node. New records land on disk as the
[`UtxoProcessor`](../../reference/Classes/UtxoProcessor.md) processes
notifications, so for "react when something happens" flows use the
[event](events.md) surface and only call [`transactions_data_get`](../../reference/Classes/Wallet.md#kaspa.Wallet.transactions_data_get) when
you need the persisted view (UI history pane, audit, export).

## Where to next

- [Events](events.md) — live `Balance` / `Pending` / `Maturity` etc.
- [Send Transaction](send-transaction.md) — outgoing flows that
  populate this history.
- [Wallet Files](wallet-files.md) — where the records are persisted.
