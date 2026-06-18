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
