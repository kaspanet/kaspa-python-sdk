---
search:
  boost: 5
---

# SilverScript

[SilverScript](https://github.com/kaspa-ng/silverscript) is a high-level
language for writing native Kaspa script-based contracts. You write a contract,
compile it to a locking script, and lock funds behind that script's
hash (a P2SH address). To spend those funds, you call one of the
contract's entrypoints, and the SDK turns that call into the unlocking
script the node accepts.

- **Compile** SilverScript source into a locking script
  ([`compile`](../../reference/SilverScript/Functions/compile.md) →
  [`CompiledContract`](../../reference/SilverScript/Classes/CompiledContract.md)).
- **Build the unlocking script** for an entrypoint call
  ([`build_sig_script`](../../reference/SilverScript/Classes/CompiledContract.md) /
  [`build_sig_script_for_covenant_decl`](../../reference/SilverScript/Classes/CompiledContract.md)).

Everything else (wrapping the locking script in a P2SH address,
building & signing the transaction, submitting) is
handled via functionality from the `kaspa` package. See [Transactions](../transactions/overview.md)
and [Scripts](../transactions/scripts.md).

!!! warning "Experimental"
    Both SilverScript and these bindings are experimental and under
    active development. The language, the compiler output, and this API
    are all subject to breaking changes and bugs. Pin your versions, test
    thoroughly, and don't lock real value behind a contract you haven't
    verified end-to-end on a test network first.

## A separate module

`kaspa.experimental.silverscript` is a sub-package — import it on
its own. The Silverscript compiler takes plain Python values (`int`, `bool`, `str`,
`bytes`, `list`/`tuple`, `dict`), not `kaspa` objects like
[`Address`](../../reference/Classes/Address.md) or
[`ScriptBuilder`](../../reference/Classes/ScriptBuilder.md), and it hands
back script `bytes`. Those `bytes` are the whole interface to core
`kaspa` SDK: compile, read
[`contract.script`](../../reference/SilverScript/Classes/CompiledContract.md),
then build the transaction with core as usual.

```python
import kaspa.experimental.silverscript as silverscript   # Silverscript
from kaspa import ScriptBuilder             # Kaspa core SDK
```

## When to use Silverscript

- **Stateful covenants** — contracts that carry state from one UTXO to
  the next (a counter, an escrow, a vault). See [Covenants](covenants.md).
- **Custom locking conditions** that are awkward to hand-assemble with
  [`ScriptBuilder`](../../reference/Classes/ScriptBuilder.md). Silverscript allows you to write in
  a typed, readable language instead of raw opcodes.

If you only need a standard P2PK or multisig, you don't
need SilverScript — see [Scripts](../transactions/scripts.md).

## An example contract

A minimal contract: a `Guard` that only lets a spend through if
`amount` is above a threshold baked in at compile time.

```python
import kaspa.experimental.silverscript as silverscript
from kaspa import ScriptBuilder, address_from_script_public_key

SOURCE = """
pragma silverscript ^0.1.0;
contract Guard(int threshold) {
    entrypoint function check(int amount) {
        require(amount > threshold);
    }
}
"""

# Compile with the constructor arg threshold = 100.
contract = silverscript.compile(SOURCE, [100])

# The redeem script
redeem = contract.script

# Wrap it in a P2SH lock and turn that into a fundable address.
spk = ScriptBuilder.from_script(redeem).create_pay_to_script_hash_script()
addr = address_from_script_public_key(spk, "testnet")
```

Send funds to `addr` and they're locked behind the contract. To spend
them, you build the unlocking script for `check(amount)` — that's
[Unlocking Scripts](unlocking-scripts.md).

## A full, live example

[`examples/silverscript/counter.py`](https://github.com/kaspanet/kaspa-python-sdk/blob/main/examples/silverscript/counter.py)
runs a stateful Counter covenant end-to-end on testnet-10. It compiles
the contract, funds it, and walks the count through `add` / `subtract`
transitions — each a real on-chain transaction. The
[Covenants](covenants.md) page walks through it.

!!! danger "Secrets handling"
    Snippets here use literal values for readability. Never handle
    production secrets this way — see
    [Security](../../getting-started/security.md).

## Next

| Page | What it covers |
| --- | --- |
| [Compiling Contracts](compiling.md) | [`compile`](../../reference/SilverScript/Functions/compile.md), constructor args, the [`CompiledContract`](../../reference/SilverScript/Classes/CompiledContract.md) surface, reading the ABI. |
| [Unlocking Scripts](unlocking-scripts.md) | Entrypoints, [`build_sig_script`](../../reference/SilverScript/Classes/CompiledContract.md), the Python → SilverScript argument mapping. |
| [Covenants](covenants.md) | Stateful contracts, [`build_sig_script_for_covenant_decl`](../../reference/SilverScript/Classes/CompiledContract.md), the Counter walkthrough. |
