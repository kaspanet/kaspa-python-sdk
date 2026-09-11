//! `ContractArtifact` — a portable ABI artifact loaded from JSON.
//!
//! The compile-free half of the module: `load_artifact` reads the same JSON
//! `CompiledContract.artifact_json` writes (and upstream `silverc -c` emits),
//! giving back an object that derives addresses and builds unlocking scripts
//! with no source and no compiler.

use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyfunction, gen_stub_pymethods};

use silverscript_abi::{SilAbiArtifact, SilContractArtifact, to_pretty_json};

use crate::{
    PyEntryAbi, PySilverScriptError, abi_entries, collect_args, entry_abi, sig_script, to_json_err,
    unknown_entry,
};

/// A portable ABI artifact: everything needed to build unlocking scripts for a
/// contract, without its source.
///
/// Obtained from `load_artifact`, never constructed directly.
#[gen_stub_pyclass]
#[pyclass(
    name = "ContractArtifact",
    module = "kaspa.experimental.silverscript",
    frozen
)]
pub struct PyContractArtifact {
    // The whole artifact, not just this contract's slice: the codec resolves
    // struct types through `structs` and addresses contracts by name.
    artifact: SilAbiArtifact,
    contract_name: String,
    abi: Vec<PyEntryAbi>,
}

impl PyContractArtifact {
    /// This artifact's selected contract.
    ///
    /// `load_artifact` resolves `contract_name` against `contracts` before
    /// constructing, and the class is frozen, so it cannot be absent.
    fn contract(&self) -> &SilContractArtifact {
        self.artifact
            .contract(&self.contract_name)
            .expect("contract_name was resolved against the artifact at load")
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyContractArtifact {
    /// The contract name this artifact was loaded for.
    #[getter]
    pub fn contract_name(&self) -> &str {
        &self.contract_name
    }

    /// The compiler version that produced the artifact.
    #[getter]
    pub fn compiler_version(&self) -> &str {
        &self.artifact.compiler_version
    }

    /// The artifact schema version. Checked by `load_artifact`.
    #[getter]
    pub fn schema_version(&self) -> u32 {
        self.artifact.schema_version
    }

    /// The compiled locking script (redeem script) bytes.
    #[getter]
    pub fn bytecode<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.contract().compiled.bytecode)
    }

    /// The 32-byte template hash recorded in the artifact.
    ///
    /// Recorded, not recomputed — `check_consistency` is what verifies it
    /// against the bytecode.
    #[getter]
    pub fn template_hash<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.contract().compiled.template_hash)
    }

    /// `(offset, len)`: byte offset and length of the contract state within the
    /// script.
    ///
    /// The same two numbers, under the same name, as
    /// `CompiledContract.state_span`.
    #[getter]
    pub fn state_span(&self) -> (usize, usize) {
        let span = &self.contract().compiled.state_span;
        (span.offset, span.len)
    }

    /// The contract ABI: one entry per callable entrypoint, ordered
    /// alphabetically by name.
    ///
    /// The same order `CompiledContract.abi` reports. To reach one entry,
    /// prefer `entry(name)` over indexing.
    #[getter]
    pub fn abi(&self) -> Vec<PyEntryAbi> {
        self.abi.clone()
    }

    /// Look up one entrypoint's ABI by name.
    ///
    /// Args:
    ///     name: The entrypoint name, as it appears in `abi`.
    ///
    /// Returns:
    ///     EntryAbi: The entrypoint's ABI.
    ///
    /// Raises:
    ///     SilverScriptError: If the artifact declares no such entrypoint. The
    ///         message matches the one `build_sig_script` raises for the same
    ///         name.
    pub fn entry(&self, name: &str) -> PyResult<PyEntryAbi> {
        entry_abi(self.contract(), name).ok_or_else(|| unknown_entry(&self.contract_name, name))
    }

    /// Build the signature (unlocking) script for an entrypoint.
    ///
    /// Identical bytes to `CompiledContract.build_sig_script` for the same
    /// call: both encode against this same artifact.
    ///
    /// Args:
    ///     function_name: The entrypoint to call.
    ///     args: Native Python values (int, bool, str, bytes, list/tuple, or
    ///         dict) matching the entrypoint's ABI input types. Omit or pass
    ///         None for an entrypoint that takes no arguments.
    ///
    /// Returns:
    ///     bytes: The signature (unlocking) script.
    ///
    /// Raises:
    ///     SilverScriptError: If the entrypoint is unknown or an argument is
    ///         invalid (wrong type, out of range, or too deeply nested).
    #[pyo3(signature = (function_name, args=None))]
    pub fn build_sig_script<'py>(
        &self,
        py: Python<'py>,
        function_name: &str,
        args: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let args = collect_args(args.as_ref())?;
        let bytes = sig_script(
            &self.artifact,
            &self.contract_name,
            function_name,
            &args,
            None,
        )?;
        Ok(PyBytes::new(py, &bytes))
    }

    /// Build the signature (unlocking) script for a covenant declaration entrypoint.
    ///
    /// Args:
    ///     function_name: The covenant entrypoint to call.
    ///     args: Native Python values matching the entrypoint's ABI input
    ///         types. Omit or pass None for an entrypoint that takes no
    ///         arguments.
    ///     is_leader: Select the leader path for covenants that distinguish a
    ///         leader from delegates (default: False).
    ///
    /// Returns:
    ///     bytes: The signature (unlocking) script.
    ///
    /// Raises:
    ///     SilverScriptError: If the entrypoint is unknown or an argument is
    ///         invalid (wrong type, out of range, or too deeply nested).
    #[pyo3(signature = (function_name, args=None, *, is_leader=false))]
    pub fn build_sig_script_for_covenant_decl<'py>(
        &self,
        py: Python<'py>,
        function_name: &str,
        args: Option<Bound<'py, PyAny>>,
        is_leader: bool,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let args = collect_args(args.as_ref())?;
        let bytes = sig_script(
            &self.artifact,
            &self.contract_name,
            function_name,
            &args,
            Some(is_leader),
        )?;
        Ok(PyBytes::new(py, &bytes))
    }

    /// Verify the artifact's internal consistency, raising if it fails.
    ///
    /// Checks the recorded template hash against the bytecode, the state span
    /// against the script, and the entries' dispatch tags for collisions —
    /// across every contract in the loaded artifact, not only the selected one.
    ///
    /// Raises:
    ///     SilverScriptError: If the artifact is inconsistent.
    ///
    /// Note:
    ///     This detects a corrupted or edited artifact. It does not prove the
    ///     bytecode was compiled from any particular source, so it does not
    ///     make an untrusted artifact trustworthy: use artifacts from a
    ///     trusted build, or compare `template_hash` against a known value.
    pub fn check_consistency(&self) -> PyResult<()> {
        self.artifact
            .check_consistency()
            .map_err(|err| PySilverScriptError::new_err(err.to_string()))
    }

    /// Serialize the artifact back to JSON.
    ///
    /// Byte-for-byte what it was loaded from, so an artifact survives a
    /// load/serialize round trip unchanged.
    ///
    /// Returns:
    ///     str: The portable artifact as pretty-printed JSON.
    ///
    /// Raises:
    ///     SilverScriptError: If the artifact cannot be serialized.
    pub fn to_json(&self) -> PyResult<String> {
        to_pretty_json(&self.artifact).map_err(to_json_err)
    }

    /// The detailed string representation.
    ///
    /// Returns:
    ///     str: The ContractArtifact as a repr string.
    pub fn __repr__(&self) -> String {
        format!(
            "ContractArtifact(name={:?}, bytecode={} bytes, entries={})",
            self.contract_name,
            self.contract().compiled.bytecode.len(),
            self.abi.len()
        )
    }
}

/// Pick the contract an artifact is being loaded for.
///
/// A named contract must exist; an unnamed one is only unambiguous when the
/// artifact holds exactly one. Both failures name what is available.
fn resolve_contract_name(
    artifact: &SilAbiArtifact,
    contract_name: Option<&str>,
) -> PyResult<String> {
    let available = || {
        artifact
            .contracts
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(", ")
    };
    match contract_name {
        Some(name) if artifact.contracts.contains_key(name) => Ok(name.to_string()),
        Some(name) => Err(PySilverScriptError::new_err(format!(
            "artifact has no contract named {name:?} (contains: {})",
            available()
        ))),
        None => match artifact.contracts.len() {
            1 => Ok(artifact.contracts.keys().next().expect("len == 1").clone()),
            0 => Err(PySilverScriptError::new_err(
                "artifact contains no contracts",
            )),
            _ => Err(PySilverScriptError::new_err(format!(
                "artifact contains several contracts ({}); pass contract_name to select one",
                available()
            ))),
        },
    }
}

/// Load a portable ABI artifact from JSON.
///
/// Takes what `CompiledContract.artifact_json` returns, or what upstream
/// `silverc -c` writes. Compile once and ship the artifact; derive addresses
/// and build unlocking scripts from it at runtime with no source and no
/// compiler.
///
/// `debug_call` and compiling with different constructor arguments need the
/// source and are not available from an artifact — an artifact describes one
/// already-compiled contract.
///
/// Args:
///     json: The artifact JSON.
///     contract_name: Which contract to select. Omit for an artifact that
///         holds exactly one (what `compile` produces).
///
/// Returns:
///     ContractArtifact: The loaded artifact.
///
/// Raises:
///     SilverScriptError: If the JSON is malformed, its schema version is
///         unsupported, or `contract_name` is absent or ambiguous.
///
/// Note:
///     Experimental. SilverScript and these bindings are under active
///     development; the API and the artifact schema may change in breaking
///     ways between releases.
#[gen_stub_pyfunction(module = "kaspa.experimental.silverscript")]
#[pyfunction]
#[pyo3(signature = (json, contract_name=None))]
pub fn load_artifact(json: &str, contract_name: Option<&str>) -> PyResult<PyContractArtifact> {
    let artifact: SilAbiArtifact = serde_json::from_str(json)
        .map_err(|err| PySilverScriptError::new_err(format!("invalid artifact JSON: {err}")))?;
    artifact
        .check_schema_version()
        .map_err(|err| PySilverScriptError::new_err(err.to_string()))?;

    let contract_name = resolve_contract_name(&artifact, contract_name)?;
    let abi = abi_entries(
        artifact
            .contract(&contract_name)
            .expect("contract_name was just resolved"),
    );
    Ok(PyContractArtifact {
        artifact,
        contract_name,
        abi,
    })
}
