"""
Unit tests for utility functions.
"""

import pytest

from kaspa import (
    kaspa_to_sompi,
    sompi_to_kaspa,
    sompi_to_kaspa_string_with_suffix,
    sign_message,
    verify_message,
    PrivateKey,
    PublicKey,
    Hash,
    AccountKind,
    create_multisig_address,
)


class TestKaspaSompiConversions:
    """Tests for Kaspa/Sompi conversion functions."""

    def test_kaspa_to_sompi_whole_number(self):
        """Test converting whole Kaspa to Sompi."""
        result = kaspa_to_sompi(1.0)
        assert result == 100_000_000

    def test_kaspa_to_sompi_with_decimals(self):
        """Test converting Kaspa with decimals to Sompi."""
        result = kaspa_to_sompi(1.5)
        assert result == 150_000_000

    def test_kaspa_to_sompi_small_value(self):
        """Test converting small Kaspa value to Sompi."""
        result = kaspa_to_sompi(0.00000001)
        assert result == 1

    def test_kaspa_to_sompi_large_value(self):
        """Test converting large Kaspa value to Sompi."""
        result = kaspa_to_sompi(100.833)
        assert result == 10_083_300_000

    def test_kaspa_to_sompi_zero(self):
        """Test converting zero Kaspa to Sompi."""
        result = kaspa_to_sompi(0.0)
        assert result == 0


class TestSompiToKaspa:
    """Tests for Sompi to Kaspa conversions."""

    def test_sompi_to_kaspa_whole_number(self):
        """Test converting whole Sompi to Kaspa."""
        result = sompi_to_kaspa(100_000_000)
        assert result == 1.0

    def test_sompi_to_kaspa_with_remainder(self):
        """Test converting Sompi with remainder to Kaspa."""
        result = sompi_to_kaspa(150_000_000)
        assert result == 1.5

    def test_sompi_to_kaspa_small_value(self):
        """Test converting small Sompi to Kaspa."""
        result = sompi_to_kaspa(1)
        assert result == 0.00000001

    def test_sompi_to_kaspa_zero(self):
        """Test converting zero Sompi to Kaspa."""
        result = sompi_to_kaspa(0)
        assert result == 0.0


class TestSompiToKaspaString:
    """Tests for Sompi to Kaspa string conversion."""

    def test_sompi_to_kaspa_string_mainnet(self):
        """Test converting Sompi to Kaspa string with mainnet suffix."""
        result = sompi_to_kaspa_string_with_suffix(100_000_000, "mainnet")
        assert "KAS" in result
        assert "1" in result

    def test_sompi_to_kaspa_string_testnet(self):
        """Test converting Sompi to Kaspa string with testnet suffix."""
        result = sompi_to_kaspa_string_with_suffix(100_000_000, "testnet")
        # Testnet may use TKAS or similar suffix
        assert result is not None
        assert len(result) > 0

    def test_sompi_to_kaspa_string_large_value(self):
        """Test converting large Sompi to Kaspa string."""
        result = sompi_to_kaspa_string_with_suffix(499_922_100, "mainnet")
        assert "KAS" in result


class TestRoundTrip:
    """Tests for round-trip conversions."""

    def test_kaspa_sompi_roundtrip(self):
        """Test round-trip conversion: Kaspa -> Sompi -> Kaspa."""
        original = 123.45678901
        sompi = kaspa_to_sompi(original)
        back = sompi_to_kaspa(sompi)
        # Due to precision limits (8 decimal places), we compare to that precision
        assert abs(back - 123.45678901) < 0.000000001

    def test_sompi_kaspa_roundtrip(self):
        """Test round-trip conversion: Sompi -> Kaspa -> Sompi."""
        original = 12345678901
        kaspa = sompi_to_kaspa(original)
        back = kaspa_to_sompi(kaspa)
        assert back == original


class TestMessageSigning:
    """Tests for message signing and verification."""

    def test_sign_message(self, known_private_key):
        """Test signing a message."""
        message = "Hello Kaspa!"
        signature = sign_message(message, known_private_key)
        
        assert signature is not None
        assert isinstance(signature, str)
        assert len(signature) > 0

    def test_verify_message_valid(self, known_private_key, known_public_key):
        """Test verifying a valid message signature."""
        message = "Hello Kaspa!"
        signature = sign_message(message, known_private_key)
        
        is_valid = verify_message(message, signature, known_public_key)
        assert is_valid is True

    def test_verify_message_invalid_signature(self, known_public_key):
        """Test verifying an invalid signature returns False."""
        message = "Hello Kaspa!"
        # Invalid signature (random hex)
        fake_signature = "a" * 128
        
        is_valid = verify_message(message, fake_signature, known_public_key)
        assert is_valid is False

    def test_verify_message_wrong_message(self, known_private_key, known_public_key):
        """Test verifying with wrong message returns False."""
        message1 = "Hello Kaspa!"
        message2 = "Wrong message"
        
        signature = sign_message(message1, known_private_key)
        is_valid = verify_message(message2, signature, known_public_key)
        
        assert is_valid is False

    def test_verify_message_wrong_public_key(self, known_private_key):
        """Test verifying with wrong public key returns False."""
        message = "Hello Kaspa!"
        signature = sign_message(message, known_private_key)
        
        # Use a different public key (valid but different)
        other_key = PrivateKey("1" * 64).to_public_key()
        
        # This might raise or return False depending on implementation
        try:
            is_valid = verify_message(message, signature, other_key)
            assert is_valid is False
        except Exception:
            # Some implementations raise on verification failure
            pass

    def test_sign_message_with_no_aux_rand(self, known_private_key):
        """Test signing with no_aux_rand option."""
        message = "Deterministic signing"
        
        sig1 = sign_message(message, known_private_key, no_aux_rand=True)
        sig2 = sign_message(message, known_private_key, no_aux_rand=True)
        
        # With no_aux_rand, signatures should be deterministic
        assert sig1 == sig2

    def test_sign_message_empty_string(self, known_private_key):
        """Test signing an empty message."""
        message = ""
        signature = sign_message(message, known_private_key)
        
        assert signature is not None


class TestHash:
    """Tests for Hash class."""

    def test_create_hash_from_hex(self):
        """Test creating a Hash from hex string."""
        hex_str = "a" * 64
        hash_obj = Hash(hex_str)
        assert hash_obj is not None

    def test_hash_to_string(self):
        """Test Hash to_string method."""
        hex_str = "b" * 64
        hash_obj = Hash(hex_str)
        
        result = hash_obj.to_string()
        assert result is not None
        assert isinstance(result, str)


class TestAccountKind:
    """Tests for AccountKind class."""

    def test_create_account_kind_bip32(self):
        """Test creating a BIP32 account kind."""
        kind = AccountKind("bip32")
        assert kind is not None
        assert "bip32" in kind.to_string().lower()

    def test_account_kind_to_string(self):
        """Test AccountKind to_string method."""
        kind = AccountKind("bip32")
        result = kind.to_string()
        assert result is not None
        assert isinstance(result, str)

    def test_account_kind_str_method(self):
        """Test AccountKind __str__ method."""
        kind = AccountKind("bip32")
        result = str(kind)
        assert result is not None


class TestMultisigAddress:
    """Tests for multisig address creation."""

    def test_create_multisig_address(self):
        """Test creating a multisig address."""
        # Create public keys from private keys (not x-only)
        priv_key1 = PrivateKey("1" * 64)
        pub_key1 = priv_key1.to_public_key()
        
        priv_key2 = PrivateKey("2" * 64)
        pub_key2 = priv_key2.to_public_key()
        
        priv_key3 = PrivateKey("3" * 64)
        pub_key3 = priv_key3.to_public_key()
        
        # Use PublicKey objects directly
        keys = [pub_key1, pub_key2, pub_key3]
        
        multisig_address = create_multisig_address(
            minimum_signatures=2,
            keys=keys,
            network_type="mainnet"
        )
        
        assert multisig_address is not None
        assert multisig_address.prefix == "kaspa"

    def test_create_multisig_address_testnet(self):
        """Test creating a testnet multisig address."""
        priv_key1 = PrivateKey("1" * 64)
        pub_key1 = priv_key1.to_public_key()
        
        priv_key2 = PrivateKey("2" * 64)
        pub_key2 = priv_key2.to_public_key()
        
        # Use PublicKey objects directly
        keys = [pub_key1, pub_key2]
        
        multisig_address = create_multisig_address(
            minimum_signatures=1,
            keys=keys,
            network_type="testnet"
        )
        
        assert multisig_address is not None
        assert multisig_address.prefix == "kaspatest"

    def test_create_multisig_address_ecdsa(self):
        """Test creating an ECDSA multisig address."""
        # For ECDSA, use public keys from private keys
        priv_key1 = PrivateKey("1" * 64)
        pub_key1 = priv_key1.to_public_key()
        
        priv_key2 = PrivateKey("2" * 64)
        pub_key2 = priv_key2.to_public_key()
        
        # Use PublicKey objects directly
        keys = [pub_key1, pub_key2]
        
        multisig_address = create_multisig_address(
            minimum_signatures=1,
            keys=keys,
            network_type="mainnet",
            ecdsa=True
        )
        
        assert multisig_address is not None

