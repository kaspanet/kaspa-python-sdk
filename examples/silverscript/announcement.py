"""Compile a SilverScript contract and build a spend, all from Python.

`kaspa.silverscript` is a separate native module (it links a different
rusty-kaspa revision than the core), but its output is plain script bytes, so it
feeds directly into the core `kaspa` transaction-construction types.
"""

from kaspa import ScriptBuilder
import kaspa.silverscript as silverscript

SOURCE = """
pragma silverscript ^0.1.0;

contract Announcement() {
    entrypoint function announce() {
        byte[] announcement = new LockingBytecodeNullData([
            27906,
            byte[]('A contract may not injure a human being or, through inaction, allow a human being to come to harm.')
        ]);

        require(tx.outputs[0].value == 0);
        require(tx.outputs[0].scriptPubKey == announcement);

        int minerFee = 1000;
        int changeAmount = tx.inputs[this.activeInputIndex].value - minerFee;
        if (changeAmount >= minerFee) {
            require(tx.outputs[1].scriptPubKey == tx.inputs[this.activeInputIndex].scriptPubKey);
            require(tx.outputs[1].value == changeAmount);
        }
    }
}
"""

if __name__ == "__main__":
    contract = silverscript.compile(SOURCE)

    print("contract :", contract.contract_name)
    print("compiler :", contract.compiler_version)
    print("script   :", len(contract.script), "bytes")
    print("abi      :")
    for fn in contract.abi:
        inputs = ", ".join(f"{i.name}: {i.type_name}" for i in fn.inputs)
        print(f"             {fn.name}({inputs})")

    # Unlocking (signature) script for the `announce` entrypoint (no args).
    sig_script = contract.build_sig_script("announce")
    print("sig script:", sig_script.hex() or "(empty)")

    # The compiled locking script flows straight into the core kaspa module:
    # wrap it as a pay-to-script-hash output.
    p2sh = ScriptBuilder.from_script(contract.script).create_pay_to_script_hash_script()
    print("p2sh spk :", p2sh.script)
