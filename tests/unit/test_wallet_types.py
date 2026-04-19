"""
Unit tests for Wallet-adjacent wrapper types.

Covers AccountId, AccountKind, PrvKeyDataId, PrvKeyDataInfo,
AccountDescriptor, WalletDescriptor, and the Wallet-related enum
wrappers.
"""

import pytest

from kaspa import (
    AccountDescriptor,
    AccountId,
    AccountKind,
    AccountsDiscoveryKind,
    CommitRevealAddressKind,
    Mnemonic,
    NewAddressKind,
    PrvKeyDataId,
    PrvKeyDataInfo,
    PrvKeyDataVariantKind,
    TransactionKind,
    WalletDescriptor,
    WalletEventType,
)
from tests.conftest import TEST_WALLET_SECRET, cleanup_wallet_files


TEST_ACCOUNT_ID_HEX = "5f9ded675fea85011a084c3c80a209693840136b6f5a60e8691df6f7d181174d"
TEST_PRV_KEY_DATA_ID_HEX = "01d843d334c65c8f"


# =============================================================================
# AccountId
# =============================================================================


class TestAccountIdCreation:
    """Tests for AccountId construction."""

    def test_create_from_valid_hex(self):
        """Test creating an AccountId from a valid hex string."""
        account_id = AccountId(TEST_ACCOUNT_ID_HEX)
        assert isinstance(account_id, AccountId)

    def test_create_from_invalid_hex_raises(self):
        """Test that an invalid hex string raises."""
        with pytest.raises(Exception):
            AccountId("not-a-valid-hex-string")


class TestAccountIdStringification:
    """Tests for AccountId string conversion."""

    def test_str_returns_hex(self):
        """Test __str__ returns the hex representation."""
        account_id = AccountId(TEST_ACCOUNT_ID_HEX)
        assert str(account_id) == TEST_ACCOUNT_ID_HEX

    def test_repr_includes_hex(self):
        """Test __repr__ includes the hex representation."""
        account_id = AccountId(TEST_ACCOUNT_ID_HEX)
        text = repr(account_id)
        assert len(text) > 0
        assert TEST_ACCOUNT_ID_HEX in text

    def test_to_hex_returns_hex(self):
        """Test to_hex() returns the hex string."""
        account_id = AccountId(TEST_ACCOUNT_ID_HEX)
        assert account_id.to_hex() == TEST_ACCOUNT_ID_HEX


class TestAccountIdEquality:
    """Tests for AccountId equality semantics."""

    def test_same_hex_is_equal(self):
        """Test two AccountIds from the same hex compare equal."""
        assert AccountId(TEST_ACCOUNT_ID_HEX) == AccountId(TEST_ACCOUNT_ID_HEX)

    def test_different_hex_is_not_equal(self):
        """Test two AccountIds from different hex compare not equal."""
        other_hex = "0" * 64
        assert AccountId(TEST_ACCOUNT_ID_HEX) != AccountId(other_hex)


# =============================================================================
# AccountKind
# =============================================================================


class TestAccountKind:
    """Tests for AccountKind construction and stringification."""

    @pytest.mark.parametrize("kind_str", ["legacy", "bip32", "multisig", "keypair"])
    def test_create_from_valid_string(self, kind_str):
        """Test creating an AccountKind from every supported string."""
        kind = AccountKind(kind_str)
        assert isinstance(kind, AccountKind)

    def test_create_from_invalid_string_raises(self):
        """Test that an invalid kind string raises."""
        with pytest.raises(Exception):
            AccountKind("not-a-real-kind")

    def test_stringification(self):
        """Test __str__ and to_string() both return the canonical kind string."""
        kind = AccountKind("bip32")
        assert str(kind) == "kaspa-bip32-standard"
        assert kind.to_string() == "kaspa-bip32-standard"

    def test_equality(self):
        """Test two AccountKinds built from the same string compare equal."""
        assert AccountKind("bip32") == AccountKind("bip32")
        assert AccountKind("bip32") != AccountKind("keypair")

    def test_repr_includes_kind_name(self):
        """Test __repr__ is non-empty and contains the kind name."""
        kind = AccountKind("bip32")
        text = repr(kind)
        assert len(text) > 0
        assert "bip32" in text


# =============================================================================
# PrvKeyDataId
# =============================================================================


class TestPrvKeyDataIdCreation:
    """Tests for PrvKeyDataId construction."""

    def test_create_from_valid_hex(self):
        """Test creating a PrvKeyDataId from a valid hex string."""
        pid = PrvKeyDataId(TEST_PRV_KEY_DATA_ID_HEX)
        assert isinstance(pid, PrvKeyDataId)

    def test_create_from_invalid_hex_raises(self):
        """Test that an invalid hex string raises."""
        with pytest.raises(Exception):
            PrvKeyDataId("xyz-not-hex")


class TestPrvKeyDataIdStringification:
    """Tests for PrvKeyDataId string conversion."""

    def test_str_returns_hex(self):
        """Test __str__ returns the hex representation."""
        pid = PrvKeyDataId(TEST_PRV_KEY_DATA_ID_HEX)
        assert str(pid) == TEST_PRV_KEY_DATA_ID_HEX

    def test_repr_includes_hex(self):
        """Test __repr__ includes the hex representation."""
        pid = PrvKeyDataId(TEST_PRV_KEY_DATA_ID_HEX)
        text = repr(pid)
        assert len(text) > 0
        assert TEST_PRV_KEY_DATA_ID_HEX in text

    def test_to_hex_returns_hex(self):
        """Test to_hex() returns the hex string."""
        pid = PrvKeyDataId(TEST_PRV_KEY_DATA_ID_HEX)
        assert pid.to_hex() == TEST_PRV_KEY_DATA_ID_HEX


class TestPrvKeyDataIdEquality:
    """Tests for PrvKeyDataId equality semantics."""

    def test_same_hex_is_equal(self):
        """Test two PrvKeyDataIds from the same hex compare equal."""
        assert PrvKeyDataId(TEST_PRV_KEY_DATA_ID_HEX) == PrvKeyDataId(
            TEST_PRV_KEY_DATA_ID_HEX
        )

    def test_different_hex_is_not_equal(self):
        """Test two PrvKeyDataIds from different hex compare not equal."""
        assert PrvKeyDataId(TEST_PRV_KEY_DATA_ID_HEX) != PrvKeyDataId("ffffffffffffffff")


# =============================================================================
# PrvKeyDataInfo
# =============================================================================


class TestPrvKeyDataInfo:
    """Tests for PrvKeyDataInfo properties."""

    async def test_info_properties(self, open_wallet):
        """Test id/name/is_encrypted properties on a created entry."""
        wallet, _ = open_wallet
        pid = await wallet.prv_key_data_create(
            wallet_secret=TEST_WALLET_SECRET,
            secret=Mnemonic.random().phrase,
            kind=PrvKeyDataVariantKind.Mnemonic,
            name="info-test",
        )
        infos = await wallet.prv_key_data_enumerate()
        assert len(infos) == 1
        info = infos[0]
        assert isinstance(info, PrvKeyDataInfo)
        assert info.id == pid
        assert info.name == "info-test"
        assert info.is_encrypted is False

    async def test_info_repr(self, open_wallet):
        """Test __repr__ includes the id and name."""
        wallet, _ = open_wallet
        await wallet.prv_key_data_create(
            wallet_secret=TEST_WALLET_SECRET,
            secret=Mnemonic.random().phrase,
            kind=PrvKeyDataVariantKind.Mnemonic,
            name="repr-test",
        )
        info = (await wallet.prv_key_data_enumerate())[0]
        text = repr(info)
        assert "PrvKeyDataInfo" in text
        assert "repr-test" in text


# =============================================================================
# AccountDescriptor
# =============================================================================


class TestAccountDescriptor:
    """Tests for AccountDescriptor properties."""

    async def test_descriptor_properties(self, open_wallet):
        """Test the key AccountDescriptor getters on a newly created account."""
        wallet, _ = open_wallet
        pid = await wallet.prv_key_data_create(
            wallet_secret=TEST_WALLET_SECRET,
            secret=Mnemonic.random().phrase,
            kind=PrvKeyDataVariantKind.Mnemonic,
        )
        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
            account_name="acct-desc",
        )
        assert isinstance(descriptor, AccountDescriptor)
        assert isinstance(descriptor.account_id, AccountId)
        assert isinstance(descriptor.kind, AccountKind)
        assert descriptor.account_name == "acct-desc"
        assert descriptor.balance is None
        assert isinstance(descriptor.receive_address, str)
        assert isinstance(descriptor.change_address, str)

    async def test_descriptor_repr(self, open_wallet):
        """Test __repr__ includes the account kind and id."""
        wallet, _ = open_wallet
        pid = await wallet.prv_key_data_create(
            wallet_secret=TEST_WALLET_SECRET,
            secret=Mnemonic.random().phrase,
            kind=PrvKeyDataVariantKind.Mnemonic,
        )
        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
            account_name="acct-repr",
        )
        text = repr(descriptor)
        assert "AccountDescriptor" in text
        assert "acct-repr" in text
        assert str(descriptor.account_id) in text


# =============================================================================
# WalletDescriptor
# =============================================================================


class TestWalletDescriptor:
    """Tests for WalletDescriptor properties."""

    async def test_descriptor_properties(self, started_wallet, unique_wallet_filename):
        """Test filename and title are independently plumbed on the descriptor."""
        distinct_title = f"title-{unique_wallet_filename}"
        try:
            await started_wallet.wallet_create(
                wallet_secret=TEST_WALLET_SECRET,
                filename=unique_wallet_filename,
                overwrite_wallet_storage=True,
                title=distinct_title,
            )
            descriptor = started_wallet.descriptor
            assert isinstance(descriptor, WalletDescriptor)
            assert descriptor.filename == unique_wallet_filename
            assert descriptor.title == distinct_title
        finally:
            cleanup_wallet_files(unique_wallet_filename)

    async def test_descriptor_repr_includes_filename(self, open_wallet):
        """Test __repr__ includes the wallet filename."""
        wallet, filename = open_wallet
        text = repr(wallet.descriptor)
        assert "WalletDescriptor" in text
        assert filename in text


# =============================================================================
# Enum Wrapper Types
# =============================================================================


@pytest.mark.parametrize(
    "enum_cls,member",
    [
        (WalletEventType, name)
        for name in ("All", "Connect", "Disconnect", "WalletOpen",
                     "WalletClose", "Balance", "AccountCreate", "SyncState")
    ]
    + [(AccountsDiscoveryKind, "Bip44")]
    + [(NewAddressKind, name) for name in ("Receive", "Change")]
    + [(CommitRevealAddressKind, name) for name in ("Receive", "Change")]
    + [
        (PrvKeyDataVariantKind, name)
        for name in ("Mnemonic", "Bip39Seed", "ExtendedPrivateKey", "SecretKey")
    ]
    + [
        (TransactionKind, name)
        for name in ("Incoming", "Outgoing", "External", "Change", "Batch",
                     "Reorg", "Stasis", "TransferIncoming", "TransferOutgoing")
    ],
)
def test_enum_member_exposed(enum_cls, member):
    """Every expected variant on the wallet-related enum wrappers is exposed."""
    assert getattr(enum_cls, member) is not None
