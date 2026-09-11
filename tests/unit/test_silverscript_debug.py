"""
Unit tests for kaspa.experimental.silverscript.debug_call (source-level
contract call debugging).
"""

import pytest

import kaspa.experimental.silverscript as silverscript

# ---------------------------------------------------------------------------
# Contract fixtures
# ---------------------------------------------------------------------------

GUARD = """
pragma silverscript ^0.1.0;
contract Guard(int threshold) {
    entry check(int amount) {
        int margin = amount - threshold;
        require(margin > 0);
    }
}
"""

MULTI = """
pragma silverscript ^0.1.0;
contract Multi(int base) {
    entry add(int amount) { require(amount > base); }
    entry sub(int amount) { require(amount < base); }
}
"""

HELPER = """
pragma silverscript ^0.1.0;
contract C() {
    function checkPositive(int v) {
        require(v > 10);
    }
    entry go(int x) {
        checkPositive(x);
    }
}
"""

LOGGER = """
pragma silverscript ^0.1.0;
contract Logger() {
    entry go(int x) {
        console.log("x is", x);
        require(x > 0);
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

BYTES4 = """
pragma silverscript ^0.1.0;
contract H(byte[4] tag) {
    entry go(byte[4] x) { require(x == tag); }
}
"""

# The Counter covenant from examples/silverscript/counter.py: state carried in
# a covenant State region, spent via generated covenant entrypoints.
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

BYTE_BOX = """
pragma silverscript ^0.1.0;
contract ByteBox(byte tag) {
    entry f(byte b) { require(b == tag); }
}
"""

BLOB = """
pragma silverscript ^0.1.0;
contract Blob(byte[] tag) {
    entry go(byte[] data) { require(data == tag); }
}
"""

# A covenant whose state and argument are scalar `byte`s — the synthesized
# output State argument has to narrow to a `byte` too.
MARKER = """
pragma silverscript ^0.1.0;
contract Marker(byte init_tag) {
    byte tag = init_tag;
    #[covenant(binding = auth, from = 1, to = 1, mode = transition)]
    function retag(State prev_state, byte next) : (State) {
        require(next != prev_state.tag);
        return(State { tag: next });
    }
}
"""

# A covenant whose state is a byte array — exercises type-directed state
# conversion (ints, int lists, and hex strings in byte positions).
TAGGED = """
pragma silverscript ^0.1.0;
contract Tagged(byte[4] init_tag) {
    byte[4] tag = init_tag;
    #[covenant(binding = auth, from = 1, to = 1, mode = transition)]
    function retag(State prev_state, byte[4] next) : (State) {
        return(State { tag: next });
    }
}
"""

# Every shape a resolved state initializer can take beyond a bare literal:
# a temporal constructor parameter, a `date(...)` literal, a unit-suffixed
# literal, and the `temporal`/`int`/`string`/`byte` cast calls. The compiler's
# constant folder leaves each of these in the AST, so the debugger has to
# decode them all.
TEMPORAL_FORMS = """
pragma silverscript ^0.1.0;
contract Forms(temporal init_deadline) {
    temporal from_param = init_deadline;
    temporal from_date = date("2030-01-01T00:00:00");
    temporal from_cast = temporal(1700000000);
    temporal from_units = 3 days;
    int from_int_cast = int(init_deadline);
    string from_string_cast = string("tagged");
    byte from_byte_cast = byte(0x07);
    entry check(int a) { require(a > 0); }
}
"""

# The minimal timelock shape: a temporal state field compared against a
# temporal argument, so the decoded state value is observable as pass/fail and
# not only as a reported variable.
DEADLINE = """
pragma silverscript ^0.1.0;
contract Deadline(temporal init_deadline) {
    temporal deadline = init_deadline;
    entry after(temporal t) { require(t >= deadline); }
}
"""

# A cov-bound covenant group: every input of the group is spent in one
# transaction, the lowest-index input is the leader and runs the declaration
# body, the rest defer to the shared delegate entrypoint. Ported from
# upstream's `cov_debug_demo` CLI fixture.
COV_GROUP = """
pragma silverscript ^0.1.0;
contract CovDebugDemo(int initial_value) {
    int value = initial_value;
    #[covenant(binding = cov, from = 2, to = 2, mode = verification)]
    function rebalance(State[] prev_states, State[] new_states) {
        require(prev_states.length == 2);
        require(prev_states[0].value == 10);
        require(prev_states[1].value == 20);
        require(new_states.length == 2);
    }
}
"""

# The same shape, but with a `#[covenant.delegate]` body that declares its own
# parameters: a delegate spend carries the delegate's arguments, not the
# leader's. Ported from upstream's `cov_distinct_delegate_args` CLI fixture.
COV_DELEGATE_ARGS = """
pragma silverscript ^0.1.0;
contract CovDistinctDelegateArgs() {
    byte dummy = 0x00;
    #[covenant(binding = cov, from = 2, to = 2)]
    function transfer(State[] prev_states, State[] new_states, int amount, bool allowed) {
        require(amount >= 0);
        require(allowed);
    }
    #[covenant.delegate]
    function authorizeDelegate(byte[] witness) {
        require(witness.length > 0);
    }
}
"""

COVENANT_ID = "11" * 32


def counter_scenario(prev_count, next_count):
    """A 1-in/1-out Counter transition: prev state in, next state out."""
    return {
        "inputs": [
            {
                "utxo_value": 5000,
                "covenant_id": COVENANT_ID,
                "state": {"count": prev_count},
            }
        ],
        "outputs": [
            {
                "value": 5000,
                "covenant_id": COVENANT_ID,
                "authorizing_input": 0,
                "state": {"count": next_count},
            }
        ],
    }


def marker_scenario(prev_tag, next_tag):
    """A 1-in/1-out Marker transition: prev byte state in, next byte state out."""
    return {
        "inputs": [
            {
                "utxo_value": 5000,
                "covenant_id": COVENANT_ID,
                "state": {"tag": prev_tag},
            }
        ],
        "outputs": [
            {
                "value": 5000,
                "covenant_id": COVENANT_ID,
                "authorizing_input": 0,
                "state": {"tag": next_tag},
            }
        ],
    }


def cov_group_scenario(active_input_index):
    """A 2-in/2-out cov-bound group, debugging one of its two inputs."""
    return {
        "active_input_index": active_input_index,
        "inputs": [
            {
                "utxo_value": 5000,
                "covenant_id": COVENANT_ID,
                "constructor_args": [10],
            },
            {
                "utxo_value": 5000,
                "covenant_id": COVENANT_ID,
                "constructor_args": [20],
            },
        ],
        "outputs": [
            {
                "value": 5000,
                "covenant_id": COVENANT_ID,
                "authorizing_input": 0,
                "constructor_args": [30],
            },
            {
                "value": 5000,
                "covenant_id": COVENANT_ID,
                "authorizing_input": 0,
                "constructor_args": [40],
            },
        ],
    }


def delegate_args_scenario(active_input_index):
    """A 2-in/0-out cov-bound group whose delegate takes its own arguments."""
    return {
        "active_input_index": active_input_index,
        "inputs": [
            {"utxo_value": 5000, "covenant_id": COVENANT_ID},
            {"utxo_value": 5000, "covenant_id": COVENANT_ID},
        ],
        "outputs": [],
    }


# ---------------------------------------------------------------------------
# Pass / fail basics
# ---------------------------------------------------------------------------

class TestDebugCallBasics:
    def test_passing_call(self):
        result = silverscript.debug_call(GUARD, "check", [150], [100])
        assert isinstance(result, silverscript.DebugCallResult)
        assert result.success is True
        assert result.error is None
        assert result.failure is None
        assert result.function_name == "check"

    def test_failing_call_reports_not_raises(self):
        result = silverscript.debug_call(GUARD, "check", [50], [100])
        assert result.success is False
        assert "verification failed" in result.error
        assert isinstance(result.failure, silverscript.FailureReport)

    def test_default_entrypoint_is_first_declared(self):
        # debug_call defaults to the first entrypoint *declared in the source*,
        # not the first in `.abi` (which is alphabetical). debug_call requires
        # source anyway, so declaration order is unambiguous here.
        result = silverscript.debug_call(GUARD, args=[150], constructor_args=[100])
        assert result.function_name == "check"
        assert result.success is True

    def test_default_entrypoint_is_declaration_order_not_alphabetical(self):
        # ORDER declares zebra first; alphabetically "alpha" would win.
        src = (
            "contract Order() {\n"
            "    entry zebra(int a)  { require(a > 0); }\n"
            "    entry alpha(int a)  { require(a > 0); }\n"
            "}\n"
        )
        assert silverscript.debug_call(src, args=[5]).function_name == "zebra"
        assert [e.name for e in silverscript.compile(src).abi] == ["alpha", "zebra"]

    def test_multi_entrypoint_selection(self):
        assert silverscript.debug_call(MULTI, "add", [20], [10]).success is True
        assert silverscript.debug_call(MULTI, "add", [5], [10]).success is False
        assert silverscript.debug_call(MULTI, "sub", [5], [10]).success is True

    def test_repr(self):
        result = silverscript.debug_call(GUARD, "check", [150], [100])
        assert repr(result) == 'DebugCallResult(function_name="check", success=True, error=None)'

    def test_repr_on_failure(self):
        result = silverscript.debug_call(GUARD, "check", [50], [100])
        assert repr(result) == (
            'DebugCallResult(function_name="check", success=False,'
            ' error="script ran, but verification failed")'
        )


# ---------------------------------------------------------------------------
# The failure report: frames, variables, rendering
# ---------------------------------------------------------------------------

class TestFailureReport:
    def _failure(self):
        return silverscript.debug_call(GUARD, "check", [50], [100]).failure

    def test_message(self):
        assert "verification failed" in self._failure().message

    def test_innermost_frame_points_at_failing_line(self):
        frame = self._failure().frames[0]
        assert frame.function_name == "check"
        # The require() is on line 6 of the GUARD source (1-based).
        assert frame.line == 6

    def test_variables_decoded_in_source_terms(self):
        variables = {v.name: v for v in self._failure().frames[0].variables}
        assert variables["amount"].value == 50
        assert variables["amount"].origin == "arg"
        assert variables["amount"].type_name == "int"
        assert variables["margin"].value == -50
        assert variables["margin"].origin == "local"
        assert variables["threshold"].value == 100
        assert variables["threshold"].origin == "ctor"

    def test_variable_display(self):
        variables = {v.name: v for v in self._failure().frames[0].variables}
        assert variables["margin"].display == "-50"

    def test_render_includes_source_context_and_variables(self):
        rendered = str(self._failure())
        assert "require(margin > 0);" in rendered
        assert "verification failed here" in rendered
        assert "margin = -50" in rendered
        assert self._failure().render() == rendered

    def test_inlined_call_stack_has_caller_frame(self):
        result = silverscript.debug_call(HELPER, "go", [3])
        assert result.success is False
        frames = result.failure.frames
        assert len(frames) == 2
        # Innermost frame first: the require() inside the helper.
        assert frames[0].function_name == "checkPositive"
        variables = {v.name: v.value for v in frames[0].variables}
        assert variables["v"] == 3
        assert "called from" in str(result.failure)

    def test_bytes_variable_decodes_to_bytes(self):
        result = silverscript.debug_call(
            BYTES4, "go", [b"\xaa\xbb\xcc\xdd"], [b"\x01\x02\x03\x04"]
        )
        assert result.success is False
        variables = {v.name: v for v in result.failure.frames[0].variables}
        assert variables["x"].value == b"\xaa\xbb\xcc\xdd"
        assert variables["tag"].value == b"\x01\x02\x03\x04"


# ---------------------------------------------------------------------------
# Console output
# ---------------------------------------------------------------------------

class TestConsole:
    def test_console_log_captured_on_success(self):
        result = silverscript.debug_call(LOGGER, "go", [7])
        assert result.success is True
        assert result.console == ["x is 7"]

    def test_console_log_captured_on_failure(self):
        result = silverscript.debug_call(LOGGER, "go", [-1])
        assert result.success is False
        assert result.console == ["x is -1"]

    def test_no_console_output(self):
        assert silverscript.debug_call(GUARD, "check", [150], [100]).console == []


# ---------------------------------------------------------------------------
# Execution tracing
# ---------------------------------------------------------------------------

class TestTrace:
    def test_trace_off_by_default(self):
        assert silverscript.debug_call(GUARD, "check", [150], [100]).trace is None

    def test_trace_records_statements_in_order(self):
        result = silverscript.debug_call(GUARD, "check", [150], [100], trace=True)
        assert result.success is True
        assert [(s.line, s.statement) for s in result.trace] == [
            (5, "int margin = amount - threshold;"),
            (6, "require(margin > 0);"),
        ]
        assert result.trace[0].function_name == "check"

    def test_trace_variables_snapshot_when_statement_reached(self):
        result = silverscript.debug_call(GUARD, "check", [150], [100], trace=True)
        first, second = result.trace
        # margin is defined by the first statement, so it appears from the
        # second step on.
        assert "margin" not in {v.name for v in first.variables}
        assert {v.name: v.value for v in second.variables} == {
            "amount": 150,
            "margin": 50,
            "threshold": 100,
        }

    def test_trace_on_failure_ends_at_failing_statement(self):
        result = silverscript.debug_call(GUARD, "check", [50], [100], trace=True)
        assert result.success is False
        assert result.failure is not None
        assert result.trace[-1].line == 6
        assert result.trace[-1].statement == "require(margin > 0);"

    def test_trace_covers_inlined_helper(self):
        result = silverscript.debug_call(HELPER, "go", [30], trace=True)
        assert result.success is True
        helper_steps = [s for s in result.trace if s.statement == "require(v > 10);"]
        assert len(helper_steps) == 1
        assert helper_steps[0].function_name == "checkPositive"
        assert {v.name: v.value for v in helper_steps[0].variables}["v"] == 30

    def test_trace_covenant_transition_records_no_pauses(self):
        # Covenant transition bodies are verified as a whole by the engine
        # (shadow evaluation), not stepped statement-by-statement — the
        # upstream CLI debugger steps them the same way. The trace is
        # present but empty; the failure report still decodes them.
        result = silverscript.debug_call(
            COUNTER, "add", [5], [0], tx=counter_scenario(10, 15), trace=True
        )
        assert result.success is True
        assert result.trace == []

    def test_trace_alongside_console(self):
        result = silverscript.debug_call(LOGGER, "go", [7], trace=True)
        assert result.console == ["x is 7"]
        assert any(s.statement == 'console.log("x is", x);' for s in result.trace)

    def test_trace_step_repr(self):
        result = silverscript.debug_call(GUARD, "check", [150], [100], trace=True)
        assert repr(result.trace[1]) == (
            'TraceStep(line=6, function_name="check",'
            ' statement="require(margin > 0);", 3 variable(s))'
        )


# ---------------------------------------------------------------------------
# The tx scenario: introspection and covenant transitions
# ---------------------------------------------------------------------------

class TestTxScenario:
    def test_default_scenario_single_output(self):
        # The default scenario has one 5000-sompi output, so the announcement
        # contract's require(tx.outputs[0].value == 0) fails.
        assert silverscript.debug_call(ANNOUNCEMENT, "announce").success is False

    def test_explicit_outputs_are_visible_to_introspection(self):
        result = silverscript.debug_call(
            ANNOUNCEMENT,
            "announce",
            tx={"inputs": [{"utxo_value": 5000}], "outputs": [{"value": 0}]},
        )
        assert result.success is True

    def test_covenant_transition_passes(self):
        result = silverscript.debug_call(
            COUNTER, "add", [5], [0], tx=counter_scenario(10, 15)
        )
        assert result.success is True

    def test_covenant_transition_wrong_next_state_fails(self):
        result = silverscript.debug_call(
            COUNTER, "add", [5], [0], tx=counter_scenario(10, 14)
        )
        assert result.success is False
        assert "verification failed" in result.error
        # The failure report decodes the covenant state that was checked.
        rendered = str(result.failure)
        assert "prev_state = {count: 10}" in rendered

    def test_covenant_second_entrypoint(self):
        result = silverscript.debug_call(
            COUNTER, "subtract", [3], [0], tx=counter_scenario(10, 7)
        )
        assert result.success is True

    def test_covenant_id_accepts_bytes(self):
        tx = counter_scenario(10, 15)
        tx["inputs"][0]["covenant_id"] = bytes.fromhex(COVENANT_ID)
        tx["outputs"][0]["covenant_id"] = bytes.fromhex(COVENANT_ID)
        assert silverscript.debug_call(COUNTER, "add", [5], [0], tx=tx).success is True

    def test_covenant_transition_with_change_output(self):
        # A plain change output has no covenant binding, so it must not count
        # toward the synthesized output State argument.
        tx = counter_scenario(10, 15)
        tx["outputs"].append({"value": 1000})
        assert silverscript.debug_call(COUNTER, "add", [5], [0], tx=tx).success is True

    def test_covenant_transition_with_raw_script_change_output(self):
        tx = counter_scenario(10, 15)
        tx["outputs"].append({"value": 1000, "p2pk_pubkey": b"\x02" * 32})
        assert silverscript.debug_call(COUNTER, "add", [5], [0], tx=tx).success is True

    def test_byte_array_state_spellings_are_equivalent(self):
        # bytes, int lists, and hex strings are all accepted in byte-array
        # state fields and produce the same simulation.
        for prev, next_ in [
            (b"\x01\x02\x03\x04", b"\xaa\xbb\xcc\xdd"),
            ([1, 2, 3, 4], [0xAA, 0xBB, 0xCC, 0xDD]),
            ("0x01020304", "0xaabbccdd"),
        ]:
            tx = {
                "inputs": [{
                    "utxo_value": 5000,
                    "covenant_id": COVENANT_ID,
                    "state": {"tag": prev},
                }],
                "outputs": [{
                    "value": 5000,
                    "covenant_id": COVENANT_ID,
                    "authorizing_input": 0,
                    "state": {"tag": next_},
                }],
            }
            result = silverscript.debug_call(
                TAGGED, "retag", [b"\xaa\xbb\xcc\xdd"], [b"\x01\x02\x03\x04"], tx=tx
            )
            assert result.success is True, f"spelling {prev!r} -> {next_!r}"


# ---------------------------------------------------------------------------
# `binding = cov` covenant groups: leader and delegate spends
# ---------------------------------------------------------------------------

class TestCovBinding:
    def test_group_leader_spend(self):
        # Input 0 is the lowest-index member, so it runs the declaration body.
        result = silverscript.debug_call(COV_GROUP, "rebalance", tx=cov_group_scenario(0))
        assert result.success is True

    def test_group_delegate_spend(self):
        # Input 1 defers to the shared delegate entrypoint, which takes no
        # arguments here because the contract declares no delegate body.
        result = silverscript.debug_call(COV_GROUP, "rebalance", tx=cov_group_scenario(1))
        assert result.success is True

    def test_group_leader_sees_every_input_state(self):
        # `prev_states[1].value == 20` comes from the companion input, which
        # the leader reads out of that input's redeem script.
        tx = cov_group_scenario(0)
        tx["inputs"][1]["constructor_args"] = [21]
        result = silverscript.debug_call(COV_GROUP, "rebalance", tx=tx)
        assert result.success is False

    def test_delegate_body_args_are_the_delegate_s_own(self):
        # A parameterised `#[covenant.delegate]` body means a delegate spend
        # carries that body's arguments, not the leader's.
        result = silverscript.debug_call(
            COV_DELEGATE_ARGS, "transfer", [b"\x01"], tx=delegate_args_scenario(1)
        )
        assert result.success is True

    def test_leader_args_unaffected_by_parameterised_delegate(self):
        result = silverscript.debug_call(
            COV_DELEGATE_ARGS, "transfer", [1, True], tx=delegate_args_scenario(0)
        )
        assert result.success is True

    def test_delegate_body_require_can_fail(self):
        # An empty witness fails the delegate's own `require`, proving the
        # argument reached the delegate body rather than being dropped.
        result = silverscript.debug_call(
            COV_DELEGATE_ARGS, "transfer", [b""], tx=delegate_args_scenario(1)
        )
        assert result.success is False

    def test_leader_body_require_can_fail(self):
        result = silverscript.debug_call(
            COV_DELEGATE_ARGS, "transfer", [1, False], tx=delegate_args_scenario(0)
        )
        assert result.success is False


# ---------------------------------------------------------------------------
# `byte` arguments and state
# ---------------------------------------------------------------------------

class TestByteValues:
    def test_byte_arg_passes(self):
        assert silverscript.debug_call(BYTE_BOX, "f", [1], [1]).success is True

    def test_byte_arg_fails(self):
        assert silverscript.debug_call(BYTE_BOX, "f", [2], [1]).success is False

    def test_single_byte_bytes_equivalent_to_int(self):
        assert silverscript.debug_call(BYTE_BOX, "f", [b"\x01"], [1]).success is True

    def test_byte_arg_out_of_range_raises(self):
        with pytest.raises(silverscript.SilverScriptError):
            silverscript.debug_call(BYTE_BOX, "f", [256], [1])

    def test_byte_array_type_name_matches_the_abi(self):
        # Both surfaces name the parameter `byte[]`; they used to disagree.
        result = silverscript.debug_call(BLOB, "go", [b"\xcc"], [b"\xaa\xbb"])
        variables = {v.name: v for v in result.failure.frames[0].variables}
        abi_name = silverscript.compile(BLOB, [b"\xaa\xbb"]).abi[0].params[0].type_name
        assert variables["data"].type_name == abi_name == "byte[]"

    def test_byte_variable_decodes_in_source_terms(self):
        variables = {
            v.name: v
            for v in silverscript.debug_call(BYTE_BOX, "f", [2], [1]).failure.frames[0].variables
        }
        assert variables["b"].type_name == "byte"
        assert variables["tag"].value == b"\x01"

    def test_covenant_byte_state_transition_passes(self):
        result = silverscript.debug_call(
            MARKER, "retag", [2], [1], tx=marker_scenario(1, 2)
        )
        assert result.success is True

    def test_covenant_byte_state_wrong_next_state_fails(self):
        # The synthesized output State is the byte the transition must produce.
        result = silverscript.debug_call(
            MARKER, "retag", [2], [1], tx=marker_scenario(1, 3)
        )
        assert result.success is False
        assert "verification failed" in result.error

    def test_covenant_byte_arg_out_of_range_raises(self):
        with pytest.raises(silverscript.SilverScriptError):
            silverscript.debug_call(MARKER, "retag", [256], [1], tx=marker_scenario(1, 2))


# ---------------------------------------------------------------------------
# `temporal` state
# ---------------------------------------------------------------------------

class TestTemporalState:
    def test_every_initializer_form_decodes(self):
        # `debug_call` used to abort outright on each of these, so a contract
        # with any temporal state -- every timelock -- was undebuggable.
        result = silverscript.debug_call(
            TEMPORAL_FORMS, "check", [0], [1700000000]
        )
        variables = {v.name: v for v in result.failure.frames[0].variables}
        decoded = {name: (v.type_name, v.value) for name, v in variables.items()}
        assert decoded == {
            "a": ("int", 0),
            "init_deadline": ("temporal", 1700000000),
            "from_param": ("temporal", 1700000000),
            "from_date": ("temporal", 1893456000000),
            "from_cast": ("temporal", 1700000000),
            "from_units": ("temporal", 259200000),
            "from_int_cast": ("int", 1700000000),
            "from_string_cast": ("string", "tagged"),
            "from_byte_cast": ("byte", b"\x07"),
        }

    def test_int_cast_retags_a_temporal_as_int(self):
        # `int(t)` and `temporal(n)` carry the same number; only the reported
        # type distinguishes them.
        variables = {
            v.name: v
            for v in silverscript.debug_call(
                TEMPORAL_FORMS, "check", [0], [1700000000]
            ).failure.frames[0].variables
        }
        assert variables["from_int_cast"].value == variables["from_param"].value
        assert variables["from_int_cast"].type_name == "int"
        assert variables["from_param"].type_name == "temporal"

    def test_deadline_reached_passes(self):
        assert silverscript.debug_call(
            DEADLINE, "after", [1700000000], [1700000000]
        ).success is True

    def test_deadline_not_reached_fails(self):
        result = silverscript.debug_call(
            DEADLINE, "after", [1699999999], [1700000000]
        )
        assert result.success is False
        assert "verification failed" in result.error

    def test_temporal_state_is_the_compiled_value_not_the_argument(self):
        # The deadline the script enforces comes from the constructor, so
        # moving it past the argument flips the outcome.
        assert silverscript.debug_call(
            DEADLINE, "after", [1700000000], [1700000001]
        ).success is False

    def test_constant_folded_initializer_is_still_unsupported(self):
        # A shared gap with the upstream CLI debugger: `is_const_expr` admits
        # constant integer arithmetic, but neither debugger decodes it. Pinned
        # so that closing it upstream is a visible change here.
        source = TEMPORAL_FORMS.replace(
            "temporal from_cast = temporal(1700000000);",
            "temporal from_cast = temporal(1700000000 + 1);",
        )
        with pytest.raises(
            silverscript.SilverScriptError, match="unsupported resolved state expression"
        ):
            silverscript.debug_call(source, "check", [0], [1700000000])


# ---------------------------------------------------------------------------
# Usage errors raise; script failures don't
# ---------------------------------------------------------------------------

class TestErrors:
    def test_invalid_source_raises(self):
        with pytest.raises(silverscript.SilverScriptError):
            silverscript.debug_call("this is not silverscript")

    def test_unknown_entrypoint_raises(self):
        with pytest.raises(silverscript.SilverScriptError):
            silverscript.debug_call(GUARD, "does_not_exist", [1], [100])

    def test_wrong_argument_type_raises(self):
        with pytest.raises(silverscript.SilverScriptError):
            silverscript.debug_call(GUARD, "check", ["not an int"], [100])

    def test_tx_must_be_dict(self):
        with pytest.raises(silverscript.SilverScriptError, match="tx must be a dict"):
            silverscript.debug_call(GUARD, "check", [150], [100], tx=[])

    def test_tx_requires_inputs(self):
        with pytest.raises(silverscript.SilverScriptError, match="inputs"):
            silverscript.debug_call(GUARD, "check", [150], [100], tx={})

    def test_tx_input_requires_utxo_value(self):
        with pytest.raises(silverscript.SilverScriptError, match="utxo_value"):
            silverscript.debug_call(GUARD, "check", [150], [100], tx={"inputs": [{}]})

    def test_tx_unknown_key_raises(self):
        with pytest.raises(silverscript.SilverScriptError, match="unknown tx input 0 key 'bogus'"):
            silverscript.debug_call(
                GUARD, "check", [150], [100],
                tx={"inputs": [{"utxo_value": 5000, "bogus": 1}]},
            )

    def test_tx_active_input_index_out_of_range_raises(self):
        with pytest.raises(silverscript.SilverScriptError, match="out of range"):
            silverscript.debug_call(
                GUARD, "check", [150], [100],
                tx={"inputs": [{"utxo_value": 5000}], "active_input_index": 1},
            )

    def test_state_must_be_dict(self):
        with pytest.raises(silverscript.SilverScriptError, match="state"):
            silverscript.debug_call(
                COUNTER, "add", [5], [0],
                tx={"inputs": [{"utxo_value": 5000, "state": 7}]},
            )

    def test_covenant_id_must_be_32_bytes(self):
        with pytest.raises(silverscript.SilverScriptError, match="32 bytes"):
            silverscript.debug_call(
                COUNTER, "add", [5], [0],
                tx={"inputs": [{"utxo_value": 5000, "covenant_id": b"\x11"}]},
            )

    def test_unknown_state_field_raises(self):
        with pytest.raises(silverscript.SilverScriptError, match="unknown state field 'nope'"):
            silverscript.debug_call(
                COUNTER, "add", [5], [0], tx=counter_scenario(10, 15) | {
                    "inputs": [{
                        "utxo_value": 5000,
                        "covenant_id": COVENANT_ID,
                        "state": {"nope": 1},
                    }],
                },
            )

    def test_misspelled_state_field_named_in_error(self):
        # A typo'd key is reported as the unknown key it is, not as the
        # missing field it shadows.
        with pytest.raises(silverscript.SilverScriptError, match="unknown state field 'cuont'"):
            silverscript.debug_call(
                COUNTER, "add", [5], [0], tx=counter_scenario(10, 15) | {
                    "inputs": [{
                        "utxo_value": 5000,
                        "covenant_id": COVENANT_ID,
                        "state": {"cuont": 10},
                    }],
                },
            )

    def test_missing_state_field_raises(self):
        with pytest.raises(silverscript.SilverScriptError, match="missing state field 'count'"):
            silverscript.debug_call(
                COUNTER, "add", [5], [0], tx=counter_scenario(10, 15) | {
                    "inputs": [{
                        "utxo_value": 5000,
                        "covenant_id": COVENANT_ID,
                        "state": {},
                    }],
                },
            )

    def test_state_validated_even_with_raw_utxo_script(self):
        # A raw utxo_script override must not bypass state validation.
        with pytest.raises(silverscript.SilverScriptError, match="unknown state field 'cuont'"):
            silverscript.debug_call(
                COUNTER, "add", [5], [0], tx=counter_scenario(10, 15) | {
                    "inputs": [{
                        "utxo_value": 5000,
                        "covenant_id": COVENANT_ID,
                        "utxo_script": b"\x51",
                        "state": {"cuont": 10},
                    }],
                },
            )

    def test_output_state_validated_even_with_raw_script(self):
        tx = counter_scenario(10, 15)
        tx["outputs"][0]["script"] = b"\x51"
        tx["outputs"][0]["state"] = {"cuont": 15}
        with pytest.raises(silverscript.SilverScriptError, match="unknown state field 'cuont'"):
            silverscript.debug_call(COUNTER, "add", [5], [0], tx=tx)

    def test_state_field_type_mismatch_raises(self):
        with pytest.raises(
            silverscript.SilverScriptError, match="state field 'count' expects int"
        ):
            silverscript.debug_call(
                COUNTER, "add", [5], [0], tx=counter_scenario(10, 15) | {
                    "inputs": [{
                        "utxo_value": 5000,
                        "covenant_id": COVENANT_ID,
                        "state": {"count": "not an int"},
                    }],
                },
            )

    def test_byte_array_state_wrong_length_raises(self):
        with pytest.raises(
            silverscript.SilverScriptError, match="state field 'tag' expects 4 bytes, got 3"
        ):
            silverscript.debug_call(
                TAGGED, "retag", [b"\xaa\xbb\xcc\xdd"], [b"\x01\x02\x03\x04"],
                tx={
                    "inputs": [{
                        "utxo_value": 5000,
                        "covenant_id": COVENANT_ID,
                        "state": {"tag": [1, 2, 3]},
                    }],
                    "outputs": [],
                },
            )


# ---------------------------------------------------------------------------
# Object semantics
# ---------------------------------------------------------------------------

class TestObjectSemantics:
    def test_result_is_frozen(self):
        result = silverscript.debug_call(GUARD, "check", [150], [100])
        with pytest.raises(AttributeError):
            result.success = False

    def test_reprs(self):
        result = silverscript.debug_call(GUARD, "check", [50], [100])
        assert repr(result.failure).startswith("FailureReport(")
        frame = result.failure.frames[0]
        assert repr(frame).startswith("FailureFrame(")
        assert repr(frame.variables[0]).startswith("DebugVariable(")
