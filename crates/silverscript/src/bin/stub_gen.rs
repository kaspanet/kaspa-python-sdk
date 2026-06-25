//! Generates `python/kaspa/experimental/silverscript/__init__.pyi`.
//!
//! Generates into a throwaway dir (set in `pyproject.toml`) so it
//! can't clobber the core crate's stub, then copies the result into the package.
use std::fs;
use std::path::Path;

use kaspa_python_sdk_core::strip_py_prefix;
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
    let content = strip_py_prefix(fs::read_to_string(generated)?, "(builtins.Exception)");
    fs::write(DEST, content)?;
    fs::remove_dir_all(&stubgen_root).ok();

    println!("wrote {DEST}");
    Ok(())
}
