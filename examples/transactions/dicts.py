from kaspa import ScriptPublicKey, TransactionOutput

def transaction_output_roundtrip():
    spk = ScriptPublicKey(0, "20236ab79a7254856a5e6c34906d0b47588dc444d7b4e9ac2acd1538727054eb0dac")
    original = TransactionOutput(1000000, spk)

    d = original.to_dict()
    print(d)

    restored = TransactionOutput.from_dict(d)

    assert original == restored

if __name__ == "__main__":
    print(transaction_output_roundtrip())