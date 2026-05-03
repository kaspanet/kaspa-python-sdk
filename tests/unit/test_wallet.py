"""
Unit tests for the Wallet class.

These tests exercise the local-store and in-process behavior of Wallet
without requiring a connected RPC node.
"""

import pytest

from kaspa import (
    Encoding,
    NetworkId,
    Resolver,
    RpcClient,
    Wallet,
    WalletDescriptor,
    WalletEventType,
)
from tests.conftest import (
    TEST_NEW_WALLET_SECRET,
    TEST_WALLET_SECRET,
    cleanup_wallet_files,
)


class TestWalletCreation:
    """Tests for Wallet construction — focused on str-vs-object arg conversion."""

    def test_create_wallet_with_string_network_id(self):
        """Test creating a Wallet with a string network id."""
        wallet = Wallet(network_id="testnet-10")
        assert isinstance(wallet, Wallet)

    def test_create_wallet_with_network_id_object(self):
        """Test creating a Wallet with a NetworkId object."""
        wallet = Wallet(network_id=NetworkId("testnet-10"))
        assert isinstance(wallet, Wallet)

    def test_create_wallet_with_encoding_enum(self):
        """Test creating a Wallet with an Encoding enum."""
        wallet = Wallet(network_id="testnet-10", encoding=Encoding.Borsh)
        assert isinstance(wallet, Wallet)

    def test_create_wallet_with_encoding_string(self):
        """Test creating a Wallet with a string encoding."""
        wallet = Wallet(network_id="testnet-10", encoding="borsh")
        assert isinstance(wallet, Wallet)


class TestWalletProperties:
    """Tests for Wallet property accessors."""

    def test_rpc_property_returns_rpc_client(self):
        """Test that the rpc property returns an RpcClient."""
        wallet = Wallet(network_id="testnet-10", resolver=Resolver())
        assert isinstance(wallet.rpc, RpcClient)

    def test_is_open_false_before_any_wallet(self):
        """Test is_open is False for a freshly created Wallet."""
        wallet = Wallet(network_id="testnet-10", resolver=Resolver())
        assert wallet.is_open is False

    def test_is_synced_false_before_connect(self):
        """Test is_synced is False before the wallet connects."""
        wallet = Wallet(network_id="testnet-10", resolver=Resolver())
        assert wallet.is_synced is False

    def test_descriptor_is_none_before_open(self):
        """Test descriptor is None before a wallet file is opened."""
        wallet = Wallet(network_id="testnet-10", resolver=Resolver())
        assert wallet.descriptor is None


class TestWalletSetNetworkId:
    """Tests for Wallet.set_network_id."""

    def test_set_network_id_with_string(self):
        """Test setting network id with a string."""
        wallet = Wallet(network_id="testnet-10", resolver=Resolver())
        wallet.set_network_id("mainnet")

    def test_set_network_id_with_object(self):
        """Test setting network id with a NetworkId instance."""
        wallet = Wallet(network_id="testnet-10", resolver=Resolver())
        wallet.set_network_id(NetworkId("testnet-10"))

    def test_set_network_id_invalid_raises(self):
        """Test that setting an invalid network id raises an error."""
        wallet = Wallet(network_id="testnet-10", resolver=Resolver())
        with pytest.raises(Exception):
            wallet.set_network_id("not-a-real-network")


class TestWalletEventListeners:
    """Tests for Wallet event listener registration."""

    @pytest.mark.parametrize("event", [WalletEventType.Balance, "balance"])
    def test_add_and_remove_listener(self, event):
        """Test registering and removing a listener with both enum and string arg forms."""
        wallet = Wallet(network_id="testnet-10", resolver=Resolver())

        def cb(event):
            _ = event

        wallet.add_event_listener(event, cb)
        wallet.remove_event_listener(event, cb)

    def test_add_listener_with_all_event(self):
        """Test registering an 'all' listener that receives every event."""
        wallet = Wallet(network_id="testnet-10", resolver=Resolver())

        def cb(event):
            _ = event

        wallet.add_event_listener(WalletEventType.All, cb)
        wallet.remove_event_listener(WalletEventType.All)

    def test_add_listener_with_args_and_kwargs(self):
        """Test registering a listener with extra positional and keyword arguments."""
        wallet = Wallet(network_id="testnet-10", resolver=Resolver())

        def cb(event, *args, **kwargs):
            _ = (event, args, kwargs)

        wallet.add_event_listener("balance", cb, 1, 2, foo="bar")
        wallet.remove_event_listener("balance", cb)

    def test_remove_listener_without_callback_clears_event(self):
        """Test that remove without a callback removes every listener for the event."""
        wallet = Wallet(network_id="testnet-10", resolver=Resolver())

        def cb(event):
            _ = event

        wallet.add_event_listener("balance", cb)
        wallet.remove_event_listener("balance")

    def test_remove_all_listeners_with_all(self):
        """Test that remove with 'all' and no callback clears every listener."""
        wallet = Wallet(network_id="testnet-10", resolver=Resolver())

        def cb(event):
            _ = event

        wallet.add_event_listener("balance", cb)
        wallet.add_event_listener("connect", cb)
        wallet.remove_event_listener(WalletEventType.All)

    def test_add_listener_invalid_event_raises(self):
        """Test that registering a listener for an unknown event raises."""
        wallet = Wallet(network_id="testnet-10", resolver=Resolver())

        def cb(event):
            _ = event

        with pytest.raises(Exception):
            wallet.add_event_listener("not-a-real-event", cb)


class TestWalletExists:
    """Tests for Wallet.exists."""

    async def test_exists_returns_false_for_missing_file(self, started_wallet, unique_wallet_filename):
        """Test exists() returns False when the wallet file does not exist."""
        assert await started_wallet.exists(unique_wallet_filename) is False

    async def test_exists_returns_true_after_create(self, open_wallet):
        """Test exists() returns True after a wallet file is created."""
        wallet, filename = open_wallet
        assert await wallet.exists(filename) is True


class TestWalletEnumerate:
    """Tests for Wallet.wallet_enumerate."""

    async def test_enumerate_returns_list(self, started_wallet):
        """Test wallet_enumerate returns a list of WalletDescriptor."""
        result = await started_wallet.wallet_enumerate()
        assert isinstance(result, list)
        for entry in result:
            assert isinstance(entry, WalletDescriptor)

    async def test_enumerate_includes_created_wallet(self, open_wallet):
        """Test that an open (created) wallet appears in wallet_enumerate."""
        wallet, filename = open_wallet
        descriptors = await wallet.wallet_enumerate()
        filenames = [d.filename for d in descriptors]
        assert filename in filenames


class TestWalletCreate:
    """Tests for Wallet.wallet_create."""

    async def test_create_wallet_file(self, started_wallet, unique_wallet_filename):
        """Test creating a new wallet file."""
        try:
            resp = await started_wallet.wallet_create(
                wallet_secret=TEST_WALLET_SECRET,
                filename=unique_wallet_filename,
                overwrite_wallet_storage=True,
                title="unit-test",
                user_hint="hint",
            )
            assert isinstance(resp, dict)
            assert started_wallet.is_open is True
            assert started_wallet.descriptor is not None
            assert started_wallet.descriptor.filename == unique_wallet_filename
            assert started_wallet.descriptor.title == "unit-test"
        finally:
            try:
                await started_wallet.wallet_close()
            except Exception:
                pass
            cleanup_wallet_files(unique_wallet_filename)

    async def test_create_wallet_without_overwrite_on_existing_raises(
        self, started_wallet, unique_wallet_filename
    ):
        """Test that creating an existing wallet without overwrite raises."""
        try:
            await started_wallet.wallet_create(
                wallet_secret=TEST_WALLET_SECRET,
                filename=unique_wallet_filename,
                overwrite_wallet_storage=True,
            )
            await started_wallet.wallet_close()

            with pytest.raises(Exception):
                await started_wallet.wallet_create(
                    wallet_secret=TEST_WALLET_SECRET,
                    filename=unique_wallet_filename,
                    overwrite_wallet_storage=False,
                )
        finally:
            try:
                await started_wallet.wallet_close()
            except Exception:
                pass
            cleanup_wallet_files(unique_wallet_filename)


class TestWalletOpenClose:
    """Tests for Wallet.wallet_open and wallet_close."""

    async def test_open_after_close(self, open_wallet):
        """Test that a wallet can be closed and re-opened."""
        wallet, filename = open_wallet
        await wallet.wallet_close()
        assert wallet.is_open is False
        resp = await wallet.wallet_open(TEST_WALLET_SECRET, True, filename)
        assert isinstance(resp, dict)
        assert wallet.is_open is True

    async def test_open_with_wrong_password_raises(self, open_wallet):
        """Test opening a wallet with the wrong password raises."""
        wallet, filename = open_wallet
        await wallet.wallet_close()
        with pytest.raises(Exception):
            await wallet.wallet_open("wrong-password", False, filename)

    async def test_close_sets_is_open_false(self, open_wallet):
        """Test that wallet_close flips is_open to False."""
        wallet, _ = open_wallet
        await wallet.wallet_close()
        assert wallet.is_open is False
        assert wallet.descriptor is None


class TestWalletRename:
    """Tests for Wallet.wallet_rename."""

    async def test_rename_title(self, open_wallet):
        """Test renaming the wallet title only."""
        wallet, _ = open_wallet
        await wallet.wallet_rename(TEST_WALLET_SECRET, "renamed", None)
        assert wallet.descriptor.title == "renamed"

    async def test_rename_filename(self, open_wallet):
        """Test renaming both the wallet file on disk and reopening."""
        wallet, filename = open_wallet
        new_filename = filename + "-renamed"
        try:
            await wallet.wallet_rename(TEST_WALLET_SECRET, None, new_filename)
            assert wallet.descriptor.filename == new_filename
        finally:
            cleanup_wallet_files(new_filename)


class TestWalletChangeSecret:
    """Tests for Wallet.wallet_change_secret."""

    async def test_change_secret_allows_reopen_with_new_password(self, open_wallet):
        """Test changing the password works and the new password can reopen."""
        wallet, filename = open_wallet
        await wallet.wallet_change_secret(TEST_WALLET_SECRET, TEST_NEW_WALLET_SECRET)
        await wallet.wallet_close()

        await wallet.wallet_open(TEST_NEW_WALLET_SECRET, False, filename)
        assert wallet.is_open is True

    async def test_change_secret_with_wrong_old_password_raises(self, open_wallet):
        """Test that providing the wrong old password raises."""
        wallet, _ = open_wallet
        with pytest.raises(Exception):
            await wallet.wallet_change_secret("wrong-old", TEST_NEW_WALLET_SECRET)


class TestWalletExportImport:
    """Tests for Wallet.wallet_export and wallet_import."""

    async def test_export_returns_hex_string(self, open_wallet):
        """Test wallet_export returns a non-empty hex string."""
        wallet, _ = open_wallet
        exported = await wallet.wallet_export(TEST_WALLET_SECRET, False)
        assert isinstance(exported, str)
        assert len(exported) > 0
        # valid hex
        int(exported, 16)

    async def test_import_with_wrong_secret_raises(self, open_wallet):
        """Test that importing with the wrong secret raises."""
        wallet, _ = open_wallet
        exported = await wallet.wallet_export(TEST_WALLET_SECRET, False)
        await wallet.wallet_close()
        with pytest.raises(Exception):
            await wallet.wallet_import("wrong-password", exported)


class TestWalletReload:
    """Tests for Wallet.wallet_reload."""

    @pytest.mark.parametrize("reactivate", [False, True])
    async def test_reload(self, open_wallet, reactivate):
        """Test reloading an open wallet with both reactivate arg values."""
        wallet, _ = open_wallet
        await wallet.wallet_reload(reactivate)
        assert wallet.is_open is True


class TestWalletBatchFlush:
    """Tests for Wallet.batch / flush."""

    async def test_batch_then_flush(self, open_wallet):
        """Test that batch followed by flush completes without error."""
        wallet, _ = open_wallet
        await wallet.batch()
        await wallet.flush(TEST_WALLET_SECRET)

    async def test_flush_with_wrong_password_raises(self, open_wallet):
        """Test that flush with the wrong password raises."""
        wallet, _ = open_wallet
        await wallet.batch()
        with pytest.raises(Exception):
            await wallet.flush("wrong-password")


class TestWalletRetainContext:
    """Tests for Wallet.retain_context."""

    async def test_retain_context_with_bytes(self, open_wallet):
        """Test storing arbitrary bytes context data."""
        wallet, _ = open_wallet
        await wallet.retain_context("ctx-name", b"payload-bytes")

    async def test_retain_context_with_none_clears(self, open_wallet):
        """Test clearing a context entry with None data."""
        wallet, _ = open_wallet
        await wallet.retain_context("ctx-name", b"payload-bytes")
        await wallet.retain_context("ctx-name", None)


class TestWalletGetStatus:
    """Tests for Wallet.get_status."""

    async def test_get_status_default(self, open_wallet):
        """Test get_status returns a dict with expected keys."""
        wallet, filename = open_wallet
        status = await wallet.get_status()
        assert isinstance(status, dict)
        assert status.get("isOpen") is True
        assert status.get("networkId") == "testnet-10"
        descriptor = status.get("walletDescriptor") or {}
        assert descriptor.get("filename") == filename

    async def test_get_status_with_name(self, open_wallet):
        """Test get_status accepts an explicit name argument and reports the wallet."""
        wallet, filename = open_wallet
        status = await wallet.get_status(filename)
        descriptor = status.get("walletDescriptor") or {}
        assert descriptor.get("filename") == filename


class TestWalletAddressBook:
    """Tests for Wallet.address_book_enumerate."""

    async def test_address_book_enumerate_raises_not_implemented(self, open_wallet):
        """Test that address_book_enumerate currently raises NotImplemented upstream."""
        wallet, _ = open_wallet
        with pytest.raises(Exception):
            await wallet.address_book_enumerate()


class TestWalletFeeRatePoller:
    """Tests for Wallet.fee_rate_poller_enable / fee_rate_poller_disable."""

    async def test_fee_rate_poller_enable_and_disable(self, started_wallet):
        """Test enabling and disabling the fee-rate poller offline."""
        await started_wallet.fee_rate_poller_enable(5)
        await started_wallet.fee_rate_poller_disable()

    async def test_fee_rate_estimate_without_rpc_raises(self, started_wallet):
        """Test fee_rate_estimate raises when not connected to RPC."""
        with pytest.raises(Exception):
            await started_wallet.fee_rate_estimate()
