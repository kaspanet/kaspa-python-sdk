"""
Unit tests for PrivateKey, PublicKey, and Keypair classes.
"""

import pytest

from kaspa import PrivateKey, PublicKey, Keypair, XOnlyPublicKey, Address


class TestPrivateKeyCreation:
    """Tests for PrivateKey construction."""

    def test_create_private_key_from_valid_hex(self, known_private_key_hex):
        """Test creating a PrivateKey from a valid hex string."""
        private_key = PrivateKey(known_private_key_hex)
        assert private_key is not None

    def test_create_private_key_from_invalid_hex_raises(self):
        """Test that creating a PrivateKey from invalid hex raises an error."""
        with pytest.raises(Exception):
            PrivateKey("not_a_valid_hex_string")

    def test_create_private_key_from_short_hex_raises(self):
        """Test that creating a PrivateKey from too short hex raises an error."""
        with pytest.raises(Exception):
            PrivateKey("abcd1234")

    def test_private_key_to_string(self, known_private_key, known_private_key_hex):
        """Test that to_string() returns the hex representation."""
        assert known_private_key.to_string() == known_private_key_hex


class TestPrivateKeyConversions:
    """Tests for PrivateKey conversion methods."""

    def test_private_key_to_public_key(self, known_private_key):
        """Test converting a private key to a public key."""
        public_key = known_private_key.to_public_key()
        assert public_key is not None
        assert isinstance(public_key, PublicKey)

    def test_private_key_to_keypair(self, known_private_key):
        """Test converting a private key to a keypair."""
        keypair = known_private_key.to_keypair()
        assert keypair is not None
        assert isinstance(keypair, Keypair)

    def test_private_key_to_address_mainnet(self, known_private_key):
        """Test generating a mainnet address from a private key."""
        address = known_private_key.to_address("mainnet")
        assert address is not None
        assert address.prefix == "kaspa"

    def test_private_key_to_address_testnet(self, known_private_key):
        """Test generating a testnet address from a private key."""
        address = known_private_key.to_address("testnet")
        assert address is not None
        assert address.prefix == "kaspatest"

    def test_private_key_to_address_ecdsa(self, known_private_key):
        """Test generating an ECDSA address from a private key."""
        address = known_private_key.to_address_ecdsa("mainnet")
        assert address is not None
        assert address.prefix == "kaspa"


class TestPublicKeyCreation:
    """Tests for PublicKey construction."""

    def test_create_public_key_from_x_only_hex(self, known_public_key_hex):
        """Test creating a PublicKey from an x-only (32-byte) hex string."""
        public_key = PublicKey(known_public_key_hex)
        assert public_key is not None

    def test_create_public_key_from_compressed_hex(self, known_compressed_public_key_hex):
        """Test creating a PublicKey from a compressed (33-byte) hex string."""
        public_key = PublicKey(known_compressed_public_key_hex)
        assert public_key is not None

    def test_create_public_key_from_full_der_hex(self):
        """Test creating a PublicKey from a full DER (65-byte) hex string."""
        full_der = "0421eb0c4270128b16c93c5f0dac48d56051a6237dae997b58912695052818e348b0a895cbd0c93a11ee7afac745929d96a4642a71831f54a7377893af71a2e2ae"
        public_key = PublicKey(full_der)
        assert public_key is not None

    def test_create_public_key_from_invalid_hex_raises(self):
        """Test that creating a PublicKey from invalid hex raises an error."""
        with pytest.raises(Exception):
            PublicKey("not_valid_hex")

    def test_public_key_to_string(self, known_public_key):
        """Test that to_string() returns a valid representation."""
        key_str = known_public_key.to_string()
        assert key_str is not None
        assert isinstance(key_str, str)
        assert len(key_str) > 0


class TestPublicKeyConversions:
    """Tests for PublicKey conversion methods."""

    def test_public_key_to_address_mainnet(self, known_public_key):
        """Test generating a mainnet address from a public key."""
        address = known_public_key.to_address("mainnet")
        assert address is not None
        assert address.prefix == "kaspa"

    def test_public_key_to_address_testnet(self, known_public_key):
        """Test generating a testnet address from a public key."""
        address = known_public_key.to_address("testnet")
        assert address is not None
        assert address.prefix == "kaspatest"

    def test_public_key_to_address_ecdsa(self, known_compressed_public_key_hex):
        """Test generating an ECDSA address from a compressed public key."""
        # ECDSA requires a compressed public key, not x-only
        public_key = PublicKey(known_compressed_public_key_hex)
        address = public_key.to_address_ecdsa("mainnet")
        assert address is not None

    def test_public_key_to_x_only_public_key(self, known_public_key):
        """Test converting a public key to an x-only public key."""
        x_only = known_public_key.to_x_only_public_key()
        assert x_only is not None
        assert isinstance(x_only, XOnlyPublicKey)


class TestXOnlyPublicKey:
    """Tests for XOnlyPublicKey class."""

    def test_create_x_only_public_key_from_hex(self, known_public_key_hex):
        """Test creating an XOnlyPublicKey from hex."""
        x_only = XOnlyPublicKey(known_public_key_hex)
        assert x_only is not None

    def test_x_only_public_key_to_string(self, known_public_key_hex):
        """Test XOnlyPublicKey to_string() method."""
        x_only = XOnlyPublicKey(known_public_key_hex)
        key_str = x_only.to_string()
        assert key_str is not None
        assert isinstance(key_str, str)

    def test_x_only_public_key_to_address(self, known_public_key_hex):
        """Test generating an address from an XOnlyPublicKey."""
        x_only = XOnlyPublicKey(known_public_key_hex)
        address = x_only.to_address("mainnet")
        assert address is not None
        assert address.prefix == "kaspa"

    def test_x_only_public_key_from_address(self, known_public_key):
        """Test creating an XOnlyPublicKey from an address."""
        address = known_public_key.to_address("mainnet")
        x_only = XOnlyPublicKey.from_address(address)
        assert x_only is not None


class TestKeypairCreation:
    """Tests for Keypair construction."""

    def test_keypair_random(self):
        """Test generating a random Keypair."""
        keypair = Keypair.random()
        assert keypair is not None
        assert keypair.private_key is not None
        assert keypair.public_key is not None

    def test_two_random_keypairs_are_different(self):
        """Test that two random keypairs are different."""
        keypair1 = Keypair.random()
        keypair2 = Keypair.random()
        assert keypair1.private_key != keypair2.private_key

    def test_keypair_from_private_key(self, known_private_key):
        """Test creating a Keypair from a PrivateKey."""
        keypair = Keypair.from_private_key(known_private_key)
        assert keypair is not None


class TestKeypairProperties:
    """Tests for Keypair properties."""

    def test_keypair_private_key_property(self, known_keypair):
        """Test accessing the private_key property."""
        private_key = known_keypair.private_key
        assert private_key is not None
        assert isinstance(private_key, str)

    def test_keypair_public_key_property(self, known_keypair):
        """Test accessing the public_key property."""
        public_key = known_keypair.public_key
        assert public_key is not None
        assert isinstance(public_key, str)

    def test_keypair_xonly_public_key_property(self, known_keypair):
        """Test accessing the xonly_public_key property."""
        xonly = known_keypair.xonly_public_key
        assert xonly is not None
        assert isinstance(xonly, str)


class TestKeypairAddresses:
    """Tests for Keypair address generation."""

    def test_keypair_to_address_mainnet(self, known_keypair):
        """Test generating a mainnet address from a keypair."""
        address = known_keypair.to_address("mainnet")
        assert address is not None
        assert address.prefix == "kaspa"

    def test_keypair_to_address_testnet(self, known_keypair):
        """Test generating a testnet address from a keypair."""
        address = known_keypair.to_address("testnet")
        assert address is not None
        assert address.prefix == "kaspatest"

    def test_keypair_to_address_ecdsa(self, known_keypair):
        """Test generating an ECDSA address from a keypair."""
        address = known_keypair.to_address_ecdsa("mainnet")
        assert address is not None


class TestKeyConsistency:
    """Tests for consistency between different key representations."""

    def test_private_key_public_key_address_consistency(self, known_private_key):
        """Test that derived keys produce the same address."""
        # Get address directly from private key
        addr1 = known_private_key.to_address("mainnet")
        
        # Get address via public key
        public_key = known_private_key.to_public_key()
        addr2 = public_key.to_address("mainnet")
        
        # Get address via keypair
        keypair = known_private_key.to_keypair()
        addr3 = keypair.to_address("mainnet")
        
        assert addr1.to_string() == addr2.to_string()
        assert addr2.to_string() == addr3.to_string()

    def test_keypair_private_key_matches_source(self, known_private_key, known_private_key_hex):
        """Test that a keypair's private key matches the source."""
        keypair = known_private_key.to_keypair()
        assert keypair.private_key == known_private_key_hex

