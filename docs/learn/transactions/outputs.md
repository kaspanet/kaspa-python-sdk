---
search:
  boost: 3
---

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

For the inverse — recovering the address an output pays to — use
[`address_from_script_public_key`](../../reference/Functions/address_from_script_public_key.md). It needs a network argument because
the script doesn't carry the prefix:

```python
from kaspa import NetworkType, address_from_script_public_key

addr = address_from_script_public_key(out.script_public_key, NetworkType.Mainnet)
```

## Pay-to-script-hash

For multisig and other custom scripts, the locking side uses a script
hash. See
[`examples/transactions/multisig.py`](https://github.com/kaspanet/kaspa-python-sdk/blob/main/examples/transactions/multisig.py)
for the full P2SH flow (address creation, multi-cosigner signing,
submission). Build the lockup with [`pay_to_script_hash_script`](../../reference/Functions/pay_to_script_hash_script.md):

```python
from kaspa import pay_to_script_hash_script

spk = pay_to_script_hash_script(redeem_script_bytes)
out = TransactionOutput(value=amount, script_public_key=spk)
```

## Change outputs

When selected inputs sum to more than `outputs + fee`, the leftover
goes to a change output you control:

```python
outputs = [
    TransactionOutput(value=amount,        script_public_key=pay_to_address_script(recipient)),
    TransactionOutput(value=change_amount, script_public_key=pay_to_address_script(change_addr)),
]
```

The [`Generator`](../../reference/Classes/Generator.md) (see [Transaction Generator](../wallet-sdk/tx-generator.md)) computes
`change_amount` for you (`selected_total − outputs − fee`) and writes
change last. Manually, do the arithmetic yourself — and re-check
after [`update_transaction_mass`](../../reference/Functions/update_transaction_mass.md) if the fee shifted.

If a tiny change output would inflate storage mass more than it's
worth, fold it into the fee instead. See
[Mass & fees](mass-and-fees.md#storage-mass-on-its-own) for sizing.

## Sompi vs KAS

Every value in the transaction API is a **sompi int**.
`1 KAS = 100_000_000 sompi`. Convert with [`kaspa_to_sompi`](../../reference/Functions/kaspa_to_sompi.md) and [`sompi_to_kaspa`](../../reference/Functions/sompi_to_kaspa.md):

```python
from kaspa import kaspa_to_sompi, sompi_to_kaspa

kaspa_to_sompi(1.5)            # 150_000_000
sompi_to_kaspa(150_000_000)    # 1.5
```

Convert only at the UI boundary — don't store KAS as a float.

## Reading outputs back

```python
for out in tx.outputs:
    print(out.value)                          # sompi
    print(out.script_public_key.version)
    print(out.script_public_key.script)       # hex
```
