"""Unit tests for the kaspa.silverscript module."""

import pytest

import kaspa.silverscript as silverscript

GUARD = """
pragma silverscript ^0.1.0;
contract Guard(int threshold) {
    entrypoint function check(int amount) {
        require(amount > threshold);
    }
}
"""

ANNOUNCEMENT = """
pragma silverscript ^0.1.0;
contract Announcement() {
    entrypoint function announce() {
        require(tx.outputs[0].value == 0);
    }
}
"""


class TestCompile:
    def test_returns_compiled_contract(self):
        contract = silverscript.compile(GUARD, [100])
        assert contract.contract_name == "Guard"
        assert contract.compiler_version
        assert isinstance(contract.script, bytes)
        assert len(contract.script) > 0

    def test_state_layout_is_pair(self):
        contract = silverscript.compile(GUARD, [100])
        assert isinstance(contract.state_layout, tuple)
        assert len(contract.state_layout) == 2

    def test_abi(self):
        contract = silverscript.compile(GUARD, [100])
        assert len(contract.abi) == 1
        entry = contract.abi[0]
        assert entry.name == "check"
        assert [(i.name, i.type_name) for i in entry.inputs] == [("amount", "int")]


class TestSigScript:
    def test_build_sig_script_encodes_int_arg(self):
        contract = silverscript.compile(GUARD, [100])
        sig = contract.build_sig_script("check", [150])
        assert isinstance(sig, bytes)
        # 150 pushed as a minimally-encoded script number; single entrypoint so
        # no function selector is appended.
        assert sig.hex() == "029600"

    def test_single_entrypoint_has_empty_sig_for_no_args(self):
        contract = silverscript.compile(ANNOUNCEMENT)
        assert contract.without_selector is True
        assert contract.build_sig_script("announce") == b""


class TestErrors:
    def test_invalid_source_raises(self):
        with pytest.raises(silverscript.SilverScriptError):
            silverscript.compile("this is not silverscript")

    def test_unknown_function_raises(self):
        contract = silverscript.compile(GUARD, [100])
        with pytest.raises(silverscript.SilverScriptError):
            contract.build_sig_script("does_not_exist", [1])

    def test_wrong_constructor_arity_raises(self):
        with pytest.raises(silverscript.SilverScriptError):
            silverscript.compile(GUARD, [])
