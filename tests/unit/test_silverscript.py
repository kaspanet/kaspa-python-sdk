"""
Unit tests for the kaspa.experimental.silverscript module.
"""

import os
import subprocess
import sys
import textwrap

import pytest

import kaspa.experimental.silverscript as silverscript

# The core `kaspa` extension links a different rusty-kaspa revision; importing it
# lets us exercise the bytes-only boundary (compile here, wrap to P2SH there).
# Guarded so the file still runs if the core script API is unavailable.
try:
    import kaspa as _kaspa

    _HAVE_CORE_SCRIPT_API = hasattr(_kaspa, "ScriptBuilder") and hasattr(
        _kaspa, "address_from_script_public_key"
    )
except Exception:  # pragma: no cover - core module should always be present
    _kaspa = None
    _HAVE_CORE_SCRIPT_API = False


# ---------------------------------------------------------------------------
# Contract fixtures
# ---------------------------------------------------------------------------

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

# Two entrypoints -> a function selector is appended to the sig script.
MULTI = """
pragma silverscript ^0.1.0;
contract Multi(int base) {
    entrypoint function add(int amount) { require(amount > base); }
    entrypoint function sub(int amount) { require(amount < base); }
}
"""

BYTES4 = """
pragma silverscript ^0.1.0;
contract H(byte[4] tag) {
    entrypoint function go(byte[4] x) { require(x == tag); }
}
"""

LIST_ARG = """
pragma silverscript ^0.1.0;
contract L() {
    entrypoint function f(int[] xs) { require(true); }
}
"""

ENTRYPOINT_RETURN = """
pragma silverscript ^0.1.0;
contract R() {
    entrypoint function f() : (int) { return(1); }
}
"""

# Mirror of silverscript-lang tutorial_rust_examples_tests.rs ::
# tutorial_rust_build_sigscript_multiple_entrypoints_example — a realistic
# pubkey/sig multi-entrypoint contract.
TRANSFER_WITH_TIMEOUT = """
pragma silverscript ^0.1.0;
contract TransferWithTimeout(pubkey sender, pubkey recipient, int timeout) {
    entrypoint function transfer(sig recipientSig) {
        require(checkSig(recipientSig, recipient));
    }
    entrypoint function reclaim(sig senderSig) {
        require(checkSig(senderSig, sender));
        require(tx.time >= timeout);
    }
}
"""

# Mirror of silverscript-lang compiler_tests.rs :: build_sig_script_builds_expected_script
# (a multi-argument entrypoint: byte[4] then int). Note: no pragma, like upstream.
BOUNDED_BYTES = """
contract BoundedBytes() {
    entrypoint function spend(byte[4] b, int i) { require(b == byte[4](i)); }
}
"""

# A real covenant contract (the Counter from examples/silverscript/counter.py):
# state is carried in covenant State, spent via build_sig_script_for_covenant_decl.
COUNTER = """
pragma silverscript ^0.1.0;
contract Counter(int init_count) {
    int count = init_count;
    #[covenant(binding = auth, from = 1, to = 1, mode = transition)]
    function add(State prev_state, int amount) : (State) {
        return({ count: prev_state.count + amount });
    }
    #[covenant(binding = auth, from = 1, to = 1, mode = transition)]
    function subtract(State prev_state, int amount) : (State) {
        return({ count: prev_state.count - amount });
    }
}
"""

I64_MAX = 2**63 - 1
I64_MIN = -(2**63)


def _run_in_subprocess(snippet: str) -> subprocess.CompletedProcess:
    """Run `snippet` in a fresh interpreter and return the completed process.

    Used for inputs that might crash the process (native stack overflow); a
    segfault there shows up as a negative `returncode` instead of taking the
    whole test session down with it.
    """
    env = dict(os.environ)
    env["PYTHONPATH"] = os.pathsep.join(p for p in sys.path if p)
    return subprocess.run(
        [sys.executable, "-c", snippet],
        capture_output=True,
        text=True,
        env=env,
        timeout=120,
    )


# ---------------------------------------------------------------------------
# Compilation basics
# ---------------------------------------------------------------------------

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

    def test_constructor_args_default_to_empty(self):
        # A no-arg constructor must be callable without passing constructor_args.
        contract = silverscript.compile(ANNOUNCEMENT)
        assert contract.contract_name == "Announcement"


# ---------------------------------------------------------------------------
# Golden locking-script bytes — the on-chain locking artifact
# ---------------------------------------------------------------------------

class TestGoldenScript:
    def test_guard_locking_script(self):
        # Pinned: the redeem script defines the P2SH address holding the funds.
        assert silverscript.compile(GUARD, [100]).script.hex() == "760164a0697551"

    def test_bytes_contract_locking_script(self):
        contract = silverscript.compile(BYTES4, [b"\x01\x02\x03\x04"])
        assert contract.script.hex() == "76040102030487697551"


# ---------------------------------------------------------------------------
# Golden unlocking (signature) scripts — the bytes that spend a UTXO
# ---------------------------------------------------------------------------

class TestGoldenSigScript:
    def test_int_arg_encoding(self):
        contract = silverscript.compile(GUARD, [100])
        # 150 pushed as a minimally-encoded script number; single entrypoint so
        # no function selector is appended.
        assert contract.build_sig_script("check", [150]).hex() == "029600"

    def test_zero_arg_encoding(self):
        contract = silverscript.compile(GUARD, [100])
        assert contract.build_sig_script("check", [0]).hex() == "00"

    def test_negative_int_encoding(self):
        contract = silverscript.compile(GUARD, [100])
        assert contract.build_sig_script("check", [-5]).hex() == "0185"

    def test_bytes_arg_encoding(self):
        contract = silverscript.compile(BYTES4, [b"\x01\x02\x03\x04"])
        assert contract.build_sig_script("go", [b"\xaa\xbb\xcc\xdd"]).hex() == "04aabbccdd"

    def test_list_arg_encoding(self):
        contract = silverscript.compile(LIST_ARG)
        assert (
            contract.build_sig_script("f", [[1, 2, 3]]).hex()
            == "18010000000000000002000000000000000300000000000000"
        )

    def test_empty_list_arg_encoding(self):
        contract = silverscript.compile(LIST_ARG)
        assert contract.build_sig_script("f", [[]]).hex() == "00"

    def test_single_entrypoint_has_empty_sig_for_no_args(self):
        contract = silverscript.compile(ANNOUNCEMENT)
        assert contract.without_selector is True
        assert contract.build_sig_script("announce") == b""

    def test_multi_entrypoint_appends_selector(self):
        contract = silverscript.compile(MULTI, [10])
        assert contract.without_selector is False
        # Each entrypoint encodes its arg plus a distinct selector.
        assert contract.build_sig_script("add", [20]).hex() == "011400"
        assert contract.build_sig_script("sub", [5]).hex() == "5551"


# ---------------------------------------------------------------------------
# Parity with silverscript-lang's own compiler tests (pinned rev 2a3961c)
# ---------------------------------------------------------------------------

class TestUpstreamParity:
    def test_multi_arg_sig_script_matches_upstream_vector(self):
        # Mirrors compiler_tests.rs :: build_sig_script_builds_expected_script.
        # Upstream builds: push byte[4] {01,02,03,04}, then i64(7); single
        # entrypoint -> no selector. We assert the exact resulting bytes.
        contract = silverscript.compile(BOUNDED_BYTES)
        assert contract.build_sig_script("spend", [b"\x01\x02\x03\x04", 7]).hex() == "040102030457"

    def test_transfer_with_timeout_multi_entrypoint(self):
        # Mirrors tutorial_rust_build_sigscript_multiple_entrypoints_example.
        sender = bytes([3]) * 32
        recipient = bytes([4]) * 32
        timeout = 1_640_000_000
        contract = silverscript.compile(TRANSFER_WITH_TIMEOUT, [sender, recipient, timeout])
        assert contract.without_selector is False
        assert [(i.name, i.type_name) for e in contract.abi for i in e.inputs] == [
            ("recipientSig", "sig"),
            ("senderSig", "sig"),
        ]

        sig = bytes([5]) * 65
        transfer = contract.build_sig_script("transfer", [sig])
        reclaim = contract.build_sig_script("reclaim", [sig])
        # 0x41 = 65-byte data push, followed by the signature bytes verbatim.
        assert transfer[0] == 0x41
        assert transfer[1:66] == sig
        # Same arg, different entrypoint -> different selector -> different bytes.
        assert transfer != reclaim


# ---------------------------------------------------------------------------
# Type safety at compile time — mirrors upstream byte/pragma rejection tests
# ---------------------------------------------------------------------------

class TestTypeSafety:
    def test_byte_from_out_of_range_int_is_rejected(self):
        # Mirrors compiler_tests.rs :: byte_variable_from_out_of_range_int_literal_is_rejected.
        with pytest.raises(silverscript.SilverScriptError):
            silverscript.compile(
                "contract B() { entrypoint function m() { byte x = 256; require(true); } }"
            )

    def test_byte_addition_is_rejected_with_message(self):
        # Mirrors compiler_tests.rs :: rejects_adding_byte_values — the upstream
        # error message must survive through the binding.
        with pytest.raises(silverscript.SilverScriptError) as exc:
            silverscript.compile(
                "contract B() { entrypoint function m() { byte x = 5; byte y = 7; require(x + y > 0); } }"
            )
        assert "byte values do not support '+'" in str(exc.value)

    def test_incompatible_pragma_is_rejected(self):
        # Mirrors compiler_tests.rs pragma-compatibility tests.
        with pytest.raises(silverscript.SilverScriptError):
            silverscript.compile(
                "pragma silverscript ^99.0.0;\ncontract B() { entrypoint function m() { require(true); } }"
            )


# ---------------------------------------------------------------------------
# Determinism & the recompile assumption — address stability
# ---------------------------------------------------------------------------

class TestDeterminism:
    def test_compile_is_deterministic(self):
        # The covenant flow re-derives a state's P2SH address to find/spend its
        # own UTXO; non-deterministic bytes would orphan funds.
        assert silverscript.compile(GUARD, [100]).script == silverscript.compile(GUARD, [100]).script

    def test_sig_script_is_deterministic(self):
        a = silverscript.compile(GUARD, [100]).build_sig_script("check", [150])
        b = silverscript.compile(GUARD, [100]).build_sig_script("check", [150])
        assert a == b

    def test_recompile_matches_original_script(self):
        # build_sig_script* recompiles from the stored source+ctor args (the
        # native CompiledContract borrows the source and can't be retained).
        # The recompiled locking script must equal the one we first returned.
        contract = silverscript.compile(GUARD, [100])
        again = silverscript.compile(GUARD, [100])
        assert contract.script == again.script


# ---------------------------------------------------------------------------
# Constructor immediates baked into the locking script
# ---------------------------------------------------------------------------

class TestConstructorState:
    def test_constructor_args_change_script(self):
        # Each constructor value is baked into the locking script (and thus the
        # address). Different args MUST produce different scripts.
        assert silverscript.compile(GUARD, [100]).script != silverscript.compile(GUARD, [101]).script

    def test_covenant_constructor_args_change_script(self):
        assert silverscript.compile(COUNTER, [0]).script != silverscript.compile(COUNTER, [5]).script


# ---------------------------------------------------------------------------
# Python -> Value -> Expr argument conversion (the binding's new logic)
# ---------------------------------------------------------------------------

class TestArgConversion:
    def test_tuple_equivalent_to_list(self):
        contract = silverscript.compile(LIST_ARG)
        assert contract.build_sig_script("f", [(1, 2, 3)]) == contract.build_sig_script("f", [[1, 2, 3]])

    def test_bytes_equivalent_to_bytearray(self):
        # Two conversion paths (PyBytes / PyByteArray) must converge.
        from_bytes = silverscript.compile(BYTES4, [b"\x01\x02\x03\x04"]).script
        from_bytearray = silverscript.compile(BYTES4, [bytearray(b"\x01\x02\x03\x04")]).script
        assert from_bytes == from_bytearray

    def test_bool_is_not_silently_coerced_to_int(self):
        # py_to_value checks bool before int by design; a bool where an int is
        # declared must be rejected, not substituted as 0/1.
        with pytest.raises(silverscript.SilverScriptError):
            silverscript.compile(GUARD, [True])

    def test_i64_max_compiles(self):
        assert silverscript.compile(GUARD, [I64_MAX]).contract_name == "Guard"

    def test_i64_min_raises_clean_domain_error(self):
        # i64::MIN is a valid i64 but its magnitude needs 9 bytes in the script
        # number encoding -> a clean SilverScriptError, not a crash.
        with pytest.raises(silverscript.SilverScriptError):
            silverscript.compile(GUARD, [I64_MIN])


# ---------------------------------------------------------------------------
# ABI / selector metadata
# ---------------------------------------------------------------------------

class TestAbi:
    def test_multi_entrypoint_abi(self):
        contract = silverscript.compile(MULTI, [10])
        assert [e.name for e in contract.abi] == ["add", "sub"]
        assert contract.without_selector is False
        for entry in contract.abi:
            assert [(i.name, i.type_name) for i in entry.inputs] == [("amount", "int")]

    def test_byte_array_input_type_name(self):
        contract = silverscript.compile(BYTES4, [b"\x01\x02\x03\x04"])
        assert [(i.name, i.type_name) for e in contract.abi for i in e.inputs] == [("x", "byte[4]")]

    def test_single_entrypoint_without_selector(self):
        assert silverscript.compile(ANNOUNCEMENT).without_selector is True


# ---------------------------------------------------------------------------
# Covenant signature path — exercised by examples/silverscript/counter.py
# ---------------------------------------------------------------------------

class TestCovenantSigScript:
    def test_build_sig_script_for_covenant_decl(self):
        # The exact call the Counter example uses to spend a covenant UTXO,
        # addressed by the friendly entrypoint name ("add"), not the mangled
        # ABI name.
        contract = silverscript.compile(COUNTER, [0])
        assert contract.build_sig_script_for_covenant_decl("add", [5]).hex() == "5500"

    def test_is_leader_flag_is_accepted(self):
        # is_leader is consensus-relevant; both values must build. (For this
        # simple 1:1 transition the bytes happen to coincide.)
        contract = silverscript.compile(COUNTER, [0])
        follower = contract.build_sig_script_for_covenant_decl("add", [5], is_leader=False)
        leader = contract.build_sig_script_for_covenant_decl("add", [5], is_leader=True)
        assert isinstance(follower, bytes)
        assert isinstance(leader, bytes)

    def test_covenant_abi_exposes_both_entrypoints(self):
        contract = silverscript.compile(COUNTER, [0])
        assert contract.without_selector is False
        assert len(contract.abi) == 2

    def test_covenant_decl_rejects_unknown_entrypoint(self):
        contract = silverscript.compile(COUNTER, [0])
        with pytest.raises(silverscript.SilverScriptError):
            contract.build_sig_script_for_covenant_decl("nope", [1])


# ---------------------------------------------------------------------------
# Compile options
# ---------------------------------------------------------------------------

class TestCompileOptions:
    def test_entrypoint_return_requires_flag(self):
        with pytest.raises(silverscript.SilverScriptError):
            silverscript.compile(ENTRYPOINT_RETURN)

    def test_entrypoint_return_compiles_with_flag(self):
        contract = silverscript.compile(ENTRYPOINT_RETURN, allow_entrypoint_return=True)
        assert contract.contract_name == "R"

    def test_record_debug_infos_does_not_change_script(self):
        # Debug info is metadata only; it must never leak into on-chain bytes
        # (that would change the address).
        plain = silverscript.compile(GUARD, [100])
        debug = silverscript.compile(GUARD, [100], record_debug_infos=True)
        assert plain.script == debug.script
        assert plain.build_sig_script("check", [150]) == debug.build_sig_script("check", [150])


# ---------------------------------------------------------------------------
# State layout semantics
# ---------------------------------------------------------------------------

class TestStateLayout:
    def test_plain_contract_has_empty_state(self):
        # state_layout marks the covenant *State* region, not constructor
        # immediates: a plain (non-covenant) contract reports (0, 0) even though
        # its constructor value is embedded elsewhere in the script.
        assert silverscript.compile(GUARD, [100]).state_layout == (0, 0)

    def test_covenant_contract_has_nonempty_state(self):
        contract = silverscript.compile(COUNTER, [0])
        start, length = contract.state_layout
        assert length > 0
        assert 0 <= start
        assert start + length <= len(contract.script)


# ---------------------------------------------------------------------------
# Template hash — canonical length-bound digest of the script's template parts
# ---------------------------------------------------------------------------

class TestTemplateHash:
    def test_is_32_bytes(self):
        contract = silverscript.compile(GUARD, [100])
        assert isinstance(contract.template_hash, bytes)
        assert len(contract.template_hash) == 32

    def test_guard_golden_digest(self):
        # Pinned against silverscript-lang's canonical template_hash() at the
        # pinned rev; a change here means the on-chain templateHash builtin
        # would no longer reproduce commitments made with this compiler.
        assert (
            silverscript.compile(GUARD, [100]).template_hash.hex()
            == "6c1fde9fea16f306d83ade5184f2db81c3f84c7e594fc41c7cc477df06e975a0"
        )

    def test_deterministic_across_recompiles(self):
        a = silverscript.compile(COUNTER, [0]).template_hash
        b = silverscript.compile(COUNTER, [0]).template_hash
        assert a == b

    def test_differs_between_templates(self):
        # Different contracts -> different template parts -> different hashes.
        assert (
            silverscript.compile(GUARD, [100]).template_hash
            != silverscript.compile(COUNTER, [0]).template_hash
        )

    def test_covenant_state_is_excluded_from_template(self):
        # The template covers only the prefix/suffix around the state region,
        # so two instances differing only in initial state share a template.
        assert (
            silverscript.compile(COUNTER, [0]).template_hash
            == silverscript.compile(COUNTER, [5]).template_hash
        )


# ---------------------------------------------------------------------------
# Cross-module bytes boundary — the central design risk
# ---------------------------------------------------------------------------

@pytest.mark.skipif(not _HAVE_CORE_SCRIPT_API, reason="core kaspa script API unavailable")
class TestCrossModule:
    def _address(self, count):
        redeem = silverscript.compile(GUARD, [count]).script
        spk = _kaspa.ScriptBuilder.from_script(
            redeem, covenants_enabled=True
        ).create_pay_to_script_hash_script()
        return _kaspa.address_from_script_public_key(spk, "testnet").to_string()

    def test_compiled_script_wraps_into_p2sh_address(self):
        # silverscript (@v2.0.1) bytes consumed by the core (@78257f2) module:
        # the whole architecture rests on this handoff working.
        assert self._address(100).startswith("kaspatest:")

    def test_p2sh_address_is_deterministic(self):
        assert self._address(100) == self._address(100)

    def test_p2sh_address_differs_by_constructor(self):
        assert self._address(100) != self._address(101)


# ---------------------------------------------------------------------------
# Error surface — must fail predictably, never panic
# ---------------------------------------------------------------------------

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

    def test_wrong_argument_type_raises(self):
        contract = silverscript.compile(GUARD, [100])
        with pytest.raises(silverscript.SilverScriptError):
            contract.build_sig_script("check", ["not an int"])

    @pytest.mark.parametrize("bad", [1.5, None, object()])
    def test_unsupported_arg_type_raises(self, bad):
        with pytest.raises(silverscript.SilverScriptError):
            silverscript.compile(GUARD, [bad])

    def test_args_must_be_list_or_tuple(self):
        contract = silverscript.compile(GUARD, [100])
        with pytest.raises(silverscript.SilverScriptError):
            contract.build_sig_script("check", 150)  # bare int, not [150]

    def test_struct_keys_must_be_strings(self):
        contract = silverscript.compile(GUARD, [100])
        with pytest.raises(silverscript.SilverScriptError):
            contract.build_sig_script("check", [{1: 2}])

    def test_spanned_error_message_includes_byte_offsets(self):
        # map_err appends "(at bytes start..end)" when the compiler error carries
        # a source span — the location signal a contract author needs. (Parse
        # errors instead render their own "--> line:col" pointer; both are
        # descriptive, this exercises the span-bearing branch.)
        with pytest.raises(silverscript.SilverScriptError) as exc:
            silverscript.compile(
                "pragma silverscript ^99.0.0;\n"
                "contract B() { entrypoint function m() { require(true); } }"
            )
        assert "at bytes" in str(exc.value)

    def test_parse_error_message_is_descriptive(self):
        # Malformed syntax still yields a located, human-readable diagnostic.
        with pytest.raises(silverscript.SilverScriptError) as exc:
            silverscript.compile("pragma silverscript ^0.1.0;\ncontract Broken( {")
        assert "-->" in str(exc.value)


# ---------------------------------------------------------------------------
# Exception & object semantics
# ---------------------------------------------------------------------------

class TestObjectSemantics:
    def test_error_is_exception_subclass(self):
        assert issubclass(silverscript.SilverScriptError, Exception)

    def test_error_message_round_trips(self):
        assert str(silverscript.SilverScriptError("boom")) == "boom"

    def test_compiled_contract_is_frozen(self):
        contract = silverscript.compile(GUARD, [100])
        with pytest.raises(AttributeError):
            contract.contract_name = "mutated"

    def test_reprs(self):
        contract = silverscript.compile(GUARD, [100])
        assert repr(contract) == 'CompiledContract(name="Guard", script=7 bytes, entrypoints=1)'
        assert repr(contract.abi[0]) == 'FunctionAbiEntry(name="check", inputs=1 input(s))'
        assert repr(contract.abi[0].inputs[0]) == 'FunctionInputAbi(name="amount", type_name="int")'


# ---------------------------------------------------------------------------
# Robustness — no input may crash the interpreter
# ---------------------------------------------------------------------------

class TestRobustness:
    def test_moderately_nested_arg_raises_cleanly(self):
        # The safe regime: a too-deep-for-the-type arg is a clean error, not a
        # crash. (1000 levels is below the native-stack cliff.)
        contract = silverscript.compile(LIST_ARG)
        deep = []
        for _ in range(1000):
            deep = [deep]
        with pytest.raises(silverscript.SilverScriptError):
            contract.build_sig_script("f", [deep])

    def test_oversized_int_is_catchable_and_does_not_crash(self):
        # Out-of-i64 ints must raise a *catchable* exception (no abort). The
        # exact type is pinned more strictly by the xfail test below.
        with pytest.raises(Exception):
            silverscript.compile(GUARD, [2**63])

    def test_oversized_int_should_raise_silverscript_error(self):
        # `py_to_value` maps pyo3's OverflowError onto the domain error, so
        # callers' `except SilverScriptError` catches an out-of-i64 arg.
        with pytest.raises(silverscript.SilverScriptError):
            silverscript.compile(GUARD, [2**63])

    def test_deeply_nested_arg_does_not_crash_process(self):
        snippet = textwrap.dedent(
            """
            import kaspa.experimental.silverscript as ss
            SRC = ("pragma silverscript ^0.1.0;\\n"
                   "contract D() { entrypoint function f(int[] xs) { require(true); } }")
            n = []
            for _ in range(20000):
                n = [n]
            try:
                ss.compile(SRC).build_sig_script("f", [n])
            except Exception:
                pass
            """
        )
        proc = _run_in_subprocess(snippet)
        # A native stack overflow shows up as a negative return code (SIGSEGV).
        assert proc.returncode == 0, f"process died with return code {proc.returncode}"
