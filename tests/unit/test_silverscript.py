"""
Unit tests for the kaspa.experimental.silverscript module.
"""

import json
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
    entry check(int amount) {
        require(amount > threshold);
    }
}
"""

ANNOUNCEMENT = """
pragma silverscript ^0.1.0;
contract Announcement() {
    entry announce() {
        require(tx.outputs[0].value == 0);
    }
}
"""

# Two entrypoints -> two distinct dispatch tags.
MULTI = """
pragma silverscript ^0.1.0;
contract Multi(int base) {
    entry add(int amount) { require(amount > base); }
    entry sub(int amount) { require(amount < base); }
}
"""

BYTES4 = """
pragma silverscript ^0.1.0;
contract H(byte[4] tag) {
    entry go(byte[4] x) { require(x == tag); }
}
"""

BLOB = """
pragma silverscript ^0.1.0;
contract Blob(byte[] tag) {
    entry go(byte[] data) { require(data == tag); }
}
"""

# A `byte` scalar in every position it can appear: constructor param,
# entrypoint param, and a struct field.
BYTE_BOX = """
pragma silverscript ^0.1.0;
contract ByteBox(byte tag) {
    entry f(byte b) { require(b == tag); }
}
"""

TAGGED = """
pragma silverscript ^0.1.0;
contract Tagged(Tag init) {
    struct Tag { byte marker; int n; }
    entry f(Tag t) { require(t.marker == init.marker); require(t.n > init.n); }
}
"""

LIST_ARG = """
pragma silverscript ^0.1.0;
contract L() {
    entry f(int[] xs) { require(true); }
}
"""

ENTRYPOINT_RETURN = """
pragma silverscript ^0.1.0;
contract R() {
    entry f() : (int) { return(1); }
}
"""

# Mirror of silverscript-lang tutorial_rust_examples_tests.rs ::
# tutorial_rust_build_sigscript_multiple_entrypoints_example — a realistic
# pubkey/sig multi-entrypoint contract.
TRANSFER_WITH_TIMEOUT = """
pragma silverscript ^0.1.0;
contract TransferWithTimeout(pubkey sender, pubkey recipient, temporal timeout) {
    entry transfer(sig recipientSig) {
        require(checkSig(recipientSig, recipient));
    }
    entry reclaim(sig senderSig) {
        require(checkSig(senderSig, sender));
        require(tx.time >= timeout);
    }
}
"""

# Mirror of silverscript-lang compiler_tests.rs :: build_sig_script_builds_expected_script
# (a multi-argument entrypoint: byte[4] then int). Note: no pragma, like upstream.
BOUNDED_BYTES = """
contract BoundedBytes() {
    entry spend(byte[4] b, int i) { require(b == i as byte[4]); }
}
"""

# Entrypoints declared out of alphabetical order, so source order and the
# artifact's BTreeMap order are visibly different.
ORDER = """
contract Order() {
    entry zebra(int a) { require(a > 0); }
    entry alpha(int a) { require(a > 0); }
    entry middle(int a) { require(a > 0); }
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
        return(State { count: prev_state.count + amount });
    }
    #[covenant(binding = auth, from = 1, to = 1, mode = transition)]
    function subtract(State prev_state, int amount) : (State) {
        return(State { count: prev_state.count - amount });
    }
}
"""

# A cov-bound covenant, unlike COUNTER's auth binding: it compiles to a leader
# entrypoint *and* a delegate, so the follower path resolves to __delegate
# rather than to the named declaration.
COV_PAIR = """
pragma silverscript ^0.1.0;
contract Pair(int init_value) {
    int value = init_value;
    #[covenant(binding = cov, from = 2, to = 2, mode = transition)]
    function carry_forward(State[] prev_states) : (State[]) {
        return(prev_states);
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
        assert isinstance(contract.bytecode, bytes)
        assert len(contract.bytecode) > 0

    def test_state_span_is_pair(self):
        contract = silverscript.compile(GUARD, [100])
        assert isinstance(contract.state_span, tuple)
        assert len(contract.state_span) == 2

    def test_abi(self):
        contract = silverscript.compile(GUARD, [100])
        assert len(contract.abi) == 1
        entry = contract.abi[0]
        assert entry.name == "check"
        assert [(i.name, i.type_name) for i in entry.params] == [("amount", "int")]

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
        assert silverscript.compile(GUARD, [100]).bytecode.hex() == "7604b08239998763757682599f6975760164a0697551676a68"

    def test_bytes_contract_locking_script(self):
        contract = silverscript.compile(BYTES4, [b"\x01\x02\x03\x04"])
        assert contract.bytecode.hex() == "760405d685098763757682549d7576040102030487697551676a68"


# ---------------------------------------------------------------------------
# Golden unlocking (signature) scripts — the bytes that spend a UTXO
# ---------------------------------------------------------------------------

class TestGoldenSigScript:
    def test_int_arg_encoding(self):
        contract = silverscript.compile(GUARD, [100])
        # 150 pushed as a minimally-encoded script number, then the entry's
        # 4-byte dispatch tag.
        assert contract.build_sig_script("check", [150]).hex() == "02960004b0823999"

    def test_zero_arg_encoding(self):
        contract = silverscript.compile(GUARD, [100])
        assert contract.build_sig_script("check", [0]).hex() == "0004b0823999"

    def test_negative_int_encoding(self):
        contract = silverscript.compile(GUARD, [100])
        assert contract.build_sig_script("check", [-5]).hex() == "018504b0823999"

    def test_bytes_arg_encoding(self):
        contract = silverscript.compile(BYTES4, [b"\x01\x02\x03\x04"])
        assert contract.build_sig_script("go", [b"\xaa\xbb\xcc\xdd"]).hex() == "04aabbccdd0405d68509"

    def test_list_arg_encoding(self):
        contract = silverscript.compile(LIST_ARG)
        assert (
            contract.build_sig_script("f", [[1, 2, 3]]).hex()
            == "1801000000000000000200000000000000030000000000000004ec1c6966"
        )

    def test_empty_list_arg_encoding(self):
        contract = silverscript.compile(LIST_ARG)
        assert contract.build_sig_script("f", [[]]).hex() == "0004ec1c6966"

    def test_single_entrypoint_emits_only_the_dispatch_tag(self):
        # Every entry carries a 4-byte dispatch tag, including a lone one that
        # takes no arguments, so this is a 5-byte push rather than b"".
        contract = silverscript.compile(ANNOUNCEMENT)
        sig = contract.build_sig_script("announce")
        assert sig.hex() == "04c7de89fa"

    def test_multi_entrypoint_appends_dispatch_tag(self):
        contract = silverscript.compile(MULTI, [10])
        # Each entrypoint encodes its arg plus its own distinct dispatch tag.
        assert contract.build_sig_script("add", [20]).hex() == "011404dc78a211"
        assert contract.build_sig_script("sub", [5]).hex() == "55048fe86423"


# ---------------------------------------------------------------------------
# Parity with silverscript-lang's own compiler tests (pinned rev 3ed9733)
# ---------------------------------------------------------------------------

class TestUpstreamParity:
    def test_multi_arg_sig_script_matches_upstream_vector(self):
        # Mirrors compiler_tests.rs :: build_sig_script_builds_expected_script.
        # Upstream builds: push byte[4] {01,02,03,04}, then i64(7), then the
        # entry's 4-byte dispatch tag. We assert the exact resulting bytes.
        contract = silverscript.compile(BOUNDED_BYTES)
        assert contract.build_sig_script("spend", [b"\x01\x02\x03\x04", 7]).hex() == "0401020304570433cd8f70"

    def test_transfer_with_timeout_multi_entrypoint(self):
        # Mirrors tutorial_rust_build_sigscript_multiple_entrypoints_example.
        sender = bytes([3]) * 32
        recipient = bytes([4]) * 32
        timeout = 1_640_000_000_000
        contract = silverscript.compile(TRANSFER_WITH_TIMEOUT, [sender, recipient, timeout])
        # Keyed by name rather than flattened across the ABI: this asserts which
        # parameter belongs to which entrypoint, which a flat list never did.
        assert [(i.name, i.type_name) for i in contract.entry("transfer").params] == [
            ("recipientSig", "sig")
        ]
        assert [(i.name, i.type_name) for i in contract.entry("reclaim").params] == [
            ("senderSig", "sig")
        ]

        sig = bytes([5]) * 65
        transfer = contract.build_sig_script("transfer", [sig])
        reclaim = contract.build_sig_script("reclaim", [sig])
        # 0x41 = 65-byte data push, followed by the signature bytes verbatim.
        assert transfer[0] == 0x41
        assert transfer[1:66] == sig
        # Same arg, different entrypoint -> different dispatch tag -> different bytes.
        assert transfer != reclaim


# ---------------------------------------------------------------------------
# Type safety at compile time — mirrors upstream byte/pragma rejection tests
# ---------------------------------------------------------------------------

class TestTypeSafety:
    def test_byte_from_out_of_range_int_is_rejected(self):
        # Mirrors compiler_tests.rs :: byte_variable_from_out_of_range_int_literal_is_rejected.
        with pytest.raises(silverscript.SilverScriptError):
            silverscript.compile(
                "contract B() { entry m() { byte x = 256; require(true); } }"
            )

    def test_byte_addition_is_rejected_with_message(self):
        # Mirrors compiler_tests.rs :: rejects_adding_byte_values — the upstream
        # error message must survive through the binding.
        with pytest.raises(silverscript.SilverScriptError) as exc:
            silverscript.compile(
                "contract B() { entry m() { byte x = 5; byte y = 7; require(x + y > 0); } }"
            )
        assert "arithmetic requires matching int or temporal operands" in str(exc.value)

    def test_incompatible_pragma_is_rejected(self):
        # Mirrors compiler_tests.rs pragma-compatibility tests.
        with pytest.raises(silverscript.SilverScriptError):
            silverscript.compile(
                "pragma silverscript ^99.0.0;\ncontract B() { entry m() { require(true); } }"
            )


# ---------------------------------------------------------------------------
# Determinism & the recompile assumption — address stability
# ---------------------------------------------------------------------------

class TestDeterminism:
    def test_compile_is_deterministic(self):
        # The covenant flow re-derives a state's P2SH address to find/spend its
        # own UTXO; non-deterministic bytes would orphan funds.
        assert silverscript.compile(GUARD, [100]).bytecode == silverscript.compile(GUARD, [100]).bytecode

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
        assert contract.bytecode == again.bytecode


# ---------------------------------------------------------------------------
# Constructor immediates baked into the locking script
# ---------------------------------------------------------------------------

class TestConstructorState:
    def test_constructor_args_change_script(self):
        # Each constructor value is baked into the locking script (and thus the
        # address). Different args MUST produce different scripts.
        assert silverscript.compile(GUARD, [100]).bytecode != silverscript.compile(GUARD, [101]).bytecode

    def test_covenant_constructor_args_change_script(self):
        assert silverscript.compile(COUNTER, [0]).bytecode != silverscript.compile(COUNTER, [5]).bytecode


# ---------------------------------------------------------------------------
# Python -> Value -> Expr argument conversion (the binding's new logic)
# ---------------------------------------------------------------------------

class TestArgConversion:
    def test_tuple_equivalent_to_list(self):
        contract = silverscript.compile(LIST_ARG)
        assert contract.build_sig_script("f", [(1, 2, 3)]) == contract.build_sig_script("f", [[1, 2, 3]])

    def test_bytes_equivalent_to_bytearray(self):
        # Two conversion paths (PyBytes / PyByteArray) must converge.
        from_bytes = silverscript.compile(BYTES4, [b"\x01\x02\x03\x04"]).bytecode
        from_bytearray = silverscript.compile(BYTES4, [bytearray(b"\x01\x02\x03\x04")]).bytecode
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
# The `byte` type — narrowing is directed by the declared type, not the value
# ---------------------------------------------------------------------------

class TestByteArguments:
    def test_abi_type_name(self):
        contract = silverscript.compile(BYTE_BOX, [1])
        assert [(i.name, i.type_name) for i in contract.abi[0].params] == [("b", "byte")]

    def test_arg_from_int(self):
        # One-byte data push, canonically encoded (OP_2), then the dispatch tag.
        contract = silverscript.compile(BYTE_BOX, [1])
        assert contract.build_sig_script("f", [2]).hex() == "52044358458f"

    def test_arg_zero_pushes_one_zero_byte(self):
        # Not OP_0: that pushes an empty item, not a one-byte one.
        contract = silverscript.compile(BYTE_BOX, [1])
        assert contract.build_sig_script("f", [0]).hex() == "0100044358458f"

    def test_arg_max(self):
        contract = silverscript.compile(BYTE_BOX, [1])
        assert contract.build_sig_script("f", [255]).hex() == "01ff044358458f"

    def test_single_byte_bytes_equivalent_to_int(self):
        contract = silverscript.compile(BYTE_BOX, [1])
        assert contract.build_sig_script("f", [b"\x07"]) == contract.build_sig_script("f", [7])

    @pytest.mark.parametrize("value", [256, -1, I64_MAX])
    def test_arg_out_of_range_raises(self, value):
        contract = silverscript.compile(BYTE_BOX, [1])
        with pytest.raises(silverscript.SilverScriptError):
            contract.build_sig_script("f", [value])

    def test_arg_rejects_multi_byte_bytes(self):
        contract = silverscript.compile(BYTE_BOX, [1])
        with pytest.raises(silverscript.SilverScriptError):
            contract.build_sig_script("f", [b"\x01\x02"])

    def test_arg_rejects_bool(self):
        # bool stays distinct from int here too — True is not 1.
        contract = silverscript.compile(BYTE_BOX, [1])
        with pytest.raises(silverscript.SilverScriptError):
            contract.build_sig_script("f", [True])

    def test_constructor_arg(self):
        assert silverscript.compile(BYTE_BOX, [1]).bytecode.hex() == (
            "76044358458f8763757682519d7576010187697551676a68"
        )

    def test_constructor_arg_single_byte_bytes_equivalent_to_int(self):
        assert silverscript.compile(BYTE_BOX, [1]).bytecode == silverscript.compile(BYTE_BOX, [b"\x01"]).bytecode

    def test_constructor_arg_is_baked_into_the_script(self):
        assert silverscript.compile(BYTE_BOX, [1]).bytecode != silverscript.compile(BYTE_BOX, [2]).bytecode

    def test_constructor_arg_out_of_range_raises(self):
        with pytest.raises(silverscript.SilverScriptError):
            silverscript.compile(BYTE_BOX, [256])

    def test_struct_field(self):
        contract = silverscript.compile(TAGGED, [{"marker": 1, "n": 5}])
        assert contract.build_sig_script("f", [{"marker": 1, "n": 6}]).hex() == "5156043e5e9af9"

    def test_struct_field_out_of_range_raises(self):
        contract = silverscript.compile(TAGGED, [{"marker": 1, "n": 5}])
        with pytest.raises(silverscript.SilverScriptError):
            contract.build_sig_script("f", [{"marker": 300, "n": 6}])

    def test_struct_constructor_field_out_of_range_raises(self):
        with pytest.raises(silverscript.SilverScriptError):
            silverscript.compile(TAGGED, [{"marker": 300, "n": 5}])

    def test_int_arg_is_not_narrowed_to_byte(self):
        # `check` declares `int`, so a byte-sized value is still encoded as a
        # script number: 0 as OP_0 and 255 as two bytes, not 0x00 / 0xff.
        contract = silverscript.compile(GUARD, [100])
        assert contract.build_sig_script("check", [0]).hex() == "0004b0823999"
        assert contract.build_sig_script("check", [255]).hex() == "02ff0004b0823999"

    def test_int_constructor_arg_is_not_narrowed_to_byte(self):
        assert silverscript.compile(GUARD, [255]).bytecode.hex() == (
            "7604b08239998763757682599f69757602ff00a0697551676a68"
        )


# ---------------------------------------------------------------------------
# ABI / dispatch-tag metadata
# ---------------------------------------------------------------------------

class TestAbi:
    def test_multi_entrypoint_abi(self):
        contract = silverscript.compile(MULTI, [10])
        assert [e.name for e in contract.abi] == ["add", "sub"]
        for entry in contract.abi:
            assert [(i.name, i.type_name) for i in entry.params] == [("amount", "int")]

    def test_abi_is_alphabetical(self):
        # MULTI can't show this — "add" precedes "sub" in both source and
        # alphabetical order. ORDER declares zebra, alpha, middle, so it can.
        assert [e.name for e in silverscript.compile(ORDER).abi] == ["alpha", "middle", "zebra"]

    def test_entry_looks_up_by_name(self):
        contract = silverscript.compile(MULTI, [10])
        entry = contract.entry("sub")
        assert entry.name == "sub"
        assert [(i.name, i.type_name) for i in entry.params] == [("amount", "int")]
        assert entry.dispatch_tag == next(e.dispatch_tag for e in contract.abi if e.name == "sub")

    def test_entry_unknown_name_raises(self):
        contract = silverscript.compile(GUARD, [100])
        with pytest.raises(silverscript.SilverScriptError) as exc:
            contract.entry("does_not_exist")
        assert str(exc.value) == "unknown entry `Guard::does_not_exist`"

    def test_entry_reaches_mangled_covenant_names(self):
        # Covenant entries are compiler-generated; entry() addresses them by
        # the name the ABI actually carries.
        contract = silverscript.compile(COUNTER, [0])
        name = "__covenant_entrypoint_auth_add"
        assert contract.entry(name).name == name

    def test_byte_array_input_type_name(self):
        contract = silverscript.compile(BYTES4, [b"\x01\x02\x03\x04"])
        assert [(i.name, i.type_name) for e in contract.abi for i in e.params] == [("x", "byte[4]")]

    def test_dynamic_byte_array_input_type_name(self):
        # `byte[]` is the SilverScript spelling; `bytes` is not a type here.
        contract = silverscript.compile(BLOB, [b"\xaa\xbb"])
        assert [(i.name, i.type_name) for e in contract.abi for i in e.params] == [("data", "byte[]")]

    def test_without_selector_property_is_gone(self):
        # SilverScript 1.0 gives every entry an unconditional dispatch tag, so
        # the "single entrypoint has no selector" case it described no longer
        # exists and the property was removed.
        assert not hasattr(silverscript.compile(ANNOUNCEMENT), "without_selector")

    def test_script_property_is_gone(self):
        # Renamed to `bytecode`, matching SilverScript 1.0.
        assert not hasattr(silverscript.compile(ANNOUNCEMENT), "script")

    def test_dispatch_tag_is_four_bytes(self):
        entry = silverscript.compile(GUARD, [100]).abi[0]
        assert isinstance(entry.dispatch_tag, bytes)
        assert len(entry.dispatch_tag) == 4
        assert entry.dispatch_tag.hex() == "b0823999"

    def test_dispatch_tag_terminates_the_sig_script(self):
        # Every signature script ends with the entry's tag as its final data
        # push, which is how the script dispatches to the right entrypoint.
        contract = silverscript.compile(MULTI, [10])
        for entry in contract.abi:
            sig = contract.build_sig_script(entry.name, [7])
            assert sig.endswith(b"\x04" + entry.dispatch_tag)

    def test_dispatch_tag_is_content_addressed(self):
        # The tag is blake3("name(type,type)")[:4], so it depends only on the
        # entry's name and parameter types — never on constructor arguments.
        a = silverscript.compile(GUARD, [100])
        b = silverscript.compile(GUARD, [101])
        assert a.bytecode != b.bytecode
        assert a.abi[0].dispatch_tag == b.abi[0].dispatch_tag

    def test_dispatch_tags_are_distinct_per_entry(self):
        contract = silverscript.compile(MULTI, [10])
        tags = {e.name: e.dispatch_tag for e in contract.abi}
        assert len(set(tags.values())) == len(tags)


# ---------------------------------------------------------------------------
# The portable ABI artifact — the same JSON upstream `silverc` emits
# ---------------------------------------------------------------------------

class TestPortableArtifact:
    def test_artifact_json_parses(self):
        artifact = json.loads(silverscript.compile(GUARD, [100]).artifact_json())
        assert artifact["schema_version"] == 1
        assert sorted(artifact) == [
            "compiler_version",
            "contracts",
            "schema_version",
            "structs",
        ]
        assert list(artifact["contracts"]) == ["Guard"]

    def test_bytecode_round_trips(self):
        # `bytecode` is a JSON array of ints, not hex — bytes() recovers it.
        contract = silverscript.compile(GUARD, [100])
        compiled = json.loads(contract.artifact_json())["contracts"]["Guard"]["compiled"]
        assert bytes(compiled["bytecode"]) == contract.bytecode

    def test_template_hash_round_trips(self):
        contract = silverscript.compile(GUARD, [100])
        compiled = json.loads(contract.artifact_json())["contracts"]["Guard"]["compiled"]
        assert bytes(compiled["template_hash"]) == contract.template_hash

    def test_state_span_matches_the_artifact_json(self):
        # The attribute and the JSON key carry the same two numbers.
        contract = silverscript.compile(COUNTER, [0])
        span = json.loads(contract.artifact_json())["contracts"]["Counter"]["compiled"][
            "state_span"
        ]
        assert (span["offset"], span["len"]) == contract.state_span
        assert span["len"] > 0

    def test_stateless_contract_has_empty_state_span(self):
        contract = silverscript.compile(GUARD, [100])
        span = json.loads(contract.artifact_json())["contracts"]["Guard"]["compiled"][
            "state_span"
        ]
        assert span == {"offset": 0, "len": 0}

    def test_dispatch_tag_is_hex_not_bytes(self):
        # The one field that is hex where the byte fields are int arrays.
        contract = silverscript.compile(GUARD, [100])
        entry = json.loads(contract.artifact_json())["contracts"]["Guard"]["entries"][
            "check"
        ]
        assert entry["dispatch_tag"] == contract.abi[0].dispatch_tag.hex()
        assert entry["dispatch_tag"] == "b0823999"

    def test_params_use_the_portable_kind_dialect(self):
        # {"kind": ...}, not the type-directed dialect the debugger accepts.
        contract = silverscript.compile(GUARD, [100])
        entry = json.loads(contract.artifact_json())["contracts"]["Guard"]["entries"][
            "check"
        ]
        assert entry["params"] == [{"name": "amount", "type": {"kind": "int"}}]

    def test_source_path_is_synthesized_from_contract_name(self):
        # There is no source file here; upstream synthesizes the same path, so
        # artifacts stay comparable across compilers.
        artifact = json.loads(silverscript.compile(GUARD, [100]).artifact_json())
        assert artifact["contracts"]["Guard"]["source_path"] == "sil/Guard.sil"

    def test_artifact_reflects_constructor_args(self):
        # Proves this is the contract's own artifact and not a static template.
        a = silverscript.compile(GUARD, [100]).artifact_json()
        b = silverscript.compile(GUARD, [101]).artifact_json()
        assert a != b

    def test_artifact_json_is_deterministic(self):
        source = silverscript.compile(GUARD, [100]).artifact_json()
        assert source == silverscript.compile(GUARD, [100]).artifact_json()

    def test_byte_arrays_wrap_at_64_values_per_line(self):
        # Upstream's formatter wraps byte arrays at exactly 64 values per line
        # (silverscript-abi/src/json.rs). This is emitted by Rust rather than
        # rebuilt with json.dumps precisely so the output stays byte-identical
        # to silverc's; this test fails loudly if anyone reimplements it.
        contract = silverscript.compile(COUNTER, [0])  # 196 bytes: 64+64+64+4
        lines = contract.artifact_json().splitlines()
        start = next(i for i, line in enumerate(lines) if '"bytecode"' in line)
        end = next(i for i, line in enumerate(lines[start:], start) if line.rstrip().endswith("],"))
        counts = [
            len([v for v in line.split(",") if v.strip()])
            for line in lines[start + 1 : end]
        ]
        assert counts[:-1] == [64] * (len(counts) - 1)
        assert sum(counts) == len(contract.bytecode)


# ---------------------------------------------------------------------------
# Loading an artifact back — spend with no source and no compiler
# ---------------------------------------------------------------------------

class TestContractArtifact:
    def test_loaded_artifact_builds_identical_sig_scripts(self):
        # The capability the whole class exists for: compile in CI, ship the
        # JSON, build the same spending bytes at runtime from the artifact
        # alone.
        contract = silverscript.compile(GUARD, [100])
        artifact = silverscript.load_artifact(contract.artifact_json())
        assert artifact.build_sig_script("check", [150]) == contract.build_sig_script(
            "check", [150]
        )

    def test_to_json_round_trips_losslessly(self):
        # load -> serialize returns the identical string, so an artifact can be
        # passed through this module without drifting from silverc's output.
        contract = silverscript.compile(GUARD, [100])
        artifact = silverscript.load_artifact(contract.artifact_json())
        assert artifact.to_json() == contract.artifact_json()

    def test_to_json_canonicalizes_rather_than_echoing_the_input(self):
        # The round trip above holds for the canonical form; it is not a
        # promise to hand back whatever JSON was loaded. Equivalent input in a
        # different shape comes back canonicalized, which is the more useful
        # guarantee anyway: two artifacts agree iff their `to_json` agree.
        canonical = silverscript.compile(GUARD, [100]).artifact_json()
        parsed = json.loads(canonical)

        compact = json.dumps(parsed, separators=(",", ":"))
        assert silverscript.load_artifact(compact).to_json() == canonical

        reordered = json.dumps({k: parsed[k] for k in reversed(list(parsed))}, indent=2)
        assert silverscript.load_artifact(reordered).to_json() == canonical

        # Unknown top-level fields are dropped (serde default, matching
        # upstream), not carried through.
        extra = json.dumps({**parsed, "__unknown__": {"x": 1}})
        assert silverscript.load_artifact(extra).to_json() == canonical

    def test_golden_sig_script_survives_the_artifact_route(self):
        # Pinned against TestGoldenSigScript: the route must not change bytes
        # that land on-chain.
        artifact = silverscript.load_artifact(silverscript.compile(GUARD, [100]).artifact_json())
        assert artifact.build_sig_script("check", [150]).hex() == "02960004b0823999"

    def test_properties_match_the_compiled_contract(self):
        contract = silverscript.compile(COUNTER, [7])
        artifact = silverscript.load_artifact(contract.artifact_json())
        assert artifact.contract_name == contract.contract_name
        assert artifact.compiler_version == contract.compiler_version
        assert artifact.bytecode == contract.bytecode
        assert artifact.template_hash == contract.template_hash
        # One name, one meaning, both routes.
        assert artifact.state_span == contract.state_span
        assert artifact.schema_version == 1

    def test_abi_order_agrees_across_routes(self):
        # One ordering rule for the module: alphabetical, whichever route you
        # took. ORDER declares zebra, alpha, middle — so source order and
        # alphabetical genuinely differ, and this would catch a regression to
        # source-ordering on either side.
        contract = silverscript.compile(ORDER)
        artifact = silverscript.load_artifact(contract.artifact_json())
        assert [e.name for e in contract.abi] == ["alpha", "middle", "zebra"]
        assert [e.name for e in artifact.abi] == [e.name for e in contract.abi]

    def test_entry_lookup_agrees_across_routes(self):
        contract = silverscript.compile(ORDER)
        artifact = silverscript.load_artifact(contract.artifact_json())
        for name in ("zebra", "alpha", "middle"):
            assert contract.entry(name).name == artifact.entry(name).name == name
            assert contract.entry(name).dispatch_tag == artifact.entry(name).dispatch_tag

    def test_entry_unknown_name_matches_build_sig_script_error(self):
        # One typo, one message — whichever method the caller reached for.
        artifact = silverscript.load_artifact(silverscript.compile(GUARD, [100]).artifact_json())
        with pytest.raises(silverscript.SilverScriptError) as lookup:
            artifact.entry("nope")
        with pytest.raises(silverscript.SilverScriptError) as build:
            artifact.build_sig_script("nope", [1])
        assert str(lookup.value) == str(build.value) == "unknown entry `Guard::nope`"

    def test_abi_entries_carry_the_same_dispatch_tags(self):
        contract = silverscript.compile(ORDER)
        artifact = silverscript.load_artifact(contract.artifact_json())
        by_name = {e.name: e.dispatch_tag for e in artifact.abi}
        assert by_name == {e.name: e.dispatch_tag for e in contract.abi}
        assert all(len(tag) == 4 for tag in by_name.values())

    def test_covenant_sig_script_round_trips(self):
        contract = silverscript.compile(COUNTER, [0])
        artifact = silverscript.load_artifact(contract.artifact_json())
        assert artifact.build_sig_script_for_covenant_decl(
            "add", [5]
        ) == contract.build_sig_script_for_covenant_decl("add", [5])

    def test_cov_bound_decl_rejects_unknown_entrypoint_on_both_paths(self):
        # The covenant_decl_entry guard is shared with CompiledContract rather
        # than reimplemented, so the follower path can't silently encode a
        # delegate call for a name the contract never declared.
        artifact = silverscript.load_artifact(silverscript.compile(COV_PAIR, [0]).artifact_json())
        for is_leader in (False, True):
            assert artifact.build_sig_script_for_covenant_decl(
                "carry_forward", is_leader=is_leader
            )
            with pytest.raises(silverscript.SilverScriptError) as exc:
                artifact.build_sig_script_for_covenant_decl("bogus", is_leader=is_leader)
            assert "unknown entry" in str(exc.value)

    def test_byte_narrowing_needs_only_the_artifact(self):
        # `byte` args are narrowed against the declared type, which the codec
        # requires — this proves that narrowing reads the artifact, not the
        # source, so it survives the round trip.
        contract = silverscript.compile(BYTE_BOX, [1])
        artifact = silverscript.load_artifact(contract.artifact_json())
        assert artifact.build_sig_script("f", [2]) == contract.build_sig_script("f", [2])
        assert artifact.build_sig_script("f", [2]).hex() == "52044358458f"
        with pytest.raises(silverscript.SilverScriptError) as exc:
            artifact.build_sig_script("f", [256])
        assert "0..=255" in str(exc.value)

    def test_unknown_entrypoint_raises(self):
        artifact = silverscript.load_artifact(silverscript.compile(GUARD, [100]).artifact_json())
        with pytest.raises(silverscript.SilverScriptError):
            artifact.build_sig_script("does_not_exist", [1])

    def test_check_consistency_passes_on_a_fresh_artifact(self):
        artifact = silverscript.load_artifact(silverscript.compile(COUNTER, [3]).artifact_json())
        assert artifact.check_consistency() is None

    def test_check_consistency_rejects_a_tampered_template_hash(self):
        # The reason the method exists: an artifact arriving from elsewhere may
        # not describe the bytecode it ships with.
        doc = json.loads(silverscript.compile(GUARD, [100]).artifact_json())
        doc["contracts"]["Guard"]["compiled"]["template_hash"][0] ^= 0xFF
        artifact = silverscript.load_artifact(json.dumps(doc))
        with pytest.raises(silverscript.SilverScriptError) as exc:
            artifact.check_consistency()
        assert "template hash mismatch" in str(exc.value)

    @pytest.mark.parametrize("bad", ["", "{not json", '{"schema_version": 1}'])
    def test_malformed_json_raises_silverscript_error(self, bad):
        # Not a json.JSONDecodeError: callers catch one exception type for
        # everything this module can refuse.
        with pytest.raises(silverscript.SilverScriptError):
            silverscript.load_artifact(bad)

    def test_unsupported_schema_version_raises(self):
        doc = json.loads(silverscript.compile(GUARD, [100]).artifact_json())
        doc["schema_version"] = 2
        with pytest.raises(silverscript.SilverScriptError) as exc:
            silverscript.load_artifact(json.dumps(doc))
        assert "schema version" in str(exc.value)

    def test_unknown_contract_name_raises_and_names_what_is_there(self):
        with pytest.raises(silverscript.SilverScriptError) as exc:
            silverscript.load_artifact(
                silverscript.compile(GUARD, [100]).artifact_json(), "Nope"
            )
        assert "Guard" in str(exc.value)

    def test_empty_artifact_raises(self):
        doc = json.loads(silverscript.compile(GUARD, [100]).artifact_json())
        doc["contracts"] = {}
        with pytest.raises(silverscript.SilverScriptError):
            silverscript.load_artifact(json.dumps(doc))

    def _two_contract_artifact(self):
        # compile() always emits exactly one contract, so a multi-contract
        # artifact has to be assembled by hand.
        doc = json.loads(silverscript.compile(GUARD, [100]).artifact_json())
        doc["contracts"].update(
            json.loads(silverscript.compile(ORDER).artifact_json())["contracts"]
        )
        return json.dumps(doc)

    def test_multi_contract_artifact_requires_a_name(self):
        with pytest.raises(silverscript.SilverScriptError) as exc:
            silverscript.load_artifact(self._two_contract_artifact())
        assert "contract_name" in str(exc.value)

    def test_multi_contract_artifact_selects_by_name(self):
        doc = self._two_contract_artifact()
        assert silverscript.load_artifact(doc, "Guard").contract_name == "Guard"
        assert silverscript.load_artifact(doc, "Order").contract_name == "Order"

    def test_multi_contract_selection_scopes_the_abi(self):
        # Selecting a contract must not leak the other contract's entries.
        artifact = silverscript.load_artifact(self._two_contract_artifact(), "Guard")
        assert [e.name for e in artifact.abi] == ["check"]

    def test_artifact_is_frozen(self):
        artifact = silverscript.load_artifact(silverscript.compile(GUARD, [100]).artifact_json())
        with pytest.raises(AttributeError):
            artifact.contract_name = "mutated"

    def test_repr(self):
        artifact = silverscript.load_artifact(silverscript.compile(GUARD, [100]).artifact_json())
        assert repr(artifact) == 'ContractArtifact(name="Guard", bytecode=25 bytes, entries=1)'


# ---------------------------------------------------------------------------
# Covenant signature path — exercised by examples/silverscript/counter.py
# ---------------------------------------------------------------------------

class TestCovenantSigScript:
    def test_build_sig_script_for_covenant_decl(self):
        # The exact call the Counter example uses to spend a covenant UTXO,
        # addressed by the friendly entrypoint name ("add"), not the mangled
        # ABI name.
        contract = silverscript.compile(COUNTER, [0])
        assert contract.build_sig_script_for_covenant_decl("add", [5]).hex() == "5504d06ddd4e"

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
        assert len(contract.abi) == 2

    def test_covenant_decl_rejects_unknown_entrypoint(self):
        contract = silverscript.compile(COUNTER, [0])
        with pytest.raises(silverscript.SilverScriptError):
            contract.build_sig_script_for_covenant_decl("nope", [1])

    def test_cov_bound_decl_rejects_unknown_entrypoint_on_both_paths(self):
        # The follower path resolves straight to __delegate without consulting
        # the declaration name, so a typo used to build a well-formed script for
        # the wrong call. COUNTER can't catch that: being auth-bound, it always
        # takes the leader path.
        contract = silverscript.compile(COV_PAIR, [0])
        for is_leader in (False, True):
            assert contract.build_sig_script_for_covenant_decl("carry_forward", is_leader=is_leader)
            with pytest.raises(silverscript.SilverScriptError) as exc:
                contract.build_sig_script_for_covenant_decl("bogus", is_leader=is_leader)
            assert "unknown entry" in str(exc.value)


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
        assert plain.bytecode == debug.bytecode
        assert plain.build_sig_script("check", [150]) == debug.build_sig_script("check", [150])


# ---------------------------------------------------------------------------
# State layout semantics
# ---------------------------------------------------------------------------

class TestStateLayout:
    def test_plain_contract_has_empty_state(self):
        # state_span marks the covenant *State* region, not constructor
        # immediates: a plain (non-covenant) contract reports (0, 0) even though
        # its constructor value is embedded elsewhere in the script.
        assert silverscript.compile(GUARD, [100]).state_span == (0, 0)

    def test_covenant_contract_has_nonempty_state(self):
        contract = silverscript.compile(COUNTER, [0])
        start, length = contract.state_span
        assert length > 0
        assert 0 <= start
        assert start + length <= len(contract.bytecode)


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
            == "b8fe43cf0bc3a8c29a9278ac2d063177e8d607e073479a949f670064b405e890"
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
        redeem = silverscript.compile(GUARD, [count]).bytecode
        spk = _kaspa.ScriptBuilder.from_script(redeem).create_pay_to_script_hash_script()
        return _kaspa.address_from_script_public_key(spk, "testnet").to_string()

    def test_compiled_script_wraps_into_p2sh_address(self):
        # silverscript (@v2.0.1) bytes consumed by the core (@c338d49) module:
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
                "contract B() { entry m() { require(true); } }"
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
        assert repr(contract) == 'CompiledContract(name="Guard", bytecode=25 bytes, entries=1)'
        assert repr(contract.abi[0]) == 'EntryAbi(name="check", params=1, dispatch_tag="b0823999")'
        assert repr(contract.abi[0].params[0]) == 'ParamAbi(name="amount", type_name="int")'


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
                   "contract D() { entry f(int[] xs) { require(true); } }")
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
