"""
Unit tests for create_transaction version / compute-commit handling.

Version 0 transactions commit a sig_op_count per input; version 1 and later
commit a compute_budget per input and may carry covenant outputs. These tests
need no node: they only build unsigned transactions from a synthetic UTXO.
"""

import pytest

from kaspa import PaymentOutput, UtxoEntryReference, create_transaction

ADDRESS = "kaspa:qpamkvhgh0kzx50gwvvp5xs8ktmqutcy3dfs9dc3w7lm9rq0zs76vf959mmrp"
COVENANT_ID = "ab" * 32


def _utxo(amount=100_000_000):
    return UtxoEntryReference.from_dict({
        "address": ADDRESS,
        "outpoint": {"transactionId": "a" * 64, "index": 0},
        "amount": amount,
        "scriptPublicKey": {"version": 0, "script": "51"},
        "blockDaaScore": 1,
        "isCoinbase": False,
        "covenantId": None,
    })


def _outputs(amount=50_000_000):
    return [{"address": ADDRESS, "amount": amount}]


def _covenant_outputs(amount=50_000_000):
    return [PaymentOutput.with_covenant(
        ADDRESS, amount, {"authorizingInput": 0, "covenantId": COVENANT_ID}
    )]


class TestCreateTransactionVersion0:
    """Version 0 (the default) commits sig_op_count and rejects version-1 inputs."""

    def test_defaults_to_version_0_with_one_sig_op(self):
        tx = create_transaction([_utxo()], _outputs(), 0)
        assert tx.version == 0
        assert tx.inputs[0].sig_op_count == 1
        assert tx.inputs[0].compute_budget == 0
        assert create_transaction([_utxo()], _outputs(), 0, sig_op_count=3).inputs[0].sig_op_count == 3

    def test_compute_budget_rejected(self):
        with pytest.raises(Exception, match="compute_budget"):
            create_transaction([_utxo()], _outputs(), 0, compute_budget=10)

    def test_covenant_output_rejected(self):
        with pytest.raises(Exception, match="covenant"):
            create_transaction([_utxo()], _covenant_outputs(), 0)


class TestCreateTransactionVersion1:
    """Version 1 commits compute_budget and allows covenant outputs."""

    def test_compute_budget_committed_per_input(self):
        tx = create_transaction([_utxo(), _utxo()], _outputs(), 0, compute_budget=10, version=1)
        assert tx.version == 1
        for tx_input in tx.inputs:
            assert tx_input.sig_op_count == 0
            assert tx_input.compute_budget == 10

    def test_missing_compute_budget_rejected(self):
        with pytest.raises(Exception, match="requires compute_budget"):
            create_transaction([_utxo()], _outputs(), 0, version=1)

    def test_sig_op_count_rejected(self):
        with pytest.raises(Exception, match="sig_op_count"):
            create_transaction([_utxo()], _outputs(), 0, sig_op_count=1, compute_budget=10, version=1)
