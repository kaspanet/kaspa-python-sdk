---
search:
  boost: 5
---

# Transactions

A Kaspa transaction has the same shape as on any UTXO chain: a list
of inputs (each spending a previous output), a list of outputs, and
a few metadata fields. The SDK exposes the underlying types —
[`Transaction`](../../reference/Classes/Transaction.md),
[`TransactionInput`](../../reference/Classes/TransactionInput.md),
[`TransactionOutput`](../../reference/Classes/TransactionOutput.md),
[`TransactionOutpoint`](../../reference/Classes/TransactionOutpoint.md),
[`UtxoEntryReference`](../../reference/Classes/UtxoEntryReference.md)
— and helpers that build, sign, mass, and serialise them.

!!! tip "Most callers don't need this section"
    The [Transaction Generator](../wallet-sdk/tx-generator.md) (and
    the managed [`Wallet`](../../reference/Classes/Wallet.md) (see [Wallet](../wallet/send-transaction.md)) on top of it)
    handles UTXO selection, mass, signing, and submission for you.
    Reach for the primitives below only when you need custom lockup
    scripts, exact input ordering, payload data, or offline signing.

## Anatomy

```
Transaction
  version, lock_time, subnetwork_id, gas, payload, mass
  inputs:  [TransactionInput, ...]
  outputs: [TransactionOutput, ...]

TransactionInput
  previous_outpoint: TransactionOutpoint(transaction_id, index)
  signature_script  (filled at sign time)
  sequence
  sig_op_count
  utxo: UtxoEntryReference

TransactionOutput
  value (sompi)
  script_public_key (lockup script)
```

What sets Kaspa apart from a Bitcoin-shaped chain:

- **Inputs carry their own UTXO context** via [`UtxoEntryReference`](../../reference/Classes/UtxoEntryReference.md), so
  the signer doesn't have to re-fetch the spent output for its amount
  and lockup. See [Inputs](inputs.md).
- **Mass replaces "byte size × rate"** as the fee model. Compute mass
  on the transaction (including a storage component derived from
  input and output values), then multiply by the prevailing fee rate.
  See [Mass & fees](mass-and-fees.md).
- **The atomic unit is the sompi**: `1 KAS = 100_000_000 sompi`.
  Every amount in the transaction surface is a sompi int. See
  [`kaspa_to_sompi`](../../reference/Functions/kaspa_to_sompi.md) and
  [`sompi_to_kaspa`](../../reference/Functions/sompi_to_kaspa.md).

## End-to-end (manual path)

This walks the manual flow using
[`sign_transaction`](../../reference/Functions/sign_transaction.md),
[`pay_to_address_script`](../../reference/Functions/pay_to_address_script.md),
and
[`update_transaction_mass`](../../reference/Functions/update_transaction_mass.md):

```python
from kaspa import (
    Transaction, TransactionInput, TransactionOutput, TransactionOutpoint,
    UtxoEntryReference, sign_transaction, pay_to_address_script,
    update_transaction_mass,
)

resp = await client.get_utxos_by_addresses({"addresses": [my_address]})
my_utxos = resp["entries"]

inputs = [
    TransactionInput(
        previous_outpoint=TransactionOutpoint(
            transaction_id=u["outpoint"]["transactionId"],
            index=u["outpoint"]["index"],
        ),
        signature_script="",          # filled at sign time
        sequence=0,
        sig_op_count=1,
        utxo=UtxoEntryReference(u),
    )
    for u in my_utxos
]

outputs = [
    TransactionOutput(value=amount,        script_public_key=pay_to_address_script(recipient)),
    TransactionOutput(value=change_amount, script_public_key=pay_to_address_script(change_addr)),
]

tx = Transaction(
    version=0, inputs=inputs, outputs=outputs,
    lock_time=0,
    subnetwork_id="0000000000000000000000000000000000000000",
    gas=0, payload="", mass=0,
)

update_transaction_mass("mainnet", tx)              # mass is signed over — fill before signing
signed = sign_transaction(tx, [private_key], verify_sig=True)

await client.submit_transaction({
    "transaction": signed,
    "allowOrphan": False,
})
```

This is what the [`Generator`](../../reference/Classes/Generator.md) (see [Transaction Generator](../wallet-sdk/tx-generator.md)) does
internally — it picks UTXOs, computes mass, signs, and yields one or
more ready-to-submit [`PendingTransaction`](../../reference/Classes/PendingTransaction.md)s.
