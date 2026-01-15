use crate::wallet::bip32::language::PyLanguage;
use kaspa_bip32::{Error, Language, Mnemonic};
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::*;
use workflow_core::hex::ToHex;

/// A BIP-39 mnemonic seed phrase.
///
/// Mnemonic phrases (also called seed phrases or recovery phrases) are
/// human-readable representations of cryptographic seeds used for HD wallet
/// generation.
#[gen_stub_pyclass]
#[pyclass(name = "Mnemonic")]
pub struct PyMnemonic(Mnemonic);

#[gen_stub_pymethods]
#[pymethods]
impl PyMnemonic {
    /// Create a mnemonic from an existing phrase.
    ///
    /// Args:
    ///     phrase: The mnemonic phrase string.
    ///     language: Optional language for the phrase (default: English).
    ///
    /// Returns:
    ///     Mnemonic: A new Mnemonic instance.
    ///
    /// Raises:
    ///     Exception: If the phrase is invalid.
    #[new]
    #[pyo3(signature = (phrase, language=None))]
    pub fn constructor(
        phrase: &str,
        #[gen_stub(override_type(type_repr = "str | Language = Language.English"))]
        language: Option<PyLanguage>,
    ) -> PyResult<Self> {
        let inner = Mnemonic::new(
            phrase,
            language.map(Language::from).unwrap_or(Language::English),
        )
        .map_err(|err| PyException::new_err(err.to_string()))?;

        Ok(Self(inner))
    }

    /// Validate a mnemonic phrase.
    ///
    /// Args:
    ///     phrase: The mnemonic phrase to validate.
    ///     language: Optional language for validation (default: English).
    ///
    /// Returns:
    ///     bool: True if the phrase is valid, False otherwise.
    #[staticmethod]
    #[pyo3(signature = (phrase, language=None))]
    pub fn validate(
        phrase: &str,
        #[gen_stub(override_type(type_repr = "str | Language = Language.English"))]
        language: Option<PyLanguage>,
    ) -> bool {
        Mnemonic::new(
            phrase,
            language.map(Language::from).unwrap_or(Language::English),
        )
        .is_ok()
    }

    /// The entropy bytes as a hex string.
    ///
    /// Returns:
    ///     str: The raw entropy in hexadecimal.
    #[getter]
    pub fn get_entropy(&self) -> String {
        self.0.get_entropy()
    }

    /// Set the entropy directly.
    ///
    /// Args:
    ///     value: The entropy as a hex string (16 or 32 bytes).
    #[setter]
    pub fn set_entropy(&mut self, value: &str) {
        // let vec = Vec::<u8>::from_hex(entropy)
        //     .unwrap_or_else(|err| panic!("invalid entropy `{entropy}`: {err}"));
        // let len = vec.len();
        // if len != 16 && len != 32 {
        //     panic!("Invalid entropy: `{entropy}`")
        // }
        self.0.set_entropy(value.to_string());
        // self.entropy = vec;
    }

    /// Generate a random mnemonic phrase.
    ///
    /// Args:
    ///     word_count: Number of words (12, 15, 18, 21, or 24). Default: 24.
    ///
    /// Returns:
    ///     Mnemonic: A new random mnemonic.
    ///
    /// Raises:
    ///     Exception: If the word count is invalid.
    #[staticmethod]
    #[pyo3(name = "random")]
    #[pyo3(signature = (word_count=None))]
    pub fn create_random(word_count: Option<u32>) -> PyResult<Self> {
        let word_count = word_count.unwrap_or(24) as usize;
        let inner = Mnemonic::random(
            word_count
                .try_into()
                .map_err(|err: Error| PyException::new_err(err.to_string()))?,
            Default::default(),
        )
        .map_err(|err: Error| PyException::new_err(err.to_string()))?;
        Ok(Self(inner))
    }

    /// The mnemonic phrase as a string.
    ///
    /// Returns:
    ///     str: The space-separated word phrase.
    #[getter]
    pub fn get_phrase(&self) -> String {
        self.0.phrase().to_string()
        // self.phrase.clone()
    }

    /// Set the mnemonic phrase.
    ///
    /// Args:
    ///     value: The mnemonic phrase string.
    #[setter]
    pub fn set_phrase(&mut self, value: String) {
        self.0.set_phrase(&value);
    }

    /// Convert the mnemonic to a seed for key derivation.
    ///
    /// Args:
    ///     password: Optional passphrase for additional security.
    ///
    /// Returns:
    ///     str: The seed as a hex string.
    ///
    /// Note:
    ///     The same mnemonic with different passwords produces
    ///     completely different seeds (and thus different wallets).
    #[pyo3(name = "to_seed")]
    #[pyo3(signature = (password=None))]
    pub fn create_seed(&self, password: Option<&str>) -> String {
        let password = password.unwrap_or_default();
        self.0.to_seed(password).as_bytes().to_vec().to_hex()
    }
}
