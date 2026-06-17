"""
Unit tests for GenesisCovenantGroup construction (instance and dict) and
its use via Transaction.populate_genesis_covenants.
"""

import pytest

from kaspa import (
    GenesisCovenantGroup,
    Transaction,
    TransactionInput,
    TransactionOutpoint,
    TransactionOutput,
    ScriptPublicKey,
    Hash,
)

SUBNETWORK_ID = bytes(20)


def _build_tx():
    """A transaction with one input and three outputs, none covenant-bound yet."""
    outpoint = TransactionOutpoint(Hash("0" * 64), 0)
    inp = TransactionInput(outpoint, b"", 0, 1)
    spk = ScriptPublicKey(0, "51")
    outputs = [
        TransactionOutput(1_000, spk),
        TransactionOutput(2_000, spk),
        TransactionOutput(500, spk),
    ]
    return Transaction(0, [inp], outputs, 0, SUBNETWORK_ID, 0, b"", 0)


class TestGenesisCovenantGroupConstruction:
    """Direct construction, properties, and repr of GenesisCovenantGroup."""

    def test_construct_from_kwargs(self):
        group = GenesisCovenantGroup(authorizing_input=0, outputs=[0, 1])
        assert group.authorizing_input == 0
        assert group.outputs == [0, 1]

    def test_construct_positional(self):
        group = GenesisCovenantGroup(2, [3, 4, 5])
        assert group.authorizing_input == 2
        assert group.outputs == [3, 4, 5]

    def test_setters(self):
        group = GenesisCovenantGroup(0, [0])
        group.authorizing_input = 7
        group.outputs = [1, 2, 3]
        assert group.authorizing_input == 7
        assert group.outputs == [1, 2, 3]

    def test_repr(self):
        group = GenesisCovenantGroup(0, [0, 1])
        assert repr(group) == "GenesisCovenantGroup(authorizing_input=0, outputs=[0, 1])"


class TestGenesisCovenantGroupFromObjectOrDict:
    """`populate_genesis_covenants` accepts a GenesisCovenantGroup instance OR an equivalent dict."""

    def test_populate_accepts_instance(self):
        tx = _build_tx()
        tx.populate_genesis_covenants([GenesisCovenantGroup(0, [0, 1])])

        outs = tx.outputs
        assert outs[0].to_dict()["covenant"] is not None
        assert outs[1].to_dict()["covenant"] is not None
        assert outs[2].to_dict()["covenant"] is None

    def test_populate_accepts_dict(self):
        tx = _build_tx()
        tx.populate_genesis_covenants([{"authorizingInput": 0, "outputs": [0, 1]}])

        outs = tx.outputs
        assert outs[0].to_dict()["covenant"] is not None
        assert outs[1].to_dict()["covenant"] is not None
        assert outs[2].to_dict()["covenant"] is None

    def test_instance_and_dict_produce_identical_tx(self):
        """The dict form must parse to a group equivalent to the instance form.

        Covenant ids are derived deterministically from the authorizing input's
        outpoint and the exact output list, so two transactions populated from
        equivalent instance/dict groups must be byte-for-byte equal.
        """
        tx_from_instance = _build_tx()
        tx_from_dict = _build_tx()
        assert tx_from_instance == tx_from_dict  # identical before population

        tx_from_instance.populate_genesis_covenants([GenesisCovenantGroup(0, [0, 1])])
        tx_from_dict.populate_genesis_covenants([{"authorizingInput": 0, "outputs": [0, 1]}])

        assert tx_from_instance == tx_from_dict

    def test_mixed_instance_and_dict_in_one_call(self):
        """A single call may mix instance and dict groups (each element is coerced)."""
        tx = _build_tx()
        tx.populate_genesis_covenants(
            [
                GenesisCovenantGroup(0, [0]),
                {"authorizingInput": 0, "outputs": [1]},
            ]
        )

        outs = tx.outputs
        assert outs[0].to_dict()["covenant"] is not None
        assert outs[1].to_dict()["covenant"] is not None
        assert outs[2].to_dict()["covenant"] is None


class TestGenesisCovenantGroupDictErrors:
    """Malformed dicts are rejected during argument coercion."""

    def test_missing_authorizing_input_raises(self):
        tx = _build_tx()
        with pytest.raises(KeyError):
            tx.populate_genesis_covenants([{"outputs": [0, 1]}])

    def test_missing_outputs_raises(self):
        tx = _build_tx()
        with pytest.raises(KeyError):
            tx.populate_genesis_covenants([{"authorizingInput": 0}])

    def test_non_dict_non_instance_raises(self):
        tx = _build_tx()
        with pytest.raises(TypeError):
            tx.populate_genesis_covenants([42])
