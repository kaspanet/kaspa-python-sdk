# Building

The project includes several shell scripts to streamline common tasks.

## Development Build Script

Use this to build development version of the package:

```bash
./build-dev
```

This script:

1. Creates/activates a virtual environment (`env/`)
2. Installs Maturin if not present
3. Generates Python stub files (`.pyi`)
4. Builds and installs the extension module in development mode

The development build is faster but not optimized. After running, activate the environment:

```bash
source env/bin/activate
```

The `kaspa` Python module can than be imported and used in Python.

## Release Build Script

Use this to build release version and wheels of the package:

```bash
./build-release
```

This script:

1. Creates/activates a virtual environment
2. Generates stub files (release mode)
3. Builds optimized wheel (`.whl`) and source distribution

Output files are placed in `target/wheels/`.

## (Approximate) CI Validation Script

```bash
./check
```

This script runs the following:

1. `cargo fmt --all` - Format Rust code
2. `cargo clippy -- -D warnings` - Lint Rust code (warnings as errors)
3. `./build-dev` - Build the extension
4. `pip install -e ".[dev,docs]"` - Install dependencies
5. `pytest tests/unit -v` - Run unit tests
6. `pytest tests/integration -v` - Run integration tests
7. `mkdocs build --strict` - Verify documentation builds

