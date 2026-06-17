"""
Unit tests for GenesisCovenantGroup construction (instance and dict) and
its use via Transaction.populate_genesis_covenants.
"""

import pytest

from kaspa import (
    CovenantBinding,
    GenesisCovenantGroup,
    PaymentOutput,
    Transaction,
    TransactionInput,
    TransactionOutpoint,
    TransactionOutput,
    ScriptPublicKey,
    Hash,
)

SUBNETWORK_ID = bytes(20)
ADDRESS = "kaspa:qr0lr4ml9fn3chekrqmjdkergxl93l4wrk3dankcgvjq776s9wn9jkdskewva"
COVENANT_ID = "ab" * 32


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


class TestCovenantBindingConstruction:
    """Direct construction, properties, and repr of CovenantBinding."""

    def test_construct(self):
        cb = CovenantBinding(0, Hash(COVENANT_ID))
        assert cb.authorizing_input == 0
        assert cb.covenant_id == Hash(COVENANT_ID)

    def test_setters(self):
        cb = CovenantBinding(0, Hash(COVENANT_ID))
        cb.authorizing_input = 5
        cb.covenant_id = Hash("cd" * 32)
        assert cb.authorizing_input == 5
        assert cb.covenant_id == Hash("cd" * 32)

    def test_repr(self):
        cb = CovenantBinding(0, Hash(COVENANT_ID))
        assert repr(cb) == f"CovenantBinding(authorizing_input=0, covenant_id=Hash('{COVENANT_ID}'))"


class TestCovenantBindingFromObjectOrDict:
    """CovenantBinding is accepted as an instance OR a {authorizingInput, covenantId} dict."""

    def test_with_covenant_accepts_dict_hex_str(self):
        po = PaymentOutput.with_covenant(
            ADDRESS, 1000, {"authorizingInput": 0, "covenantId": COVENANT_ID}
        )
        instance = PaymentOutput.with_covenant(ADDRESS, 1000, CovenantBinding(0, Hash(COVENANT_ID)))
        assert po == instance

    def test_with_covenant_accepts_dict_hash_instance(self):
        po = PaymentOutput.with_covenant(
            ADDRESS, 1000, {"authorizingInput": 3, "covenantId": Hash(COVENANT_ID)}
        )
        instance = PaymentOutput.with_covenant(ADDRESS, 1000, CovenantBinding(3, Hash(COVENANT_ID)))
        assert po == instance

    def test_transaction_output_accepts_dict(self):
        spk = ScriptPublicKey(0, "51")
        from_dict = TransactionOutput(1000, spk, {"authorizingInput": 0, "covenantId": COVENANT_ID})
        from_instance = TransactionOutput(1000, spk, CovenantBinding(0, Hash(COVENANT_ID)))
        assert from_dict == from_instance
        assert from_dict.to_dict()["covenant"] == {
            "authorizingInput": 0,
            "covenantId": COVENANT_ID,
        }

    def test_missing_authorizing_input_raises(self):
        with pytest.raises(KeyError):
            PaymentOutput.with_covenant(ADDRESS, 1000, {"covenantId": COVENANT_ID})

    def test_missing_covenant_id_raises(self):
        with pytest.raises(KeyError):
            PaymentOutput.with_covenant(ADDRESS, 1000, {"authorizingInput": 0})

    def test_invalid_covenant_id_raises(self):
        with pytest.raises(Exception):
            PaymentOutput.with_covenant(ADDRESS, 1000, {"authorizingInput": 0, "covenantId": "xyz"})


class TestCovenantRoundTrip:
    """Covenant-bound outputs survive to_dict/from_dict round-trips with a flat covenant shape."""

    def test_transaction_output_covenant_roundtrip(self):
        spk = ScriptPublicKey(0, "51")
        out = TransactionOutput(1000, spk, CovenantBinding(0, Hash(COVENANT_ID)))

        restored = TransactionOutput.from_dict(out.to_dict())

        assert out == restored
        assert restored.to_dict()["covenant"] == {"authorizingInput": 0, "covenantId": COVENANT_ID}

    def test_transaction_covenant_roundtrip(self):
        outpoint = TransactionOutpoint(Hash("0" * 64), 0)
        inp = TransactionInput(outpoint, b"", 0, 1)
        spk = ScriptPublicKey(0, "51")
        bound = TransactionOutput(1000, spk, CovenantBinding(0, Hash("cd" * 32)))
        unbound = TransactionOutput(500, spk)
        tx = Transaction(0, [inp], [bound, unbound], 0, SUBNETWORK_ID, 0, b"", 0)

        restored = Transaction.from_dict(tx.to_dict())

        assert tx == restored
        assert restored.outputs[0].to_dict()["covenant"] == {
            "authorizingInput": 0,
            "covenantId": "cd" * 32,
        }
        assert restored.outputs[1].to_dict()["covenant"] is None
