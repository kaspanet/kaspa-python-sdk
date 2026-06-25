//! Generates `python/kaspa/experimental/silverscript/__init__.pyi`.
//!
//! Generates into a throwaway dir (set in `pyproject.toml`) so it
//! can't clobber the core crate's stub, then copies the result into the package.
use std::fs;
use std::path::Path;

use pyo3_stub_gen::Result;

const CRATE_DIR: &str = "crates/silverscript";
const DEST: &str = "python/kaspa/experimental/silverscript/__init__.pyi";

fn main() -> Result<()> {
    let stub = silverscript::stub_info()?;
    stub.generate()?;

    let stubgen_root = format!("{CRATE_DIR}/_stubgen");
    let candidates = [
        format!("{stubgen_root}/kaspa/experimental/silverscript.pyi"),
        format!("{stubgen_root}/kaspa/experimental/silverscript/__init__.pyi"),
    ];
    let generated = candidates
        .iter()
        .find(|p| Path::new(p).exists())
        .unwrap_or_else(|| panic!("stub not generated; looked in {candidates:?}"));

    fs::create_dir_all("python/kaspa/experimental/silverscript")?;
    let content = strip_py_prefix_from_exceptions(fs::read_to_string(generated)?);
    fs::write(DEST, content)?;
    fs::remove_dir_all(&stubgen_root).ok();

    println!("wrote {DEST}");
    Ok(())
}

/// Strips the `Py` prefix from exception class names in the stub: pyo3-stub-gen
/// emits the Rust ident (e.g. `PySilverScriptError`) instead of the
/// `#[pyclass(name = "...")]` value.
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
