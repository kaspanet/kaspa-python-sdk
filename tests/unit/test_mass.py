"""
Unit tests for transaction mass helpers.
"""

from kaspa import calculate_storage_mass, NetworkId


class TestCalculateStorageMass:
    """Tests for the calculate_storage_mass function."""

    def test_normal_case_returns_value(self):
        """A typical transaction yields a finite storage mass."""
        net = NetworkId("mainnet")
        result = calculate_storage_mass(net, [1_000_000, 1_000_000], [500_000, 500_000, 500_000])
        assert isinstance(result, int)
        assert result == 4_000_000

    def test_zero_amount_inputs_do_not_panic(self):
        """Inputs whose amount mean rounds down to 0 must not divide-by-zero.

        Regression guard for rusty-kaspa v2.0.1's `calc_storage_mass` change, which
        clamps the input-amount mean to a minimum of 1. With >2 inputs the arithmetic
        path runs `storm_param / mean_ins`; before the clamp, all-zero input amounts
        made `mean_ins == 0` and the call panicked. It must now return a value instead.
        """
        net = NetworkId("mainnet")
        # 3 inputs forces the arithmetic path; zero amounts make the raw mean 0.
        result = calculate_storage_mass(net, [0, 0, 0], [1_000_000, 1_000_000])
        assert isinstance(result, int)
        assert result == 0

    def test_input_sum_less_than_count_does_not_panic(self):
        """The clamp also covers a nonzero input sum smaller than the input count."""
        net = NetworkId("mainnet")
        # sum(amounts) = 2 < 3 inputs -> raw integer mean is 0 -> clamped to 1.
        result = calculate_storage_mass(net, [1, 1, 0], [1_000_000, 1_000_000])
        assert isinstance(result, int)
        assert result == 0
