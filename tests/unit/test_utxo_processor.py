from kaspa import NetworkId, UtxoProcessor


def test_set_coinbase_transaction_maturity_daa_smoke():
    UtxoProcessor.set_coinbase_transaction_maturity_daa(NetworkId("testnet-10"), 1000)


def test_set_user_transaction_maturity_daa_smoke():
    UtxoProcessor.set_user_transaction_maturity_daa(NetworkId("testnet-10"), 100)
