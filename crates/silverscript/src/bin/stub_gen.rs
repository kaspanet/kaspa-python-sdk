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
    // `SilverScriptError` is now a `#[pyclass(extends = PyException)]` (via the
    // shared `create_py_exception!` macro), so pyo3-stub-gen captures it into the
    // generated stub automatically — no manual append.
    let content = fs::read_to_string(generated)?;
    fs::write(DEST, content)?;
    fs::remove_dir_all(&stubgen_root).ok();

    println!("wrote {DEST}");
    Ok(())
}
