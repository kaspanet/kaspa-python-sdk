# Documentation

Reference for how the documentation site is built, including stub generation, API reference generation, and doc comment format.

## Site Overview

The documentation site is built with [MkDocs](https://www.mkdocs.org/) using the [Material theme](https://squidfunk.github.io/mkdocs-material/). Configuration lives in `mkdocs.yml`.

| Plugin | Purpose |
|--------|---------|
| `mkdocs-gen-files` | Runs `docs/gen_ref_pages.py` to generate API reference pages |
| `mkdocstrings` | Renders docstrings from `kaspa.pyi` into HTML |
| `mike` | Handles documentation versioning |

## Stub Generation Pipeline

Type stub files (`.pyi`) serve two purposes:

1. **IDE support** - Autocompletion and type checking for the native module
2. **API documentation** - Source for auto-generated API Reference pages

### Stub Generation

Stubs are generated automatically during `./build-dev` and `./build-release`, or manually:

```bash
cargo run --bin stub-gen
```

The generator (`src/bin/stub_gen.rs`) performs:

1. `pyo3-stub-gen` extracts types and signatures from Rust source
2. Post-processing fixes enum names (`Py` prefix removal), RPC method signatures
3. Appends RPC TypedDicts from `kaspa_rpc.pyi` (manually maintained)
4. Outputs `kaspa.pyi` in project root

### API Reference Generation

At docs build time (`mkdocs build` or `mkdocs serve`):

1. `docs/gen_ref_pages.py` parses `kaspa.pyi` for classes, functions, and enums
2. Extracts `Category:` tags from docstrings
3. Groups objects by category (Core, RPC, Wallet)
4. Generates `reference/*.md` pages (one per class/function)
5. `mkdocstrings` renders final HTML from the docstrings

## Building Docs

```bash
mkdocs serve              # Local preview at http://127.0.0.1:8000
mkdocs build --strict     # Production build to site/
```

## Doc Comment Format

### Required Attributes

| Item Type | Required Attribute |
|-----------|-------------------|
| Class | `#[gen_stub_pyclass]` before `#[pyclass]` |
| Methods | `#[gen_stub_pymethods]` before `#[pymethods]` |
| Function | `#[gen_stub_pyfunction]` before `#[pyfunction]` |

### Class/Struct Documentation

```rust
/// Brief description of the class.
///
/// Extended description with additional details.
#[gen_stub_pyclass]
#[pyclass(name = "ClassName")]
pub struct PyClassName(pub NativeType);
```

### Method/Function Documentation

Uses Google-style docstrings for [mkdocstrings](https://mkdocstrings.github.io/):

```rust
/// Brief description of the method.
///
/// Args:
///     param_name: Description of the parameter.
///
/// Returns:
///     ReturnType: Description of return value.
///
/// Raises:
///     Exception: When the exception occurs.
#[new]
pub fn constructor(param_name: &str) -> PyResult<Self> {
    // ...
}
```

### Docstring Sections

| Section | Purpose |
|---------|---------|
| `Args:` | Parameter documentation |
| `Returns:` | Return value documentation |
| `Raises:` | Exception documentation |
| `Note:` | Additional information |
| `Example:` | Code examples |
| `Category:` | API Reference category |

## Categories

Categories organize items in the API Reference. Specified via `Category:` tag in docstrings.

| Category | Contents |
|----------|----------|
| `Core/Types` | Address, NetworkId, Hash, ScriptPublicKey |
| `Core/Utils` | Utility functions |
| `RPC/Core` | RpcClient, Resolver |
| `RPC/Messages` | Request/Response types |
| `RPC/Types` | RPC-related types |
| `Wallet/Core` | Mnemonic, derivation utilities |
| `Wallet/Keys` | PrivateKey, PublicKey, XPrv, XPub, generators |
| `Wallet/Transactions` | Transaction, Generator, PaymentOutput |

Auto-categorization rules (when no `Category:` specified):

- `*Request` / `*Response` suffix → `RPC/Messages`
- `Rpc*` prefix → `RPC/Types`
- All others → `Other`

Categories are defined in `docs/gen_ref_pages.py`.
