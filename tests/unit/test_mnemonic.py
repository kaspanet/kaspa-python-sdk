"""
Unit tests for the Mnemonic class.
"""

import pytest

from kaspa import Mnemonic, Language
from tests.conftest import TEST_MNEMONIC_PHRASE


class TestMnemonicCreation:
    """Tests for Mnemonic construction."""

    def test_create_mnemonic_from_valid_phrase(self):
        """Test creating a Mnemonic from a valid 24-word phrase."""
        mnemonic = Mnemonic(phrase=TEST_MNEMONIC_PHRASE)
        assert isinstance(mnemonic, Mnemonic)
        assert mnemonic.phrase == TEST_MNEMONIC_PHRASE

    def test_create_mnemonic_from_invalid_phrase_raises(self):
        """Test that creating a Mnemonic from an invalid phrase raises an error."""
        with pytest.raises(Exception):
            Mnemonic(phrase="invalid mnemonic phrase that is not valid")

    def test_create_mnemonic_with_wrong_word_count_raises(self):
        """Test that a mnemonic with wrong word count raises an error."""
        with pytest.raises(Exception):
            Mnemonic(phrase="word1 word2 word3")

    def test_random_mnemonic_creates_valid_phrase(self):
        """Test that random() creates a valid Mnemonic."""
        mnemonic = Mnemonic.random()
        assert mnemonic.validate(mnemonic.phrase)

    def test_random_mnemonic_12_words(self):
        """Test creating a random 12-word mnemonic."""
        mnemonic = Mnemonic.random(word_count=12)
        words = mnemonic.phrase.split()
        assert len(words) == 12
        assert mnemonic.validate(mnemonic.phrase)

    def test_two_random_mnemonics_are_different(self):
        """Test that two random mnemonics are different."""
        mnemonic1 = Mnemonic.random()
        mnemonic2 = Mnemonic.random()
        assert mnemonic1.phrase != mnemonic2.phrase


class TestMnemonicValidation:
    """Tests for Mnemonic validation."""

    def test_validate_valid_phrase_returns_true(self):
        """Test that validate() returns True for a valid phrase."""
        assert Mnemonic.validate(TEST_MNEMONIC_PHRASE) is True

    def test_validate_invalid_phrase_returns_false(self):
        """Test that validate() returns False for an invalid phrase."""
        assert Mnemonic.validate("invalid phrase") is False

    def test_validate_with_language_parameter(self):
        """Test validate() with explicit Language parameter."""
        # English is the default/only supported language
        assert Mnemonic.validate(TEST_MNEMONIC_PHRASE,
                                 Language.English) is True


class TestMnemonicProperties:
    """Tests for Mnemonic properties."""

    def test_phrase_property(self, known_mnemonic):
        """Test that phrase property returns the mnemonic phrase."""
        assert known_mnemonic.phrase == TEST_MNEMONIC_PHRASE

    def test_entropy_property(self, known_mnemonic):
        """Test that entropy property returns a hex string."""
        entropy = known_mnemonic.entropy
        assert isinstance(entropy, str)
        # Entropy should be a hex string
        assert all(c in '0123456789abcdef' for c in entropy.lower())


class TestMnemonicSeed:
    """Tests for Mnemonic seed generation."""

    def test_to_seed_without_password(self, known_mnemonic):
        """Test generating a seed without a password."""
        seed = known_mnemonic.to_seed()
        assert isinstance(seed, str)
        # Seed should be a hex string (64 bytes = 128 hex chars)
        assert len(seed) == 128
        assert all(c in '0123456789abcdef' for c in seed.lower())

    def test_to_seed_with_password(self, known_mnemonic):
        """Test generating a seed with a password (25th word)."""
        seed = known_mnemonic.to_seed("my_password")
        assert isinstance(seed, str)
        assert len(seed) == 128

    def test_to_seed_with_empty_password(self, known_mnemonic):
        """Test generating a seed with an empty password."""
        seed = known_mnemonic.to_seed("")
        assert isinstance(seed, str)

    def test_same_mnemonic_same_seed(self):
        """Test that the same mnemonic produces the same seed."""
        mnemonic1 = Mnemonic(phrase=TEST_MNEMONIC_PHRASE)
        mnemonic2 = Mnemonic(phrase=TEST_MNEMONIC_PHRASE)

        seed1 = mnemonic1.to_seed()
        seed2 = mnemonic2.to_seed()

        assert seed1 == seed2

    def test_different_password_different_seed(self, known_mnemonic):
        """Test that different passwords produce different seeds."""
        seed1 = known_mnemonic.to_seed("password1")
        seed2 = known_mnemonic.to_seed("password2")

        assert seed1 != seed2

    def test_password_vs_no_password_different_seed(self, known_mnemonic):
        """Test that using a password produces a different seed than no password."""
        seed_no_password = known_mnemonic.to_seed()
        seed_with_password = known_mnemonic.to_seed("any_password")

        assert seed_no_password != seed_with_password


class TestMnemonicDeterminism:
    """Tests for deterministic behavior of mnemonics."""

    def test_known_mnemonic_produces_known_seed(self, known_mnemonic):
        """Test that the known mnemonic produces a consistent seed."""
        seed = known_mnemonic.to_seed()
        # The seed should be consistent across runs
        # We just verify it's a valid 64-byte hex string
        assert len(seed) == 128

        # Generate again and verify consistency
        seed2 = known_mnemonic.to_seed()
        assert seed == seed2
