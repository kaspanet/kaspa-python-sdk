---
search:
  boost: 3
---

# Unlocking Scripts

To spend a UTXO locked behind a compiled contract, you call
one of the contract's entrypoints with arguments, and the compiler emits
the bytes that satisfy the lock.

```python
# Genesis with min spend threshold of 100
contract = silverscript.compile(SOURCE, [100])

# Spend 150
sig_script = contract.build_sig_script("check", [150])
```

[`build_sig_script`](../../reference/SilverScript/Classes/CompiledContract.md)
takes the entrypoint `name` and a list of `args`, and returns the
unlocking script `bytes`. Those bytes go into a transaction input's
`signature_script`. For a covenant entrypoint, use
[`build_sig_script_for_covenant_decl`](../../reference/SilverScript/Classes/CompiledContract.md)
instead — see [Covenants](covenants.md).

## Calling an entrypoint

Only functions marked `entrypoint` are callable from a spend. Pass the
entrypoint name and a list of positional arguments, in the order the
entrypoint declares them; the compiler emits the right unlocking script —
including any selector it needs to pick the function when a contract has
several entrypoints. You don't construct or read these bytes yourself —
you put them on the input (see
[Spending a locked UTXO](#spending-a-locked-utxo)).

```python
# check(int amount) — args are a positional list matching the parameters.
sig_script = contract.build_sig_script("check", [150])
```

An entrypoint that takes no arguments is called with no args (or an empty
list):

```python
announcement.build_sig_script("announce")   # no-arg entrypoint: call with just the name
```

## Argument types

`args` are native Python values, mapped to the entrypoint's declared
SilverScript types (the `type_name`s you can read off the
[ABI](compiling.md#reading-the-abi)):

| SilverScript type | Python value |
| --- | --- |
| `int` | `int` (must fit in a signed 64-bit integer) |
| `bool` | `bool` (a real bool — not `0`/`1`) |
| `byte[N]` | `bytes` / `bytearray` of length `N` |
| `pubkey` | `bytes` (an x-only public key) |
| `sig` | `bytes` (a signature) |
| `T[]` | `list` or `tuple` of `T` |
| struct / `State` | `dict` |

A few rules worth knowing:

- **`bool` is distinct from `int`.** `True` is not `1` here — pass the
  type the entrypoint declares.
- **`list` and `tuple` are interchangeable** for array arguments.
- **Out-of-range and mistyped values raise
  [`SilverScriptError`](../../reference/SilverScript/Exceptions/SilverScriptError.md)**,
  not a Python `OverflowError` or `TypeError`. An `int` outside the
  signed 64-bit range, a `byte[4]` given five bytes, or a deeply nested
  argument all fail cleanly instead of producing a bad script.

```python
contract.build_sig_script("check", [2**63])   # raises SilverScriptError
```

## Spending a locked UTXO

The unlocking script is one piece of a P2SH spend. The input's
`signature_script` must reveal the redeem script and satisfy it, so
you concatenate the contract call with the pushed redeem script:

```python
from kaspa import ScriptBuilder

contract = silverscript.compile(SOURCE, [100])
call = contract.build_sig_script("check", [150])

# Push the redeem script so it rides along in the same signature_script.
redeem = bytes.fromhex(
    ScriptBuilder().add_data(contract.script).to_string()
)
signature_script = call + redeem
```

Put `signature_script` on the
[`TransactionInput`](../../reference/Classes/TransactionInput.md) that
spends the locked UTXO. The full P2SH mechanics — wrapping the lock,
building the address, the spend side — are in
[Transactions → Scripts](../transactions/scripts.md).

## Building twice recompiles

A [`CompiledContract`](../../reference/SilverScript/Classes/CompiledContract.md)
stores its source and constructor args, not a borrowed parse tree. So
each call to
[`build_sig_script`](../../reference/SilverScript/Classes/CompiledContract.md)
recompiles the contract from scratch before assembling the script. It's
deterministic — the same call always yields the same bytes — but each
call pays the full compile cost. That matters only if you build many
unlocking scripts in a hot loop; for one spend per transaction, it's
irrelevant.

Next: stateful contracts that carry state from one UTXO to the next —
[Covenants](covenants.md).
