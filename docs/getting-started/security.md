---
search:
  boost: 3
---

# Security

The SDK gives you direct access to mnemonics, seeds, private keys, and wallet
files. Anyone who obtains them can spend your funds. Treat them with care.

## What counts as secret material

| Type | Compromise consequence |
| --- | --- |
| **Mnemonic phrase** / **BIP-39 seed** | Full recovery of every wallet derived from it |
| **Extended private key (XPrv)** | Full control of every account derived under it |
| **Private key** | Full control of every UTXO that pays its address |
| **Wallet secret** (password) | Decrypts the on-disk wallet file |
| **Wallet file** (`.kaspa/`) | Encrypted, but only as strong as the password |

## Why Python makes this hard

Python wasn't designed for handling secrets. Be aware of:

- **No secure memory.** `str` and `bytes` are immutable — you can't reliably
  zero them after use. Copies linger in interpreter caches, tracebacks, and the
  garbage collector until reclaimed.
- **Leaks via `repr` and logging.** Frameworks happily dump local variables in
  exception traces, debuggers, and structured logs. A `PrivateKey` in scope
  during an unhandled exception can land in your error tracker.
- **Process introspection.** Other code in the same interpreter (debuggers,
  profilers, third-party libraries, malicious dependencies) can read your
  memory. Supply-chain risk is real — audit what you `pip install`.
- **Shell history and env dumps.** Secrets passed as CLI args show up in
  `ps`/`history`; secrets in `os.environ` show up in crash reports and
  subprocess inheritance.

## Handling secrets in Python

- **Source secrets at runtime, never from source code.** Use environment
  variables loaded from a `.env` file (with `python-dotenv`), an OS keyring
  (`keyring`), a cloud secret manager (AWS Secrets Manager, GCP Secret
  Manager, HashiCorp Vault), or `getpass.getpass()` for interactive prompts.
- **Keep secrets in narrow scope.** Load, use, drop the reference. Don't
  attach them to long-lived objects, module globals, or class attributes.
- **Strip `print()` and disable verbose logging in production.** Filter
  sensitive fields out of structured logs before they reach disk or a SaaS
  collector.
- **Isolate signing.** For high-value keys, sign in a separate process,
  container, or hardware device — not the same Python process that handles
  network I/O.
- **Pin and audit dependencies.** Use a lockfile, review transitive deps, and
  prefer `pip install --require-hashes` for production installs.

## Operational rules

1. **Never commit secrets to git.** Add wallet storage paths and any
   `*.json`, `seed.txt`, `mnemonic.txt` artefacts to `.gitignore` *before*
   generating real keys.
2. **Never paste a real mnemonic into source code, an issue, a chat message,
   an LLM prompt, or a screenshot.**
3. **Don't reuse mainnet keys for testing.** Use a fresh testnet mnemonic.
4. **Use a wallet passphrase ("25th word") for high-value wallets.**
5. **Store backups offline** — paper, hardware-encrypted USB, hardware
   wallet. Not iCloud Notes.
6. **Generate keys on a machine you trust** — not a shared CI runner.

## In these docs

Code samples pass literal hex strings and short passwords inline for
readability. **That is not how to handle real secrets.** Replace inline
strings with values sourced from a secret manager, environment variable,
hardware wallet, or interactive prompt.
