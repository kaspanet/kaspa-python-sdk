"""
Shared helpers for wallet-related tests.

Used by both `tests/unit/test_wallet_accounts.py` and
`tests/integration/test_wallet.py` to avoid duplicating the
prv_key_data_create boilerplate.
"""

from kaspa import Mnemonic, PrvKeyDataId, PrvKeyDataVariantKind

from tests.conftest import TEST_WALLET_SECRET


# 64-char hex string = 32 bytes, a valid secp256k1 scalar.
TEST_SECRET_KEY_RAW = "a" * 64


async def create_mnemonic_key(
    wallet, name: str = "unit-test-mnemonic", phrase: str | None = None
) -> PrvKeyDataId:
    """Create a Mnemonic-kind prv key data entry and return its id."""
    return await wallet.prv_key_data_create(
        wallet_secret=TEST_WALLET_SECRET,
        secret=phrase if phrase is not None else Mnemonic.random().phrase,
        kind=PrvKeyDataVariantKind.Mnemonic,
        name=name,
    )


async def create_secret_key(
    wallet, name: str = "unit-test-secret-key"
) -> PrvKeyDataId:
    """Create a SecretKey-kind prv key data entry and return its id."""
    return await wallet.prv_key_data_create(
        wallet_secret=TEST_WALLET_SECRET,
        secret=TEST_SECRET_KEY_RAW,
        kind=PrvKeyDataVariantKind.SecretKey,
        name=name,
    )
