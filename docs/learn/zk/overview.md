---
search:
  boost: 5
---

# ZK Scripts

Kaspa can verify a [RISC Zero](https://risczero.com/) zero-knowledge
proof natively on-chain, via the `OpZkPrecompile` script opcode. The SDK's
[`ZkScriptBuilder`](../../reference/Classes/ZkScriptBuilder.md) turns a
RISC Zero **receipt** (the proof artifact) into the two scripts of a
P2SH lock: a **redeem script** that only unlocks when the node verifies
a proof for a fixed program, and a **sig script** that carries the proof
which satisfies it.

The statement the chain checks is exactly:

> "Program `image_id` ran and produced public output whose hash is
> `journal_hash`."

Anyone holding a valid proof for that pair can spend the UTXO — the
proof *is* the authorization, no signature involved. The SDK does not
generate proofs; you bring a receipt from a RISC Zero prover, and the
bindings transcribe it into script bytes. Verification itself happens on
the node.

## Four RISC Zero terms

| Term | What it is | Size |
| --- | --- | --- |
| **image id** | A digest of the guest program's compiled binary — *which program ran*. | 32 bytes |
| **journal** | The program's public output — what it chose to reveal. | varies |
| **journal hash** | A digest of the journal — pins *what the program output*. | 32 bytes |
| **receipt** | The proof plus the journal; the artifact you verify. | see below |

## Groth16 vs succinct receipts

RISC Zero hands you the proof in one of two forms, and the builder has a
parallel path for each:

- **Groth16 receipt** — the STARK proof compressed into a Groth16
  SNARK. A few hundred bytes, constant-size, cheap to verify. This is
  the practical on-chain choice; it relies on RISC Zero's public
  trusted-setup ceremony.
- **Succinct receipt** — the STARK proof directly. No trusted setup,
  but hundreds of kilobytes, and verifying it needs extra material
  (a `control_id` identifying the recursion circuit, and a
  `hash_fn_id` — currently only `"poseidon2"`).

## The surface

- **Build the lock and the spend**
  ([`ZkScriptBuilder`](../../reference/Classes/ZkScriptBuilder.md) →
  [`FinalizedR0Script`](../../reference/Classes/FinalizedR0Script.md)) —
  a staged builder: commit to a program, then finalize with a proof.
  See [Locking & Redeeming](locking-and-redeeming.md).
- **Compose scripts by hand**
  (the `append_*` / `push_*` fragment methods,
  [`prepare_r0_groth16_proof`](../../reference/Functions/prepare_r0_groth16_proof.md),
  [`prepare_r0_succinct_witness`](../../reference/Functions/prepare_r0_succinct_witness.md)) —
  lower-level pieces for embedding a verifier inside a larger script.
  See [Fragments & Helpers](fragments.md).

Errors — a call in the wrong builder state, a malformed id, an
undecodable receipt — raise
[`ZkError`](../../reference/Exceptions/ZkError.md) from
`kaspa.exceptions`.

Everything else (wrapping the redeem script in a P2SH address, building
and submitting the transactions) is ordinary `kaspa` work — see
[Transactions](../transactions/overview.md) and
[Scripts](../transactions/scripts.md).

## A minimal lock

```python
from kaspa import ZkScriptBuilder, address_from_script_public_key, pay_to_script_hash_script

# From your prover: the guest program's 32-byte image id (hex).
IMAGE_ID = "75641a540ee2ad9ee5902bcdcdb8b55c0bef4a28287309b858f97b1356c6c2e0"

builder = ZkScriptBuilder.new_r0(covenants_enabled=True)
builder.commit_to_groth16(IMAGE_ID)

# The redeem script, wrapped in a P2SH lock and turned into a fundable address.
spk = pay_to_script_hash_script(builder.script())
addr = address_from_script_public_key(spk, "testnet")
```

Send funds to `addr` and they're locked behind the proof. To spend them,
you finalize the builder with a receipt — that's
[Locking & Redeeming](locking-and-redeeming.md).

## A full, live example

[`examples/zk/groth16_onchain.py`](https://github.com/kaspanet/kaspa-python-sdk/blob/main/examples/zk/groth16_onchain.py)
runs the whole lifecycle end-to-end on testnet-10: it builds the zk
scripts from a vendored Groth16 proof set, locks a funding UTXO into the
P2SH address (commit), then spends it back by presenting the proof
(redeem) — each a real on-chain transaction, with the redeem verified by
`OpZkPrecompile`.

!!! danger "Secrets handling"
    Snippets here use literal values for readability. Never handle
    production secrets this way — see
    [Security](../../getting-started/security.md).

## Next

| Page | What it covers |
| --- | --- |
| [Locking & Redeeming](locking-and-redeeming.md) | The builder state machine, `commit_to_*` / `finalize_with_*`, [`FinalizedR0Script`](../../reference/Classes/FinalizedR0Script.md), and the on-chain spend. |
| [Fragments & Helpers](fragments.md) | The `append_*` / `push_*` fragment methods, fixed-journal verifiers, and the standalone `prepare_*` decoders. |
