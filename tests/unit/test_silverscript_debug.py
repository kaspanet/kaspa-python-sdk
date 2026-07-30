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
    entrypoint function check(int amount) {
        int margin = amount - threshold;
        require(margin > 0);
    }
}
"""

MULTI = """
pragma silverscript ^0.1.0;
contract Multi(int base) {
    entrypoint function add(int amount) { require(amount > base); }
    entrypoint function sub(int amount) { require(amount < base); }
}
"""

HELPER = """
pragma silverscript ^0.1.0;
contract C() {
    function checkPositive(int v) {
        require(v > 10);
    }
    entrypoint function go(int x) {
        checkPositive(x);
    }
}
"""

LOGGER = """
pragma silverscript ^0.1.0;
contract Logger() {
    entrypoint function go(int x) {
        console.log("x is", x);
        require(x > 0);
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

BYTES4 = """
pragma silverscript ^0.1.0;
contract H(byte[4] tag) {
    entrypoint function go(byte[4] x) { require(x == tag); }
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
        return({ count: prev_state.count + amount });
    }
    #[covenant(binding = auth, from = 1, to = 1, mode = transition)]
    function subtract(State prev_state, int amount) : (State) {
        return({ count: prev_state.count - amount });
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

    def test_default_entrypoint_is_first_abi_entry(self):
        result = silverscript.debug_call(GUARD, args=[150], constructor_args=[100])
        assert result.function_name == "check"
        assert result.success is True

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
