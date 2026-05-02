---
search:
  boost: 3
---

# Key Management

Everything on this page is BIP-39 compatible. The SDK gives you
[`Mnemonic`](../../reference/Classes/Mnemonic.md) for the
human-readable phrase, the seed bytes it produces, and
[`XPrv`](../../reference/Classes/XPrv.md) for the master extended
private key from that seed. From an [`XPrv`](../../reference/Classes/XPrv.md) you [derive](derivation.md) child keys.

Read [Security](../../getting-started/security.md) before generating a
real mnemonic.

## Generate a mnemonic

[`Mnemonic.random()`](../../reference/Classes/Mnemonic.md) generates a fresh phrase:

```python
from kaspa import Mnemonic

m = Mnemonic.random()                  # 24 words, default
m12 = Mnemonic.random(word_count=12)   # 12 words
print(m.phrase)
```

## Restore from a mnemonic

```python
from kaspa import Mnemonic

phrase = "abandon abandon abandon ... about"

if Mnemonic.validate(phrase):
    m = Mnemonic(phrase)
else:
    raise ValueError("invalid mnemonic")
```

[`Mnemonic.validate(phrase)`](../../reference/Classes/Mnemonic.md) checks word membership, length, and the
BIP-39 checksum, and returns a bool. The [`Mnemonic(phrase)`](../../reference/Classes/Mnemonic.md) constructor
raises on the same conditions, so the explicit [`validate()`](../../reference/Classes/Mnemonic.md) call is
only useful when you want to surface a friendlier error than the
underlying exception.

## Validation

```python
from kaspa import Mnemonic, Language

ok = Mnemonic.validate(phrase)                       # English assumed
ok_en = Mnemonic.validate(phrase, Language.English)  # explicit
```

The [`Language`](../../reference/Enums/Language.md) enum names the BIP-39 wordlist (English by default).

## Convert to a seed

```python
from kaspa import Mnemonic, XPrv

m = Mnemonic.random()
seed = m.to_seed()                            # 64-byte BIP-39 seed
seed_with_passphrase = m.to_seed("25th-word") # different seed; same mnemonic

xprv = XPrv(seed)
```

!!! info "Passphrase"
    The optional passphrase (the "25th word") changes the seed. The
    same mnemonic with different passphrases produces different
    wallets. An attacker who recovers the mnemonic alone gets nothing
    without the passphrase.

## Inspect entropy

The `entropy` property exposes the underlying random bits as a hex
string — the raw input the BIP-39 phrase encodes:

```python
m = Mnemonic.random()
print(m.entropy)                    # hex
m.entropy = "<new entropy hex>"     # advanced; rebuilds the phrase
```

You rarely need to set `entropy` directly. Two cases that come up:
re-creating a `Mnemonic` from entropy emitted by another tool, and
debugging a vector mismatch against a third-party implementation.

## Languages

```python
from kaspa import Mnemonic, Language

m = Mnemonic(phrase, Language.English)
```

English is the default and what every Kaspa example uses. Other
BIP-39 wordlists exist on the enum but are rarely used — if you don't
know you need a non-English wordlist, use English.

## Hex private keys ([`PrivateKey`](../../reference/Classes/PrivateKey.md))

For one-key accounts (a single secp256k1 secret, no derivation),
skip the mnemonic entirely:

```python
from kaspa import PrivateKey

key = PrivateKey("<64-char hex>")
addr = key.to_address("testnet-10")
```

The wallet's keypair accounts use the same 64-char hex string. When
calling [`prv_key_data_create`](../../reference/Classes/Wallet.md#kaspa.Wallet.prv_key_data_create), pass it as `secret` with
[`kind=PrvKeyDataVariantKind.SecretKey`](../../reference/Enums/PrvKeyDataVariantKind.md) — see
[Wallet → Accounts → Keypair accounts](../wallet/accounts.md#keypair-accounts).

## Where to next

- [Derivation](derivation.md) — turn the `XPrv` into addresses.
- [Transaction Generator](tx-generator.md) — sign a transaction with a
  key you derived.
- [Security](../../getting-started/security.md) — secret-handling rules
  before any of the above touches mainnet.
