---
search:
  boost: 3
---

# Compiling Contracts

[`compile`](../../reference/SilverScript/Functions/compile.md) turns
SilverScript source into a
[`CompiledContract`](../../reference/SilverScript/Classes/CompiledContract.md) —
the locking script plus everything needed to build unlocking scripts
to spend.

```python
import kaspa.experimental.silverscript as silverscript

contract = silverscript.compile(source, constructor_args=None)
```

`source` is the contract as a string. `constructor_args` is a list of
values for the contract's constructor parameters. Omit it (or pass `None` / `[]`) for a contract with
no parameters.

A compile failure — a syntax error, a type error, an incompatible
`pragma` — raises
[`SilverScriptError`](../../reference/SilverScript/Exceptions/SilverScriptError.md).
The message points to the byte span of the offending source.

```python
try:
    silverscript.compile("contract B() { entry m() { byte x = 256; } }")
except silverscript.SilverScriptError as e:
    print(e)        # ... (at bytes <start>..<end>)
```

## Constructor args embed state into the script

Constructor arguments are compiled **into** the locking script. Two
different argument sets produce two different scripts, and so two
different P2SH addresses:

```python
silverscript.compile(SOURCE, [100]).bytecode != silverscript.compile(SOURCE, [101]).bytecode
```

This is a key mental model for SilverScript. A `Counter` at `count = 0` and the
same `Counter` at `count = 5` are different scripts, and as a result, are different
P2SH addresses. Compilation is deterministic — the same source and args
always produce identical bytes — so you can re-derive an address at any
time from known source and state.

## The compiled contract

A [`CompiledContract`](../../reference/SilverScript/Classes/CompiledContract.md)
is read-only. Its properties:

| Property | What it is |
| --- | --- |
| [`bytecode`](../../reference/SilverScript/Classes/CompiledContract.md) | The locking (redeem) script `bytes`. |
| [`contract_name`](../../reference/SilverScript/Classes/CompiledContract.md) | The contract name from the source. |
| [`compiler_version`](../../reference/SilverScript/Classes/CompiledContract.md) | The compiler version that produced it. |
| [`abi`](../../reference/SilverScript/Classes/CompiledContract.md) | One [`EntryAbi`](../../reference/SilverScript/Classes/EntryAbi.md) per callable entrypoint, in source order. |
| [`state_layout`](../../reference/SilverScript/Classes/CompiledContract.md) | `(start, len)`: the byte offset and length of the contract state within the script. |
| [`template_hash`](../../reference/SilverScript/Classes/CompiledContract.md) | 32-byte digest over the script's template parts, matching the `templateHash()` builtin. |

## Reading the ABI

The [`abi`](../../reference/SilverScript/Classes/CompiledContract.md) tells
you which entrypoints a contract exposes and what arguments each one
takes.

Each [`EntryAbi`](../../reference/SilverScript/Classes/EntryAbi.md) has a
`name`, a list of `params`, and a `dispatch_tag`. Each
[`ParamAbi`](../../reference/SilverScript/Classes/ParamAbi.md) has a `name`
and a `type_name` — the SilverScript type, e.g. `"int"`, `"byte[32]"`,
`"pubkey"`, `"sig"`.

```python
contract = silverscript.compile(SOURCE, [100])

for entry in contract.abi:
    args = ", ".join(f"{p.type_name} {p.name}" for p in entry.params)
    print(f"{entry.name}({args})")
# check(int amount)
```

`dispatch_tag` is the entrypoint's four-byte identity —
`blake3("name(type,type)")[:4]`. It is content-addressed, so it depends only
on the entrypoint's name and parameter types, never on constructor arguments:
every instance of a contract shares the same tags. It is also the final data
push of every signature script built for that entrypoint.

Next: turn one of these entrypoints into an unlocking script in
[Unlocking Scripts](unlocking-scripts.md).
