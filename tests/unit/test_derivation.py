"""
Unit tests for XPrv, XPub, and DerivationPath classes.
"""

import pytest

from kaspa import XPrv, XPub, DerivationPath, Mnemonic, PrivateKey, PublicKey


class TestXPrvCreation:
    """Tests for XPrv construction."""

    def test_create_xprv_from_seed(self, known_mnemonic):
        """Test creating an XPrv from a mnemonic seed."""
        seed = known_mnemonic.to_seed()
        xprv = XPrv(seed)
        assert xprv is not None

    def test_create_xprv_from_xprv_string(self, known_master_xprv_string):
        """Test creating an XPrv from an xprv string."""
        xprv = XPrv.from_xprv(known_master_xprv_string)
        assert xprv is not None

    def test_create_xprv_from_invalid_seed_raises(self):
        """Test that creating an XPrv from an invalid seed raises an error."""
        with pytest.raises(Exception):
            XPrv("invalid_seed")


class TestXPrvProperties:
    """Tests for XPrv properties."""

    def test_xprv_property(self, known_xprv_from_mnemonic):
        """Test accessing the xprv property."""
        xprv_str = known_xprv_from_mnemonic.xprv
        assert xprv_str is not None
        assert isinstance(xprv_str, str)
        # Default prefix is 'kprv'
        assert xprv_str.startswith("kprv")

    def test_xprv_private_key_property(self, known_xprv_from_mnemonic):
        """Test accessing the private_key property."""
        private_key = known_xprv_from_mnemonic.private_key
        assert private_key is not None
        assert isinstance(private_key, str)

    def test_xprv_depth_property(self, known_xprv_from_mnemonic):
        """Test accessing the depth property."""
        depth = known_xprv_from_mnemonic.depth
        assert depth == 0  # Master key has depth 0

    def test_xprv_chain_code_property(self, known_xprv_from_mnemonic):
        """Test accessing the chain_code property."""
        chain_code = known_xprv_from_mnemonic.chain_code
        assert chain_code is not None
        assert isinstance(chain_code, str)

    def test_xprv_parent_fingerprint_property(self, known_xprv_from_mnemonic):
        """Test accessing the parent_fingerprint property."""
        fingerprint = known_xprv_from_mnemonic.parent_fingerprint
        assert fingerprint is not None

    def test_xprv_child_number_property(self, known_xprv_from_mnemonic):
        """Test accessing the child_number property."""
        child_number = known_xprv_from_mnemonic.child_number
        assert child_number == 0  # Master key has child number 0


class TestXPrvDerivation:
    """Tests for XPrv derivation."""

    def test_derive_child_hardened(self, known_xprv_from_mnemonic):
        """Test deriving a hardened child key."""
        child = known_xprv_from_mnemonic.derive_child(0, True)
        assert child is not None
        assert child.depth == 1

    def test_derive_child_normal(self, known_xprv_from_mnemonic):
        """Test deriving a normal (non-hardened) child key."""
        child = known_xprv_from_mnemonic.derive_child(0, False)
        assert child is not None
        assert child.depth == 1

    def test_derive_path_string(self, known_xprv_from_mnemonic):
        """Test deriving keys using a path string."""
        # Standard Kaspa derivation path: m/44'/111111'/0'
        derived = known_xprv_from_mnemonic.derive_path("m/44'/111111'/0'")
        assert derived is not None
        assert derived.depth == 3

    def test_derive_path_object(self, known_xprv_from_mnemonic):
        """Test deriving keys using a DerivationPath object."""
        path = DerivationPath("m/44'/111111'/0'")
        derived = known_xprv_from_mnemonic.derive_path(path)
        assert derived is not None
        assert derived.depth == 3

    def test_derive_full_receive_path(self, known_xprv_from_mnemonic):
        """Test deriving a full receive address path."""
        # m/44'/111111'/0'/0/0 - first receive address
        derived = known_xprv_from_mnemonic.derive_path("m/44'/111111'/0'/0/0")
        assert derived is not None
        assert derived.depth == 5

    def test_derive_full_change_path(self, known_xprv_from_mnemonic):
        """Test deriving a full change address path."""
        # m/44'/111111'/0'/1/0 - first change address
        derived = known_xprv_from_mnemonic.derive_path("m/44'/111111'/0'/1/0")
        assert derived is not None
        assert derived.depth == 5


class TestXPrvConversions:
    """Tests for XPrv conversion methods."""

    def test_xprv_to_xpub(self, known_xprv_from_mnemonic):
        """Test converting XPrv to XPub."""
        xpub = known_xprv_from_mnemonic.to_xpub()
        assert xpub is not None
        assert isinstance(xpub, XPub)

    def test_xprv_to_private_key(self, known_xprv_from_mnemonic):
        """Test converting XPrv to PrivateKey."""
        private_key = known_xprv_from_mnemonic.to_private_key()
        assert private_key is not None
        assert isinstance(private_key, PrivateKey)

    def test_xprv_to_string(self, known_xprv_from_mnemonic):
        """Test XPrv to_string() method."""
        xprv_str = known_xprv_from_mnemonic.to_string()
        assert xprv_str is not None
        assert xprv_str.startswith("kprv")

    def test_xprv_into_string_with_prefix(self, known_xprv_from_mnemonic):
        """Test XPrv into_string() with custom prefix."""
        ktrv_str = known_xprv_from_mnemonic.into_string("ktrv")
        assert ktrv_str.startswith("ktrv")
        
        xprv_str = known_xprv_from_mnemonic.into_string("xprv")
        assert xprv_str.startswith("xprv")


class TestXPubCreation:
    """Tests for XPub construction."""

    def test_create_xpub_from_xprv(self, known_xprv_from_mnemonic):
        """Test creating an XPub from an XPrv."""
        xpub = known_xprv_from_mnemonic.to_xpub()
        assert xpub is not None

    def test_create_xpub_from_string(self, known_xprv_from_mnemonic):
        """Test creating an XPub from an xpub string."""
        xpub = known_xprv_from_mnemonic.to_xpub()
        xpub_str = xpub.xpub
        
        xpub2 = XPub(xpub_str)
        assert xpub2 is not None


class TestXPubProperties:
    """Tests for XPub properties."""

    def test_xpub_property(self, known_xprv_from_mnemonic):
        """Test accessing the xpub property."""
        xpub = known_xprv_from_mnemonic.to_xpub()
        xpub_str = xpub.xpub
        assert xpub_str is not None
        assert isinstance(xpub_str, str)
        assert xpub_str.startswith("kpub")

    def test_xpub_depth_property(self, known_xprv_from_mnemonic):
        """Test accessing the depth property."""
        xpub = known_xprv_from_mnemonic.to_xpub()
        assert xpub.depth == 0

    def test_xpub_chain_code_property(self, known_xprv_from_mnemonic):
        """Test accessing the chain_code property."""
        xpub = known_xprv_from_mnemonic.to_xpub()
        chain_code = xpub.chain_code
        assert chain_code is not None


class TestXPubDerivation:
    """Tests for XPub derivation."""

    def test_derive_child_normal(self, known_xprv_from_mnemonic):
        """Test deriving a normal (non-hardened) child from XPub."""
        xpub = known_xprv_from_mnemonic.to_xpub()
        child = xpub.derive_child(0, False)
        assert child is not None
        assert child.depth == 1

    def test_derive_path(self, known_xprv_from_mnemonic):
        """Test deriving using a path string from XPub."""
        xpub = known_xprv_from_mnemonic.to_xpub()
        derived = xpub.derive_path("m/0/1")
        assert derived is not None
        assert derived.depth == 2


class TestXPubConversions:
    """Tests for XPub conversion methods."""

    def test_xpub_to_public_key(self, known_xprv_from_mnemonic):
        """Test converting XPub to PublicKey."""
        xpub = known_xprv_from_mnemonic.to_xpub()
        public_key = xpub.to_public_key()
        assert public_key is not None
        assert isinstance(public_key, PublicKey)

    def test_xpub_to_str_with_prefix(self, known_xprv_from_mnemonic):
        """Test XPub into_string() with custom prefix."""
        xpub = known_xprv_from_mnemonic.to_xpub()
        xpub_str = xpub.into_string("xpub")
        assert xpub_str.startswith("xpub")


class TestDerivationPath:
    """Tests for DerivationPath class."""

    def test_create_derivation_path(self):
        """Test creating a DerivationPath."""
        path = DerivationPath("m/44'/111111'/0'")
        assert path is not None

    def test_derivation_path_to_string(self):
        """Test DerivationPath to_string() method."""
        path = DerivationPath("m/44'/111111'/0'")
        path_str = path.to_string()
        assert "44'" in path_str
        assert "111111'" in path_str

    def test_derivation_path_is_empty(self):
        """Test is_empty() method."""
        path = DerivationPath("m")
        assert path.is_empty() is True
        
        path2 = DerivationPath("m/0")
        assert path2.is_empty() is False

    def test_derivation_path_length(self):
        """Test length() method."""
        path = DerivationPath("m/44'/111111'/0'")
        assert path.length() == 3

    def test_derivation_path_push(self):
        """Test push() method for adding child components."""
        path = DerivationPath("m")
        path.push(44, True)  # Hardened
        path.push(0, False)  # Normal
        
        assert path.length() == 2

    def test_derivation_path_parent(self):
        """Test parent() method."""
        path = DerivationPath("m/44'/111111'/0'")
        parent = path.parent()
        
        assert parent is not None
        assert parent.length() == 2


class TestDerivationConsistency:
    """Tests for derivation consistency."""

    def test_same_seed_same_keys(self, known_mnemonic):
        """Test that the same seed produces the same keys."""
        seed = known_mnemonic.to_seed()
        
        xprv1 = XPrv(seed)
        xprv2 = XPrv(seed)
        
        derived1 = xprv1.derive_path("m/44'/111111'/0'/0/0")
        derived2 = xprv2.derive_path("m/44'/111111'/0'/0/0")
        
        assert derived1.private_key == derived2.private_key

    def test_xprv_xpub_derive_same_address(self, known_xprv_from_mnemonic):
        """Test that XPrv and XPub derive to the same public key for non-hardened paths."""
        # Derive to account level first (hardened derivation)
        account_xprv = known_xprv_from_mnemonic.derive_path("m/44'/111111'/0'")
        account_xpub = account_xprv.to_xpub()
        
        # Now derive non-hardened paths from both
        receive_xprv = account_xprv.derive_path("m/0/0")
        receive_xpub = account_xpub.derive_path("m/0/0")
        
        # Get public keys
        pubkey_from_xprv = receive_xprv.to_xpub().to_public_key()
        pubkey_from_xpub = receive_xpub.to_public_key()
        
        # Addresses should match
        addr1 = pubkey_from_xprv.to_address("mainnet")
        addr2 = pubkey_from_xpub.to_address("mainnet")
        
        assert addr1.to_string() == addr2.to_string()

