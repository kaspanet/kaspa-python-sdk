# Multi-signature transactions

You have N cosigners and want a 2-of-3 (or M-of-N) wallet. Each cosigner
holds their own xprv; spending requires M signatures.

This is a how-to. For the underlying derivation and transaction
generation primitives, see
[Wallet SDK → Derivation](../learn/wallet-sdk/derivation.md) and
[Wallet SDK → Transaction Generator](../learn/wallet-sdk/tx-generator.md).

## Build a 2-of-3 multisig address

```python
from kaspa import (
    create_multisig_address, NetworkType, PublicKey,
    PrivateKeyGenerator,
)

# Each cosigner derives the same account-level public key, with their
# own cosigner_index. Public keys (not private!) are exchanged.
gen0 = PrivateKeyGenerator(xprv=cosigner_0_xprv, is_multisig=True,
                            account_index=0, cosigner_index=0)
gen1 = PrivateKeyGenerator(xprv=cosigner_1_xprv, is_multisig=True,
                            account_index=0, cosigner_index=1)
gen2 = PrivateKeyGenerator(xprv=cosigner_2_xprv, is_multisig=True,
                            account_index=0, cosigner_index=2)

pubkeys = [
    PublicKey(gen0.receive_key(0).to_public_key().to_string()),
    PublicKey(gen1.receive_key(0).to_public_key().to_string()),
    PublicKey(gen2.receive_key(0).to_public_key().to_string()),
]

multisig = create_multisig_address(
    minimum_signatures=2,
    keys=pubkeys,
    network_type=NetworkType.Mainnet,
)
print(multisig.to_string())
```

Send funds to `multisig.to_string()` and you've created a 2-of-3 UTXO
set.

## Spend from the multisig

```python
from kaspa import Generator, NetworkId, PaymentOutput

gen = Generator(
    network_id=NetworkId("mainnet"),
    entries=multisig_utxos,           # UTXOs paying to multisig.address
    change_address=multisig,           # change goes back to the multisig
    outputs=[PaymentOutput(recipient, amount)],
    minimum_signatures=2,              # accurate mass calculation
)

for pending in gen:
    # Two of the three cosigners' keys
    pending.sign([cosigner_0_key, cosigner_1_key])
    tx_id = await pending.submit(client)
```

`minimum_signatures=2` matters: the `Generator` budgets mass for the
expected number of signatures. Skip it and the resulting transaction is
under-massed and the node will reject it.

## Coordinating signatures across machines

For a real multisig, the cosigners aren't co-resident. The wire-format
hand-off is:

1. **Coordinator** runs `gen` and collects each `pending.transaction` (or
   the dict form `pending.transaction.to_dict()`).
2. **Each signer** receives the unsigned transaction, signs only their
   inputs with `pending.create_input_signature(...)`, and ships the
   signature back.
3. **Coordinator** assembles the signatures into the
   `signature_script` for each input via `pending.fill_input(i, ...)`,
   then submits.

See `pending.create_input_signature(input_index, private_key,
sighash_type=SighashType.All)` and `pending.fill_input(i, script_bytes)` —
both are documented in
[Wallet SDK → Transaction Generator](../learn/wallet-sdk/tx-generator.md).

## Notes

- All cosigners must use the **same `account_index`** — the multisig
  address is a function of the public keys at that level, and any
  mismatch produces a different address.
- `cosigner_index` differentiates the cosigners; it's not a "first /
  second / third signer" ordering, it's a deterministic position used
  during derivation. Pick once and document it alongside the wallet.
- Multisig addresses are `ScriptHash`-version (see
  [Addresses](../learn/addresses.md)).
- For derivation across more than one address index, every cosigner
  derives in lockstep — `gen0.receive_key(i)`, `gen1.receive_key(i)`,
  `gen2.receive_key(i)` for the same `i`.
