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

Integration tests require network access. By default they connect to `mainnet` via the Public Node Network (PNN) resolver.

```bash
# Default: mainnet via Resolver
pytest tests/integration -v

# Target a specific network and/or direct RPC server
pytest tests/integration -v --network-id testnet-10 --rpc-url ws://host:port
```

### CLI options

- `--network-id` — Kaspa network ID (default: `mainnet`). Examples: `mainnet`, `testnet-10`.
- `--rpc-url` — Direct RPC server WebSocket URL (optional). Bypasses the resolver and connects directly to the given node.

Note: resolver tests always target `mainnet` since PNN only exposes mainnet nodes.

## Test Fixtures

Shared test fixtures are defined in `tests/conftest.py`. These provide deterministic test values (keys, addresses, etc.) used throughout tests.
