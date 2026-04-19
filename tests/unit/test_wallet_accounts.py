"""
Unit tests for Wallet account and private-key-data operations.

Covers prv_key_data_* and accounts_* methods that execute locally without
requiring a live RPC connection.
"""

import pytest

from kaspa import (
    AccountDescriptor,
    AccountId,
    AccountKind,
    Address,
    Mnemonic,
    NetworkId,
    NewAddressKind,
    PrvKeyDataId,
    PrvKeyDataInfo,
    PrvKeyDataVariantKind,
    TransactionKind,
)
from tests.conftest import TEST_WALLET_SECRET
from tests.wallet_helpers import (
    create_mnemonic_key as _create_mnemonic_key,
    create_secret_key as _create_secret_key,
)


# =============================================================================
# prv_key_data_*
# =============================================================================


class TestPrvKeyDataCreate:
    """Tests for Wallet.prv_key_data_create."""

    async def test_create_mnemonic_returns_id(self, open_wallet):
        """Test creating a Mnemonic-kind prv key data returns a PrvKeyDataId."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        assert isinstance(pid, PrvKeyDataId)

    async def test_create_secret_key_returns_id(self, open_wallet):
        """Test creating a SecretKey-kind prv key data returns a PrvKeyDataId."""
        wallet, _ = open_wallet
        pid = await _create_secret_key(wallet)
        assert isinstance(pid, PrvKeyDataId)

    async def test_create_with_kind_string(self, open_wallet):
        """Test that the kind argument accepts a string alias."""
        wallet, _ = open_wallet
        pid = await wallet.prv_key_data_create(
            wallet_secret=TEST_WALLET_SECRET,
            secret=Mnemonic.random().phrase,
            kind="mnemonic",
            name="string-kind-test",
        )
        assert isinstance(pid, PrvKeyDataId)

    async def test_create_with_invalid_kind_string_raises(self, open_wallet):
        """Test that an unsupported kind string raises."""
        wallet, _ = open_wallet
        with pytest.raises(Exception):
            await wallet.prv_key_data_create(
                wallet_secret=TEST_WALLET_SECRET,
                secret=Mnemonic.random().phrase,
                kind="not-a-real-kind",
            )

    async def test_create_with_wrong_wallet_secret_raises(self, open_wallet):
        """Test that an incorrect wallet secret raises."""
        wallet, _ = open_wallet
        with pytest.raises(Exception):
            await wallet.prv_key_data_create(
                wallet_secret="wrong-password",
                secret=Mnemonic.random().phrase,
                kind=PrvKeyDataVariantKind.Mnemonic,
            )


class TestPrvKeyDataEnumerate:
    """Tests for Wallet.prv_key_data_enumerate."""

    async def test_enumerate_empty(self, open_wallet):
        """Test enumerate returns an empty list when no entries are stored."""
        wallet, _ = open_wallet
        infos = await wallet.prv_key_data_enumerate()
        assert infos == []

    async def test_enumerate_includes_created_entries(self, open_wallet):
        """Test enumerate lists every created entry as a PrvKeyDataInfo."""
        wallet, _ = open_wallet
        pid_a = await _create_mnemonic_key(wallet, name="key-a")
        pid_b = await _create_secret_key(wallet, name="key-b")

        infos = await wallet.prv_key_data_enumerate()
        assert len(infos) == 2
        for info in infos:
            assert isinstance(info, PrvKeyDataInfo)

        ids = {str(info.id) for info in infos}
        assert str(pid_a) in ids
        assert str(pid_b) in ids


class TestPrvKeyDataGet:
    """Tests for Wallet.prv_key_data_get."""

    async def test_get_returns_info(self, open_wallet):
        """Test prv_key_data_get returns a PrvKeyDataInfo for a known id."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet, name="getter")
        info = await wallet.prv_key_data_get(TEST_WALLET_SECRET, pid)
        assert isinstance(info, PrvKeyDataInfo)
        assert info.id == pid
        assert info.name == "getter"

    async def test_get_accepts_hex_string_id(self, open_wallet):
        """Test prv_key_data_get accepts a hex string id."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        info = await wallet.prv_key_data_get(TEST_WALLET_SECRET, str(pid))
        assert isinstance(info, PrvKeyDataInfo)

    async def test_get_with_wrong_password_raises(self, open_wallet):
        """Test that an incorrect password raises."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        with pytest.raises(Exception):
            await wallet.prv_key_data_get("wrong-password", pid)


class TestPrvKeyDataRemove:
    """Tests for Wallet.prv_key_data_remove."""

    async def test_remove_raises_not_implemented(self, open_wallet):
        """Test that prv_key_data_remove currently raises NotImplemented upstream."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        with pytest.raises(Exception):
            await wallet.prv_key_data_remove(TEST_WALLET_SECRET, pid)


# =============================================================================
# accounts_*
# =============================================================================


class TestAccountsCreate:
    """Tests for Wallet.accounts_create_bip32 / accounts_create_keypair."""

    async def test_create_bip32_returns_descriptor(self, open_wallet):
        """Test accounts_create_bip32 returns an AccountDescriptor."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
            account_name="bip32-acct",
        )
        assert isinstance(descriptor, AccountDescriptor)
        assert isinstance(descriptor.account_id, AccountId)
        assert isinstance(descriptor.kind, AccountKind)
        assert descriptor.account_name == "bip32-acct"
        assert descriptor.receive_address is not None
        assert descriptor.change_address is not None

    async def test_create_bip32_with_explicit_account_index(self, open_wallet):
        """Test accounts_create_bip32 honors an explicit account_index.

        Two accounts derived from the same key but different indexes must
        produce distinct addresses — otherwise the argument is ignored.
        """
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        default_desc = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
            account_name="default",
            account_index=0,
        )
        indexed_desc = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
            account_name="indexed",
            account_index=7,
        )
        assert isinstance(indexed_desc, AccountDescriptor)
        assert indexed_desc.account_id != default_desc.account_id
        assert indexed_desc.receive_address != default_desc.receive_address

    async def test_create_keypair_returns_descriptor(self, open_wallet):
        """Test accounts_create_keypair returns an AccountDescriptor."""
        wallet, _ = open_wallet
        pid = await _create_secret_key(wallet)
        descriptor = await wallet.accounts_create_keypair(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
            ecdsa=False,
            account_name="kp-acct",
        )
        assert isinstance(descriptor, AccountDescriptor)

    async def test_create_bip32_with_wrong_password_raises(self, open_wallet):
        """Test accounts_create_bip32 raises with the wrong wallet secret."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        with pytest.raises(Exception):
            await wallet.accounts_create_bip32(
                wallet_secret="wrong",
                prv_key_data_id=pid,
            )


class TestAccountsImport:
    """Tests for Wallet.accounts_import_bip32 / accounts_import_keypair."""

    async def test_import_bip32_without_rpc_connection_raises(self, open_wallet):
        """Test accounts_import_bip32 raises when no RPC connection is available."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        with pytest.raises(Exception):
            await wallet.accounts_import_bip32(
                wallet_secret=TEST_WALLET_SECRET,
                prv_key_data_id=pid,
                account_name="imported",
            )


class TestAccountsEnsureDefault:
    """Tests for Wallet.accounts_ensure_default."""

    async def test_ensure_default_bip32_creates_account(self, open_wallet):
        """Test ensure_default with AccountKind('bip32') returns a descriptor."""
        wallet, _ = open_wallet
        descriptor = await wallet.accounts_ensure_default(
            wallet_secret=TEST_WALLET_SECRET,
            account_kind=AccountKind("bip32"),
            mnemonic_phrase=Mnemonic.random().phrase,
        )
        assert isinstance(descriptor, AccountDescriptor)

    async def test_ensure_default_is_idempotent(self, open_wallet):
        """Test ensure_default returns the existing account on a second call."""
        wallet, _ = open_wallet
        first = await wallet.accounts_ensure_default(
            wallet_secret=TEST_WALLET_SECRET,
            account_kind=AccountKind("bip32"),
            mnemonic_phrase=Mnemonic.random().phrase,
        )
        second = await wallet.accounts_ensure_default(
            wallet_secret=TEST_WALLET_SECRET,
            account_kind=AccountKind("bip32"),
            mnemonic_phrase=Mnemonic.random().phrase,
        )
        assert first.account_id == second.account_id


class TestAccountsEnumerate:
    """Tests for Wallet.accounts_enumerate."""

    async def test_enumerate_empty(self, open_wallet):
        """Test accounts_enumerate returns [] when no accounts exist."""
        wallet, _ = open_wallet
        accounts = await wallet.accounts_enumerate()
        assert accounts == []

    async def test_enumerate_includes_created_accounts(self, open_wallet):
        """Test accounts_enumerate returns an entry for every created account."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
            account_name="acct-1",
        )
        accounts = await wallet.accounts_enumerate()
        assert len(accounts) == 1
        assert isinstance(accounts[0], AccountDescriptor)


class TestAccountsGet:
    """Tests for Wallet.accounts_get."""

    async def test_accounts_get_with_account_id(self, open_wallet):
        """Test accounts_get accepts an AccountId instance."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
        )
        await wallet.accounts_get(descriptor.account_id)

    async def test_accounts_get_with_hex_string(self, open_wallet):
        """Test accounts_get accepts a hex string id."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
        )
        await wallet.accounts_get(str(descriptor.account_id))

    async def test_accounts_get_missing_raises(self, open_wallet):
        """Test accounts_get raises for an unknown account id."""
        wallet, _ = open_wallet
        bogus = AccountId("0" * 64)
        with pytest.raises(Exception):
            await wallet.accounts_get(bogus)


class TestAccountsRename:
    """Tests for Wallet.accounts_rename."""

    async def test_rename_account(self, open_wallet):
        """Test renaming an account updates its descriptor name."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
            account_name="old-name",
        )
        await wallet.accounts_rename(
            wallet_secret=TEST_WALLET_SECRET,
            account_id=descriptor.account_id,
            name="new-name",
        )
        accounts = await wallet.accounts_enumerate()
        names = [a.account_name for a in accounts]
        assert "new-name" in names

    async def test_rename_with_wrong_password_raises(self, open_wallet):
        """Test accounts_rename raises with the wrong wallet secret."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
        )
        with pytest.raises(Exception):
            await wallet.accounts_rename(
                wallet_secret="wrong",
                account_id=descriptor.account_id,
                name="whatever",
            )
        # The failed rename leaves the store's modified flag set.
        # Trigger a successful save via a valid rename so teardown's wallet_close
        # doesn't panic.
        # This feels like an upstream bug in rk native
        await wallet.accounts_rename(
            wallet_secret=TEST_WALLET_SECRET,
            account_id=descriptor.account_id,
            name="cleanup",
        )


class TestAccountsCreateNewAddress:
    """Tests for Wallet.accounts_create_new_address."""

    async def test_create_receive_address(self, open_wallet):
        """Test deriving a new receive address returns an Address."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
        )
        address = await wallet.accounts_create_new_address(
            descriptor.account_id, NewAddressKind.Receive
        )
        assert isinstance(address, Address)
        assert address.prefix == "kaspatest"

    async def test_create_change_address_with_string_kind(self, open_wallet):
        """Test deriving a change address with a string kind argument."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
        )
        address = await wallet.accounts_create_new_address(
            descriptor.account_id, "change"
        )
        assert isinstance(address, Address)

    async def test_create_address_invalid_kind_string_raises(self, open_wallet):
        """Test an unknown NewAddressKind string raises."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
        )
        with pytest.raises(Exception):
            await wallet.accounts_create_new_address(
                descriptor.account_id, "not-a-real-kind"
            )


class TestAccountsActivate:
    """Tests for Wallet.accounts_activate."""

    async def test_activate_specific_account(self, open_wallet):
        """Test activating a single account id."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
        )
        await wallet.accounts_activate([descriptor.account_id])

    async def test_activate_all_accounts(self, open_wallet):
        """Test activating every account when account_ids is None."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
        )
        await wallet.accounts_activate()


# =============================================================================
# transactions_*
# =============================================================================


class TestTransactionsDataGet:
    """Tests for Wallet.transactions_data_get."""

    async def test_data_get_returns_empty_history(self, open_wallet):
        """Test transactions_data_get returns an empty history for a new account."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
        )
        resp = await wallet.transactions_data_get(
            account_id=descriptor.account_id,
            network_id=NetworkId("testnet-10"),
            start=0,
            end=10,
        )
        assert isinstance(resp, dict)
        assert resp.get("total") == 0
        assert resp.get("transactions") == []

    async def test_data_get_accepts_filter(self, open_wallet):
        """Test transactions_data_get accepts a list of TransactionKind filters."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
        )
        resp = await wallet.transactions_data_get(
            account_id=descriptor.account_id,
            network_id=NetworkId("testnet-10"),
            start=0,
            end=10,
            filter=[TransactionKind.Incoming, TransactionKind.Outgoing],
        )
        assert isinstance(resp, dict)
        assert resp.get("transactions") == []


class TestTransactionsReplaceNote:
    """Tests for Wallet.transactions_replace_note."""

    async def test_replace_note_missing_transaction_raises(self, open_wallet):
        """Test replacing a note on an unknown transaction id raises."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
        )
        with pytest.raises(Exception):
            await wallet.transactions_replace_note(
                account_id=descriptor.account_id,
                network_id=NetworkId("testnet-10"),
                transaction_id="0" * 64,
                note="note",
            )


class TestTransactionsReplaceMetadata:
    """Tests for Wallet.transactions_replace_metadata."""

    async def test_replace_metadata_missing_transaction_raises(self, open_wallet):
        """Test replacing metadata on an unknown transaction id raises."""
        wallet, _ = open_wallet
        pid = await _create_mnemonic_key(wallet)
        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
        )
        with pytest.raises(Exception):
            await wallet.transactions_replace_metadata(
                account_id=descriptor.account_id,
                network_id=NetworkId("testnet-10"),
                transaction_id="0" * 64,
                metadata="meta",
            )
