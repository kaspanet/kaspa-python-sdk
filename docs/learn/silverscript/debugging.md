---
search:
  boost: 3
---

# Debugging

[`debug_call`](../../reference/SilverScript/Functions/debug_call.md)
runs a contract entrypoint locally through SilverScript's source-level
debug engine — the same engine behind the upstream CLI debugger. It
compiles the contract with debug info, builds a synthetic transaction
that spends the contract's P2SH UTXO, executes the spend, and reports
the outcome in *source* terms: which statement failed, the call stack,
and the value of every variable in scope — not raw stack bytes.

There is no stepping or breakpoint interface: a Python script isn't an
IDE, so the session always runs to completion and hands you the full
story afterwards.

```python
import kaspa.experimental.silverscript as silverscript

GUARD = """
pragma silverscript ^0.1.0;
contract Guard(int threshold) {
    entrypoint function check(int amount) {
        int margin = amount - threshold;
        require(margin > 0);
    }
}
"""

result = silverscript.debug_call(GUARD, "check", args=[50], constructor_args=[100])
result.success            # False
print(result.failure)     # the formatted source-level report
```

```text
error: script ran, but verification failed
   --> 6:9
   |
 5 |         int margin = amount - threshold;
 6 |         require(margin > 0);
   |         ^^^^^^^^^^^^^^^^^^^^ verification failed here
   |   amount = 50, margin = -50, threshold = 100
   |
```

A failing script is reported in the
[`DebugCallResult`](../../reference/SilverScript/Classes/DebugCallResult.md),
not raised; only usage errors (bad source, unknown entrypoint, a
malformed `tx` scenario) raise `SilverScriptError`. When
`function_name` is omitted the contract's first entrypoint is called.

## The failure report

`result.failure` is a
[`FailureReport`](../../reference/SilverScript/Classes/FailureReport.md):
`message` is the engine error, and `frames` is the inlined call stack
at the failure, innermost first. Each
[`FailureFrame`](../../reference/SilverScript/Classes/FailureFrame.md)
carries the function name, the 1-based source `line`, and its
`variables` — every
[`DebugVariable`](../../reference/SilverScript/Classes/DebugVariable.md)
decoded back to a native Python value:

```python
frame = result.failure.frames[0]
for var in frame.variables:
    print(var.name, var.type_name, var.origin, var.value)
# amount    int  arg    50
# margin    int  local  -50
# threshold int  ctor   100
```

`origin` tells you where each value comes from: `local`, `arg`,
`state` (a contract field), `ctor` (a constructor argument), or
`const`. `str(result.failure)` (or `.render()`) formats the whole
report with source context, the way shown above.

Calls into non-entrypoint functions are inlined by the compiler; the
report reconstructs them as separate frames, so a `require` failing
inside a helper shows both the helper's variables and the caller's.

## Console output

`console.log(...)` statements execute during the simulation and are
captured in order on `result.console` — on success and failure alike:

```python
result = silverscript.debug_call(LOGGER, "go", args=[7])
result.console   # ['x is 7']
```

## The transaction scenario

Contracts that introspect the spending transaction (`tx.outputs`,
covenant state) need a transaction to introspect. By default
`debug_call` synthesizes a minimal one — a single 5000-sompi contract
input and a single 5000-sompi output. Pass `tx` to control the shape:

```python
result = silverscript.debug_call(
    ANNOUNCEMENT,
    "announce",
    tx={
        "inputs": [{"utxo_value": 5000}],
        "outputs": [{"value": 0}],
    },
)
```

The scenario dict accepts `version` (default 1), `lock_time`
(default 0), `active_input_index` (which input's spend is being
debugged, default 0), `inputs`, and `outputs`.

Each **input** accepts: `utxo_value` (required), `covenant_id`
(32 bytes or hex), `state` (a dict of the contract state fields
carried by the spent UTXO), `constructor_args` (per-input override),
`prev_txid`, `prev_index`, `sequence`, `sig_op_count`, and raw-bytes
overrides `signature_script` / `utxo_script`. On the active input,
`signature_script` only replaces the bytes carried by the synthetic
transaction (what sighash and introspection see) — the debug session
still executes the entrypoint call built from `function_name`/`args`.

Each **output** accepts: `value` (required), `covenant_id`,
`authorizing_input`, `state` (the post-transition contract state),
`constructor_args`, and raw-bytes overrides `script` / `p2pk_pubkey`.

## Debugging covenant transitions

For covenant contracts (see [Covenants](covenants.md)) the scenario is
where you express the state transition: the input carries the previous
state, the output carries the expected next state, and both bind the
same covenant id. Call the transition by its source-level name and
pass only the source-level arguments — the hidden verified-state
parameter is synthesized from the scenario's output states:

```python
COVENANT_ID = "11" * 32

result = silverscript.debug_call(
    COUNTER,
    "add",
    args=[5],
    constructor_args=[0],
    tx={
        "inputs": [
            {"utxo_value": 5000, "covenant_id": COVENANT_ID, "state": {"count": 10}}
        ],
        "outputs": [
            {
                "value": 5000,
                "covenant_id": COVENANT_ID,
                "authorizing_input": 0,
                "state": {"count": 15},
            }
        ],
    },
)
result.success   # True — 10 + 5 == 15
```

Change the output state to `{"count": 14}` and the run fails, with the
report showing `prev_state = {count: 10}` and `amount = 5` — the
mismatch is immediately visible. This is the fast inner loop for
covenant development: iterate here in milliseconds, then take the
contract on-chain (see the worked example in [Covenants](covenants.md)).

## Simulation, not validation

`debug_call` executes a *synthetic* transaction so you can debug the
contract logic itself. It says nothing about fees, mass, maturity, or
the exact transaction you will broadcast — the network validates the
real transaction when you submit it. `debug_call` answers *why* a
contract call fails, in source terms; it does not answer *will the
network accept this transaction*.

## What this page didn't cover

- Writing and compiling contracts: [Compiling Contracts](compiling.md).
- Building the unlocking script for a real spend:
  [Unlocking Scripts](unlocking-scripts.md).
- Covenant declarations and the on-chain transition flow:
  [Covenants](covenants.md).
