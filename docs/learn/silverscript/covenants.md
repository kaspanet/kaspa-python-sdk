---
search:
  boost: 3
---

# Covenants

A **covenant** is a contract that carries its state from one UTXO to the
next.

The state lives in the locking script, so each state results in its own P2SH
address (see
[Compiling → Constructor args bake state](compiling.md#constructor-args-bake-state-into-the-script)).
A transition spends the UTXO at the current address and creates a new
output at a new address (which is derived from the newly created covenant state).

## Calling a covenant entrypoint

Spend a covenant entrypoint with
[`build_sig_script_for_covenant_decl`](../../reference/SilverScript/Classes/CompiledContract.md),
not
[`build_sig_script`](unlocking-scripts.md). It finds the covenant's
declaration entrypoint and builds the unlocking script for it:

```python
contract = silverscript.compile(SOURCE, [current_count])
call = contract.build_sig_script_for_covenant_decl("add", [amount])
```

The `is_leader` keyword (default `False`) matters only for covenants that
spend several UTXOs of the same covenant in one transition and pick one
input as the **leader**. The leader input carries the entrypoint's
arguments and runs the covenant's logic; the other inputs are
**delegates** — they take no arguments and only prove they belong to the
same covenant, deferring to the leader. Build the leader's unlocking
script with `is_leader=True` and each delegate's with `is_leader=False`.

The Counter here never needs it. Its `binding = auth` compiles to a single
entrypoint with no leader/delegate split, so `is_leader` is ignored — you
reach for `is_leader=True` only when building the leader input of a
multi-input covenant (`binding = cov`).

The rest is the same as a plain spend: the input's `signature_script` is
the covenant call followed by the pushed redeem script (see
[Unlocking Scripts → Spending a locked UTXO](unlocking-scripts.md#spending-a-locked-utxo)).

## State transition via transactions

Covenant state advances (transitions) via transactions on the Kaspa network. Actual on-chain side of a covenant is core `kaspa` work, not the compiler's.
A transition is a transaction that:

1. **Spends** the current covenant UTXO, with the covenant call as its
   `signature_script`.
2. **Creates** a new output at the next state's address, carrying the
   covenant forward with a
   [`CovenantBinding`](../../reference/Classes/CovenantBinding.md) so the
   chain keeps the same covenant id.

The first transaction (genesis) locks an ordinary funding UTXO into the
initial state. It derives the covenant id with
[`populate_genesis_covenants`](../../reference/Classes/Transaction.md) and a
[`GenesisCovenantGroup`](../../reference/Classes/GenesisCovenantGroup.md).
Every later transition reuses that id.

## Worked example: Counter

[`examples/silverscript/counter.py`](https://github.com/kaspanet/kaspa-python-sdk/blob/main/examples/silverscript/counter.py)
runs the whole loop live on testnet-10. The contract holds one `count`
in covenant state. Each spend is a 1:1 transition that updates the count
and re-locks the funds:

```
genesis        count = 0
add(5)         count -> 5
subtract(3)    count -> 2
```

The contract:

```silverscript
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
```

A few things from the example worth noting:

- **State is in the address.** `count = 0` and `count = 5` are different
  scripts at different addresses. The example re-derives each address
  from the source and the count alone.
- **No signing on the transition.** The Counter checks no signature, so
  the unlocking script is just the covenant call plus the redeem script —
  the transition is permissionless. Only the genesis funding input (a
  normal P2PK UTXO) is signed.
- **The covenant id carries over.** Genesis derives it; each transition
  binds the new output to it with a
  [`CovenantBinding`](../../reference/Classes/CovenantBinding.md).

The compiler's role is small: compile the contract at the current count,
and build the covenant call. Transaction building, fee sizing, covenant
binding, and submission are all core `kaspa` — see
[Transactions](../transactions/overview.md).

!!! warning "Experimental"
    Covenants are the newest and least-settled part of SilverScript.
    Treat the contract above as a teaching example, not a template for
    production value. Verify everything on a test network first.
