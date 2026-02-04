import asyncio
from kaspa import (
    Hash,
    Mnemonic,
    Opcodes,
    PSKT,
    Resolver,
    RpcClient,
    ScriptBuilder,
    TransactionInput,
    TransactionOutpoint,
    UtxoEntryReference,
    XPrv,
    address_from_script_public_key,
    calculate_transaction_mass,
    create_transaction,
    sign_transaction,
)


def derive(seed, account_index):
    xprv = XPrv(seed).derive_path(f"m/45'/111111'/{account_index}'")
    xpub = xprv.to_xpub()
    prv = xprv.derive_child(1).to_private_key()
    pub = xpub.derive_child(1).to_public_key()
    return prv, pub


async def main():
    #######################################################
    # Derive 3 accounts to use for Multisig PSKT demo
    ####################################################### 
    seed = Mnemonic((
        'predict cloud noise economy home stereo tag cancel adult pistol act remove '
        'equip cricket man summer neutral black art miracle foam world clown say'
    )).to_seed()
    
    prv1, pub1 = derive(seed, 0)
    print(f'Account 1:\n - prv: {prv1.to_string()}\n - pub: {pub1.to_string()}\n')

    prv2, pub2 = derive(seed, 1)
    print(f'Account 2:\n - prv: {prv2.to_string()}\n - pub: {pub2.to_string()}\n')

    prv3, pub3 = derive(seed, 2)
    print(f'Account 3:\n - prv: {prv3.to_string()}\n - pub: {pub3.to_string()}\n')

    #######################################################
    # Create Multisig address
    #######################################################
    redeem_script = ScriptBuilder()\
        .add_i64(2)\
        .add_data(pub1.to_x_only_public_key().to_string())\
        .add_data(pub2.to_x_only_public_key().to_string())\
        .add_data(pub3.to_x_only_public_key().to_string())\
        .add_i64(3)\
        .add_op(Opcodes.OpCheckMultiSig)
    spk = redeem_script.create_pay_to_script_hash_script()
    address = address_from_script_public_key(spk, "testnet")

    print(f"Multisig address: {address}")

    while True:
        if input("Send funds to address (y to proceed): ") == "y":
            break

    #######################################################
    # Get address's UTXOs
    #######################################################
    client = RpcClient(resolver=Resolver(), network_id='testnet-10')
    await client.connect(strategy='fallback')
    utxos = await client.get_utxos_by_addresses(request={'addresses': [address]})
    utxos = utxos["entries"]
    utxos = sorted(utxos, key=lambda x: x['utxoEntry']['amount'], reverse=True)
    total = sum(item["utxoEntry"]["amount"] for item in utxos)
    print(utxos)
    # utxo = utxos["entries"][0]

    #######################################################
    # Placeholder TX for fee calculation
    #######################################################
    # outputs = [
    #     {"address": address, "amount": int(total)}
    # ]
    # tx = create_transaction(utxos, outputs, 0, None, 2)
    # mass = calculate_transaction_mass("testnet-10", tx)

    #######################################################
    # Get feerates & create actual TX
    #######################################################
    # fee_rates = await client.get_fee_estimate()
    # fee_rate = int(fee_rates["estimate"]["priorityBucket"]["feerate"])

    # outputs = [
    #     {"address": address, "amount": int(total - (fee_rate * mass)), "scriptPublicKey": ""}
    # ]
    # tx = create_transaction(utxos, outputs, 0, None, 1)
    # tx_signed = sign_transaction(tx, [prv1], True)

    #######################################################
    # Create PSKT
    #######################################################
    pskt = PSKT()
    pskt_serialized = pskt.serialize()
    print(pskt_serialized)

    #######################################################
    # Create input
    #######################################################
    input0 = TransactionInput.from_dict({
        'previousOutpoint': { 'transactionId': 'c38eb7191a2e0df6089b05cf7df9c92dc559db618184b11cbb8c5ba30b024bce', 'index': 1 },
        'signatureScript': '',
        'sequence': 0,
        'sigOpCount': 1,
        'utxo': {
            'utxo': {
                'address': 'kaspatest:prganzek6uhsn4rv29g6qkeh8rduae6n3ul0xk5fnzjtugqhfaxcx0ee2dn47',
                'outpoint': {'transactionId': 'c38eb7191a2e0df6089b05cf7df9c92dc559db618184b11cbb8c5ba30b024bce', 'index': 1},
                'amount': 98699920028,
                'scriptPublicKey': '0000aa20d1d98b36d72f09d46c5151a05b3738dbcee7538f3ef35a8998a4be20174f4d8387',
                'blockDaaScore': 354263497,
                'isCoinbase': False
            }
        }
    })
    # pskt = pskt.input(input)

    # previous_outpoint = TransactionOutpoint(
    #     transaction_id=Hash(utxo["outpoint"]['transactionId']),
    #     index=utxo["outpoint"]['index']
    # )
    # input_0 = TransactionInput(
    #     previous_outpoint=previous_outpoint,
    #     signature_script=b"",
    #     sequence=0,
    #     sig_op_count=2,
    #     utxo=None
    # )
    # pskt.to_constructor().input(input_0)
    # print(pskt.serialize())

if __name__ == "__main__":
    asyncio.run(main())