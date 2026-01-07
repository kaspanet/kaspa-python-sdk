# Kaspa Python SDK

> [!CAUTION]
> **This repository is a proof of concept. Intended solely to assess feasibility of moving Kaspa Python SDK into its own repository.**
>
> **Do NOT use this repository.**
> 
> **The current `kaspa` Python package source can be found [here](https://github.com/aspectron/rusty-kaspa/tree/python).**

---

A Python SDK for interacting with the Kaspa blockchain, built as a native extension module from Rust using [PyO3](https://pyo3.rs/) and [Maturin](https://www.maturin.rs/).

This project wraps the [rusty-kaspa](https://github.com/kaspanet/rusty-kaspa) implementation to provide Python bindings for Kaspa functionality.

## Documentation

Full documentation is available at the [documentation site](https://smartgoo.github.io/kaspa-python-sdk/), including:

- [Installation Guide](https://smartgoo.github.io/kaspa-python-sdk/getting-started/installation/)
- [Quickstart](https://smartgoo.github.io/kaspa-python-sdk/getting-started/quickstart/)
- [API Reference](https://smartgoo.github.io/kaspa-python-sdk/reference/)

## Quick Install

```bash
pip install kaspa
```

## Contributing

Interested in contributing? See the [Contributing Guide](https://smartgoo.github.io/kaspa-python-sdk/contributing/) for:

- [Development Setup](https://smartgoo.github.io/kaspa-python-sdk/contributing/setup/) - Environment setup and build instructions
- [Architecture](https://smartgoo.github.io/kaspa-python-sdk/contributing/architecture/) - How the Rust-PyO3-Python bridge works
- [PyO3 Reference](https://smartgoo.github.io/kaspa-python-sdk/contributing/pyo3-reference/) - Detailed technical reference for CPython/PyO3 integration
- [Development Workflow](https://smartgoo.github.io/kaspa-python-sdk/contributing/development-workflow/) - Building, testing, and iterating
- [Code Style](https://smartgoo.github.io/kaspa-python-sdk/contributing/code-style/) - Coding conventions

## License

This project is licensed under the ISC License.
