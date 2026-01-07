# Architecture & Design

This guide provides a high-level overview of the Kaspa Python SDK architecture.
## Project Structure Overview

```
kaspa-python-sdk/
├── src/                  # Rust source code
│   ├── lib.rs
│   └── ...
├── tests/                # Python test suite
│   ├── unit/             # Unit tests
│   └── integration/      # Integration tests (network required)
├── docs/                 # Documentation source (MkDocs)
├── examples/             # Example Python scripts
├── kaspa_rpc.pyi         # Manually maintained RPC stubs
├── kaspa.pyi             # Auto-generated full type stubs
├── build-dev             # Development build script
├── build-release         # Release build script
└── check                 # CI validation script
```

For convenience, code inside `src/` closely mirrors native rusty-kaspa file layout.

## High-Level Overview

The Kaspa Python SDK is a **native Python extension module** built from Rust source code. It wraps the [rusty-kaspa](https://github.com/kaspanet/rusty-kaspa) library to expose Kaspa functionality to Python.

```
┌─────────────────────────────────────────────────────────────┐
│                     Python Application                       │
│                    (import kaspa)                            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   kaspa Python Module                        │
│              (Native Extension - .so/.dylib/.pyd)            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      PyO3 Bindings                           │
│        (Rust ↔ CPython C-API Bridge)                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     Rust Wrappers                            │
│              (PyAddress, PyRpcClient, etc.)                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      rusty-kaspa                             │
│            (Native Kaspa Implementation)                     │
└─────────────────────────────────────────────────────────────┘
```

## Key Technologies

| Technology | Purpose |
|------------|---------|
| [PyO3](https://pyo3.rs/) | Rust library providing Python C-API bindings |
| [Maturin](https://www.maturin.rs/) | Build tool for Rust-based Python packages |
| [pyo3-stub-gen](https://github.com/Jij-Inc/pyo3-stub-gen) | Generates `.pyi` type stub files |

## Design Principles

The SDK follows these core principles:

1. **Wrappers *should* perform type conversion only** - To the extent possible, wrapper code should only convert types between Python and Rust. Logic lives in rusty-kaspa.

2. **Use rusty-kaspa native logic** - To the extent possible, when logic is needed, leverage rusty-kaspa's implementation rather than reimplementing in this repository.

3. **Mirror rusty-kaspa structure** - The directory structure approximately mirrors rusty-kaspa's workspace layout for easier navigation.

!!! note "WASM Features Dependency"
    This repository relies on rusty-kaspa features gated behind WASM feature flags. The WASM features are a required dependency.

## The Newtype Pattern

The SDK uses the [newtype pattern](https://doc.rust-lang.org/rust-by-example/generics/new_types.html) extensively. Most Python-exposed structs wrap a rusty-kaspa native type:

```rust
// A wrapper around rusty-kaspa's Address type
#[pyclass(name = "Address")]
pub struct PyAddress(Address);
```

The wrapper then implements methods that delegate to the inner type:

```rust
#[pymethods]
impl PyAddress {
    #[new]
    pub fn constructor(address: &str) -> PyResult<PyAddress> {
        Ok(PyAddress(address.try_into().map_err(
            |err: AddressError| PyException::new_err(err.to_string()),
        )?))
    }

    pub fn to_string(&self) -> String {
        self.0.address_to_string()  // Delegates to inner type
    }
}
```
