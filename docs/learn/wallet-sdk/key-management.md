# Key Management

Everything on this page is BIP-39 compatible. The SDK gives you
[`Mnemonic`](../../reference/Classes/Mnemonic.md) for the
human-readable phrase, the seed bytes it produces, and
[`XPrv`](../../reference/Classes/XPrv.md) for the master extended
private key from that seed. From an `XPrv` you derive child keys —
that's the next page, [Derivation](derivation.md).

Read [Security](../../getting-started/security.md) before generating a
real mnemonic.

## Generate a mnemonic

```python
from kaspa import Mnemonic

m = Mnemonic.random()                  # 24 words, default
m12 = Mnemonic.random(word_count=12)   # 12 words
print(m.phrase)
```

24 words is the recommended default — more entropy, lower brute-force
risk. 12 words is supported for compatibility with tools that emit
them.

## Restore from a mnemonic

```python
from kaspa import Mnemonic

phrase = "abandon abandon abandon ... about"

if Mnemonic.validate(phrase):
    m = Mnemonic(phrase)
else:
    raise ValueError("invalid mnemonic")
```

`Mnemonic.validate(phrase)` checks word membership, length, and the
BIP-39 checksum. Returns a bool; never raises.

## Validation

```python
from kaspa import Mnemonic, Language

ok = Mnemonic.validate(phrase)                       # English assumed
ok_en = Mnemonic.validate(phrase, Language.English)  # explicit
```

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

## Hex private keys (`SecretKey`)

For one-key accounts (a single secp256k1 secret, no derivation),
skip the mnemonic entirely:

```python
from kaspa import PrivateKey

key = PrivateKey("<64-char hex>")
addr = key.to_address("testnet-10")
```

The 64-char hex string is what
[Wallet → Keypair Accounts](../wallet/keypair.md) takes as the
`secret` input to
`prv_key_data_create(kind=PrvKeyDataVariantKind.SecretKey)`.

## Where to next

- [Derivation](derivation.md) — turn the `XPrv` into addresses.
- [Transaction Generator](tx-generator.md) — sign a transaction with a
  key you derived.
- [Security](../../getting-started/security.md) — secret-handling rules
  before any of the above touches mainnet.
