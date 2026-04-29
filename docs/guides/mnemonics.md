# Generate or restore a mnemonic

How to produce, validate, and convert BIP-39 mnemonic phrases. For the
teaching version of this material, see
[Wallet SDK → Key Management](../learn/wallet-sdk/key-management.md).

Read [Security](../getting-started/security.md) before generating real
secrets.

## Generate

```python
from kaspa import Mnemonic

# 24 words (default)
m = Mnemonic.random()

# 12 words
m12 = Mnemonic.random(word_count=12)

print(m.phrase)
```

## Restore from a phrase

```python
from kaspa import Mnemonic

phrase = "abandon abandon abandon ... about"

if not Mnemonic.validate(phrase):
    raise ValueError("invalid mnemonic")

m = Mnemonic(phrase)
```

`Mnemonic.validate(phrase)` returns `True` / `False`; it does not raise.
Pass `Language.English` (or another wordlist) as the second argument to
validate against a specific language.

## Convert to a seed

```python
seed = m.to_seed()                            # 64-byte BIP-39 seed
seed_passphrased = m.to_seed("25th-word")     # different seed; same mnemonic
```

The optional passphrase changes the seed — the same mnemonic with
different passphrases produces different wallets. An attacker who
captures the mnemonic alone gets nothing without the passphrase.

## Convert to an `XPrv`

```python
from kaspa import XPrv
xprv = XPrv(seed)
```

From here, derive child keys with
[`PrivateKeyGenerator`](../learn/wallet-sdk/derivation.md) or load the
mnemonic into a managed [Wallet](../learn/wallet/private-keys.md).

## Raw entropy

The entropy is exposed as a hex string — useful when round-tripping
through a tool that emits entropy rather than words:

```python
m = Mnemonic.random()
print(m.entropy)

m.entropy = "<hex from external tool>"
print(m.phrase)   # rebuilt from the new entropy
```
