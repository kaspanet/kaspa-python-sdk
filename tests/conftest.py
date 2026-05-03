"""
Shared fixtures for Kaspa Python SDK tests.
"""

import os
import shutil
import uuid

import pytest
import pytest_asyncio

from kaspa import (
    Mnemonic,
    PrivateKey,
    PublicKey,
    Keypair,
    XPrv,
    Address,
    RpcClient,
    Resolver,
    Wallet,
)


def pytest_addoption(parser):
    parser.addoption(
        "--network-id",
        default="mainnet",
        help="Kaspa network ID (default: mainnet)",
    )
    parser.addoption(
        "--rpc-url",
        default=None,
        help="Direct RPC server WebSocket URL (bypasses resolver)",
    )


# =============================================================================
# Test Vectors - Deterministic values for reproducible tests
# =============================================================================

TEST_MNEMONIC_PHRASE = (
    "hunt bitter praise lift buyer topic crane leopard uniform network inquiry over "
    "grain pass match crush marine strike doll relax fortune trumpet sunny silk"
)

TEST_PRIVATE_KEY_HEX = "b7e151628aed2a6abf7158809cf4f3c762e7160f38b4da56a784d9045190cfef"

TEST_PUBLIC_KEY_HEX = "dff1d77f2a671c5f36183726db2341be58feae1da2deced843240f7b502ba659"

TEST_COMPRESSED_PUBLIC_KEY_HEX = "02dff1d77f2a671c5f36183726db2341be58feae1da2deced843240f7b502ba659"

TEST_MASTER_XPRV = (
    "kprv5y2qurMHCsXYrNfU3GCihuwG3vMqFji7PZXajMEqyBkNh9UZUJgoHYBLTKu1eM4MvUtomcXPQ3Sw9HZ5ebbM4byoUciHo1zrPJBQfqpLorQ"
)

# Burn address
TEST_MAINNET_ADDRESS = "kaspa:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqkx9awp4e"


# =============================================================================
# Mnemonic Fixtures
# =============================================================================

@pytest.fixture
def known_mnemonic() -> Mnemonic:
    """Return a Mnemonic object from the known test phrase."""
    return Mnemonic(phrase=TEST_MNEMONIC_PHRASE)


# =============================================================================
# Key Fixtures
# =============================================================================

@pytest.fixture
def known_private_key() -> PrivateKey:
    """Return a PrivateKey object from the known test hex."""
    return PrivateKey(TEST_PRIVATE_KEY_HEX)


@pytest.fixture
def known_public_key() -> PublicKey:
    """Return a PublicKey object from the known test hex."""
    return PublicKey(TEST_PUBLIC_KEY_HEX)


@pytest.fixture
def known_keypair(known_private_key) -> Keypair:
    """Return a Keypair derived from the known private key."""
    return known_private_key.to_keypair()


# =============================================================================
# XPrv/XPub Fixtures
# =============================================================================

@pytest.fixture
def known_xprv_from_mnemonic(known_mnemonic) -> XPrv:
    """Return an XPrv derived from the known mnemonic seed."""
    seed = known_mnemonic.to_seed()
    return XPrv(seed)


# =============================================================================
# Address Fixtures
# =============================================================================

@pytest.fixture
def known_mainnet_address() -> Address:
    """Return an Address object from the known mainnet address."""
    return Address(TEST_MAINNET_ADDRESS)


# =============================================================================
# Integration Test Fixtures (Network Required)
# =============================================================================

@pytest.fixture(scope="session")
def network_id(request):
    return request.config.getoption("--network-id")


@pytest.fixture(scope="session")
def rpc_url(request):
    return request.config.getoption("--rpc-url")


@pytest_asyncio.fixture(scope="session")
async def rpc_client(network_id, rpc_url):
    """
    Session-scoped async fixture for RPC client.

    This fixture is used for integration tests that require network access.
    Configure via pytest options: --network-id and --rpc-url.
    """
    if rpc_url:
        client = RpcClient(url=rpc_url, network_id=network_id)
    else:
        client = RpcClient(resolver=Resolver(), network_id=network_id)
    await client.connect()
    yield client
    await client.disconnect()


@pytest.fixture(scope="session")
def test_address(network_id):
    """Address for the currently targeted network.

    Returns a testnet address for testnet-* network IDs, otherwise mainnet.
    """
    if network_id.startswith("testnet"):
        return "kaspatest:qr0lr4ml9fn3chekrqmjdkergxl93l4wrk3dankcgvjq776s9wn9jhtkdksae"
    return TEST_MAINNET_ADDRESS


# =============================================================================
# Wallet Fixtures (Unit-test scope — no network required)
# =============================================================================

# Default passwords used across wallet unit tests.
TEST_WALLET_SECRET = "test-wallet-secret"
TEST_NEW_WALLET_SECRET = "test-wallet-secret-new"

# Directory where the local wallet store persists wallet files.
_KASPA_LOCAL_STORE_DIR = os.path.expanduser("~/.kaspa")


def cleanup_wallet_files(filename: str) -> None:
    """Remove any wallet/transaction artifacts left by a test run.

    Sweeps both `~/.kaspa/` (default LocalStore folder) and the current
    working directory. The CWD sweep covers `wallet_rename` with a bare
    filename — upstream `Storage::rename_sync` treats the argument as a
    plain `PathBuf`, so the renamed file lands in the pytest CWD, not
    `~/.kaspa/`.
    """
    root = os.path.join(_KASPA_LOCAL_STORE_DIR, filename)
    cwd_root = os.path.join(os.getcwd(), filename)
    candidates = (
        f"{root}.wallet",
        f"{root}.transactions",
        root,
        cwd_root,
        f"{cwd_root}.wallet",
        f"{cwd_root}.transactions",
    )
    for candidate in candidates:
        if os.path.isdir(candidate):
            shutil.rmtree(candidate, ignore_errors=True)
        else:
            try:
                os.remove(candidate)
            except (FileNotFoundError, IsADirectoryError, PermissionError):
                pass


@pytest.fixture
def unique_wallet_filename() -> str:
    """Return a unique wallet filename (without extension) for isolated tests."""
    return f"unit-test-{uuid.uuid4().hex[:12]}"


@pytest_asyncio.fixture
async def started_wallet():
    """A started Wallet (testnet-10) with no wallet file opened.

    Yields the Wallet; `stop()` is awaited on teardown.
    """
    wallet = Wallet(network_id="testnet-10", resolver=Resolver())
    await wallet.start()
    try:
        yield wallet
    finally:
        if wallet.is_open:
            await wallet.wallet_close()
        await wallet.stop()


@pytest_asyncio.fixture
async def open_wallet(unique_wallet_filename):
    """A started Wallet with a freshly created wallet file opened.

    Yields (wallet, filename). Cleans up the wallet file on teardown.
    """
    wallet = Wallet(network_id="testnet-10", resolver=Resolver())
    await wallet.start()
    # The wallet title is also used as the filename when the wallet is
    # re-imported from an exported payload; keep it unique per-test so
    # export/import flows don't collide with other tests on disk.
    title = unique_wallet_filename
    try:
        await wallet.wallet_create(
            wallet_secret=TEST_WALLET_SECRET,
            filename=unique_wallet_filename,
            overwrite_wallet_storage=True,
            title=title,
        )
        yield wallet, unique_wallet_filename
    finally:
        if wallet.is_open:
            await wallet.wallet_close()
        await wallet.stop()
        cleanup_wallet_files(unique_wallet_filename)
        cleanup_wallet_files(title)


@pytest_asyncio.fixture
async def connected_wallet(unique_wallet_filename, network_id, rpc_url):
    """A started Wallet with a fresh wallet file opened and RPC connected.

    Yields (wallet, filename). Disconnects + closes + cleans up on teardown.
    Requires `--rpc-url` to be passed (integration tier).
    """
    wallet = Wallet(network_id=network_id, url=rpc_url)
    await wallet.start()
    title = unique_wallet_filename
    try:
        await wallet.wallet_create(
            wallet_secret=TEST_WALLET_SECRET,
            filename=unique_wallet_filename,
            overwrite_wallet_storage=True,
            title=title,
        )
        await wallet.connect(block_async_connect=True)
        yield wallet, unique_wallet_filename
    finally:
        if wallet.rpc.is_connected:
            await wallet.disconnect()
        if wallet.is_open:
            await wallet.wallet_close()
        await wallet.stop()
        cleanup_wallet_files(unique_wallet_filename)
        cleanup_wallet_files(title)
