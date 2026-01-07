# Transactions

This guide covers building, signing, and broadcasting transactions with the Kaspa Python SDK.

!!! danger "Security Warning"
    **Handle Private Keys Securely**

    **These examples do not use proper private key/mnemonic/seed handling.** This is omitted here for brevity.

    Never store your private keys in plain text, or directly in source code. Store securely offline. Anyone with access to this phrase has full control over your funds.

## Using the Generator

The `Generator` class handles UTXO selection, fee calculation, and change management:

```python
import asyncio
from kaspa import (
    RpcClient, Resolver, Generator, PaymentOutput,
    Address, PrivateKey, NetworkId
)

async def send_payment():
    # Connect to network
    client = RpcClient(resolver=Resolver(), network_id="mainnet")
    await client.connect()
    
    try:
        # Your private key and address
        private_key = PrivateKey("your-private-key-hex")
        my_address = private_key.to_address("mainnet")
        
        # Fetch UTXOs
        utxos = await client.get_utxos_by_addresses({
            "addresses": [my_address.to_string()]
        })
        
        # Define payment
        recipient = Address("kaspa:recipient-address...")
        amount = 500_000_000  # 5 KAS in sompi
        
        # Create generator
        generator = Generator(
            network_id=NetworkId("mainnet"),
            entries=utxos["entries"],
            change_address=my_address,
            outputs=[PaymentOutput(recipient, amount)],
        )
        
        # Process transactions
        for pending_tx in generator:
            pending_tx.sign([private_key])
            tx_id = await pending_tx.submit(client)
            print(f"Submitted: {tx_id}")
        
        # Get summary
        summary = generator.summary()
        print(f"Total fees: {summary.fees}")
        print(f"Transactions: {summary.transactions}")
        
    finally:
        await client.disconnect()

asyncio.run(send_payment())
```

## Generator Options

```python
from kaspa import Generator, NetworkId, PaymentOutput

generator = Generator(
    # Required
    network_id=NetworkId("mainnet"),
    entries=utxo_entries,           # List of UTXOs
    change_address=my_address,       # Where to send change
    
    # Optional
    outputs=[payment1, payment2],    # Payment outputs
    payload=b"optional-data",        # OP_RETURN data
    priority_fee=1000,               # Additional fee in sompi
    priority_entries=priority_utxos, # UTXOs to use first
    sig_op_count=1,                  # Signature operations per input
    minimum_signatures=1,            # For multisig estimation
)
```

## Estimating Transactions

Transactions can be estimated prior to submission.

```python
from kaspa import Generator, estimate_transactions

# Using Generator.estimate()
generator = Generator(
    network_id="mainnet",
    entries=utxos,
    change_address=my_address,
    outputs=[PaymentOutput(recipient, amount)],
)

summary = generator.estimate()
print(f"Estimated fee: {summary.fees} sompi")
print(f"Number of transactions: {summary.transactions}")
print(f"UTXOs consumed: {summary.utxos}")

# Using standalone function
summary = estimate_transactions(
    network_id="mainnet",
    entries=utxos,
    change_address=my_address,
    outputs=[{"address": recipient, "amount": amount}],
)
```

## Pending Transactions

The `PendingTransaction` represents a transaction ready for signing:

```python
for pending_tx in generator:
    # Transaction properties
    print(f"ID: {pending_tx.id}")
    print(f"Payment amount: {pending_tx.payment_amount}")
    print(f"Change amount: {pending_tx.change_amount}")
    print(f"Fee: {pending_tx.fee_amount}")
    print(f"Mass: {pending_tx.mass}")
    print(f"Type: {pending_tx.transaction_type}")
    
    # Get UTXOs being spent
    utxo_refs = pending_tx.get_utxo_entries()
    
    # Get addresses involved
    addresses = pending_tx.addresses()
    
    # Access underlying transaction
    tx = pending_tx.transaction
```

## Signing Transactions

### Simple Signing

```python
# Sign with one or more private keys
pending_tx.sign([private_key])

# Or sign with multiple keys for multisig
pending_tx.sign([key1, key2, key3])
```

### Per-Input Signing

For more control, sign each input individually:

```python
for i, utxo in enumerate(pending_tx.get_utxo_entries()):
    pending_tx.sign_input(i, private_key)
```

### Custom Signature Scripts

For advanced use cases (like multisig):

```python
from kaspa import SighashType

# Create signature
signature = pending_tx.create_input_signature(
    input_index=0,
    private_key=private_key,
    sighash_type=SighashType.All
)

# Set custom signature script
pending_tx.fill_input(0, signature_script_bytes)
```

## Manual Transaction Building

Transactions can be built manually:

```python
from kaspa import (
    Transaction, TransactionInput, TransactionOutput,
    TransactionOutpoint, ScriptPublicKey, UtxoEntryReference,
    sign_transaction
)

# Create inputs from UTXOs
inputs = []
for utxo in my_utxos:
    outpoint = TransactionOutpoint(
        transaction_id=utxo["outpoint"]["transactionId"],
        index=utxo["outpoint"]["index"]
    )
    tx_input = TransactionInput(
        previous_outpoint=outpoint,
        signature_script="",  # Will be filled when signing
        sequence=0,
        sig_op_count=1,
        utxo=UtxoEntryReference(utxo)
    )
    inputs.append(tx_input)

# Create outputs
outputs = [
    TransactionOutput(
        value=amount,
        script_public_key=pay_to_address_script(recipient)
    ),
    TransactionOutput(
        value=change_amount,
        script_public_key=pay_to_address_script(change_address)
    )
]

# Build transaction
tx = Transaction(
    version=0,
    inputs=inputs,
    outputs=outputs,
    lock_time=0,
    subnetwork_id="0000000000000000000000000000000000000000",
    gas=0,
    payload="",
    mass=0
)

# Calculate and update mass
from kaspa import update_transaction_mass
update_transaction_mass("mainnet", tx)

# Sign
signed_tx = sign_transaction(tx, [private_key], verify_sig=True)
```

## Transaction Mass and Fees

Kaspa uses a mass-based fee model:

```python
from kaspa import (
    calculate_transaction_mass,
    calculate_transaction_fee,
    calculate_storage_mass,
    maximum_standard_transaction_mass
)

# Get maximum allowed mass
max_mass = maximum_standard_transaction_mass()
print(f"Max mass: {max_mass}")

# Calculate transaction mass
mass = calculate_transaction_mass("mainnet", tx)
print(f"Transaction mass: {mass}")

# Calculate required fee
fee = calculate_transaction_fee("mainnet", tx)
print(f"Required fee: {fee} sompi")

# Calculate storage mass component
storage_mass = calculate_storage_mass(
    network_id="mainnet",
    input_values=[1000000, 2000000],
    output_values=[2500000, 400000]
)
```

## Submitting Transactions

```python
# Using PendingTransaction
tx_id = await pending_tx.submit(client)

# Manual submission
result = await client.submit_transaction({
    "transaction": tx.serialize_to_dict(),
    "allowOrphan": False
})
```

## Helper Functions

### Create Single Transaction

```python
from kaspa import create_transaction

tx = create_transaction(
    utxo_entry_source=utxos,
    outputs=[{"address": "kaspa:...", "amount": 100000000}],
    priority_fee=1000,
    payload=None,
    sig_op_count=1
)
```

### Create Multiple Transactions

```python
from kaspa import create_transactions

result = create_transactions(
    network_id="mainnet",
    entries=utxos,
    change_address=my_address,
    outputs=[{"address": "kaspa:...", "amount": 100000000}],
    priority_fee=1000,
)

for pending in result["transactions"]:
    pending.sign([private_key])
    # submit...

print(f"Summary: {result['summary']}")
```

## Multi-Signature Transactions

```python
from kaspa import (
    Generator, create_multisig_address,
    PublicKey, NetworkType
)

# Create multisig address (2-of-3)
pubkeys = [PublicKey(k) for k in [key1_pub, key2_pub, key3_pub]]
multisig_addr = create_multisig_address(2, pubkeys, NetworkType.Mainnet)

# Build transaction spending from multisig
generator = Generator(
    network_id="mainnet",
    entries=multisig_utxos,
    change_address=multisig_addr,
    outputs=[PaymentOutput(recipient, amount)],
    minimum_signatures=2,  # For accurate mass calculation
)

for pending_tx in generator:
    # Collect signatures from 2 of 3 signers
    pending_tx.sign([signer1_key, signer2_key])
    tx_id = await pending_tx.submit(client)
```

## Unit Conversions

```python
from kaspa import kaspa_to_sompi, sompi_to_kaspa, sompi_to_kaspa_string_with_suffix

# KAS to sompi
sompi = kaspa_to_sompi(1.5)  # 150,000,000 sompi

# Sompi to KAS
kas = sompi_to_kaspa(150000000)  # 1.5 KAS

# Formatted string
formatted = sompi_to_kaspa_string_with_suffix(150000000, "mainnet")
# "1.5 KAS"
```
