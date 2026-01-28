from kaspa import ScriptPublicKey, TransactionOutput

def transaction_output_roundtrip():
    spk = ScriptPublicKey(0, "51")
    original = TransactionOutput(1000000, spk)

    d = original.to_dict()
    print(d)

    restored = TransactionOutput.from_dict(d)

    assert original == restored

if __name__ == "__main__":
    print(transaction_output_roundtrip())