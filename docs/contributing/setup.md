# Development Setup

This guide details local development environment setup.

## Setup Process

**1.) Setup the Rust & rusty-kaspa environment:**

This SDK is built from Rust source code and leverages native rusty-kaspa. You'll need the full Rust/rusty-kaspa environment: [rusty-kaspa installation guide](https://github.com/kaspanet/rusty-kaspa?tab=readme-ov-file#installation).

**2.) Ensure Python 3.10 - 3.14 is installed, as well as pip.**

**3.) Clone the repository:**

```bash
git clone https://github.com/kaspanet/kaspa-python-sdk.git
cd kaspa-python-sdk
```

**4.) Install optional development & docs dependencies:**

For development and testing, install the optional dependency groups:

```bash
# Activate the virtual environment first
source env/bin/activate

# Install dev dependencies (pytest, pytest-asyncio)
pip install -e ".[dev]"

# Install docs dependencies (mkdocs, mkdocstrings, etc.)
pip install -e ".[docs]"

# Or install both at once
pip install -e ".[dev,docs]"
```

## Verify environment

The project includes a `check` script that automates various builds, lints, and tests to help the development process. It can be used to verify environment is working:

```bash
./check
```

See [building workflows - ./check](building.md#approximate-ci-validation-script) for details.

The locally built `kaspa` module can then be imported and used from Python.
