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

Each fragment appends the on-chain Groth16 verifier and documents the
stack it expects the sig script to have set up:

- **[`append_r0_groth16_verifier(image_id)`](../../reference/Classes/ZkScriptBuilder.md)**
  — expects `[..., journal_hash, compressed_proof]` on the stack.
- **[`append_r0_groth16_verifier_dynamic_image_id()`](../../reference/Classes/ZkScriptBuilder.md)**
  — the same verifier, but the image id is taken from the stack rather
  than baked into the script: expects
  `[..., journal_hash, compressed_proof, image_id]`.
  `append_r0_groth16_verifier(image_id)` is exactly `add_data(image_id)`
  followed by this.

### Fixed-journal variants

A normal verifier takes the journal (hash) from the spender at unlock
time. The `*_with_fixed_journal` variants **bake it into the script**
instead, so the UTXO can only ever be unlocked by a proof producing that
one specific output — a smaller, more rigid lock:

- **[`append_r0_groth16_verifier_with_fixed_journal(image_id, journal_hash)`](../../reference/Classes/ZkScriptBuilder.md)**
  — expects only `[..., compressed_proof]`.

```python
from kaspa import ZkScriptBuilder

# A redeem script bound to one program AND one specific output.
builder = ZkScriptBuilder.new_r0()
builder.append_r0_groth16_verifier_with_fixed_journal(image_id, journal_hash)
redeem = builder.script()
```

## Proof pushes

The spend-side counterpart decodes a receipt and pushes its proof
material onto the builder — typically while assembling a sig script by
hand:

- **[`push_r0_groth16_proof(receipt)`](../../reference/Classes/ZkScriptBuilder.md)**
  — push the compressed proof bytes. The script layout is responsible
  for placing the journal hash under it.

## Standalone decoder

When you want the proof material as a plain value — no builder — the
module-level function decodes a receipt directly:

```python
from kaspa import prepare_r0_groth16_proof

proof = prepare_r0_groth16_proof(receipt)      # compressed proof bytes, hex
```

[`prepare_r0_groth16_proof`](../../reference/Functions/prepare_r0_groth16_proof.md)
returns exactly the bytes `finalize_with_groth16_proof` pushes into the
sig script.

All of these raise
[`ZkError`](../../reference/Exceptions/ZkError.md) on a malformed id or
an undecodable receipt.
