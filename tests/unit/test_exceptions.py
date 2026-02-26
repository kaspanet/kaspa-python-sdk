"""
Unit tests for the custom Exceptions.
"""


class TestExceptionsSubmodule:
    """Tests for Exceptions submodule existence."""

    def test_exceptions_submodule_exists(self):
        """Test that the exceptions submodule is importable."""
        from kaspa import exceptions

        assert exceptions is not None
