"""
Unit tests for PublicKeyGenerator and PrivateKeyGenerator classes.
"""

from kaspa import (
    PublicKeyGenerator,
    PrivateKeyGenerator,
    PublicKey,
    PrivateKey,
    Address,
)
from tests.conftest import TEST_MASTER_XPRV


class TestPublicKeyGeneratorCreation:
    """Tests for PublicKeyGenerator construction."""

    def test_create_from_master_xprv(self):
        """Test creating a PublicKeyGenerator from a master xprv string."""
        pubkey_gen = PublicKeyGenerator.from_master_xprv(
            TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )
        assert isinstance(pubkey_gen, PublicKeyGenerator)

    def test_create_from_master_xprv_different_account(self):
        """Test creating a PublicKeyGenerator for a different account index."""
        pubkey_gen = PublicKeyGenerator.from_master_xprv(
            TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=1
        )
        assert isinstance(pubkey_gen, PublicKeyGenerator)

    def test_create_from_xpub(self, known_xprv_from_mnemonic):
        """Test creating a PublicKeyGenerator from an xpub string."""
        # Derive to account level and get xpub
        account_xprv = known_xprv_from_mnemonic.derive_path("m/44'/111111'/0'")
        account_xpub = account_xprv.to_xpub()
        xpub_str = account_xpub.xpub

        pubkey_gen = PublicKeyGenerator.from_xpub(xpub_str)
        assert isinstance(pubkey_gen, PublicKeyGenerator)


class TestPublicKeyGeneratorReceiveKeys:
    """Tests for PublicKeyGenerator receive key generation."""

    def test_receive_pubkey_single(self):
        """Test generating a single receive public key."""
        pubkey_gen = PublicKeyGenerator.from_master_xprv(
            TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        pubkey = pubkey_gen.receive_pubkey(0)
        assert isinstance(pubkey, PublicKey)

    def test_receive_pubkeys_range(self):
        """Test generating a range of receive public keys."""
        pubkey_gen = PublicKeyGenerator.from_master_xprv(
            TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        pubkeys = pubkey_gen.receive_pubkeys(0, 10)
        assert len(pubkeys) == 10
        for key in pubkeys:
            assert isinstance(key, PublicKey)

    def test_receive_pubkey_as_string(self):
        """Test generating a receive public key as string."""
        pubkey_gen = PublicKeyGenerator.from_master_xprv(
            TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        key_str = pubkey_gen.receive_pubkey_as_string(0)
        assert isinstance(key_str, str)
        assert len(key_str) > 0

    def test_receive_pubkeys_as_strings(self):
        """Test generating multiple receive public keys as strings."""
        pubkey_gen = PublicKeyGenerator.from_master_xprv(
            TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        key_strs = pubkey_gen.receive_pubkeys_as_strings(0, 5)
        assert len(key_strs) == 5
        for key_str in key_strs:
            assert isinstance(key_str, str)


class TestPublicKeyGeneratorReceiveAddresses:
    """Tests for PublicKeyGenerator receive address generation."""

    def test_receive_address_single(self):
        """Test generating a single receive address."""
        pubkey_gen = PublicKeyGenerator.from_master_xprv(
            TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        address = pubkey_gen.receive_address("mainnet", 0)
        assert isinstance(address, Address)
        assert address.prefix == "kaspa"

    def test_receive_addresses_range(self):
        """Test generating a range of receive addresses."""
        pubkey_gen = PublicKeyGenerator.from_master_xprv(
            TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        addresses = pubkey_gen.receive_addresses("mainnet", 0, 10)
        assert len(addresses) == 10
        for addr in addresses:
            assert isinstance(addr, Address)
            assert addr.prefix == "kaspa"

    def test_receive_address_as_string(self):
        """Test generating a receive address as string."""
        pubkey_gen = PublicKeyGenerator.from_master_xprv(
            TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        addr_str = pubkey_gen.receive_address_as_string("mainnet", 0)
        assert addr_str.startswith("kaspa:")

    def test_receive_addresses_as_strings(self):
        """Test generating multiple receive addresses as strings."""
        pubkey_gen = PublicKeyGenerator.from_master_xprv(
            TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        addr_strs = pubkey_gen.receive_addresses_as_strings("mainnet", 0, 5)
        assert len(addr_strs) == 5
        for addr_str in addr_strs:
            assert addr_str.startswith("kaspa:")


class TestPublicKeyGeneratorChangeKeys:
    """Tests for PublicKeyGenerator change key generation."""

    def test_change_pubkey_single(self):
        """Test generating a single change public key."""
        pubkey_gen = PublicKeyGenerator.from_master_xprv(
            TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        pubkey = pubkey_gen.change_pubkey(0)
        assert isinstance(pubkey, PublicKey)

    def test_change_pubkeys_range(self):
        """Test generating a range of change public keys."""
        pubkey_gen = PublicKeyGenerator.from_master_xprv(
            TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        pubkeys = pubkey_gen.change_pubkeys(0, 10)
        assert len(pubkeys) == 10

    def test_change_address_single(self):
        """Test generating a single change address."""
        pubkey_gen = PublicKeyGenerator.from_master_xprv(
            TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        address = pubkey_gen.change_address("mainnet", 0)
        assert address.prefix == "kaspa"

    def test_change_addresses_range(self):
        """Test generating a range of change addresses."""
        pubkey_gen = PublicKeyGenerator.from_master_xprv(
            TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        addresses = pubkey_gen.change_addresses("mainnet", 0, 10)
        assert len(addresses) == 10


class TestPublicKeyGeneratorDifferentNetworks:
    """Tests for PublicKeyGenerator with different networks."""

    def test_receive_address_testnet(self):
        """Test generating testnet receive addresses."""
        pubkey_gen = PublicKeyGenerator.from_master_xprv(
            TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        address = pubkey_gen.receive_address("testnet", 0)
        assert address.prefix == "kaspatest"

    def test_change_address_testnet(self):
        """Test generating testnet change addresses."""
        pubkey_gen = PublicKeyGenerator.from_master_xprv(
            TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        address = pubkey_gen.change_address("testnet", 0)
        assert address.prefix == "kaspatest"


class TestPublicKeyGeneratorToString:
    """Tests for PublicKeyGenerator serialization."""

    def test_to_string(self):
        """Test serializing PublicKeyGenerator to string."""
        pubkey_gen = PublicKeyGenerator.from_master_xprv(
            TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        gen_str = pubkey_gen.to_string()
        assert isinstance(gen_str, str)


class TestPrivateKeyGeneratorCreation:
    """Tests for PrivateKeyGenerator construction."""

    def test_create_from_string(self):
        """Test creating a PrivateKeyGenerator from an xprv string."""
        privkey_gen = PrivateKeyGenerator(
            xprv=TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )
        assert privkey_gen is not None

    def test_create_from_xprv_instance(self, known_xprv_from_mnemonic):
        """Test creating a PrivateKeyGenerator from an XPrv instance."""
        privkey_gen = PrivateKeyGenerator(
            xprv=known_xprv_from_mnemonic,
            is_multisig=False,
            account_index=0
        )
        assert privkey_gen is not None

    def test_create_from_string_multisig(self):
        """Test creating a PrivateKeyGenerator for multisig from string."""
        privkey_gen = PrivateKeyGenerator(
            xprv=TEST_MASTER_XPRV,
            is_multisig=True,
            account_index=0,
            cosigner_index=0
        )
        assert privkey_gen is not None

    def test_create_from_xprv_instance_multisig(self, known_xprv_from_mnemonic):
        """Test creating a PrivateKeyGenerator for multisig from XPrv instance."""
        privkey_gen = PrivateKeyGenerator(
            xprv=known_xprv_from_mnemonic,
            is_multisig=True,
            account_index=0,
            cosigner_index=0
        )
        assert privkey_gen is not None

    def test_string_and_xprv_produce_same_keys(self, known_xprv_from_mnemonic):
        """Test that creating from string vs XPrv produces the same keys."""
        xprv_string = known_xprv_from_mnemonic.to_string()

        gen_from_string = PrivateKeyGenerator(
            xprv=xprv_string,
            is_multisig=False,
            account_index=0
        )
        gen_from_xprv = PrivateKeyGenerator(
            xprv=known_xprv_from_mnemonic,
            is_multisig=False,
            account_index=0
        )

        # Both should produce identical keys
        key_from_string = gen_from_string.receive_key(0)
        key_from_xprv = gen_from_xprv.receive_key(0)

        assert key_from_string.to_string() == key_from_xprv.to_string()


class TestPrivateKeyGeneratorKeys:
    """Tests for PrivateKeyGenerator key generation."""

    def test_receive_key(self):
        """Test generating a receive private key."""
        privkey_gen = PrivateKeyGenerator(
            xprv=TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        private_key = privkey_gen.receive_key(0)
        assert isinstance(private_key, PrivateKey)

    def test_change_key(self):
        """Test generating a change private key."""
        privkey_gen = PrivateKeyGenerator(
            xprv=TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        private_key = privkey_gen.change_key(0)
        assert isinstance(private_key, PrivateKey)

    def test_receive_key_different_indices(self):
        """Test that different indices produce different keys."""
        privkey_gen = PrivateKeyGenerator(
            xprv=TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        key0 = privkey_gen.receive_key(0)
        key1 = privkey_gen.receive_key(1)

        assert key0.to_string() != key1.to_string()


class TestKeyGeneratorConsistency:
    """Tests for consistency between PublicKeyGenerator and PrivateKeyGenerator."""

    def test_public_private_generators_produce_matching_keys(self):
        """Test that public and private generators produce matching key pairs."""
        pubkey_gen = PublicKeyGenerator.from_master_xprv(
            TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        privkey_gen = PrivateKeyGenerator(
            xprv=TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        # Get receive address from public key generator
        addr_from_pubgen = pubkey_gen.receive_address("mainnet", 0)

        # Get receive private key and derive address
        private_key = privkey_gen.receive_key(0)
        addr_from_privgen = private_key.to_address("mainnet")

        assert addr_from_pubgen.to_string() == addr_from_privgen.to_string()

    def test_change_keys_consistency(self):
        """Test consistency between change key generators."""
        pubkey_gen = PublicKeyGenerator.from_master_xprv(
            TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        privkey_gen = PrivateKeyGenerator(
            xprv=TEST_MASTER_XPRV,
            is_multisig=False,
            account_index=0
        )

        # Get change address from public key generator
        addr_from_pubgen = pubkey_gen.change_address("mainnet", 0)

        # Get change private key and derive address
        private_key = privkey_gen.change_key(0)
        addr_from_privgen = private_key.to_address("mainnet")

        assert addr_from_pubgen.to_string() == addr_from_privgen.to_string()
