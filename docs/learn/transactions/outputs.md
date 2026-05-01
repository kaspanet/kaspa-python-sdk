# Outputs

A transaction's outputs are the new UTXOs it creates. Each carries a
value (in sompi) and a *locking script* — the conditions a future
spender must satisfy.

## Types involved

```
TransactionOutput
  value (sompi)
  script_public_key: ScriptPublicKey

ScriptPublicKey
  version (int)
  script  (hex bytes — the lockup conditions)
```

[`TransactionOutput`](../../reference/Classes/TransactionOutput.md)
pairs the amount with the script that locks it.
[`ScriptPublicKey`](../../reference/Classes/ScriptPublicKey.md) is the
script itself: a version byte plus the encoded program a future
spender must satisfy.

## Build an output

Pay-to-address is the common case. Build the lockup script with
[`pay_to_address_script`](../../reference/Functions/pay_to_address_script.md):

```python
from kaspa import Address, TransactionOutput, pay_to_address_script

recipient = Address("kaspa:qz...")
out = TransactionOutput(
    value=500_000_000,                              # 5 KAS in sompi
    script_public_key=pay_to_address_script(recipient),
)
```

For the inverse — recovering the address that an output pays to — use
`address_from_script_public_key`:

```python
from kaspa import NetworkType, address_from_script_public_key

addr = address_from_script_public_key(out.script_public_key, NetworkType.Mainnet)
```

That call needs a network argument because the script doesn't carry
the prefix — you have to tell the decoder which network you're
displaying for.

## Pay-to-script-hash

For multisig and other custom scripts, lockups go through a script
hash. `pay_to_script_hash_script(redeem_script)` produces the locking
side; the spender later supplies the redeem script plus signatures
via `pay_to_script_hash_signature_script(...)` at sign time.

```python
from kaspa import pay_to_script_hash_script

spk = pay_to_script_hash_script(redeem_script_bytes)
out = TransactionOutput(value=amount, script_public_key=spk)
```

For the multisig flow (address creation, multi-cosigner signing,
submission), see the
[Multi-signature transactions](../../guides/multisig.md) recipe.

## Change outputs

When selected inputs sum to more than `amount + fee`, the leftover
goes to a change output you control:

```python
outputs = [
    TransactionOutput(value=amount,        script_public_key=pay_to_address_script(recipient)),
    TransactionOutput(value=change_amount, script_public_key=pay_to_address_script(change_addr)),
]
```

The [Generator](../wallet-sdk/tx-generator.md) computes
`change_amount` for you (selected total − outputs − fee) and writes
change last. When building manually, do the arithmetic yourself —
including re-checking after `update_transaction_mass` if the fee
shifted.

If `change_amount` is too small to be worth a separate output, fold
it into the fee — paying a slightly inflated fee beats dust.

## Sompi vs KAS

Every value in the output surface (and everywhere else in the
transaction API) is a **sompi int**. `1 KAS = 100_000_000 sompi`.

```python
from kaspa import kaspa_to_sompi, sompi_to_kaspa

kaspa_to_sompi(1.5)            # 150_000_000
sompi_to_kaspa(150_000_000)    # 1.5
```

Convert only at the UI boundary. Don't store KAS as a float
internally — everything in the SDK assumes integer sompi.

## Reading outputs back

```python
for out in tx.outputs:
    print(out.value)                          # sompi
    print(out.script_public_key.version)
    print(out.script_public_key.script)       # hex
```

## Where to next

- [Addresses](../addresses.md) — what a `pay_to_address_script` is
  actually pointing at.
- [Inputs](inputs.md) — the other half of a transaction.
- [Mass & fees](mass-and-fees.md) — output values feed into the
  storage-mass component of the fee.
