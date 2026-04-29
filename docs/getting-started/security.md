# Security

Working with cryptocurrency means working with secret material. The same key
that lets you spend funds also lets anyone else who obtains it spend them. The
SDK doesn't try to hide this — it gives you direct access to mnemonics, seeds,
private keys, and wallet files. Treat them with care.

## What counts as secret material

| Type | Where it appears | Compromise consequence |
| --- | --- | --- |
| **Mnemonic phrase** | `Mnemonic.phrase`, the words in a `Mnemonic` instance | Full recovery of every wallet derived from it |
| **BIP-39 seed (64 bytes)** | `mnemonic.to_seed(...)`, the input to `XPrv(seed)` | Same as the mnemonic |
| **Extended private key (XPrv)** | `XPrv` instance, `xprv.xprv` string | Full control of every account derived under it |
| **Private key** | `PrivateKey`, `private_key.to_string()`, hex export | Full control of every UTXO that pays its address |
| **Wallet secret** | The password passed to `wallet_create`, `prv_key_data_create`, `accounts_send`, etc. | Decrypts the on-disk wallet file |
| **Wallet file** (`.kaspa/`) | The directory the managed `Wallet` writes to | Encrypted, but a weak password is not enough — the file holds every key |

## Rules

1. **Never commit secrets to git.** Add the wallet storage directory (`~/.kaspa` or whatever you configured) and any `*.json`, `*.kaspa`, `seed.txt`, `mnemonic.txt` artefacts to `.gitignore` *before* generating real keys.
2. **Never paste a real mnemonic into source code, an issue, a chat message, an LLM prompt, or a screenshot.** The examples throughout these docs use placeholder phrases for a reason.
3. **Don't print or log secret material in production.** The example snippets print mnemonics for clarity; strip those `print()` calls before shipping.
4. **Don't reuse mainnet keys for testing.** Generate a fresh testnet mnemonic and fund it from the testnet faucet.
5. **Use a wallet passphrase ("25th word") for high-value wallets.** A passphrase changes the seed; an attacker with the mnemonic alone gets nothing without it.
6. **Store backups offline.** Paper, hardware-encrypted USB, hardware wallet — not iCloud Notes.
7. **Generate keys on a machine you trust.** Building a release on a shared CI runner and deriving keys there is not the same as deriving them locally.

## In these docs

Code samples in **Learn** and **Guides** show how the SDK works — they pass
literal hex strings and short passwords inline so the snippet is readable.
**That is not how to handle real secrets.** When you adapt a snippet to
production, replace the inline strings with secrets sourced from your secret
manager, environment variables, hardware wallet, or interactive prompt.

If a page documents an operation that touches secret material, it links here
instead of repeating this warning in full.

## When something does leak

If a mnemonic, seed, or private key leaves your control:

1. Move every UTXO out of the affected wallet *immediately* — to a freshly
   derived wallet from a *new* mnemonic, not the same one.
2. Stop using the leaked mnemonic. Don't try to "rotate the passphrase" or
   "skip account 0" — derive a new wallet from new entropy.
3. Audit any service that accepted that wallet's signed messages or extended
   public key.
