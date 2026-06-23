"""Run a SilverScript Counter covenant live on testnet-10, one tx at a time.

`kaspa.silverscript` compiles the contract to script bytes; the core `kaspa`
module builds and submits the transactions. The Counter holds one `count` in
covenant state, and each spend is a 1:1 transition that updates it and re-locks
the funds into the next Counter UTXO:

    genesis        count = 0
    add(5)         count + 5
    subtract(3)    count - 3

The count is baked into the locking script, so every count has its own P2SH
address. The contract checks no signature, so transitions are permissionless:
the unlocking script is just the covenant call plus the redeem script. Only the
funding input (a normal P2PK UTXO) is signed.

Setup:
    export KASPA_RPC_URL=<your testnet-10 node, e.g. ws://127.0.0.1:17210>
    python examples/silverscript/counter.py

Prints a funding address and waits for you to send it testnet KAS.
"""

import asyncio
import os
from dataclasses import dataclass

from kaspa import (
    Address,
    CovenantBinding,
    GenesisCovenantGroup,
    Hash,
    Keypair,
    PrivateKey,
    RpcClient,
    ScriptBuilder,
    ScriptPublicKey,
    Transaction,
    TransactionInput,
    TransactionOutpoint,
    TransactionOutput,
    UtxoEntryReference,
    address_from_script_public_key,
    calculate_transaction_mass,
    sign_transaction,
)
import kaspa.silverscript as silverscript

NETWORK_ID = "testnet-10"
NETWORK_TYPE = "testnet"
RPC_URL = os.environ.get("KASPA_RPC_URL")
SUBNETWORK_ID = bytes(20)
TX_VERSION = 1
COMPUTE_BUDGET = 10

# Silverscript source
SOURCE = """
pragma silverscript ^0.1.0;

contract Counter(int init_count) {
    int count = init_count;

    #[covenant(binding = auth, from = 1, to = 1, mode = transition)]
    function add(State prev_state, int amount) : (State) {
        return({ count: prev_state.count + amount });
    }

    #[covenant(binding = auth, from = 1, to = 1, mode = transition)]
    function subtract(State prev_state, int amount) : (State) {
        require(prev_state.count - amount >= 0);
        return({ count: prev_state.count - amount });
    }
}
"""

# =============================================================================
# Helpers for scripts, addresses, and Counter state
# =============================================================================

def lock_script(count: int) -> ScriptPublicKey:
    """Generate the P2SH locking script for the Counter at `count`.

    Args:
        count: The counter value baked into the contract.

    Returns:
        The P2SH (pay-to-script-hash) locking script.
    """
    redeem = silverscript.compile(SOURCE, [count]).script
    return ScriptBuilder.from_script(redeem, covenants_enabled=True).create_pay_to_script_hash_script()


def address(count: int) -> Address:
    """Encode the address of the P2SH locking script for the Counter at `count`.

    Args:
        count: The counter value baked into the contract.

    Returns:
        The Counter's address for `count`.
    """
    return address_from_script_public_key(lock_script(count), NETWORK_TYPE)


def unlock_script(count: int, function: str, amount: int) -> bytes:
    """Generate the unlocking script for a transition.

    The script is the covenant call for `function(amount)` followed by the
    pushed redeem script.

    Args:
        count: The current counter value, which selects the contract script.
        function: The covenant function to call ("add" or "subtract").
        amount: The argument passed to `function`.

    Returns:
        The unlocking (signature) script bytes.
    """
    contract = silverscript.compile(SOURCE, [count])
    call = contract.build_sig_script_for_covenant_decl(function, [amount])

    # Push the redeem script (hex -> bytes so it concatenates with the call).
    redeem = bytes.fromhex(
        ScriptBuilder(covenants_enabled=True).add_data(contract.script).to_string()
    )
    return call + redeem


@dataclass
class Counter:
    """Stores the live Counter UTXO state in memory.

    Attributes:
        txid: Id of the transaction that created this UTXO.
        count: The counter value held in covenant state.
        value: The UTXO amount, in sompi.
        covenant_id: The covenant's on-chain id, constant across transitions.
    """
    txid: str
    count: int
    value: int
    covenant_id: str

    @property
    def outpoint(self) -> TransactionOutpoint:
        # The Counter is always output 0 of its tx.
        return TransactionOutpoint(Hash(self.txid), 0)

    @property
    def utxo(self) -> UtxoEntryReference:
        # Rebuild the UTXO entry the node would return, so we can spend it.
        spk = lock_script(self.count)
        return UtxoEntryReference.from_dict({
            "address": address(self.count).to_string(),
            "outpoint": {"transactionId": self.txid, "index": 0},
            "utxoEntry": {
                "amount": self.value,
                "scriptPublicKey": {"version": spk.version, "script": spk.script},
                "blockDaaScore": 0,
                "isCoinbase": False,
                "covenantId": self.covenant_id,
            },
        })


# =============================================================================
# Helpers for building transactions
# =============================================================================

async def build_counter_tx(
    client: RpcClient,
    spend: TransactionInput,
    value_in: int,
    count: int,
    covenant: CovenantBinding | None,
) -> tuple[Transaction, int]:
    """Size and build a tx that spends `spend` into one Counter output at `count`.

    The fee is paid out of `value_in`. Mass (and so the fee) doesn't depend on
    the output amount, so it's measured on a draft first.

    Args:
        client: RPC client used to fetch the fee estimate.
        spend: The input being spent.
        value_in: The input's value, in sompi; the fee comes out of this.
        count: The counter value for the new Counter output.
        covenant: The output's CovenantBinding, or None to leave it unbound
            (the genesis case, where the caller derives the binding afterward).

    Returns:
        A tuple of (transaction, output_value), where output_value is
        `value_in` minus the fee.
    """
    spk = lock_script(count)

    # Use a draft tx to measure mass (and as a result fee)
    draft = Transaction(
        TX_VERSION, [spend], [TransactionOutput(value_in, spk, covenant)],
        lock_time=0, subnetwork_id=SUBNETWORK_ID, gas=0, payload=b"", mass=0,
    )
    mass = calculate_transaction_mass(NETWORK_ID, draft)
    estimate = await client.get_fee_estimate()
    fee = mass * int(estimate["estimate"]["priorityBucket"]["feerate"])
    value_out = value_in - fee

    # Rebuild with the fee deducted and the measured mass.
    tx = Transaction(
        TX_VERSION, [spend], [TransactionOutput(value_out, spk, covenant)],
        lock_time=0, subnetwork_id=SUBNETWORK_ID, gas=0, payload=b"", mass=mass,
    )
    return tx, value_out


async def genesis(client: RpcClient, funder_key: PrivateKey, funding_utxos: list[dict]) -> Counter:
    """Lock the funding UTXO into the first Counter (count = 0).

    Args:
        client: RPC client used to size and submit the tx.
        funder_key: Private key for the P2PK funding input.
        funding_utxos: Candidate funding UTXOs; the largest is spent.

    Returns:
        The genesis Counter (count = 0).
    """
    # Spend the largest funding UTXO (a normal P2PK output).
    funding = max(funding_utxos, key=lambda u: u["utxoEntry"]["amount"])
    spend = TransactionInput(
        TransactionOutpoint(Hash(funding["outpoint"]["transactionId"]), funding["outpoint"]["index"]),
        b"",
        sequence=0,
        sig_op_count=0,
        compute_budget=COMPUTE_BUDGET,
        utxo=UtxoEntryReference.from_dict(funding),
    )

    # Build the output unbound, then derive its covenant id from the funding input
    tx, value = await build_counter_tx(client, spend, funding["utxoEntry"]["amount"], count=0, covenant=None)
    tx.populate_genesis_covenants([GenesisCovenantGroup(authorizing_input=0, outputs=[0])])
    covenant_id = tx.outputs[0].to_dict()["covenant"]["covenantId"]

    signed = sign_transaction(tx, [funder_key], True)
    result = await client.submit_transaction({"transaction": signed, "allowOrphan": False})
    return Counter(result["transactionId"], count=0, value=value, covenant_id=covenant_id)


async def transition(client: RpcClient, counter: Counter, function: str, amount: int) -> Counter:
    """Spend the current Counter into the next by calling `function(amount)`.

    Args:
        client: RPC client used to size and submit the tx.
        counter: The current Counter UTXO being spent.
        function: The covenant function to call ("add" or "subtract").
        amount: The argument passed to `function`.

    Returns:
        The next Counter, carrying the same covenant id.
    """
    new_count = counter.count + amount if function == "add" else counter.count - amount
    
    # Spend the current Counter UTXO; the unlocking script runs the transition.
    spend = TransactionInput(
        counter.outpoint,
        unlock_script(counter.count, function, amount),
        sequence=0,
        sig_op_count=0,
        compute_budget=COMPUTE_BUDGET,
        utxo=counter.utxo,
    )

    # The covenant id carries over: the new output must keep the spent UTXO's id
    binding = CovenantBinding(authorizing_input=0, covenant_id=Hash(counter.covenant_id))
    tx, value = await build_counter_tx(client, spend, counter.value, new_count, binding)

    # No signing — the transition is permissionless as contract does not check sig
    result = await client.submit_transaction({"transaction": tx, "allowOrphan": False})
    return Counter(result["transactionId"], count=new_count, value=value, covenant_id=counter.covenant_id)


# =============================================================================
# Helpers for RPC
# =============================================================================

async def wait_for_funds(client: RpcClient, addr: Address) -> list[dict]:
    """Poll until `addr` has at least one UTXO, then return them.

    Args:
        client: RPC client used to query UTXOs.
        addr: The address to watch for funds.

    Returns:
        The UTXO entries found at `addr`.
    """
    while True:
        result = await client.get_utxos_by_addresses({"addresses": [addr]})
        if result["entries"]:
            return result["entries"]
        await asyncio.sleep(5)


async def wait_until_accepted(client: RpcClient, counter: Counter) -> None:
    """Poll the Counter's address until its output appears on-chain.

    Each count has its own address, so the output landing there means the
    transaction was accepted. Match on the txid in case a stale UTXO from an
    earlier run is already sitting at this (deterministic) address.

    Args:
        client: RPC client used to query UTXOs.
        counter: The Counter whose acceptance is awaited.
    """
    addr = address(counter.count)
    while True:
        result = await client.get_utxos_by_addresses({"addresses": [addr]})
        if any(e["outpoint"]["transactionId"] == counter.txid for e in result["entries"]):
            return
        await asyncio.sleep(1)


# =============================================================================
# Main
# =============================================================================

def show_step(label: str, counter: Counter) -> None:
    """Print the current Counter state.

    Args:
        label: A heading printed above the state.
        counter: The Counter whose state is printed.
    """
    print(label)
    print(f"  count     {counter.count}")
    print(f"  address   {address(counter.count).to_string()}")
    print(f"  covenant  {counter.covenant_id}")
    print(f"  value     {counter.value:,} sompi")
    print(f"  tx        {counter.txid}")
    print()


async def main() -> None:
    if not RPC_URL:
        raise SystemExit("Set KASPA_RPC_URL to your testnet-10 node URL (e.g. ws://127.0.0.1:17210).")

    print(f"SilverScript Counter — live on {NETWORK_ID}\n")

    client = RpcClient(url=RPC_URL, network_id=NETWORK_ID)
    print(f"Connecting to RPC host {RPC_URL}...")
    await client.connect(strategy="fallback")
    print(f"Connected\n")

    try:
        # Every run of this script uses a new (random) key
        keypair = Keypair.random()
        funder_key = PrivateKey(keypair.private_key)
        funding_address = keypair.to_address(NETWORK_TYPE)
        print("Fund this address with testnet KAS (TKAS):")
        print(f"{funding_address.to_string()}\n")
        print("Polling for funds (automatically continues within a few seconds of funding)...\n")
        funding_utxos = await wait_for_funds(client, funding_address)

        # Genesis transaction: create Counter covernant with count = 0
        counter = await genesis(client, funder_key, funding_utxos)
        await wait_until_accepted(client, counter)
        show_step("genesis      count = 0", counter)

        # Add 5 transaction: the output re-locks the funds at the count=5 address
        prev = counter.count
        counter = await transition(client, counter, "add", 5)
        await wait_until_accepted(client, counter)
        show_step(f"add(5)       count {prev} -> {counter.count}", counter)

        # Subtract 3 transaction: the output re-locks the funds at the count=2 address
        prev = counter.count
        counter = await transition(client, counter, "subtract", 3)
        await wait_until_accepted(client, counter)
        show_step(f"subtract(3)  count {prev} -> {counter.count}", counter)

        print(
            f"Final count = {counter.count}\n")
    finally:
        await client.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
