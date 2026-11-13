"""
Unit tests for RpcClient request wiring that need no node and no event loop.

An RpcClient method parses and validates its request dict synchronously,
*before* handing the call off to the async runtime. So a malformed request
raises immediately -- without a connection or a running event loop -- whereas a
well-formed request would instead fail later with "no running event loop". These
tests assert the former and use the latter as the discriminator that proves
validation happens up front.
"""

import pytest

from kaspa import RpcClient

# An offline client: constructed but never connected.
URL = "ws://127.0.0.1:17110"


def _client():
    return RpcClient(url=URL, network_id="mainnet")


class TestRpcClientConstruction:
    def test_construct_offline(self):
        client = _client()
        assert isinstance(client, RpcClient)
        assert client.is_connected is False


class TestGetSeqCommitLaneProofWiring:
    """get_seq_commit_lane_proof is exposed and validates its request
    (blockHash + laneKey) synchronously, before any network/await."""

    def test_missing_required_field_rejected_synchronously(self):
        with pytest.raises(Exception, match="laneKey"):
            _client().get_seq_commit_lane_proof({"blockHash": "a" * 64})

    def test_malformed_block_hash_rejected_synchronously(self):
        with pytest.raises(Exception) as exc_info:
            _client().get_seq_commit_lane_proof(
                {"blockHash": "not-a-hash", "laneKey": "a" * 64}
            )
        # Rejected by hash parsing, not by the async runtime: confirms validation
        # runs before the await (no event loop required).
        assert "no running event loop" not in str(exc_info.value)


class TestGetUtxosByAddressesV2Wiring:
    """get_utxos_by_addresses_v2 is exposed and validates its request
    (addresses + optional bounds/cursor/limit) synchronously."""

    def test_missing_addresses_rejected_synchronously(self):
        with pytest.raises(Exception, match="addresses"):
            _client().get_utxos_by_addresses_v2({"limit": 10})

    def test_malformed_address_rejected_synchronously(self):
        with pytest.raises(Exception) as exc_info:
            _client().get_utxos_by_addresses_v2({"addresses": ["not-an-address"]})
        assert "no running event loop" not in str(exc_info.value)

    def test_malformed_cursor_rejected_synchronously(self):
        with pytest.raises(Exception) as exc_info:
            _client().get_utxos_by_addresses_v2({
                "addresses": ["kaspa:qpamkvhgh0kzx50gwvvp5xs8ktmqutcy3dfs9dc3w7lm9rq0zs76vf959mmrp"],
                "cursor": {"startDaaScore": "not-a-number"},
            })
        assert "no running event loop" not in str(exc_info.value)


class TestGetUtxosByCovenantIdsWiring:
    """get_utxos_by_covenant_ids is exposed and validates its request
    (covenantIds + scriptPublicKeys) synchronously."""

    def test_missing_required_field_rejected_synchronously(self):
        with pytest.raises(Exception, match="scriptPublicKeys"):
            _client().get_utxos_by_covenant_ids({"covenantIds": ["a" * 64]})

    def test_malformed_covenant_id_rejected_synchronously(self):
        with pytest.raises(Exception) as exc_info:
            _client().get_utxos_by_covenant_ids(
                {"covenantIds": ["not-a-hash"], "scriptPublicKeys": []}
            )
        assert "no running event loop" not in str(exc_info.value)


class TestGetTransactionsByAcceptingBlockWiring:
    """get_transactions_by_accepting_block is exposed and validates its request
    (acceptingBlock selector + transactionIds) synchronously."""

    def test_missing_required_field_rejected_synchronously(self):
        with pytest.raises(Exception, match="transactionIds"):
            _client().get_transactions_by_accepting_block(
                {"acceptingBlock": {"daaScore": 1}}
            )

    def test_unknown_selector_rejected_synchronously(self):
        with pytest.raises(Exception) as exc_info:
            _client().get_transactions_by_accepting_block(
                {"acceptingBlock": {"blueScore": 1}, "transactionIds": ["a" * 64]}
            )
        assert "no running event loop" not in str(exc_info.value)

    def test_malformed_hash_rejected_synchronously(self):
        with pytest.raises(Exception) as exc_info:
            _client().get_transactions_by_accepting_block(
                {"acceptingBlock": {"hash": "not-a-hash"}, "transactionIds": ["a" * 64]}
            )
        assert "no running event loop" not in str(exc_info.value)
