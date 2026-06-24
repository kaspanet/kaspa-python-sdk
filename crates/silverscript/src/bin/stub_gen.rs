//! Generates `python/kaspa/silverscript/__init__.pyi`.
//!
//! pyo3-stub-gen writes one `.pyi` per module under the `python-source` root
//! from `pyproject.toml`. We point that root at a throwaway `_stubgen/` dir so
//! generation can never touch the core crate's `python/kaspa/__init__.pyi`,
//! then copy just the SilverScript stub into the package as `__init__.pyi`.
//!
//! Run from the repository root (e.g. `cargo run -p kaspa-python-sdk-silverscript
//! --bin stub-gen --no-default-features`).

use std::fs;
use std::path::Path;

use pyo3_stub_gen::Result;

const CRATE_DIR: &str = "crates/silverscript";
const DEST: &str = "python/kaspa/silverscript/__init__.pyi";

fn main() -> Result<()> {
    let stub = silverscript::stub_info()?;
    stub.generate()?;

    // generate() writes `<crate>/_stubgen/kaspa/silverscript.pyi` (single
    // module) — fall back to the package form just in case.
    let stubgen_root = format!("{CRATE_DIR}/_stubgen");
    let candidates = [
        format!("{stubgen_root}/kaspa/silverscript.pyi"),
        format!("{stubgen_root}/kaspa/silverscript/__init__.pyi"),
    ];
    let generated = candidates
        .iter()
        .find(|p| Path::new(p).exists())
        .unwrap_or_else(|| panic!("stub not generated; looked in {candidates:?}"));

    fs::create_dir_all("python/kaspa/silverscript")?;
    // `SilverScriptError` is a `#[pyclass(extends = PyException)]` (via the shared
    // `create_py_exception!` macro), so pyo3-stub-gen captures it automatically —
    // but for exception classes it emits the Rust ident (`PySilverScriptError`)
    // rather than the `#[pyclass(name = "...")]` value. Strip the `Py` prefix so
    // the stub matches the Python name, mirroring the core crate's stub_gen.
    let content = strip_py_prefix_from_exceptions(fs::read_to_string(generated)?);
    fs::write(DEST, content)?;
    fs::remove_dir_all(&stubgen_root).ok();

    println!("wrote {DEST}");
    Ok(())
}

/// Removes the `Py` prefix from exception class names in the generated stub.
/// pyo3-stub-gen emits the Rust ident for `extends = PyException` classes
/// (e.g. `class PySilverScriptError(builtins.Exception)`) rather than the
/// `#[pyclass(name = "...")]` value, so we rewrite them to the Python name.
/// Mirrors `strip_py_prefix_from_exceptions` in the core crate's `src/bin/stub_gen.rs`.
fn strip_py_prefix_from_exceptions(content: String) -> String {
    let mut exception_names: Vec<String> = Vec::new();

    for line in content.lines() {
        if let Some(start) = line.find("class Py")
            && line.contains("(builtins.Exception)")
        {
            let after_class = &line[start + 6..];
            if let Some(paren_pos) = after_class.find('(') {
                let class_name = &after_class[..paren_pos];
                if class_name.starts_with("Py") {
                    exception_names.push(class_name.to_string());
                }
            }
        }
    }

    let mut result = content;
    for py_name in &exception_names {
        if let Some(stripped) = py_name.strip_prefix("Py") {
            result = result.replace(py_name, stripped);
        }
    }

    result
}
