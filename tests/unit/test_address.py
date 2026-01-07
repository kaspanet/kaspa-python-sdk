"""
Unit tests for the Address class.
"""

import pytest

from kaspa import Address, PublicKey, ScriptPublicKey, pay_to_address_script, address_from_script_public_key


class TestAddressCreation:
    """Tests for Address construction and validation."""

    def test_create_address_from_valid_mainnet_string(self, known_mainnet_address_string):
        """Test creating an Address from a valid mainnet address string."""
        address = Address(known_mainnet_address_string)
        assert address is not None
        assert address.to_string() == known_mainnet_address_string

    def test_create_address_from_valid_testnet_string(self):
        """Test creating an Address from a valid testnet address string."""
        # Testnet addresses have 'kaspatest:' prefix
        testnet_address_str = "kaspatest:qr0lr4ml9fn3chekrqmjdkergxl93l4wrk3dankcgvjq776s9wn9jhtkdksae"
        address = Address(testnet_address_str)
        assert address is not None
        assert address.prefix == "kaspatest"

    def test_create_address_from_invalid_string_raises(self):
        """Test that creating an Address from an invalid string raises an error."""
        with pytest.raises(Exception):
            Address("invalid_address_string")

    def test_validate_valid_address_returns_true(self, known_mainnet_address_string):
        """Test that validate() returns True for a valid address."""
        assert Address.validate(known_mainnet_address_string) is True

    def test_validate_invalid_address_returns_false(self):
        """Test that validate() returns False for an invalid address."""
        assert Address.validate("invalid_address") is False

    def test_validate_empty_string_returns_false(self):
        """Test that validate() returns False for an empty string."""
        assert Address.validate("") is False


class TestAddressProperties:
    """Tests for Address properties and methods."""

    def test_address_prefix_mainnet(self, known_mainnet_address):
        """Test that mainnet addresses have 'kaspa' prefix."""
        assert known_mainnet_address.prefix == "kaspa"

    def test_address_version(self, known_mainnet_address):
        """Test that address version is accessible."""
        version = known_mainnet_address.version
        assert version is not None
        assert isinstance(version, str)

    def test_address_to_string(self, known_mainnet_address, known_mainnet_address_string):
        """Test to_string() returns the original address string."""
        assert known_mainnet_address.to_string() == known_mainnet_address_string

    def test_address_payload(self, known_mainnet_address):
        """Test that payload property returns the bech32 encoded payload."""
        payload = known_mainnet_address.payload
        assert payload is not None
        assert isinstance(payload, str)
        assert len(payload) > 0
        # Payload should be the address without the prefix and colon
        full_address = known_mainnet_address.to_string()
        expected_payload = full_address.split(":")[1]
        assert payload == expected_payload

    def test_address_short(self, known_mainnet_address):
        """Test that short() returns a shortened address representation."""
        n = 4
        short_addr = known_mainnet_address.short(n)
        assert short_addr is not None
        assert isinstance(short_addr, str)
        # Should contain the prefix
        assert short_addr.startswith(known_mainnet_address.prefix + ":")
        # Should contain the ellipsis pattern
        assert "...." in short_addr

class TestAddressFromKey:
    """Tests for creating addresses from keys."""

    def test_address_from_public_key_mainnet(self, known_public_key):
        """Test creating a mainnet address from a public key."""
        address = known_public_key.to_address("mainnet")
        assert address is not None
        assert address.prefix == "kaspa"

    def test_address_from_public_key_testnet(self, known_public_key):
        """Test creating a testnet address from a public key."""
        address = known_public_key.to_address("testnet")
        assert address is not None
        assert address.prefix == "kaspatest"

    def test_address_from_private_key_mainnet(self, known_private_key):
        """Test creating a mainnet address from a private key."""
        address = known_private_key.to_address("mainnet")
        assert address is not None
        assert address.prefix == "kaspa"

    def test_address_from_keypair_mainnet(self, known_keypair):
        """Test creating a mainnet address from a keypair."""
        address = known_keypair.to_address("mainnet")
        assert address is not None
        assert address.prefix == "kaspa"

    def test_address_from_keypair_ecdsa(self, known_keypair):
        """Test creating an ECDSA address from a keypair."""
        address = known_keypair.to_address_ecdsa("mainnet")
        assert address is not None
        assert address.prefix == "kaspa"

    def test_private_key_and_keypair_produce_same_address(self, known_private_key, known_keypair):
        """Test that the same address is produced from private key and its keypair."""
        addr_from_privkey = known_private_key.to_address("mainnet")
        addr_from_keypair = known_keypair.to_address("mainnet")
        assert addr_from_privkey.to_string() == addr_from_keypair.to_string()


class TestScriptPublicKeyAddress:
    """Tests for address creation from ScriptPublicKey."""

    def test_pay_to_address_script(self, known_mainnet_address):
        """Test creating a ScriptPublicKey from an address."""
        spk = pay_to_address_script(known_mainnet_address)
        assert spk is not None
        assert isinstance(spk, ScriptPublicKey)

    def test_address_from_script_public_key_roundtrip(self, known_mainnet_address):
        """Test roundtrip: address -> ScriptPublicKey -> address."""
        spk = pay_to_address_script(known_mainnet_address)
        recovered_address = address_from_script_public_key(spk, "mainnet")
        assert recovered_address.to_string() == known_mainnet_address.to_string()

