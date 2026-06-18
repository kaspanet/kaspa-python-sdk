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
    create_transaction,
    covenant_id,
)

SUBNETWORK_ID = bytes(20)
ADDRESS = "kaspa:qr0lr4ml9fn3chekrqmjdkergxl93l4wrk3dankcgvjq776s9wn9jkdskewva"
COVENANT_ID = "ab" * 32
# A P2PK script-public-key matching ADDRESS, reused for synchronous UTXO fixtures.
SCRIPT_PUBLIC_KEY = "20852be1b87fca94453a35027c550a3ccdbebb5913106029f3a8bf18152bf93bffac"


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


class TestCovenantKeyOptional:
    """The `covenant` key is optional when coercing an output from a dict.

    Mirrors the WASM SDK, where `PaymentOutput.try_cast_from` reads `covenant`
    via `Reflect::get` (absent -> None) rather than throwing on a missing key.
    """

    def test_transaction_output_from_dict_without_covenant_key(self):
        """from_dict succeeds on a dict with no `covenant` key (covenant -> None)."""
        out = TransactionOutput.from_dict(
            {"value": 1000, "scriptPublicKey": {"version": 0, "script": "51"}}
        )
        assert out.to_dict()["covenant"] is None

    def test_transaction_output_from_dict_explicit_none_covenant(self):
        """An explicit `covenant: None` is accepted too (value may be None)."""
        out = TransactionOutput.from_dict(
            {"value": 1000, "scriptPublicKey": {"version": 0, "script": "51"}, "covenant": None}
        )
        assert out.to_dict()["covenant"] is None

    def test_payment_output_dict_without_covenant_key(self):
        """A plain {address, amount} output dict coerces (no `covenant` key required).

        Exercised through `create_transaction`, whose `outputs` argument coerces
        each dict via the same `PaymentOutput` from-dict path used by Generator,
        wallet send, etc.
        """
        entry = {
            "address": ADDRESS,
            "outpoint": {"transactionId": "a" * 64, "index": 0},
            "amount": 5_000_000,
            "scriptPublicKey": {"version": 0, "script": SCRIPT_PUBLIC_KEY},
            "blockDaaScore": 0,
            "isCoinbase": False,
            "covenantId": None,
        }
        tx = create_transaction([entry], [{"address": ADDRESS, "amount": 1_000_000}], 0)
        assert tx.outputs[0].to_dict()["covenant"] is None

    def test_payment_output_dict_explicit_none_covenant(self):
        """An explicit `covenant: None` on a payment-output dict is accepted too."""
        entry = {
            "address": ADDRESS,
            "outpoint": {"transactionId": "a" * 64, "index": 0},
            "amount": 5_000_000,
            "scriptPublicKey": {"version": 0, "script": SCRIPT_PUBLIC_KEY},
            "blockDaaScore": 0,
            "isCoinbase": False,
            "covenantId": None,
        }
        tx = create_transaction(
            [entry], [{"address": ADDRESS, "amount": 1_000_000, "covenant": None}], 0
        )
        assert tx.outputs[0].to_dict()["covenant"] is None

    def test_payment_output_dict_still_accepts_covenant(self):
        """The covenant key still works when present (no regression)."""
        entry = {
            "address": ADDRESS,
            "outpoint": {"transactionId": "a" * 64, "index": 0},
            "amount": 5_000_000,
            "scriptPublicKey": {"version": 0, "script": SCRIPT_PUBLIC_KEY},
            "blockDaaScore": 0,
            "isCoinbase": False,
            "covenantId": None,
        }
        tx = create_transaction(
            [entry],
            [
                {
                    "address": ADDRESS,
                    "amount": 1_000_000,
                    "covenant": {"authorizingInput": 0, "covenantId": COVENANT_ID},
                }
            ],
            0,
        )
        assert tx.outputs[0].to_dict()["covenant"] == {
            "authorizingInput": 0,
            "covenantId": COVENANT_ID,
        }


class TestPopulateGenesisCovenantsValidation:
    """Native (consensus) validation failures surface as Exception with the
    message intact.

    The validation logic itself is tested upstream in rusty-kaspa; these confirm
    the binding forwards each documented failure mode. They are distinct from the
    dict-coercion errors in TestGenesisCovenantGroupDictErrors. `_build_tx()` has
    1 input and 3 outputs.
    """

    def test_authorizing_input_out_of_bounds(self):
        tx = _build_tx()
        with pytest.raises(
            Exception, match="authorizing input index 1 is out of bounds for 1 inputs"
        ):
            tx.populate_genesis_covenants([GenesisCovenantGroup(1, [0])])

    def test_output_index_out_of_bounds(self):
        tx = _build_tx()
        with pytest.raises(Exception, match="output index 3 is out of bounds for 3 outputs"):
            tx.populate_genesis_covenants([GenesisCovenantGroup(0, [3])])

    def test_outputs_not_strictly_ordered(self):
        tx = _build_tx()
        with pytest.raises(Exception, match="outputs are not strictly ordered"):
            tx.populate_genesis_covenants([GenesisCovenantGroup(0, [1, 0])])

    def test_outputs_overlap_across_groups(self):
        tx = _build_tx()
        with pytest.raises(Exception, match="output index 1 appears in more than one group"):
            tx.populate_genesis_covenants(
                [GenesisCovenantGroup(0, [0, 1]), GenesisCovenantGroup(0, [1, 2])]
            )

    def test_output_already_populated(self):
        tx = _build_tx()
        tx.populate_genesis_covenants([GenesisCovenantGroup(0, [0])])
        with pytest.raises(
            Exception, match="output index 0 covenant field is already populated"
        ):
            tx.populate_genesis_covenants([GenesisCovenantGroup(0, [0])])


class TestCovenantIdFunction:
    """`covenant_id(outpoint, auth_outputs)` is a deterministic hash of the
    authorizing outpoint and the (positionally-indexed) output list."""

    @staticmethod
    def _outpoint(prefix="0"):
        return TransactionOutpoint(Hash(prefix + "0" * 63), 0)

    def test_deterministic(self):
        op = self._outpoint()
        spk = ScriptPublicKey(0, "51")
        outs = [TransactionOutput(1000, spk), TransactionOutput(2000, spk)]
        assert covenant_id(op, outs) == covenant_id(op, outs)
        assert isinstance(covenant_id(op, outs), Hash)

    def test_sensitive_to_inputs(self):
        """Any change to the outpoint or the output list (value, count, order)
        yields a different id."""
        op = self._outpoint()
        spk = ScriptPublicKey(0, "51")
        o0 = TransactionOutput(1000, spk)
        o1 = TransactionOutput(2000, spk)
        base = covenant_id(op, [o0, o1])
        assert base != covenant_id(self._outpoint("1"), [o0, o1])  # outpoint
        assert base != covenant_id(op, [TransactionOutput(1001, spk), o1])  # value
        assert base != covenant_id(op, [o0])  # count
        assert base != covenant_id(op, [o1, o0])  # index/order

    def test_matches_populate_genesis_covenants(self):
        """The free function reproduces the id that populate_genesis_covenants
        writes, for a group whose output indices are contiguous from 0 (so the
        function's positional enumerate matches the transaction output indices).
        """
        op = TransactionOutpoint(Hash("0" * 64), 0)
        inp = TransactionInput(op, b"", 0, 1)
        spk = ScriptPublicKey(0, "51")
        o0 = TransactionOutput(1000, spk)
        o1 = TransactionOutput(2000, spk)
        tx = Transaction(
            0, [inp], [o0, o1, TransactionOutput(500, spk)], 0, SUBNETWORK_ID, 0, b"", 0
        )
        tx.populate_genesis_covenants([GenesisCovenantGroup(0, [0, 1])])

        expected = covenant_id(op, [o0, o1]).to_string()
        assert tx.outputs[0].to_dict()["covenant"]["covenantId"] == expected
        assert tx.outputs[1].to_dict()["covenant"]["covenantId"] == expected
