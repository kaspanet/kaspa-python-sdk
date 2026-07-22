---
search:
  boost: 3
---

# Locking & Redeeming

[`ZkScriptBuilder`](../../reference/Classes/ZkScriptBuilder.md) is a
**staged builder**: the redeem script must be bound to a program before
a proof can be supplied, and neither step can happen twice. The native
Rust SDK enforces that order at compile time; the Python binding mirrors
it at runtime — a call in the wrong state raises
[`ZkError`](../../reference/Exceptions/ZkError.md) instead of producing
a broken script.

```
              commit_to_groth16(image_id)
  unbounded ─────────────────────────────────► groth16-bounded
      │                                              │
      │  commit_to_succinct(                         │ finalize_with_groth16_proof(
      │     image_id, control_id, hash_fn_id?)       │     receipt, journal_hash)
      ▼                                              ▼
  succinct-bounded ──────────────────────────► FinalizedR0Script
              finalize_with_succinct_proof(     (builder is consumed)
                  receipt, journal)
```

You must finalize with the *matching* proof form: a Groth16 finalize on
a Groth16-bounded builder, a succinct finalize on a succinct-bounded
one. `repr(builder)` shows the current state.

Byte arguments — image ids, journal hashes, receipts — accept a hex
string, `bytes`, or a list of ints throughout.

## Creating a builder

[`ZkScriptBuilder.new_r0`](../../reference/Classes/ZkScriptBuilder.md)
is a static factory (there is no constructor — matching the WASM SDK, so
future non-RISC-Zero backends can add their own factories):

```python
from kaspa import ZkScriptBuilder

builder = ZkScriptBuilder.new_r0(covenants_enabled=True)
```

- **`covenants_enabled`** (default `True`) selects the post-Toccata
  script limits. The sig script's pushes — the redeem script itself,
  and for succinct proofs the witness — exceed the pre-Toccata 520-byte
  element limit, so finalizing always fails when this is `False`; only
  pass `False` to build fragments under pre-Toccata rules.
- **`sigop_script_units`** overrides the script units charged per
  signature operation; omit it for the engine default.

## Committing: the lock side

Committing binds the builder to a specific program and writes the
verifier into the redeem script:

```python
builder.commit_to_groth16(image_id)                       # Groth16
builder.commit_to_succinct(image_id, control_id)          # succinct
```

The succinct form also takes a `hash_fn_id` (currently only
`"poseidon2"`, which is the default when omitted). After a commit,
[`script()`](../../reference/Classes/ZkScriptBuilder.md) returns the
redeem script (hex) — hash it into the P2SH lock the usual way (see
[Scripts → Wrapping in P2SH](../transactions/scripts.md#wrapping-in-p2sh)):

```python
from kaspa import address_from_script_public_key, pay_to_script_hash_script

spk = pay_to_script_hash_script(builder.script())
p2sh_address = address_from_script_public_key(spk, "testnet")
```

Funds sent to `p2sh_address` are now locked behind the proof.

## Finalizing: the spend side

Finalizing decodes the receipt, assembles the spending script, and
consumes the builder:

```python
finalized = builder.finalize_with_groth16_proof(receipt, journal_hash)
finalized.redeem_script   # the commit script — same bytes script() returned
finalized.sig_script      # the proof-bearing spending script
```

[`FinalizedR0Script`](../../reference/Classes/FinalizedR0Script.md)
holds both scripts, hex-encoded. `sig_script` embeds the journal hash,
the compressed proof, and the redeem script itself — it goes on the
spending input's `signature_script` verbatim.

The succinct twin is
[`finalize_with_succinct_proof(receipt, journal)`](../../reference/Classes/ZkScriptBuilder.md);
its `journal` argument is likewise the 32-byte journal digest.

Two properties worth knowing:

- **Finalize consumes the builder.** After a successful finalize (or a
  [`drain()`](../../reference/Classes/ZkScriptBuilder.md)), `script()`
  returns `""` and further mutating calls raise
  [`ZkError`](../../reference/Exceptions/ZkError.md).
- **A failed finalize does not.** If the receipt won't decode or a push
  exceeds the script limits, the builder survives with its state and
  committed script intact — fix the input and finalize again.

## Spending the locked UTXO

The redeem transaction differs from an ordinary P2SH spend in three
ways, all visible in
[`examples/zk/groth16_onchain.py`](https://github.com/kaspanet/kaspa-python-sdk/blob/main/examples/zk/groth16_onchain.py):

```python
from kaspa import TransactionInput, TransactionOutpoint, Hash

spend = TransactionInput(
    TransactionOutpoint(Hash(commit_txid), 0),
    finalized.sig_script,     # the proof is the authorization
    sequence=0,
    sig_op_count=0,
    compute_budget=1600,      # covers the Groth16 precompile
    utxo=p2sh_utxo,
)
```

- **No signature.** The proof alone satisfies the lock, so the redeem
  input carries no Schnorr signature and the transaction is submitted
  unsigned. That makes the redeem **permissionless**: anyone holding a
  valid proof for the `(image_id, journal_hash)` pair can spend the
  UTXO.
- **Compute budget.** Proof verification is metered: a Groth16
  verification is priced at 140,000 grams (1,400 budget units) and a
  succinct one at 250,000 grams (2,500 units), plus the surrounding
  script opcodes — so the redeem input needs a `compute_budget` with
  headroom (the example uses 1600 for Groth16). Too low and the redeem
  fails.
- **Activation.** `OpZkPrecompile` only runs where Toccata is active —
  pre-check the virtual DAA score against the activation score before
  submitting.

Everything else — sizing the fee from
[`calculate_transaction_mass`](../../reference/Functions/calculate_transaction_mass.md),
building the [`Transaction`](../../reference/Classes/Transaction.md),
submission — is ordinary transaction plumbing; see
[Transactions](../transactions/overview.md).

Next: composing scripts by hand from the verifier and proof pieces —
[Fragments & Helpers](fragments.md).
