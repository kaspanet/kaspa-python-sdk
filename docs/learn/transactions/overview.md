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

Most of the time you'll use the higher-level
[Transaction Generator](../wallet-sdk/tx-generator.md) (or the
managed [Wallet](../wallet/send-transaction.md) on top of it). This
section covers the primitives underneath, so you can build manually
when you need custom lockup scripts, exact input ordering, payload
data, or offline signing.

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

- **Inputs carry their own UTXO context** via `UtxoEntryReference`, so
  the signer doesn't have to re-fetch the spent output for its amount
  and lockup. See [Inputs](inputs.md).
- **Mass replaces "byte size × rate"** as the fee model. Compute mass
  on the transaction (including a storage component derived from
  input and output values), then multiply by the prevailing fee rate.
  See [Mass & fees](mass-and-fees.md).
- **The atomic unit is the sompi**: `1 KAS = 100_000_000 sompi`.
  Every amount in the transaction surface is a sompi int — convert
  only at the UI boundary.

## End-to-end (manual path)

```python
from kaspa import (
    Transaction, TransactionInput, TransactionOutput, TransactionOutpoint,
    UtxoEntryReference, sign_transaction, pay_to_address_script,
    update_transaction_mass,
)

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

update_transaction_mass("mainnet", tx)
signed = sign_transaction(tx, [private_key], verify_sig=True)

await client.submit_transaction({
    "transaction": signed.serialize_to_dict(),
    "allowOrphan": False,
})
```

This is what the [Generator](../wallet-sdk/tx-generator.md) does
internally — it picks UTXOs, computes mass, signs, and yields one or
more ready-to-submit `PendingTransaction`s. Reach for the manual path
when you need control the Generator doesn't expose.

## In this section

- **[Inputs](inputs.md)** — `TransactionInput`,
  `TransactionOutpoint`, `UtxoEntryReference`, and why inputs carry
  their UTXO context.
- **[Outputs](outputs.md)** — `TransactionOutput`, `ScriptPublicKey`,
  the lockup scripts that pay an address.
- **[Mass & fees](mass-and-fees.md)** — computing mass, storage mass,
  the fee market, and when to call `update_transaction_mass`.
- **[Signing](signing.md)** — `sign_transaction`, `SighashType`,
  per-input signing, multi-key flows.
- **[Submission](submission.md)** — `submit_transaction`, what
  "submitted" means, and how confirmation works.
- **[Metadata fields](metadata.md)** — `version`, `lock_time`,
  `subnetwork_id`, `gas`, `payload` — the fields you mostly leave
  alone.
- **[Serialization](serialization.md)** — `to_dict()` / `from_dict()`
  for round-tripping through other systems.

## Where to next

- [Wallet SDK → Transaction Generator](../wallet-sdk/tx-generator.md)
  — the high-level coin selector + signer.
- [Wallet → Send Transaction](../wallet/send-transaction.md) — the
  managed Wallet's send surface.
- [Kaspa Concepts](../concepts.md) — UTXO model, mass, fees,
  maturity.
