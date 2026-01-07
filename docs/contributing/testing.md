# Testing

## Unit Tests

```bash
# All unit tests
pytest tests/unit -v

# Run a specific test file:
pytest tests/unit/test_address.py -v

# Run a specific test:
pytest tests/unit/test_address.py::test_address_validation -v
```

## Integration Tests

```bash
pytest tests/integration -v
```

## Test Fixtures

Shared test fixtures are defined in `tests/conftest.py`. These provide deterministic test values:

```python
# Example fixtures available
known_mnemonic_phrase    # Known test mnemonic
known_private_key        # Known test private key
known_mainnet_address    # Known valid address
testnet_rpc_client       # Connected RPC client (integration tests)
```

