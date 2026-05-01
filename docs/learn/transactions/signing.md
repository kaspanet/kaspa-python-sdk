# Signing

Signing fills each input's `signature_script` with proof that the
spender controls the key the UTXO is locked to. Kaspa defaults to
**Schnorr** signatures over secp256k1; ECDSA is supported for inputs
locked to ECDSA addresses. Multisig inputs combine multiple
signatures under a script-hash lockup.

For Schnorr vs ECDSA on the addressing side, see
[Addresses → Versions](../addresses.md#versions).

## Sign a manually built transaction

```python
from kaspa import sign_transaction, update_transaction_mass

update_transaction_mass("mainnet", tx)        # do this first — mass is signed over
signed = sign_transaction(tx, [private_key], verify_sig=True)
```

[`sign_transaction(tx, signers, verify_sig)`](../../reference/Functions/sign_transaction.md):

- `signers` — a list of
  [`PrivateKey`](../../reference/Classes/PrivateKey.md). The signer
  for each input is inferred from the input's UTXO lockup; pass every
  key any input needs.
- `verify_sig=True` — verify each signature after writing it. Cheap
  insurance during development; disable in performance-sensitive
  paths once you trust the inputs.

Sign after mass is filled in, before submission. Mass is part of the
signed payload, so changing inputs, outputs, or mass *after* signing
invalidates the signature.

## Sign a generator-produced PendingTransaction

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
for i, _ in enumerate(pending.get_utxo_entries()):
    pending.sign_input(i, key_for(i))
```

`sign_input` is the right surface when different inputs need different
signers (mixed-key wallets, partially-co-signed flows).

## SighashType

The hash that gets signed describes *which parts of the transaction*
the signature commits to.
[`SighashType.All`](../../reference/Enums/SighashType.md) is the
default and the only one most code should use.

```python
from kaspa import SighashType

print(list(SighashType))
# All, _None, Single, AllAnyOneCanPay, NoneAnyOneCanPay, SingleAnyOneCanPay
```

- **`All`** — signs every input and every output. Standard.
- **`_None`** — signs inputs only; outputs can be modified. Rare;
  underscore-prefixed because `None` is a Python keyword.
- **`Single`** — signs the input being spent and the matching output
  by index.
- **`*AnyOneCanPay`** — variants that *don't* sign the other inputs,
  letting cosigners add inputs after the fact.

Leave it at `All` unless you have a specific protocol or co-signing
reason. The non-`All` modes are for advanced flows like collaborative
coin joins.

## Build a signature without filling the input

When you need raw signature bytes — e.g. to send to a co-signer for
aggregation — use `create_input_signature`:

```python
from kaspa import SighashType, create_input_signature

sig_hex = create_input_signature(
    tx,
    input_index=0,
    private_key=key,
    sighash_type=SighashType.All,
)
```

The same method exists on `PendingTransaction`
(`pending.create_input_signature(...)`); write the resulting script
back with `pending.fill_input(...)`.

## Multisig and sig_op_count

Two fields interact with mass when you sign:

- **`sig_op_count`** on each input — number of signature ops the
  input actually performs. `1` for a single-key spend, `M` for an
  `M`-of-`N` multisig.
- **`minimum_signatures`** passed to
  `update_transaction_mass(..., minimum_signatures=M)` and
  `calculate_transaction_mass` — tells the mass calculator how big
  the filled-in signature script will be.

Wrong values yield wrong mass and either rejected (too low) or wasted
(too high) fees. The Generator handles this when you pass
`sig_op_count` and `minimum_signatures` to the constructor.

For the full multisig flow (address creation, multi-cosigner signing,
submission), see
[Multi-signature transactions](../../guides/multisig.md).

## Where to next

- [Submission](submission.md) — what to do with a signed transaction.
- [Mass & fees](mass-and-fees.md) — fill mass before signing, not after.
- [Multi-signature transactions](../../guides/multisig.md) — cosigner
  flow end to end.
