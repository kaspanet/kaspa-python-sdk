"""Shared demo variables and bootstrap helper for the wallet examples.

All wallet examples import the same mnemonic, secret, and network id from
this module so they derive addresses from a single deterministic seed.

Each example uses its own `FILENAME` so the on-disk wallets stay
independent.
"""

from kaspa import AccountKind, PrvKeyDataVariantKind, Wallet

BIP32_KIND = AccountKind("bip32")

FIXED_MNEMONIC_PHRASE = (
    "abandon abandon abandon abandon abandon abandon abandon abandon "
    "abandon abandon abandon abandon abandon abandon abandon abandon "
    "abandon abandon abandon abandon abandon abandon abandon art"
)
WALLET_SECRET = "example-wallet-secret"
NETWORK_ID = "testnet-10"


async def open_or_create_wallet(wallet: Wallet, filename: str, *, title: str) -> str:
    """Open an existing demo wallet or create a fresh one.

    First run: creates the wallet, stores the shared demo mnemonic under
    the name "demo-key", and creates a BIP32 account at index 0.
    Subsequent runs: reopens the file and returns the same account via
    `accounts_ensure_default`, which is idempotent.
    """
    if await wallet.exists(filename):
        opened = await wallet.wallet_open(WALLET_SECRET, True, filename)
        print(f"Opened existing wallet file {filename!r}: {opened}\n")
        descriptor = await wallet.accounts_ensure_default(
            wallet_secret=WALLET_SECRET,
            account_kind=BIP32_KIND,
        )
        return descriptor.account_id

    created = await wallet.wallet_create(
        wallet_secret=WALLET_SECRET,
        filename=filename,
        overwrite_wallet_storage=False,
        title=title,
        user_hint="example",
    )
    print(f"Created new wallet file {filename!r}: {created}")

    prv_key_id = await wallet.prv_key_data_create(
        wallet_secret=WALLET_SECRET,
        secret=FIXED_MNEMONIC_PHRASE,
        kind=PrvKeyDataVariantKind.Mnemonic,
        payment_secret=None,
        name="demo-key",
    )
    print(f"Created PrvKeyDataId: {prv_key_id}\n")

    descriptor = await wallet.accounts_create_bip32(
        wallet_secret=WALLET_SECRET,
        prv_key_data_id=prv_key_id,
        payment_secret=None,
        account_name="demo-acct",
        account_index=0,
    )
    print(f"Created BIP32 account: {descriptor}\n")

    return descriptor.account_id
