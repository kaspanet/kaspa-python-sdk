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

Only functions declared with `entry` are callable from a spend. Pass the
entrypoint name and a list of positional arguments, in the order the
entrypoint declares them; the compiler emits the right unlocking script —
including the four-byte dispatch tag that selects the entrypoint. You don't
construct or read these bytes yourself —
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
| `temporal` | `int` (a time value in milliseconds) |
| `byte` | `int` in `0..=255` (a one-byte `bytes` also works) |
| `byte[]` | `bytes` / `bytearray` (not a `list`) |
| `byte[N]` | `bytes` / `bytearray` of length `N` |
| `string` | `str` (encoded as UTF-8) |
| `pubkey` | `bytes` (a 32-byte x-only public key) |
| `sig` | `bytes` (a 65-byte signature — 64 plus the sighash type byte) |
| `datasig` | `bytes` (a 64-byte signature over a message, for `checkMsgSig`) |
| `T[]` | `list` or `tuple` of `T` |
| struct / `State` | `dict` |

A few rules worth knowing:

- **`bool` is distinct from `int`.** `True` is not `1` here — pass the
  type the entrypoint declares.
- **Which Python value means what is decided by the declared type, not the
  value.** `1` is a `byte` for a `byte` parameter and an `int` for an `int`
  one; a small `int` is never silently treated as a `byte`.
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
    ScriptBuilder().add_data(contract.bytecode).to_string()
)
signature_script = call + redeem
```

Put `signature_script` on the
[`TransactionInput`](../../reference/Classes/TransactionInput.md) that
spends the locked UTXO. The full P2SH mechanics — wrapping the lock,
building the address, the spend side — are in
[Transactions → Scripts](../transactions/scripts.md).

## Compile once, build many

[`compile`](../../reference/SilverScript/Functions/compile.md) does all
the expensive work up front: it parses the source, compiles the contract,
and builds its portable ABI artifact once, then keeps them on the
[`CompiledContract`](../../reference/SilverScript/Classes/CompiledContract.md).

Each later call to
[`build_sig_script`](../../reference/SilverScript/Classes/CompiledContract.md)
works from that stored artifact — it converts your arguments to the
declared parameter types, pushes them, and appends the entrypoint's
four-byte dispatch tag. Nothing is recompiled. Building many unlocking
scripts from one contract is cheap, and it's deterministic: the same call
always yields the same bytes.

Next: stateful contracts that carry state from one UTXO to the next —
[Covenants](covenants.md).
