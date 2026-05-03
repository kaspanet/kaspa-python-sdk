"""
Integration tests for RPC-dependent Wallet methods.

Requires a reachable Kaspa node — run with `--network-id testnet-10 --rpc-url <ws-url>`.
The module skips entirely when `--rpc-url` is not supplied, so the unit tier
on a CI box without network access still passes.
"""

import asyncio

import pytest

from kaspa import (
    AccountDescriptor,
    AccountKind,
    AccountsDiscoveryKind,
    CommitRevealAddressKind,
    Fees,
    Wallet,
)

from tests.conftest import TEST_MNEMONIC_PHRASE, TEST_WALLET_SECRET
from tests.wallet_helpers import create_mnemonic_key, create_secret_key


# Most of these tests exercise failure paths where the wallet has no UTXOs
# and the binding should raise — there is no need for a funded account.
SYNC_TIMEOUT_SECONDS = 60


@pytest.fixture(autouse=True)
def _require_rpc_url(rpc_url):
    """Skip this module entirely when no --rpc-url is supplied."""
    if not rpc_url:
        pytest.skip("integration tests require --rpc-url")


async def _wait_for_sync(wallet: Wallet, timeout: float = SYNC_TIMEOUT_SECONDS) -> bool:
    """Poll wallet.is_synced until True or timeout elapses."""
    deadline = asyncio.get_event_loop().time() + timeout
    while asyncio.get_event_loop().time() < deadline:
        if wallet.is_synced:
            return True
        await asyncio.sleep(0.5)
    return wallet.is_synced


# =============================================================================
# connect / disconnect
# =============================================================================


class TestWalletConnect:
    """Tests for Wallet.connect and Wallet.disconnect."""

    async def test_connect_eventually_syncs(self, connected_wallet):
        """Test that a connected wallet eventually flips is_synced True."""
        wallet, _ = connected_wallet
        assert await _wait_for_sync(wallet) is True

    async def test_disconnect_clears_sync_state(self, connected_wallet):
        """Test that disconnect drops is_synced back to False."""
        wallet, _ = connected_wallet
        await _wait_for_sync(wallet)
        await wallet.disconnect()
        # is_synced is event-driven; allow a short grace period for the flip.
        for _ in range(20):
            if wallet.is_synced is False:
                break
            await asyncio.sleep(0.25)
        assert wallet.is_synced is False


# =============================================================================
# accounts_import_keypair
# =============================================================================


class TestAccountsImportKeypair:
    """Tests for Wallet.accounts_import_keypair (requires RPC)."""

    async def test_import_keypair_returns_keypair_descriptor(self, connected_wallet):
        """Test importing a SecretKey-kind prv key data yields a keypair account."""
        wallet, _ = connected_wallet
        pid = await create_secret_key(wallet)
        descriptor = await wallet.accounts_import_keypair(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
            ecdsa=False,
            account_name="imported-keypair",
        )
        assert isinstance(descriptor, AccountDescriptor)
        assert descriptor.kind == AccountKind("keypair")


# =============================================================================
# accounts_discovery
# =============================================================================


class TestAccountsDiscovery:
    """Tests for Wallet.accounts_discovery (requires RPC)."""

    async def test_discovery_with_known_mnemonic(self, connected_wallet):
        """Test running discovery against a known mnemonic returns an index."""
        wallet, _ = connected_wallet
        last_index = await wallet.accounts_discovery(
            discovery_kind=AccountsDiscoveryKind.Bip44,
            address_scan_extent=2,
            account_scan_extent=1,
            bip39_mnemonic=TEST_MNEMONIC_PHRASE,
        )
        assert isinstance(last_index, int)
        assert last_index >= 0


# =============================================================================
# accounts_estimate / accounts_send
# =============================================================================


class TestAccountsEstimateAndSend:
    """Tests for Wallet.accounts_estimate and Wallet.accounts_send (requires RPC)."""

    async def test_estimate_with_no_utxos_raises(self, connected_wallet):
        """Test estimate on a fresh (unfunded) account raises insufficient-funds."""
        wallet, _ = connected_wallet
        pid = await create_mnemonic_key(wallet)
        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
        )
        await wallet.accounts_activate([descriptor.account_id])
        with pytest.raises(Exception):
            await wallet.accounts_estimate(
                account_id=descriptor.account_id,
                priority_fee_sompi=Fees(0),
            )

    async def test_send_with_no_utxos_raises(self, connected_wallet):
        """Test send from a fresh (unfunded) account raises insufficient-funds."""
        wallet, _ = connected_wallet
        pid = await create_mnemonic_key(wallet)
        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
        )
        await wallet.accounts_activate([descriptor.account_id])
        with pytest.raises(Exception):
            await wallet.accounts_send(
                wallet_secret=TEST_WALLET_SECRET,
                account_id=descriptor.account_id,
                priority_fee_sompi=Fees(0),
            )


# =============================================================================
# accounts_get_utxos
# =============================================================================


class TestAccountsGetUtxos:
    """Tests for Wallet.accounts_get_utxos (requires RPC)."""

    async def test_get_utxos_on_fresh_account_returns_empty(self, connected_wallet):
        """Test get_utxos on a freshly created account returns an empty list."""
        wallet, _ = connected_wallet
        pid = await create_mnemonic_key(wallet)
        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
        )
        await wallet.accounts_activate([descriptor.account_id])
        utxos = await wallet.accounts_get_utxos(descriptor.account_id)
        assert utxos == []


# =============================================================================
# accounts_transfer
# =============================================================================


class TestAccountsTransfer:
    """Tests for Wallet.accounts_transfer (requires RPC)."""

    async def test_transfer_with_no_utxos_raises(self, connected_wallet):
        """Test transferring between two unfunded accounts raises insufficient-funds."""
        wallet, _ = connected_wallet
        pid = await create_mnemonic_key(wallet)
        source = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
            account_name="source",
            account_index=0,
        )
        destination = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
            account_name="destination",
            account_index=1,
        )
        await wallet.accounts_activate([source.account_id, destination.account_id])
        with pytest.raises(Exception):
            await wallet.accounts_transfer(
                wallet_secret=TEST_WALLET_SECRET,
                source_account_id=source.account_id,
                destination_account_id=destination.account_id,
                transfer_amount_sompi=1,
            )


# =============================================================================
# accounts_commit_reveal / accounts_commit_reveal_manual
# =============================================================================


class TestAccountsCommitReveal:
    """Tests for Wallet.accounts_commit_reveal and _manual (requires RPC)."""

    async def test_commit_reveal_on_fresh_account_raises(self, connected_wallet):
        """Test commit_reveal on an unfunded account raises."""
        wallet, _ = connected_wallet
        pid = await create_mnemonic_key(wallet)
        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
        )
        await wallet.accounts_activate([descriptor.account_id])
        with pytest.raises(Exception):
            await wallet.accounts_commit_reveal(
                wallet_secret=TEST_WALLET_SECRET,
                account_id=descriptor.account_id,
                address_type=CommitRevealAddressKind.Receive,
                address_index=0,
                script_sig=b"\x00",
                commit_amount_sompi=1000,
                reveal_fee_sompi=100,
            )

    async def test_commit_reveal_manual_on_fresh_account_raises(self, connected_wallet):
        """Test commit_reveal_manual on an unfunded account raises."""
        wallet, _ = connected_wallet
        pid = await create_mnemonic_key(wallet)
        descriptor = await wallet.accounts_create_bip32(
            wallet_secret=TEST_WALLET_SECRET,
            prv_key_data_id=pid,
        )
        await wallet.accounts_activate([descriptor.account_id])
        with pytest.raises(Exception):
            await wallet.accounts_commit_reveal_manual(
                wallet_secret=TEST_WALLET_SECRET,
                account_id=descriptor.account_id,
                script_sig=b"\x00",
                reveal_fee_sompi=100,
            )
