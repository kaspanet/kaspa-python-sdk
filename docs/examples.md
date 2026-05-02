---
search:
  boost: 3
---

# Examples

Runnable examples live in the SDK repository, not in these docs. Each
script is self-contained and demonstrates one feature end-to-end.

[Browse examples on GitHub →](https://github.com/kaspanet/kaspa-python-sdk/tree/main/examples)

## What's there

- **[`rpc/`](https://github.com/kaspanet/kaspa-python-sdk/tree/main/examples/rpc)**
  — connecting via the resolver, calling RPC methods, subscribing to
  notifications.
- **[`wallet/`](https://github.com/kaspanet/kaspa-python-sdk/tree/main/examples/wallet)**
  — managed `Wallet` lifecycle: creating, opening, accounts, sending,
  export/import.
- **[`transactions/`](https://github.com/kaspanet/kaspa-python-sdk/tree/main/examples/transactions)**
  — building transactions with the `Generator`, `UtxoContext`,
  multisig, and KRC-20 deploys.
- **[`derivation.py`](https://github.com/kaspanet/kaspa-python-sdk/blob/main/examples/derivation.py)**
  — BIP-32 / BIP-44 key derivation.
- **[`mnemonic.py`](https://github.com/kaspanet/kaspa-python-sdk/blob/main/examples/mnemonic.py)**
  — generating and restoring mnemonics.
- **[`message_signing.py`](https://github.com/kaspanet/kaspa-python-sdk/blob/main/examples/message_signing.py)**
  — signing and verifying messages with a private key.
- **[`addresses.py`](https://github.com/kaspanet/kaspa-python-sdk/blob/main/examples/addresses.py)**
  — encoding, parsing, and validating Kaspa addresses.

## Running them

Clone the repo, install the SDK, and run any script directly:

```bash
git clone https://github.com/kaspanet/kaspa-python-sdk
cd kaspa-python-sdk
pip install kaspa
python examples/rpc/all_calls.py
```

See [Installation](getting-started/installation.md) for build-from-source
instructions.
