---
search:
  boost: 3
---

# Scripts

A [`ScriptPublicKey`](../../reference/Classes/ScriptPublicKey.md) is the lock on every output. Most of the time it's
generated for you — [`pay_to_address_script`](../../reference/Functions/pay_to_address_script.md) builds the standard
pay-to-pubkey lock, and both the
[`Generator`](../../reference/Classes/Generator.md) (see [Transaction Generator](../wallet-sdk/tx-generator.md)) and the high-level
[`Wallet`](../../reference/Classes/Wallet.md) (see [Wallet](../wallet/overview.md)) API use it under the hood. This page
is for the cases where you need a non-standard lock: multisig redeem
scripts, KRC‑20 / inscription envelopes, time-locked spends, covenant
prototypes.

The fluent builder is
[`ScriptBuilder`](../../reference/Classes/ScriptBuilder.md); the
opcode set lives in
[`Opcodes`](../../reference/Enums/Opcodes.md).

## When you'd reach for this

- **Multisig.** Building the redeem script behind an `M`-of-`N`
  pay-to-script-hash address. Worked example:
  [`examples/transactions/multisig.py`](https://github.com/kaspanet/kaspa-python-sdk/blob/main/examples/transactions/multisig.py).
- **KRC‑20 / inscription envelopes.** Embedding token-protocol JSON
  in a commit-reveal script. Worked example:
  [`examples/transactions/krc20_deploy.py`](https://github.com/kaspanet/kaspa-python-sdk/blob/main/examples/transactions/krc20_deploy.py).
- **Custom locking conditions.** Time-locked or hash-locked spends
  and covenant prototypes. Advanced — you should already know what
  opcodes you need.

## Building a script

[`ScriptBuilder`](../../reference/Classes/ScriptBuilder.md) is a chained-builder: every method returns the
builder so you can compose a script in one expression.

```python
from kaspa import Opcodes, ScriptBuilder

script = ScriptBuilder()\
    .add_data(pubkey_xonly_hex)\
    .add_op(Opcodes.OpCheckSig)

print(script.to_string())   # hex of the assembled script
```

The four input families:

- **[`add_op(op)`](../../reference/Classes/ScriptBuilder.md) / [`add_ops([op, ...])`](../../reference/Classes/ScriptBuilder.md)** — push a single opcode or a
  sequence. `op` is an [`Opcodes`](../../reference/Enums/Opcodes.md) enum member or its integer value.
- **[`add_data(bytes_or_hex)`](../../reference/Classes/ScriptBuilder.md)** — push raw bytes (signatures, public
  keys, payloads). The builder picks the right `OP_PUSHDATA*` variant
  for the size automatically. Accepts a hex string, `bytes`, or a
  list of ints.
- **[`add_i64(n)`](../../reference/Classes/ScriptBuilder.md)** — push a signed integer. Used for `M` and `N` in
  multisig, and for sequence / lock-time scalars.
- **[`add_lock_time(daa_score)`](../../reference/Classes/ScriptBuilder.md) / [`add_sequence(seq)`](../../reference/Classes/ScriptBuilder.md)** — push
  DAA-score and sequence values, for time-locked spends.

For the full opcode catalog see the
[`Opcodes`](../../reference/Enums/Opcodes.md) reference.

## Wrapping in P2SH

The output side commits to the *hash* of the redeem script, not the
script itself. [`create_pay_to_script_hash_script`](../../reference/Classes/ScriptBuilder.md) produces the
locking
[`ScriptPublicKey`](../../reference/Classes/ScriptPublicKey.md);
[`pay_to_script_hash_script`](../../reference/Functions/pay_to_script_hash_script.md)
is the equivalent free function when you have the redeem-script bytes
directly:

```python
from kaspa import address_from_script_public_key

spk = script.create_pay_to_script_hash_script()
p2sh_address = address_from_script_public_key(spk, "testnet-10")
```

Place `spk` on a [`TransactionOutput`](../../reference/Classes/TransactionOutput.md), or share `p2sh_address` so
funds can be sent to it. See [`address_from_script_public_key`](../../reference/Functions/address_from_script_public_key.md) for the network argument.

When *spending* a P2SH output, the input's `signature_script` must
reveal the redeem script *and* a satisfying signature. For the
single-signature case, the convenience helper writes the witness for
you ([`create_input_signature`](../../reference/Classes/PendingTransaction.md), [`encode_pay_to_script_hash_signature_script`](../../reference/Classes/ScriptBuilder.md), [`fill_input`](../../reference/Classes/PendingTransaction.md)):

```python
sig = pending.create_input_signature(input_index=0, private_key=key)
witness = script.encode_pay_to_script_hash_signature_script(sig)
pending.fill_input(0, witness)
```

[`pay_to_script_hash_signature_script(redeem_script, signature)`](../../reference/Functions/pay_to_script_hash_signature_script.md)
is the equivalent functional form when you no longer have the original
builder in scope. For multisig — where the witness needs more than
one signature — you build the `signature_script` manually with
[`ScriptBuilder`](../../reference/Classes/ScriptBuilder.md); see the example linked below.

## Two real shapes

### Multisig redeem (M-of-N)

[`create_multisig_address`](../../reference/Functions/create_multisig_address.md) produces the same lockup as a one-shot helper; this section
shows what's happening underneath:

```python
redeem = ScriptBuilder()\
    .add_i64(2)\
    .add_data(pub_a.to_x_only_public_key().to_string())\
    .add_data(pub_b.to_x_only_public_key().to_string())\
    .add_data(pub_c.to_x_only_public_key().to_string())\
    .add_i64(3)\
    .add_op(Opcodes.OpCheckMultiSig)

spk = redeem.create_pay_to_script_hash_script()
```

This is a 2-of-3 Schnorr multisig: integer `M`, the public keys
([`XOnlyPublicKey`](../../reference/Classes/XOnlyPublicKey.md) for Schnorr), integer `N`, then [`Opcodes.OpCheckMultiSig`](../../reference/Enums/Opcodes.md). ECDSA
multisig uses [`Opcodes.OpCheckMultiSigECDSA`](../../reference/Enums/Opcodes.md) and full (compressed) [`PublicKey`](../../reference/Classes/PublicKey.md)s
instead. The mass calculator needs to know about the multiple
signatures the input will eventually hold — pass `sig_op_count=N` per
input and `minimum_signatures=M` to the
[`Generator`](../../reference/Classes/Generator.md) (see [Transaction Generator](../wallet-sdk/tx-generator.md)). See
[Signing → Multisig](signing.md#multisig-and-sig_op_count) for how
those fields feed mass.

The spending side (collecting `M` signatures and packing them into
each input's `signature_script`) is non-trivial. The full flow is in
[`examples/transactions/multisig.py`](https://github.com/kaspanet/kaspa-python-sdk/blob/main/examples/transactions/multisig.py).

### KRC‑20 / inscription envelope

```python
import json

script = ScriptBuilder()\
    .add_data(pub.to_x_only_public_key().to_string())\
    .add_op(Opcodes.OpCheckSig)\
    .add_op(Opcodes.OpFalse)\
    .add_op(Opcodes.OpIf)\
    .add_data(b"kasplex")\
    .add_i64(0)\
    .add_data(json.dumps(payload, separators=(",", ":")).encode())\
    .add_op(Opcodes.OpEndIf)
```

The `OpFalse` / `OpIf` block is unreachable execution — a
conventional way to embed protocol data without affecting whether
the script can be satisfied. The reveal-stage input later spends a
P2SH output committing to this script, putting the embedded payload
on-chain.

The two-stage commit/reveal flow itself (fund the P2SH address in a
commit transaction, then spend it back to yourself in a reveal
transaction once the commit is mature) lives in
[`examples/transactions/krc20_deploy.py`](https://github.com/kaspanet/kaspa-python-sdk/blob/main/examples/transactions/krc20_deploy.py).
The managed [`Wallet`](../../reference/Classes/Wallet.md) wraps this as
[`accounts_commit_reveal`](../../reference/Classes/Wallet.md) /
[`accounts_commit_reveal_manual`](../../reference/Classes/Wallet.md), keyed by a
[`CommitRevealAddressKind`](../../reference/Enums/CommitRevealAddressKind.md).

## Inspecting an unknown script

When you hold a [`ScriptPublicKey`](../../reference/Classes/ScriptPublicKey.md) from somewhere else — a UTXO
returned by [`get_utxos_by_addresses`](../../reference/Classes/RpcClient.md#kaspa.RpcClient.get_utxos_by_addresses), an output read off-chain — the
classification predicates
([`is_script_pay_to_pubkey`](../../reference/Functions/is_script_pay_to_pubkey.md),
[`is_script_pay_to_pubkey_ecdsa`](../../reference/Functions/is_script_pay_to_pubkey_ecdsa.md),
[`is_script_pay_to_script_hash`](../../reference/Functions/is_script_pay_to_script_hash.md))
tell you which lockup family it is:

```python
from kaspa import (
    is_script_pay_to_pubkey,
    is_script_pay_to_pubkey_ecdsa,
    is_script_pay_to_script_hash,
)

script_bytes = utxo.script_public_key.script
if is_script_pay_to_pubkey(script_bytes):
    ...        # Schnorr P2PK
elif is_script_pay_to_pubkey_ecdsa(script_bytes):
    ...        # ECDSA P2PK
elif is_script_pay_to_script_hash(script_bytes):
    ...        # P2SH — needs the redeem script to spend
```

Use them to pick the signing path, filter UTXOs the wallet can
actually spend, or audit a transaction's outputs.

## SighashType and advanced flows

Scripts and sighash variants interact when you write protocols that
intentionally let signed transactions be amended.
[`SighashType.All`](../../reference/Enums/SighashType.md) — the
default — commits to every input and every output and is the only
one ordinary scripts should use. The `_None`, `Single`, and
`*AnyOneCanPay` variants exist for coinjoins and partial co-signing
flows; don't reach for them without a spec to follow. See
[Signing → SighashType](signing.md#sighashtype).

## What this page didn't cover

- The full multisig orchestration (deriving cosigner keys, exchanging
  partial signatures, submission):
  [`examples/transactions/multisig.py`](https://github.com/kaspanet/kaspa-python-sdk/blob/main/examples/transactions/multisig.py).
- Commit-reveal mechanics (two-stage submission, fee budget, the
  maturity gate between commit and reveal):
  [`examples/transactions/krc20_deploy.py`](https://github.com/kaspanet/kaspa-python-sdk/blob/main/examples/transactions/krc20_deploy.py).
- The full opcode catalog:
  [`Opcodes`](../../reference/Enums/Opcodes.md) reference.
