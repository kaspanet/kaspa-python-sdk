---
search:
  boost: 3
---

# Signing

Signing fills each input's `signature_script` with proof that the
spender controls the key the UTXO is locked to. Kaspa defaults to
**Schnorr** signatures over secp256k1; ECDSA is supported for inputs
locked to ECDSA addresses. Multisig inputs combine multiple
signatures under a script-hash lockup.

For Schnorr vs ECDSA on the addressing side, see
[Addresses → Versions](../addresses.md#versions).

## Sign a manually built transaction

Run [`update_transaction_mass`](../../reference/Functions/update_transaction_mass.md) first, then [`sign_transaction`](../../reference/Functions/sign_transaction.md):

```python
from kaspa import sign_transaction, update_transaction_mass

update_transaction_mass("mainnet", tx)        # mass is signed over — fill first
signed = sign_transaction(tx, [private_key], verify_sig=True)
```

[`sign_transaction(tx, signers, verify_sig)`](../../reference/Functions/sign_transaction.md):

- `signers` — a list of
  [`PrivateKey`](../../reference/Classes/PrivateKey.md). The signer
  for each input is inferred from the input's UTXO lockup; pass every
  key any input needs.
- `verify_sig=True` — verify each signature after writing it. Raises
  on mismatch (a corrupt input or wrong key). Cheap insurance during
  development; disable in performance-sensitive paths once you trust
  the inputs.

Changing inputs, outputs, or mass *after* signing invalidates the
signature — sign last.

## Sign a generator-produced PendingTransaction

A [`PendingTransaction`](../../reference/Classes/PendingTransaction.md) yielded by [`Generator`](../../reference/Classes/Generator.md) (see [Transaction Generator](../wallet-sdk/tx-generator.md)) signs in one call:

```python
for pending in gen:
    pending.sign([key])              # all inputs at once
    await pending.submit(client)
```

For multisig, hand in every cosigner's key:

```python
pending.sign([key1, key2, key3])
```

For per-input control:

```python
for i in range(len(pending.get_utxo_entries())):
    pending.sign_input(i, key_for(i))
```

[`sign_input`](../../reference/Classes/PendingTransaction.md) is the right surface when different inputs need different
signers (mixed-key wallets, partially-co-signed flows).

## SighashType

The hash that gets signed describes *which parts of the transaction*
the signature commits to.
[`SighashType.All`](../../reference/Enums/SighashType.md) is the
default and the only one most code should use — it signs every input
and every output.

The other variants (`_None`, `Single`, and the `*AnyOneCanPay`
flavors) exist for advanced collaborative flows like coinjoins,
where cosigners add inputs or outputs after one party signs. If
you don't have a specific protocol reason, leave it at `All`.

## Build a signature without filling the input

When you need raw signature bytes — e.g. to send to a co-signer for
aggregation — use [`create_input_signature`](../../reference/Functions/create_input_signature.md):

```python
from kaspa import SighashType, create_input_signature

sig_hex = create_input_signature(
    tx,
    input_index=0,
    private_key=key,
    sighash_type=SighashType.All,
)
```

The same method exists on [`PendingTransaction`](../../reference/Classes/PendingTransaction.md)
([`pending.create_input_signature(...)`](../../reference/Classes/PendingTransaction.md)); write the resulting script
back with [`pending.fill_input(...)`](../../reference/Classes/PendingTransaction.md).

## Compute a sighash without signing

When the key isn't available in-process — an external or hardware
signer, multisig digest distribution, or verifying a signature
against the exact digest the node checks — use
[`compute_sighash`](../../reference/Functions/compute_sighash.md) to
get the signature hash for an input without signing it:

```python
from kaspa import compute_sighash, sign_script_hash

digest = compute_sighash(tx, input_index=0)   # Hash, SighashType.All
sig_hex = sign_script_hash(digest.to_hex(), private_key)
```

Every input must carry its UTXO entry — the sighash commits to each
input's script public key and amount. Schnorr is the default; pass
`ecdsa=True` for inputs locked to ECDSA addresses (an extra hash
round over the Schnorr digest). The blob from
[`sign_script_hash`](../../reference/Functions/sign_script_hash.md)
is a complete signature script for a single-key (P2PK) input; for
script-hash lockups, wrap it with
`pay_to_script_hash_signature_script`.

## Multisig and sig_op_count

Two fields interact with mass when you sign:

- **`sig_op_count`** on each input — number of signature ops the
  input actually performs. `1` for a single-key spend, `M` for an
  `M`-of-`N` multisig.
- **`minimum_signatures`** passed to
  [`update_transaction_mass(..., minimum_signatures=M)`](../../reference/Functions/update_transaction_mass.md) and
  [`calculate_transaction_mass`](../../reference/Functions/calculate_transaction_mass.md) — tells the mass calculator how big
  the filled-in signature script will be.

Wrong values yield wrong mass — the transaction is rejected (mass
too low) or pays more fee than it needs (mass too high). The
[`Generator`](../../reference/Classes/Generator.md) handles this when you pass `sig_op_count` and
`minimum_signatures` to the constructor.

For the full multisig flow, see
[`examples/transactions/multisig.py`](https://github.com/kaspanet/kaspa-python-sdk/blob/main/examples/transactions/multisig.py).
