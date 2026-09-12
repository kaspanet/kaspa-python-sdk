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
Where the compiler records the location, the message ends with the byte span of
the offending source:

```python
try:
    silverscript.compile("pragma silverscript ^9.9.9; contract B() {}")
except silverscript.SilverScriptError as e:
    print(e)
# unsupported feature: SilverScript compiler cannot support pragmas that cover
# future major versions ... (at bytes 20..26)
```

Not every error carries one. A type error names the offending declaration
instead, and a syntax error arrives as the parser's own caret diagram:

```python
silverscript.compile("contract B() { entry m() { byte x = 256; } }")
# SilverScriptError: unsupported feature: variable 'x' expects byte
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
| [`abi`](../../reference/SilverScript/Classes/CompiledContract.md) | One [`EntryAbi`](../../reference/SilverScript/Classes/EntryAbi.md) per callable entrypoint, ordered alphabetically by name. |
| [`entry(name)`](../../reference/SilverScript/Classes/CompiledContract.md) | One entrypoint's [`EntryAbi`](../../reference/SilverScript/Classes/EntryAbi.md), by name. |
| [`state_span`](../../reference/SilverScript/Classes/CompiledContract.md) | `(offset, len)`: the byte offset and length of the contract state within the script. |
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

# One entrypoint, by name:
entry = contract.entry("check")
print(entry.params[0].type_name)   # int

# Or enumerate them all:
for entry in contract.abi:
    args = ", ".join(f"{p.type_name} {p.name}" for p in entry.params)
    print(f"{entry.name}({args})")
# check(int amount)
```

`abi` is ordered alphabetically by entrypoint name — the same order a
[`ContractArtifact`](../../reference/SilverScript/Classes/ContractArtifact.md)
reports, so an entrypoint keeps its position whichever route you reached it
by. Declaration order isn't available: it exists only while the source is
parsed, and a compiled artifact doesn't carry it.
[`entry(name)`](../../reference/SilverScript/Classes/CompiledContract.md)
raises
[`SilverScriptError`](../../reference/SilverScript/Exceptions/SilverScriptError.md)
for a name the contract doesn't declare — the same error `build_sig_script`
gives for the same typo.

`dispatch_tag` is the entrypoint's four-byte identity —
`blake3("name(type,type)")[:4]`. It is content-addressed over the name and the
*resolved* parameter types, so every instance of a contract normally shares the
same tags, whatever its constructor arguments. The one exception is a parameter
whose array length is itself a constructor parameter: `entry take(byte[n] blob)`
on `contract Sized(int n)` resolves to `byte[4]` for `n = 4` and `byte[8]` for
`n = 8` — different types, and so different tags. Read the tag off the instance
you are spending rather than caching it across instances of such a contract.

The tag is also the final data push of every signature script built for that
entrypoint.

## The portable artifact

Behind the ABI is a *portable artifact* — the full machine-readable
description of the compiled contract. `compile()` builds it once, and it is
what each `build_sig_script` call encodes against.
[`artifact_json()`](../../reference/SilverScript/Classes/CompiledContract.md)
hands it to you as JSON:

```python
import json

contract = silverscript.compile(SOURCE, [100])
artifact = json.loads(contract.artifact_json())

print(artifact["contracts"]["Guard"]["entries"]["check"]["dispatch_tag"])
# b0823999
```

For the same source and constructor arguments this is byte-for-byte what the
upstream `silverc` compiler writes, so an artifact produced here and one
produced by `silverc contract.sil` are interchangeable. That makes it a
portable build output: compile in CI, commit the artifact, and let other
tooling read it without a compiler.

Three things about the encoding are worth knowing before you parse it:

- **`bytecode` and `template_hash` are JSON arrays of integers, not hex.**
  Use `bytes(...)` to recover the raw value:
  `bytes(artifact["contracts"]["Guard"]["compiled"]["bytecode"]) == contract.bytecode`.
- **`dispatch_tag` *is* hex** — an eight-character string, unlike the two
  fields above.
- **Types are tagged objects**, e.g. `{"kind": "int"}`. The matching value
  form, `{"kind": "int", "value": 100}`, is the *portable* dialect — what
  `silverc --constructor-args` reads. It is not the type-directed JSON the
  SilverScript debugger CLI and `.test.json` fixtures use. The two look
  similar and are not interchangeable; if you are hand-writing JSON for
  upstream tooling, check which one that tool wants.

`state_span` appears here as `compiled.state_span`, an object with the keys
`offset` and `len` rather than a two-tuple — the same two numbers the
attribute gives you.

## Loading an artifact back

[`load_artifact`](../../reference/SilverScript/Functions/load_artifact.md)
reads that JSON back into a
[`ContractArtifact`](../../reference/SilverScript/Classes/ContractArtifact.md):
a contract you can derive an address from and spend, with no source and no
compiler.

```python
from pathlib import Path

# In CI, once:
Path("guard.json").write_text(silverscript.compile(SOURCE, [100]).artifact_json())

# At runtime, from guard.json alone:
artifact = silverscript.load_artifact(Path("guard.json").read_text())

artifact.bytecode                           # the same redeem script
artifact.build_sig_script("check", [150])   # the same unlocking script
```

The bytes are identical to the ones the
[`CompiledContract`](../../reference/SilverScript/Classes/CompiledContract.md)
would have produced. Both encode against this same artifact — it is the only
thing `build_sig_script` ever reads.

A `ContractArtifact` carries `contract_name`, `compiler_version`,
`schema_version`, `bytecode`, `template_hash`, `state_span`, `abi` and
`entry(name)`, plus both `build_sig_script` methods and `to_json()`. Two
things it can't do,
because they need the source: [`debug_call`](debugging.md), and compiling with
different constructor arguments — those are a different contract, and so a
different artifact.

If the JSON holds more than one contract, name the one you want:
`load_artifact(text, "Guard")`. Omitting the name is fine for anything
`compile()` produced, which is always a single contract.

### Checking an artifact you didn't build

`load_artifact` checks that the JSON parses and that its schema version is one
this release understands. It does not check that the artifact describes itself
consistently — `check_consistency()` does:

```python
artifact.check_consistency()   # raises SilverScriptError if it doesn't hold together
```

That verifies the recorded `template_hash` against the bytecode, the state span
against the script, and the entrypoints' dispatch tags for collisions. It
catches corruption and casual edits.

It cannot tell you the bytecode was compiled from any particular source —
nothing inside an artifact can. An artifact is exactly as trustworthy as where
you got it: take it from a build you control, or compare its `template_hash`
against a value you already trust.

Next: turn one of these entrypoints into an unlocking script in
[Unlocking Scripts](unlocking-scripts.md).
