# PyO3 Reference

This page provides a brief summary of core concepts for the Rust-PyO3-CPython integration.

## Python Memory Model

In CPython, all data is stored on a private heap managed by CPython. The lifetime of all objects on this heap is managed via reference counting.

All data is stored on the heap in a `PyObject*` container class. This includes primitives (integers, strings, etc.) as well as custom classes and so on.

The `PyObject*` container has a field `ob_refcnt` which tracks the reference count for the given object. As references are created, this field is incremented using CPython's `Py_INCREF()` function. As references are dropped, it is decremented using `Py_DECREF()`.

When the reference count of data on the heap (again, inside of a `PyObject*`) reaches 0, the memory is deallocated.

## CPython & The C-API

CPython is the original, de facto, and most-widely used Python implementation. CPython is not *Python the language*.

CPython is effectively the runtime for *Python the language*:

- Compiles Python code into bytecode
- Runs the bytecode on its Python Virtual Machine
- Provides functionality like the standard library, built-in functions, etc.

The Python C-API exposes (a huge set of) functionality for integrating with CPython. Including APIs for object creation on the Python heap, reference count management, executing Python functions, and so on. This allows CPython to load external libraries, called **extension modules**, making them usable from your Python program. Extension modules can be written in any language that can interact with Python's C-API.

### C-API Concepts & Terminology

**Owned reference**
:   A pointer to data on the Python heap. The holder contributes responsibility to the object's lifetime by incrementing (`Py_INCREF()`) and decrementing (`Py_DECREF()`) the reference count of the object on the Python heap.

**Borrowed reference**
:   A pointer to data on the Python heap. The holder does **not** increment/decrement the refcount and must not outlive the actual owner (risking a dangling pointer if the object is destroyed).

## PyO3

[PyO3](https://pyo3.rs/) is a Rust library that provides deep integration with Python's C-API. This allows the creation of Python extension modules from Rust source code.

PyO3 provides:

- **Low-level bindings**: One-to-one bindings to Python's C-API via Rust's FFI functionality
- **High-level abstractions**: Safe abstractions that bridge the Rust and Python memory models, helping avoid common pitfalls

### The `FromPyObject` Trait

Understanding [PyO3's `FromPyObject` trait](https://pyo3.rs/v0.27.2/conversions/traits#extract-and-the-frompyobject-trait) is essential for working with this codebase.

All types exposed to Python (primitives and custom) implement the `FromPyObject` trait. This trait provides the `extract()` method which is used to bridge Python heap data to Rust.

Key points:

- `FromPyObject` is implemented out of the box for primitive types
- Automatically implemented for structs/enums decorated with `#[pyclass]` attribute
- The standard implementation performs a **clone** under the hood, requiring `Clone` trait
- Results in cloning data from the Python heap into Rust-managed memory
- Can be manually implemented to extend functionality and/or change default behavior

### Custom Type Function Parameters

Custom types are structs or enums decorated with the `#[pyclass]` attribute.

PyO3 offers several methods for specifying parameters of custom types in Python-exposed functions. Each method has its own underlying mechanics. The following covers the approaches used in this project.

#### `T` (Owned Rust Value)

**Use when:** Rust ownership of function args is needed.

```rust
fn process(value: MyType) -> PyResult<()>
```

- An owned Rust value
- Requires `Clone` trait on the `#[pyclass]` type
- Clones data from the Python heap into Rust-managed memory
- Standard Rust ownership applies from there
- The original object on the Python heap remains unchanged
- The CPython managed reference count remains unchanged

#### `Py<T>` (GIL-Independent Reference)

**Use when:** Storing Python objects (references) long-term in Rust.

```rust
fn store(value: Py<MyType>) -> PyResult<()>
```

- A GIL-independent Rust-owned reference to a value on the Python heap
- Not bound to Python GIL, but can be bound when Python interpreter is needed
- A Rust-owned reference to data on the Python heap
- The CPython managed reference count increments on creation and decrements on drop

#### `Bound<'py, T>` (GIL-Bound Owned Reference)

**Use when:** Rust ownership (of a reference) with GIL is required (e.g., creating new Python objects from Rust).

```rust
fn create(py: Python<'py>) -> PyResult<Bound<'py, MyType>>
```

- A GIL-bound reference to a value on the Python heap
- A Rust-owned reference to data on the Python heap
- The CPython managed reference count increments on creation and decrements on drop

#### `&Bound<'py, T>` (Borrowed Reference)

**Use when:** Reading or mutating without taking ownership.

```rust
fn read(value: &Bound<'py, MyType>) -> PyResult<()>
```

- A GIL-bound borrowed reference to a value on the Python heap
- "Borrowed reference" is a CPython concept: a pointer to a `PyObject*` on the Python heap, without incrementing/decrementing refcount
- CPython guarantees the pointer is valid as long as some owning reference keeps the object alive
- The CPython managed reference count remains unchanged

### Parameter Types Reference Table

| Type | Category | Lifetime / GIL Binding | Ownership / Reference Counting | Access to Embedded Rust Data | Common Use Cases | Performance Notes |
|------|----------|------------------------|--------------------------------|------------------------------|------------------|-------------------|
| `T` | Owned Rust value | No lifetime (static-like). Completely outside of GIL | Owns a **cloned** copy in Rust memory | Direct access (`obj.field`) | Function args when you want an owned copy (e.g., to store or move). Requires `T: Clone`. | Involves cloning the Rust data; avoid for large structs. |
| `Py<T>` | GIL-independent owned reference | No `'py` lifetime | Owns a strong Python reference (refcount +1) | Via `.bind(py).borrow()` or similar (need GIL) | Storing Python objects long-term (e.g., in `#[pyclass]` fields, across GIL releases, or threads). | GIL-independent; safe to hold without GIL, but most operations require reacquiring GIL. Convert from `Bound` via `.unbind()`. |
| `Bound<'py, T>` | GIL-bound owned reference | Tied to `'py` (proof GIL is held) | Owns a strong Python reference (refcount +1) | Via `.borrow()` / `.borrow_mut()` | Temporary ownership within a GIL-held scope (e.g., returning new objects, intermediate values). | Preferred for most operations; efficient Python API calls. |
| `&Bound<'py, T>` | Borrowed reference | Tied to `'py` (proof GIL is held) | Borrows (no refcount change) | Via `.borrow()` (immutable) or `.borrow_mut()` (mutable, with runtime checks) | Function arguments (most common); reading/mutating without taking ownership. | Zero-cost borrow; idiomatic for args. |

## Limitations & Challenges

Several limitations require workarounds or reimplementation in this repository:

### Private Code in rusty-kaspa

Private structs, struct fields, methods, etc. in rusty-kaspa cannot be leveraged by this repository. This results in new/re-implementation when such code is needed.

### Enum Exposure

Rust enums from rusty-kaspa cannot be exposed directly to Python. A Python interface-compatible enum must be defined. To reduce boilerplate, macros are provided:

- `wrap_unit_enum_for_py!` - For enums without associated data
- `wrap_c_enum_for_py!` - For enums with explicit discriminant values

### Error Propagation

Rust errors from rusty-kaspa cannot be propagated directly. Currently, `map_err()` is used extensively to convert to a generic Python exception:

```rust
pub fn constructor(address: &str) -> PyResult<PyAddress> {
    Ok(PyAddress(address.try_into().map_err(
        |err: AddressError| PyException::new_err(err.to_string()),
    )?))
}
```

!!! note "Future Work"
    Explicit custom exception types are planned to improve error handling.

### WASM Feature Dependency

This repository relies on rusty-kaspa features gated behind WASM feature flags. The WASM features are a required dependency.

### Future Improvements

Long term, to alleviate some challenges, there may be an opportunity to add "bindings primitives" to rusty-kaspa native. This would reduce the need for workarounds in this repository.

## Additional Resources

- [PyO3 User Guide](https://pyo3.rs/)
- [PyO3 API Documentation](https://docs.rs/pyo3/latest/pyo3/)
- [Python C-API Reference](https://docs.python.org/3/c-api/index.html)
- [Rust FFI Guide](https://doc.rust-lang.org/nomicon/ffi.html)

