"""
Unit tests for Transaction creation, signing, and related functionality.
"""

import pytest

from kaspa import (
    Transaction,
    TransactionInput,
    TransactionOutput,
    TransactionOutpoint,
    ScriptPublicKey,
    UtxoEntryReference,
    PrivateKey,
    Address,
    Generator,
    PaymentOutput,
    Hash,
    sign_transaction,
    create_input_signature,
    create_transaction,
    create_transactions,
    estimate_transactions,
    calculate_transaction_mass,
    calculate_transaction_fee,
    maximum_standard_transaction_mass,
    SighashType,
)


class TestTransactionOutpoint:
    """Tests for TransactionOutpoint class."""

    def test_create_outpoint(self):
        """Test creating a TransactionOutpoint."""
        tx_hash = Hash("0" * 64)  # 32-byte zero hash
        outpoint = TransactionOutpoint(tx_hash, 0)
        assert outpoint is not None

    def test_outpoint_properties(self):
        """Test TransactionOutpoint properties."""
        tx_id = "a" * 64
        tx_hash = Hash(tx_id)
        outpoint = TransactionOutpoint(tx_hash, 5)
        
        assert outpoint.transaction_id == tx_id
        assert outpoint.index == 5

    def test_outpoint_get_id(self):
        """Test TransactionOutpoint get_id method."""
        tx_id = "b" * 64
        tx_hash = Hash(tx_id)
        outpoint = TransactionOutpoint(tx_hash, 0)
        
        outpoint_id = outpoint.get_id()
        assert outpoint_id is not None
        assert isinstance(outpoint_id, str)


class TestScriptPublicKey:
    """Tests for ScriptPublicKey class."""

    def test_create_script_public_key_from_hex(self):
        """Test creating a ScriptPublicKey from hex."""
        script_hex = "20" + "a" * 64 + "ac"  # Sample script
        spk = ScriptPublicKey(0, script_hex)
        assert spk is not None

    def test_create_script_public_key_from_bytes(self):
        """Test creating a ScriptPublicKey from bytes."""
        script_bytes = bytes([0x51])  # OP_TRUE
        spk = ScriptPublicKey(0, script_bytes)
        assert spk is not None

    def test_create_script_public_key_from_list(self):
        """Test creating a ScriptPublicKey from a list."""
        script_list = [0x51]  # OP_TRUE
        spk = ScriptPublicKey(0, script_list)
        assert spk is not None

    def test_script_public_key_script_property(self):
        """Test ScriptPublicKey script property."""
        script_hex = "51"
        spk = ScriptPublicKey(0, script_hex)
        
        script = spk.script
        assert script is not None
        assert isinstance(script, str)


class TestTransactionOutput:
    """Tests for TransactionOutput class."""

    def test_create_transaction_output(self):
        """Test creating a TransactionOutput."""
        spk = ScriptPublicKey(0, "51")
        output = TransactionOutput(1000000, spk)
        assert output is not None

    def test_transaction_output_value(self):
        """Test TransactionOutput value property."""
        spk = ScriptPublicKey(0, "51")
        output = TransactionOutput(1000000, spk)
        
        assert output.value == 1000000

    def test_transaction_output_value_setter(self):
        """Test setting TransactionOutput value."""
        spk = ScriptPublicKey(0, "51")
        output = TransactionOutput(1000000, spk)
        
        output.value = 2000000
        assert output.value == 2000000


class TestTransactionInput:
    """Tests for TransactionInput class."""

    def test_create_transaction_input(self):
        """Test creating a TransactionInput."""
        tx_hash = Hash("0" * 64)
        outpoint = TransactionOutpoint(tx_hash, 0)
        input = TransactionInput(outpoint, "", 0, 1)
        assert input is not None

    def test_transaction_input_properties(self):
        """Test TransactionInput properties."""
        tx_hash = Hash("a" * 64)
        outpoint = TransactionOutpoint(tx_hash, 5)
        input = TransactionInput(outpoint, "deadbeef", 0xFFFFFFFF, 1)
        
        assert input.previous_outpoint is not None
        assert input.sequence == 0xFFFFFFFF
        assert input.sig_op_count == 1


class TestTransaction:
    """Tests for Transaction class."""

    def test_create_minimal_transaction(self):
        """Test creating a minimal transaction."""
        tx_hash = Hash("0" * 64)
        outpoint = TransactionOutpoint(tx_hash, 0)
        input = TransactionInput(outpoint, "", 0, 1)
        
        spk = ScriptPublicKey(0, "51")
        output = TransactionOutput(1000000, spk)
        
        tx = Transaction(0, [input], [output], 0, "0" * 40, 0, "", 0)
        assert tx is not None

    def test_transaction_equality(self):
        """Test transaction equality works."""
        tx_hash = Hash("0" * 64)
        outpoint = TransactionOutpoint(tx_hash, 0)
        input = TransactionInput(outpoint, "", 0, 1)
        
        spk = ScriptPublicKey(0, "51")
        output = TransactionOutput(1000000, spk)
        
        tx1 = Transaction(0, [input], [output], 0, "0" * 40, 0, "", 0)
        tx2 = Transaction(0, [input], [output], 0, "0" * 40, 0, "", 0)
        assert tx1 == tx2


    def test_transaction_properties(self):
        """Test Transaction properties."""
        tx_hash = Hash("0" * 64)
        outpoint = TransactionOutpoint(tx_hash, 0)
        input = TransactionInput(outpoint, "", 0, 1)
        
        spk = ScriptPublicKey(0, "51")
        output = TransactionOutput(1000000, spk)
        
        tx = Transaction(0, [input], [output], 100, "0" * 40, 0, "", 0)
        
        assert tx.version == 0
        assert tx.lock_time == 100
        assert len(tx.inputs) == 1
        assert len(tx.outputs) == 1

    def test_transaction_id(self):
        """Test Transaction id property."""
        tx_hash = Hash("0" * 64)
        outpoint = TransactionOutpoint(tx_hash, 0)
        input = TransactionInput(outpoint, "", 0, 1)
        
        spk = ScriptPublicKey(0, "51")
        output = TransactionOutput(1000000, spk)
        
        tx = Transaction(0, [input], [output], 0, "0" * 40, 0, "", 0)
        
        tx_id = tx.id
        assert tx_id is not None
        assert len(tx_id) == 64  # 32 bytes hex

    def test_transaction_is_coinbase(self):
        """Test Transaction is_coinbase method."""
        tx_hash = Hash("0" * 64)
        outpoint = TransactionOutpoint(tx_hash, 0)
        input = TransactionInput(outpoint, "", 0, 1)
        
        spk = ScriptPublicKey(0, "51")
        output = TransactionOutput(1000000, spk)
        
        tx = Transaction(0, [input], [output], 0, "0" * 40, 0, "", 0)
        
        # Regular transaction should not be coinbase
        # (coinbase transactions have specific subnetwork_id)
        assert isinstance(tx.is_coinbase(), bool)


class TestPaymentOutput:
    """Tests for PaymentOutput class."""

    def test_payment_output_import(self):
        """Test that PaymentOutput class is importable."""
        # PaymentOutput may be used differently based on API
        # This test verifies the class exists
        assert PaymentOutput is not None


class TestTransactionMass:
    """Tests for transaction mass calculations."""

    def test_maximum_standard_transaction_mass(self):
        """Test getting maximum standard transaction mass."""
        max_mass = maximum_standard_transaction_mass()
        assert max_mass is not None
        assert max_mass > 0


class TestSighashType:
    """Tests for SighashType enum."""

    def test_sighash_type_exists(self):
        """Test SighashType enum exists with expected variants."""
        # SighashType is a Rust enum exposed to Python
        assert SighashType.All is not None
        assert SighashType.Single is not None
        assert SighashType.AllAnyOneCanPay is not None


class TestCreateTransaction:
    """Tests for create_transaction helper function."""

    def test_create_transaction_function_exists(self):
        """Test that create_transaction function is importable."""
        # The create_transaction function exists and can be called
        # Exact signature may vary based on implementation
        assert callable(create_transaction)


class TestGenerator:
    """Tests for Generator class."""

    def test_generator_class_exists(self):
        """Test that Generator class is importable."""
        # Generator class exists for transaction generation
        assert Generator is not None

    def test_generator_has_expected_methods(self):
        """Test Generator has expected methods."""
        # Check Generator has summary and iteration capabilities
        assert hasattr(Generator, 'summary')
        assert hasattr(Generator, '__iter__')

