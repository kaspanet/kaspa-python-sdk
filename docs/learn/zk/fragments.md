---
search:
  boost: 3
---

# Fragments & Helpers

The `commit_to_*` / `finalize_with_*` flow in
[Locking & Redeeming](locking-and-redeeming.md) covers the standard
lock. The builder also exposes the pieces it is made of — verifier
**fragments** and proof **pushes** — for composing a script by hand:
embedding the verifier inside a larger redeem script, or baking the
expected journal into the lock itself. These methods work in any
builder state (except consumed) and don't advance the state machine.

## Pushing data

- **[`add_data(data)`](../../reference/Classes/ZkScriptBuilder.md)** —
  push raw bytes with canonical encoding, e.g. the caller-owned journal
  or a redeem script. Accepts hex, `bytes`, or a list of ints.
- **[`script()`](../../reference/Classes/ZkScriptBuilder.md)** — the
  bytes built so far (hex), without consuming.
- **[`drain()`](../../reference/Classes/ZkScriptBuilder.md)** — the same
  bytes, but the builder is consumed. Matching the WASM SDK — and unlike
  [`ScriptBuilder.drain`](../../reference/Classes/ScriptBuilder.md) —
  the builder is not reusable afterwards: subsequent mutating calls
  raise [`ZkError`](../../reference/Exceptions/ZkError.md).

## Verifier fragments

Each fragment appends the on-chain verifier for one proof form and
documents the stack it expects the sig script to have set up:

- **[`append_r0_groth16_verifier(image_id)`](../../reference/Classes/ZkScriptBuilder.md)**
  — expects `[..., journal_hash, compressed_proof]` on the stack.
- **[`append_r0_succinct_verifier(image_id, control_id, hash_fn_id=None)`](../../reference/Classes/ZkScriptBuilder.md)**
  — expects `[..., claim, control_index, control_digests, seal, journal]`.

### Fixed-journal variants

A normal verifier takes the journal (hash) from the spender at unlock
time. The `*_with_fixed_journal` variants **bake it into the script**
instead, so the UTXO can only ever be unlocked by a proof producing that
one specific output — a smaller, more rigid lock:

- **[`append_r0_groth16_verifier_with_fixed_journal(image_id, journal_hash)`](../../reference/Classes/ZkScriptBuilder.md)**
  — expects only `[..., compressed_proof]`.
- **[`append_r0_succinct_verifier_with_fixed_journal(image_id, control_id, hash_fn_id=None, *, journal)`](../../reference/Classes/ZkScriptBuilder.md)**
  — expects only `[..., claim, control_index, control_digests, seal]`.

```python
from kaspa import ZkScriptBuilder

# A redeem script bound to one program AND one specific output.
builder = ZkScriptBuilder.new_r0(covenants_enabled=True)
builder.append_r0_groth16_verifier_with_fixed_journal(image_id, journal_hash)
redeem = builder.script()
```

## Proof pushes

The spend-side counterparts decode a receipt and push its proof
material onto the builder — typically while assembling a sig script by
hand:

- **[`push_r0_groth16_proof(receipt)`](../../reference/Classes/ZkScriptBuilder.md)**
  — push the compressed proof bytes. The script layout is responsible
  for placing the journal hash under it.
- **[`push_r0_succinct_witness(receipt)`](../../reference/Classes/ZkScriptBuilder.md)**
  — push the witness items (claim, control index, control digests,
  seal); push the caller-owned journal afterwards, on top.

## Standalone decoders

When you want the proof material as plain values — no builder — the
module-level functions decode a receipt directly:

```python
from kaspa import prepare_r0_groth16_proof, prepare_r0_succinct_witness

proof = prepare_r0_groth16_proof(receipt)      # compressed proof bytes, hex

parts = prepare_r0_succinct_witness(receipt)   # R0SuccinctWitnessParts
parts.claim             # SHA-256 digest of the receipt claim
parts.control_index     # inclusion-proof leaf index (u32 LE)
parts.control_digests   # flattened sibling digests
parts.seal              # the flattened STARK seal
```

[`prepare_r0_groth16_proof`](../../reference/Functions/prepare_r0_groth16_proof.md)
returns exactly the bytes `finalize_with_groth16_proof` pushes into the
sig script;
[`prepare_r0_succinct_witness`](../../reference/Functions/prepare_r0_succinct_witness.md)
returns an
[`R0SuccinctWitnessParts`](../../reference/Classes/R0SuccinctWitnessParts.md)
with the four witness fields, hex-encoded, in the order the verifier
consumes them (the journal is caller-owned and excluded).

All of these raise
[`ZkError`](../../reference/Exceptions/ZkError.md) on a malformed id, an
undecodable receipt, or an unsupported `hash_fn_id`.
