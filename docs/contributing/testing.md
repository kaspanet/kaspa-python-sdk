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

Shared test fixtures are defined in `tests/conftest.py`. These provide deterministic test values (keys, addresses, etc.) used throughout tests.
