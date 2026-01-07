# Installation

This guide covers installation of the Kaspa Python SDK.

This Python package can be [installed from PyPi](#installation-via-pypi) or [built from source](#installation-from-source). **With any crypto related library, it is always recommended to build from trusted source code.**


## Requirements

- Python versions 3.9 thru 3.14 are supported
- pip package manager


## Installation via PyPi

```bash
pip install kaspa
```

The following OS/architectures have pre-built wheels available on PyPi:

| Platform | Architecture | Status |
|----------|--------------|--------|
| Linux    | x86_64       | Supported |
| Linux    | aarch64      | Supported |
| macOS    | x86_64       | Supported |
| macOS    | Apple Silicon (arm64) | Supported |
| Windows  | x86_64       | Supported |

Support for alternate builds will require [building from source](#installation-from-source). Building from sdist is not yet supported, as such an sdist is not available on PyPi.


## Installation from Source

1. To build the Python SDK from source, the full [rusty-kaspa](https://github.com/kaspanet/rusty-kaspa) build environment is required. Follow set up instructions in the [Installation section of Rusty Kaspa README](https://github.com/kaspanet/rusty-kaspa?tab=readme-ov-file#installation).

2. Clone the Kaspa Python SDK repository:
```bash
# Clone the repository
git clone https://github.com/smartgoo/kaspa-python-sdk.git
cd kaspa-python-sdk
```

3. Run `./build-release` sh script to build source and built (wheel) dists. The built wheel (`.whl`) file path will be printed: `Built wheel for CPython 3.x to <filepath>`. The `.whl` file can be copied to another location or machine (of the same OS/architecture).

4. Install the wheel built by the prior step using `pip install <.whl filepath>`.


## Optional Dependencies

### Development (and testing):

```bash
pip install kaspa[dev] # Installs from PyPi pyproject.toml

pip install -e ".[dev]" # Installs from local pyproject.toml
```

This includes pytest and pytest-asyncio for running tests.

### Documentation

```bash
pip install kaspa[docs] # Installs from PyPi pyproject.toml

pip install -e ".[docs]" # Installs from local pyproject.toml
```

This includes MkDocs related packages for generating this documentation website.
