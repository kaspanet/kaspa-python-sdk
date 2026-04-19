"""
Shared helpers for wallet-related tests.

Used by both `tests/unit/test_wallet_accounts.py` and
`tests/integration/test_wallet.py` to avoid duplicating the
prv_key_data_create boilerplate.
"""

from kaspa import Mnemonic, PrvKeyDataId, PrvKeyDataVariantKind

from tests.conftest import TEST_WALLET_SECRET


# The SecretKey variant of PrvKeyDataVariantKind passes the `secret` String
# straight to `Secret::from(secret)` (see `prv_key_data_create` in
# `src/wallet/core/wallet.rs`), so upstream wallet-core reads its UTF-8
# bytes directly as the secp256k1 key material — NOT as a hex string.
# 32 repeated "a" chars = 32 bytes of 0x61, a valid secp256k1 scalar.
TEST_SECRET_KEY_RAW = "a" * 32


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
