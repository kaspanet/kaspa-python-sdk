//! Generates `python/kaspa/silverscript/__init__.pyi`.
//!
//! pyo3-stub-gen writes one `.pyi` per module under the `python-source` root
//! from `pyproject.toml`. We point that root at a throwaway `_stubgen/` dir so
//! generation can never touch the core crate's `python/kaspa/__init__.pyi`,
//! then copy just the SilverScript stub into the package as `__init__.pyi`.
//!
//! Run from the repository root (e.g. `cargo run -p kaspa-silverscript
//! --bin stub-gen --no-default-features`).

use std::fs;
use std::path::Path;

use pyo3_stub_gen::Result;

const CRATE_DIR: &str = "crates/kaspa-silverscript";
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
    let mut content = fs::read_to_string(generated)?;
    // `SilverScriptError` is declared via `create_exception!`, which pyo3-stub-gen
    // doesn't capture — append it so it appears in the reference + type checking.
    if !content.contains("class SilverScriptError") {
        content.push_str(
            "\n@typing.final\nclass SilverScriptError(builtins.Exception):\n    \
             r\"\"\"Raised when SilverScript compilation or signature-script construction fails.\"\"\"\n    ...\n",
        );
    }
    fs::write(DEST, content)?;
    fs::remove_dir_all(&stubgen_root).ok();

    println!("wrote {DEST}");
    Ok(())
}
