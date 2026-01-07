"""
Unit tests for ScriptBuilder and Opcodes classes.
"""

import pytest

from kaspa import (
    ScriptBuilder,
    Opcodes,
    ScriptPublicKey,
    Address,
    pay_to_script_hash_script,
    pay_to_script_hash_signature_script,
    is_script_pay_to_pubkey,
    is_script_pay_to_pubkey_ecdsa,
    is_script_pay_to_script_hash,
)


class TestScriptBuilderCreation:
    """Tests for ScriptBuilder construction."""

    def test_create_empty_script_builder(self):
        """Test creating an empty ScriptBuilder."""
        builder = ScriptBuilder()
        assert builder is not None

    def test_create_script_builder_from_script_hex(self):
        """Test creating a ScriptBuilder from a hex script."""
        # Simple OP_TRUE script
        script_hex = "51"  # OP_TRUE
        builder = ScriptBuilder.from_script(script_hex)
        assert builder is not None

    def test_create_script_builder_from_script_bytes(self):
        """Test creating a ScriptBuilder from bytes."""
        script_bytes = bytes([0x51])  # OP_TRUE
        builder = ScriptBuilder.from_script(script_bytes)
        assert builder is not None

    def test_create_script_builder_from_script_list(self):
        """Test creating a ScriptBuilder from a list of integers."""
        script_list = [0x51]  # OP_TRUE
        builder = ScriptBuilder.from_script(script_list)
        assert builder is not None


class TestScriptBuilderOperations:
    """Tests for ScriptBuilder operations."""

    def test_add_op_with_opcode_enum(self):
        """Test adding an opcode using the Opcodes enum."""
        builder = ScriptBuilder()
        result = builder.add_op(Opcodes.OpTrue)
        assert result is not None
        # Method should return self for chaining
        assert isinstance(result, ScriptBuilder)

    def test_add_op_with_int(self):
        """Test adding an opcode using an integer."""
        builder = ScriptBuilder()
        result = builder.add_op(0x51)  # OP_TRUE
        assert result is not None

    def test_add_ops_with_opcode_list(self):
        """Test adding multiple opcodes."""
        builder = ScriptBuilder()
        result = builder.add_ops([Opcodes.OpTrue, Opcodes.OpVerify])
        assert result is not None

    def test_add_ops_with_int_list(self):
        """Test adding multiple opcodes as integers."""
        builder = ScriptBuilder()
        result = builder.add_ops([0x51, 0x69])  # OP_TRUE, OP_VERIFY
        assert result is not None

    def test_add_data_hex_string(self):
        """Test adding data as a hex string."""
        builder = ScriptBuilder()
        result = builder.add_data("deadbeef")
        assert result is not None

    def test_add_data_bytes(self):
        """Test adding data as bytes."""
        builder = ScriptBuilder()
        result = builder.add_data(bytes([0xde, 0xad, 0xbe, 0xef]))
        assert result is not None

    def test_add_data_list(self):
        """Test adding data as a list of integers."""
        builder = ScriptBuilder()
        result = builder.add_data([0xde, 0xad, 0xbe, 0xef])
        assert result is not None

    def test_add_i64(self):
        """Test adding an i64 value."""
        builder = ScriptBuilder()
        result = builder.add_i64(12345)
        assert result is not None

    def test_add_lock_time(self):
        """Test adding a lock time."""
        builder = ScriptBuilder()
        result = builder.add_lock_time(1000000)
        assert result is not None

    def test_add_sequence(self):
        """Test adding a sequence number."""
        builder = ScriptBuilder()
        result = builder.add_sequence(0xFFFFFFFF)
        assert result is not None


class TestScriptBuilderChaining:
    """Tests for ScriptBuilder method chaining."""

    def test_chain_multiple_operations(self):
        """Test chaining multiple operations."""
        builder = ScriptBuilder()
        result = (
            builder
            .add_op(Opcodes.OpTrue)
            .add_op(Opcodes.OpVerify)
            .add_data("deadbeef")
        )
        assert result is not None

    def test_build_simple_script(self):
        """Test building a simple script."""
        builder = ScriptBuilder()
        builder.add_op(Opcodes.OpDup)
        builder.add_op(Opcodes.OpBlake2b)
        builder.add_op(Opcodes.OpEqualVerify)
        builder.add_op(Opcodes.OpCheckSig)
        
        script_str = builder.to_string()
        assert script_str is not None
        assert len(script_str) > 0


class TestScriptBuilderOutput:
    """Tests for ScriptBuilder output methods."""

    def test_to_string(self):
        """Test converting script to string."""
        builder = ScriptBuilder()
        builder.add_op(Opcodes.OpTrue)
        
        script_str = builder.to_string()
        assert script_str is not None
        assert isinstance(script_str, str)

    def test_drain(self):
        """Test draining the script."""
        builder = ScriptBuilder()
        builder.add_op(Opcodes.OpTrue)
        
        script = builder.drain()
        assert script is not None
        assert isinstance(script, str)


class TestScriptBuilderP2SH:
    """Tests for ScriptBuilder P2SH (Pay-to-Script-Hash) functionality."""

    def test_create_pay_to_script_hash_script(self):
        """Test creating a P2SH script."""
        # Create a simple redeem script
        redeem_script_builder = ScriptBuilder()
        redeem_script_builder.add_op(Opcodes.OpTrue)
        
        p2sh_spk = redeem_script_builder.create_pay_to_script_hash_script()
        assert p2sh_spk is not None
        assert isinstance(p2sh_spk, ScriptPublicKey)

    def test_encode_pay_to_script_hash_signature_script(self):
        """Test encoding a P2SH signature script."""
        # Create a redeem script
        redeem_script_builder = ScriptBuilder()
        redeem_script_builder.add_op(Opcodes.OpTrue)
        
        # Encode signature script (with empty signature for testing)
        signature = "00"
        sig_script = redeem_script_builder.encode_pay_to_script_hash_signature_script(signature)
        assert sig_script is not None
        assert isinstance(sig_script, str)


class TestCanonicalDataSize:
    """Tests for canonical data size calculation."""

    def test_canonical_data_size_small(self):
        """Test canonical data size for small data."""
        small_data = "ab"  # 1 byte
        size = ScriptBuilder.canonical_data_size(small_data)
        assert size >= 1

    def test_canonical_data_size_medium(self):
        """Test canonical data size for medium data."""
        medium_data = "ab" * 40  # 40 bytes
        size = ScriptBuilder.canonical_data_size(medium_data)
        assert size >= 40

    def test_canonical_data_size_bytes(self):
        """Test canonical data size with bytes input."""
        data = bytes([0x00] * 32)
        size = ScriptBuilder.canonical_data_size(data)
        assert size >= 32


class TestOpcodes:
    """Tests for Opcodes enum."""

    def test_opcode_values(self):
        """Test that opcode values are correct."""
        assert Opcodes.OpFalse.value == 0x00
        assert Opcodes.OpTrue.value == 0x51
        assert Opcodes.OpReturn.value == 0x6a
        assert Opcodes.OpDup.value == 0x76
        assert Opcodes.OpEqualVerify.value == 0x88
        assert Opcodes.OpCheckSig.value == 0xac

    def test_opcode_stack_operations(self):
        """Test stack operation opcodes."""
        assert Opcodes.OpDrop.value == 0x75
        assert Opcodes.OpDup.value == 0x76
        assert Opcodes.OpSwap.value == 0x7c

    def test_opcode_crypto_operations(self):
        """Test crypto operation opcodes."""
        assert Opcodes.OpSHA256.value == 0xa8
        assert Opcodes.OpBlake2b.value == 0xaa
        assert Opcodes.OpCheckSig.value == 0xac
        assert Opcodes.OpCheckSigVerify.value == 0xad
        assert Opcodes.OpCheckMultiSig.value == 0xae

    def test_opcode_numeric_operations(self):
        """Test numeric opcodes."""
        assert Opcodes.Op2.value == 0x52
        assert Opcodes.Op3.value == 0x53
        assert Opcodes.Op16.value == 0x60


class TestScriptHelperFunctions:
    """Tests for script helper functions."""

    def test_pay_to_script_hash_script_from_hex(self):
        """Test pay_to_script_hash_script with hex input."""
        redeem_script = "51"  # OP_TRUE
        result = pay_to_script_hash_script(redeem_script)
        assert result is not None

    def test_pay_to_script_hash_signature_script(self):
        """Test pay_to_script_hash_signature_script."""
        redeem_script = "51"  # OP_TRUE
        signature = "00"
        result = pay_to_script_hash_signature_script(redeem_script, signature)
        assert result is not None
        assert isinstance(result, str)


class TestScriptTypeDetection:
    """Tests for script type detection functions."""

    def test_is_script_pay_to_pubkey(self, known_public_key):
        """Test detecting pay-to-pubkey scripts."""
        # Create a P2PK script
        address = known_public_key.to_address("mainnet")
        # For a proper P2PK test, we'd need the actual script
        # This is a basic test to ensure the function exists and runs
        result = is_script_pay_to_pubkey("00")
        assert isinstance(result, bool)

    def test_is_script_pay_to_pubkey_ecdsa(self):
        """Test detecting pay-to-pubkey-ecdsa scripts."""
        result = is_script_pay_to_pubkey_ecdsa("00")
        assert isinstance(result, bool)

    def test_is_script_pay_to_script_hash(self):
        """Test detecting pay-to-script-hash scripts."""
        result = is_script_pay_to_script_hash("00")
        assert isinstance(result, bool)

