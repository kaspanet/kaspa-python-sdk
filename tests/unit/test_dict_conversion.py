"""
Unit tests for to_dict/from_dict conversion methods on consensus client types.
"""

import pytest

from kaspa import (
    Transaction,
    TransactionInput,
    TransactionOutput,
    TransactionOutpoint,
    ScriptPublicKey,
    UtxoEntry,
    UtxoEntryReference,
    Hash,
)


class TestTransactionOutpointDict:
    """Tests for TransactionOutpoint to_dict/from_dict methods."""

    def test_outpoint_to_dict(self):
        """Test TransactionOutpoint to_dict method."""
        tx_hash = Hash("a" * 64)
        outpoint = TransactionOutpoint(tx_hash, 5)

        d = outpoint.to_dict()
        assert isinstance(d, dict)
        assert "transactionId" in d
        assert "index" in d
        assert d["index"] == 5

    def test_outpoint_from_dict_roundtrip(self):
        """Test TransactionOutpoint to_dict/from_dict round-trip."""
        tx_hash = Hash("a" * 64)
        original = TransactionOutpoint(tx_hash, 5)

        d = original.to_dict()
        restored = TransactionOutpoint.from_dict(d)

        assert original == restored


class TestTransactionOutputDict:
    """Tests for TransactionOutput to_dict/from_dict methods."""

    def test_output_to_dict(self):
        """Test TransactionOutput to_dict method."""
        spk = ScriptPublicKey(0, "51")
        output = TransactionOutput(1000000, spk)

        d = output.to_dict()
        assert isinstance(d, dict)
        assert "value" in d
        assert "scriptPublicKey" in d
        assert d["value"] == 1000000

    def test_output_from_dict_roundtrip(self):
        """Test TransactionOutput to_dict/from_dict round-trip."""
        spk = ScriptPublicKey(0, "51")
        original = TransactionOutput(1000000, spk)

        d = original.to_dict()
        restored = TransactionOutput.from_dict(d)

        assert original == restored


class TestTransactionInputDict:
    """Tests for TransactionInput to_dict/from_dict methods."""

    def test_input_to_dict(self):
        """Test TransactionInput to_dict method."""
        tx_hash = Hash("a" * 64)
        outpoint = TransactionOutpoint(tx_hash, 5)
        input = TransactionInput(outpoint, "deadbeef", 0xFFFFFFFF, 1)

        d = input.to_dict()
        assert isinstance(d, dict)
        assert "previousOutpoint" in d
        assert "signatureScript" in d
        assert "sequence" in d
        assert "sigOpCount" in d
        assert "utxo" in d

    def test_input_from_dict_roundtrip(self):
        """Test TransactionInput to_dict/from_dict round-trip."""
        tx_hash = Hash("a" * 64)
        outpoint = TransactionOutpoint(tx_hash, 5)
        original = TransactionInput(outpoint, "deadbeef", 0xFFFFFFFF, 1)

        d = original.to_dict()
        restored = TransactionInput.from_dict(d)

        assert original == restored

    def test_input_from_dict_without_compute_budget_defaults_zero(self):
        """A v0-shape input dict (no `computeBudget` key) defaults `compute_budget` to 0.

        Regression guard for rusty-kaspa v2.0.1's transaction-v0 deserialization fix
        (#1052): legacy/pre-covenant input JSON omits `computeBudget`, and it must still
        deserialize rather than raise a KeyError.
        """
        d = {
            "previousOutpoint": {"transactionId": "a" * 64, "index": 0},
            "signatureScript": "01",
            "sequence": 0,
            "sigOpCount": 1,
            "utxo": None,
        }
        restored = TransactionInput.from_dict(d)
        assert restored.compute_budget == 0

    def test_input_from_dict_with_compute_budget_preserved(self):
        """An explicit `computeBudget` key is honored by from_dict."""
        d = {
            "previousOutpoint": {"transactionId": "a" * 64, "index": 0},
            "signatureScript": "01",
            "sequence": 0,
            "sigOpCount": 1,
            "computeBudget": 7,
            "utxo": None,
        }
        restored = TransactionInput.from_dict(d)
        assert restored.compute_budget == 7


class TestTransactionDict:
    """Tests for Transaction to_dict/from_dict methods."""

    def test_transaction_to_dict(self):
        """Test Transaction to_dict method."""
        tx_hash = Hash("0" * 64)
        outpoint = TransactionOutpoint(tx_hash, 0)
        input = TransactionInput(outpoint, "", 0, 1)

        spk = ScriptPublicKey(0, "51")
        output = TransactionOutput(1000000, spk)

        tx = Transaction(0, [input], [output], 100, "0" * 40, 0, "", 0)

        d = tx.to_dict()
        assert isinstance(d, dict)
        assert "id" in d
        assert "version" in d
        assert "inputs" in d
        assert "outputs" in d
        assert "lockTime" in d
        assert "subnetworkId" in d
        assert "gas" in d
        assert "payload" in d
        assert "mass" in d
        assert "storageMass" in d
        assert d["mass"] == d["storageMass"]

    def test_transaction_storage_mass_alias(self):
        """`storage_mass` mirrors `mass` (kept as an alias for WASM & back-compat)."""
        tx_hash = Hash("0" * 64)
        outpoint = TransactionOutpoint(tx_hash, 0)
        input = TransactionInput(outpoint, "", 0, 1)
        spk = ScriptPublicKey(0, "51")
        output = TransactionOutput(1000000, spk)

        tx = Transaction(0, [input], [output], 100, "0" * 40, 0, "", 1234)
        assert tx.mass == 1234
        assert tx.storage_mass == 1234

        tx.storage_mass = 5678
        assert tx.mass == 5678
        tx.mass = 99
        assert tx.storage_mass == 99

    def test_transaction_from_dict_roundtrip(self):
        """Test Transaction to_dict/from_dict round-trip."""
        tx_hash = Hash("0" * 64)
        outpoint = TransactionOutpoint(tx_hash, 0)
        input = TransactionInput(outpoint, "", 0, 1)

        spk = ScriptPublicKey(0, "51")
        output = TransactionOutput(1000000, spk)

        original = Transaction(0, [input], [output], 100, "0" * 40, 0, "", 0)

        d = original.to_dict()
        restored = Transaction.from_dict(d)

        assert original == restored

    def test_transaction_from_dict_accepts_v0_shape(self):
        """Transaction.from_dict accepts a v0-shape dict.

        Regression guard for rusty-kaspa v2.0.1's transaction-v0 deserialization fix
        (#1052): a legacy transaction uses the `mass` key (no `storageMass`) and its
        inputs omit `computeBudget`. Both must deserialize, with `compute_budget`
        defaulting to 0 and `mass`/`storage_mass` taking the `mass` value.
        """
        v0_input = {
            "previousOutpoint": {"transactionId": "01" * 32, "index": 0},
            "signatureScript": "01",
            "sequence": 0,
            "sigOpCount": 1,
            "utxo": None,
        }
        v0_tx = {
            "id": "0" * 64,
            "version": 0,
            "inputs": [v0_input],
            "outputs": [],
            "subnetworkId": "0" * 40,
            "lockTime": 0,
            "gas": 0,
            "mass": 1,
            "payload": "",
        }

        tx = Transaction.from_dict(v0_tx)
        assert tx.version == 0
        assert tx.mass == 1
        assert tx.storage_mass == 1
        assert tx.inputs[0].compute_budget == 0


class TestUtxoEntryDict:
    """Tests for UtxoEntry to_dict/from_dict methods."""

    def test_utxo_entry_to_dict(self):
        """Test UtxoEntry to_dict method."""
        entry_dict = {
            "address": "kaspa:qr0lr4ml9fn3chekrqmjdkergxl93l4wrk3dankcgvjq776s9wn9jkdskewva",
            "outpoint": {"transactionId": "a" * 64, "index": 0},
            "amount": 1000000,
            "scriptPublicKey": {"version": 0, "script": "20852be1b87fca94453a35027c550a3ccdbebb5913106029f3a8bf18152bf93bffac"},
            "blockDaaScore": 12345,
            "isCoinbase": False,
            "covenantId": None,
        }
        entry = UtxoEntry.from_dict(entry_dict)

        d = entry.to_dict()
        assert isinstance(d, dict)
        assert "address" in d
        assert "outpoint" in d
        assert "amount" in d
        assert "scriptPublicKey" in d
        assert "blockDaaScore" in d
        assert "isCoinbase" in d

    def test_utxo_entry_from_dict_roundtrip(self):
        """Test UtxoEntry to_dict/from_dict round-trip."""
        entry_dict = {
            "address": "kaspa:qr0lr4ml9fn3chekrqmjdkergxl93l4wrk3dankcgvjq776s9wn9jkdskewva",
            "outpoint": {"transactionId": "a" * 64, "index": 0},
            "amount": 1000000,
            "scriptPublicKey": {"version": 0, "script": "20852be1b87fca94453a35027c550a3ccdbebb5913106029f3a8bf18152bf93bffac"},
            "blockDaaScore": 12345,
            "isCoinbase": False,
            "covenantId": None,
        }
        original = UtxoEntry.from_dict(entry_dict)

        d = original.to_dict()
        restored = UtxoEntry.from_dict(d)

        assert original == restored


class TestUtxoEntryReferenceDict:
    """Tests for UtxoEntryReference to_dict/from_dict methods."""

    def test_utxo_entry_reference_to_dict(self):
        """Test UtxoEntryReference to_dict method produces flat format."""
        entry_dict = {
            "address": "kaspa:qr0lr4ml9fn3chekrqmjdkergxl93l4wrk3dankcgvjq776s9wn9jkdskewva",
            "outpoint": {"transactionId": "a" * 64, "index": 0},
            "amount": 1000000,
            "scriptPublicKey": {"version": 0, "script": "20852be1b87fca94453a35027c550a3ccdbebb5913106029f3a8bf18152bf93bffac"},
            "blockDaaScore": 12345,
            "isCoinbase": False,
            "covenantId": None,
        }
        entry_ref = UtxoEntryReference.from_dict(entry_dict)

        d = entry_ref.to_dict()
        assert isinstance(d, dict)
        # Flat format keys
        assert "address" in d
        assert "outpoint" in d
        assert "amount" in d
        assert "scriptPublicKey" in d
        assert "blockDaaScore" in d
        assert "isCoinbase" in d
        # Should NOT have nested utxoEntry
        assert "utxoEntry" not in d

    def test_utxo_entry_reference_from_dict_flat_format(self):
        """Test UtxoEntryReference from_dict with flat format."""
        entry_dict = {
            "address": "kaspa:qr0lr4ml9fn3chekrqmjdkergxl93l4wrk3dankcgvjq776s9wn9jkdskewva",
            "outpoint": {"transactionId": "a" * 64, "index": 0},
            "amount": 1000000,
            "scriptPublicKey": {"version": 0, "script": "20852be1b87fca94453a35027c550a3ccdbebb5913106029f3a8bf18152bf93bffac"},
            "blockDaaScore": 12345,
            "isCoinbase": False,
            "covenantId": None,
        }
        entry_ref = UtxoEntryReference.from_dict(entry_dict)
        assert entry_ref.amount == 1000000

    def test_utxo_entry_reference_from_dict_nested_format(self):
        """Test UtxoEntryReference from_dict with nested format (compatible with utxos returned via RPC)."""
        entry_dict = {
            "address": "kaspa:qr0lr4ml9fn3chekrqmjdkergxl93l4wrk3dankcgvjq776s9wn9jkdskewva",
            "outpoint": {"transactionId": "a" * 64, "index": 0},
            "utxoEntry": {
                "amount": 2000000,
                "scriptPublicKey": {"version": 0, "script": "20852be1b87fca94453a35027c550a3ccdbebb5913106029f3a8bf18152bf93bffac"},
                "blockDaaScore": 12345,
                "isCoinbase": False,
                "covenantId": None,
            },
        }
        entry_ref = UtxoEntryReference.from_dict(entry_dict)
        assert entry_ref.amount == 2000000

    def test_utxo_entry_reference_from_dict_roundtrip(self):
        """Test UtxoEntryReference to_dict/from_dict round-trip."""
        entry_dict = {
            "address": "kaspa:qr0lr4ml9fn3chekrqmjdkergxl93l4wrk3dankcgvjq776s9wn9jkdskewva",
            "outpoint": {"transactionId": "a" * 64, "index": 0},
            "amount": 1000000,
            "scriptPublicKey": {"version": 0, "script": "20852be1b87fca94453a35027c550a3ccdbebb5913106029f3a8bf18152bf93bffac"},
            "blockDaaScore": 12345,
            "isCoinbase": False,
            "covenantId": None,
        }
        original = UtxoEntryReference.from_dict(entry_dict)

        d = original.to_dict()
        restored = UtxoEntryReference.from_dict(d)

        assert original == restored
