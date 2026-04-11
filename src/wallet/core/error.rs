use crate::error::IntoPyResult;
use kaspa_wallet_core::error::Error as NativeError;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

pub struct Error(NativeError);

impl From<NativeError> for Error {
    fn from(err: NativeError) -> Self {
        Error(err)
    }
}

impl<T> IntoPyResult<T> for std::result::Result<T, NativeError> {
    fn into_py_result(self) -> PyResult<T> {
        self.map_err(|e| PyErr::from(Error(e)))
    }
}

crate::py_error_map! {
NativeError::Custom(_) => PyWalletCustomError, "WalletCustomError";
NativeError::WalletKeys(_) => PyWalletKeysError, "WalletKeysError";
NativeError::AccountSelection => PyWalletAccountSelectionError, "WalletAccountSelectionError";
NativeError::KaspaRpcClientResult(_) => PyWalletKaspaRpcClientResultError, "WalletKaspaRpcClientResultError";
NativeError::RpcError(_) => PyWalletRpcError, "WalletRpcError";
NativeError::KaspaWorkflowRpcError(_) => PyWalletKaspaWorkflowRpcError, "WalletKaspaWorkflowRpcError";
NativeError::NotWrpcClient => PyWalletNotWrpcClientError, "WalletNotWrpcClientError";
NativeError::BIP32Error(_) => PyWalletBIP32Error, "WalletBIP32Error";
NativeError::Decode(_) => PyWalletDecodeError, "WalletDecodeError";
NativeError::PoisonError(_) => PyWalletPoisonError, "WalletPoisonError";
NativeError::Secp256k1Error(_) => PyWalletSecp256k1Error, "WalletSecp256k1Error";
NativeError::CoreSignError(_) => PyWalletCoreSignError, "WalletCoreSignError";
NativeError::SerdeJson(_) => PyWalletSerdeJsonError, "WalletSerdeJsonError";
NativeError::NoWalletInStorage(_) => PyWalletNoWalletInStorageError, "WalletNoWalletInStorageError";
NativeError::WalletAlreadyExists=> PyWalletAlreadyExistsError, "WalletAlreadyExistsError";
NativeError::WalletNameNotAllowed => PyWalletNameNotAllowedError, "WalletNameNotAllowedError";
NativeError::WalletNotOpen=> PyWalletNotOpenError, "WalletNotOpenError";
NativeError::NotConnected => PyWalletNotConnectedError, "WalletNotConnectedError";
NativeError::MissingNetworkId => PyWalletMissingNetworkIdError, "WalletMissingNetworkIdError";
NativeError::RpcApiVersion(_, _)=> PyWalletRpcApiVersionError, "WalletRpcApiVersionError";
NativeError::InvalidNetworkId(_)=> PyWalletInvalidNetworkIdError, "WalletInvalidNetworkIdError";
NativeError::InvalidNetworkType(_, _) => PyWalletInvalidNetworkTypeError, "WalletInvalidNetworkTypeError";
NativeError::InvalidNetworkSuffix(_)=> PyWalletInvalidNetworkSuffixError, "WalletInvalidNetworkSuffixError";
NativeError::MissingNetworkSuffix(_)=> PyWalletMissingNetworkSuffixError, "WalletMissingNetworkSuffixError";
NativeError::UnexpectedExtraSuffixToken(_)=> PyWalletUnexpectedExtraSuffixTokenError, "WalletUnexpectedExtraSuffixTokenError";
NativeError::NetworkTypeConnected => PyWalletNetworkTypeConnectedError, "WalletNetworkTypeConnectedError";
NativeError::NetworkType(_) => PyWalletNetworkTypeError, "WalletNetworkTypeError";
NativeError::NetworkId(_) => PyWalletNetworkIdError, "WalletNetworkIdError";
NativeError::MissingUtxoIndex => PyWalletMissingUtxoIndexError, "WalletMissingUtxoIndexError";
NativeError::InvalidFilename(_) => PyWalletInvalidFilenameError, "WalletInvalidFilenameError";
NativeError::Io(_)=> PyWalletIoError, "WalletIoError";
NativeError::JsValue(_) => PyWalletJsValueError, "WalletJsValueError";
NativeError::DecodeError(_) => PyWalletBase64DecodeError, "WalletBase64DecodeError";
NativeError::WorkflowWasm(_)=> PyWalletWorkflowWasmError, "WalletWorkflowWasmError";
NativeError::WorkflowStore(_) => PyWalletWorkflowStoreError, "WalletWorkflowStoreError";
NativeError::Address(_) => PyWalletAddressError, "WalletAddressError";
NativeError::SerdeWasmBindgen(_)=> PyWalletSerdeWasmBindgenError, "WalletSerdeWasmBindgenError";
NativeError::FasterHexError(_)=> PyWalletFasterHexError, "WalletFasterHexError";
NativeError::ParseFloatError(_) => PyWalletParseFloatError, "WalletParseFloatError";
NativeError::Chacha20poly1305(_)=> PyWalletChacha20poly1305Error, "WalletChacha20poly1305Error";
NativeError::WalletDecrypt(_) => PyWalletDecryptError, "WalletDecryptError";
NativeError::FromUtf8Error(_) => PyWalletFromUtf8Error, "WalletFromUtf8Error";
NativeError::ScriptBuilderError(_)=> PyWalletScriptBuilderError, "WalletScriptBuilderError";
NativeError::Argon2(_)=> PyWalletArgon2Error, "WalletArgon2Error";
NativeError::Argon2ph(_)=> PyWalletArgon2phError, "WalletArgon2phError";
NativeError::VarError(_)=> PyWalletVarError, "WalletVarError";
NativeError::PrivateKeyNotFound(_)=> PyWalletPrivateKeyNotFoundError, "WalletPrivateKeyNotFoundError";
NativeError::PrivateKeyAlreadyExists(_) => PyWalletPrivateKeyAlreadyExistsError, "WalletPrivateKeyAlreadyExistsError";
NativeError::AccountAlreadyExists(_)=> PyWalletAccountAlreadyExistsError, "WalletAccountAlreadyExistsError";
NativeError::XPrvSupport=> PyWalletXPrvSupportError, "WalletXPrvSupportError";
NativeError::KeyId(_) => PyWalletKeyIdError, "WalletKeyIdError";
NativeError::WalletSecretRequired => PyWalletSecretRequiredError, "WalletSecretRequiredError";
NativeError::SecretIsEmpty(_) => PyWalletSecretIsEmptyError, "WalletSecretIsEmptyError";
NativeError::Aborted=> PyWalletAbortedError, "WalletAbortedError";
NativeError::TryFromEnum(_) => PyWalletTryFromEnumError, "WalletTryFromEnumError";
NativeError::AccountFactoryNotFound(_)=> PyWalletAccountFactoryNotFoundError, "WalletAccountFactoryNotFoundError";
NativeError::AccountNotFound(_) => PyWalletAccountNotFoundError, "WalletAccountNotFoundError";
NativeError::AccountNotActive(_)=> PyWalletAccountNotActiveError, "WalletAccountNotActiveError";
NativeError::InvalidAccountId(_)=> PyWalletInvalidAccountIdError, "WalletInvalidAccountIdError";
NativeError::InvalidKeyDataId(_)=> PyWalletInvalidKeyDataIdError, "WalletInvalidKeyDataIdError";
NativeError::InvalidAccountKind => PyWalletInvalidAccountKindError, "WalletInvalidAccountKindError";
NativeError::InsufficientFunds { .. } => PyWalletInsufficientFundsError, "WalletInsufficientFundsError";
NativeError::Utf8Error(_) => PyWalletUtf8Error, "WalletUtf8Error";
NativeError::ParseIntError(_) => PyWalletParseIntError, "WalletParseIntError";
NativeError::DuplicateUtxoEntry => PyWalletDuplicateUtxoEntryError, "WalletDuplicateUtxoEntryError";
NativeError::ToValue(_) => PyWalletToValueError, "WalletToValueError";
NativeError::NoRecordsFound => PyWalletNoRecordsFoundError, "WalletNoRecordsFoundError";
NativeError::NotImplemented => PyWalletNotImplementedError, "WalletNotImplementedError";
NativeError::ResidentWallet => PyWalletResidentWalletError, "WalletResidentWalletError";
NativeError::ResidentAccount=> PyWalletResidentAccountError, "WalletResidentAccountError";
NativeError::Bip32WatchAccount=> PyWalletBip32WatchAccountError, "WalletBip32WatchAccountError";
NativeError::Bip32WatchXpubRequired => PyWalletBip32WatchXpubRequiredError, "WalletBip32WatchXpubRequiredError";
NativeError::AccountKindFeature => PyWalletAccountKindFeatureError, "WalletAccountKindFeatureError";
NativeError::AccountAddressDerivationCaps => PyWalletAccountAddressDerivationCapsError, "WalletAccountAddressDerivationCapsError";
NativeError::DowncastError(_) => PyWalletDowncastError, "WalletDowncastError";
NativeError::ConsensusClient(_) => PyWalletConsensusClientError, "WalletConsensusClientError";
NativeError::ConsensusWasm(_) => PyWalletConsensusWasmError, "WalletConsensusWasmError";
NativeError::GeneratorFeesInSweepTransaction=> PyWalletGeneratorFeesInSweepTransactionError, "WalletGeneratorFeesInSweepTransactionError";
NativeError::GeneratorNoFeesForFinalTransaction => PyWalletGeneratorNoFeesForFinalTransactionError, "WalletGeneratorNoFeesForFinalTransactionError";
NativeError::GeneratorChangeAddressNetworkTypeMismatch=> PyWalletGeneratorChangeAddressNetworkTypeMismatchError, "WalletGeneratorChangeAddressNetworkTypeMismatchError";
NativeError::GeneratorPaymentOutputNetworkTypeMismatch=> PyWalletGeneratorPaymentOutputNetworkTypeMismatchError, "WalletGeneratorPaymentOutputNetworkTypeMismatchError";
NativeError::GeneratorPaymentOutputZeroAmount => PyWalletGeneratorPaymentOutputZeroAmountError, "WalletGeneratorPaymentOutputZeroAmountError";
NativeError::GeneratorIncludeFeesRequiresOneOutput=> PyWalletGeneratorIncludeFeesRequiresOneOutputError, "WalletGeneratorIncludeFeesRequiresOneOutputError";
NativeError::GeneratorTransactionOutputsAreTooHeavy { .. }=> PyWalletGeneratorTransactionOutputsAreTooHeavyError, "WalletGeneratorTransactionOutputsAreTooHeavyError";
NativeError::GeneratorTransactionIsTooHeavy => PyWalletGeneratorTransactionIsTooHeavyError, "WalletGeneratorTransactionIsTooHeavyError";
NativeError::StorageMassExceedsMaximumTransactionMass { .. }=> PyWalletStorageMassExceedsMaximumTransactionMassError, "WalletStorageMassExceedsMaximumTransactionMassError";
NativeError::InvalidRange(_, _) => PyWalletInvalidRangeError, "WalletInvalidRangeError";
NativeError::MultisigCreateError(_) => PyWalletMultisigCreateError, "WalletMultisigCreateError";
NativeError::TxScriptError(_) => PyWalletTxScriptError, "WalletTxScriptError";
NativeError::LegacyAccountNotInitialized=> PyWalletLegacyAccountNotInitializedError, "WalletLegacyAccountNotInitializedError";
NativeError::AssocPrvKeyDataIds(_, _) => PyWalletAssocPrvKeyDataIdsError, "WalletAssocPrvKeyDataIdsError";
NativeError::AssocPrvKeyDataIdsEmpty=> PyWalletAssocPrvKeyDataIdsEmptyError, "WalletAssocPrvKeyDataIdsEmptyError";
NativeError::InvalidExtendedPublicKey(_, _) => PyWalletInvalidExtendedPublicKeyError, "WalletInvalidExtendedPublicKeyError";
NativeError::MissingDaaScore(_) => PyWalletMissingDaaScoreError, "WalletMissingDaaScoreError";
NativeError::ListenerId => PyWalletListenerIdError, "WalletListenerIdError";
NativeError::MassCalculationError => PyWalletMassCalculationError, "WalletMassCalculationError";
NativeError::TransactionFeesAreTooHigh=> PyWalletTransactionFeesAreTooHighError, "WalletTransactionFeesAreTooHighError";
NativeError::InvalidArgument(_) => PyWalletInvalidArgumentError, "WalletInvalidArgumentError";
NativeError::BigInt(_)=> PyWalletBigIntError, "WalletBigIntError";
NativeError::InvalidMnemonicPhrase=> PyWalletInvalidMnemonicPhraseError, "WalletInvalidMnemonicPhraseError";
NativeError::InvalidTransactionKind(_)=> PyWalletInvalidTransactionKindError, "WalletInvalidTransactionKindError";
NativeError::CipherMessageTooShort=> PyWalletCipherMessageTooShortError, "WalletCipherMessageTooShortError";
NativeError::InvalidPrivateKeyLength=> PyWalletInvalidPrivateKeyLengthError, "WalletInvalidPrivateKeyLengthError";
NativeError::InvalidPublicKeyLength => PyWalletInvalidPublicKeyLengthError, "WalletInvalidPublicKeyLengthError";
NativeError::Metrics(_) => PyWalletMetricsError, "WalletMetricsError";
NativeError::NotSynced=> PyWalletNotSyncedError, "WalletNotSyncedError";
NativeError::Pskt(_)=> PyWalletPsktError, "WalletPsktError";
NativeError::PendingTransactionFromPSKTError(_) => PyWalletPendingTransactionFromPSKTError, "WalletPendingTransactionFromPSKTError";
NativeError::AddressNotFound=> PyWalletAddressNotFoundError, "WalletAddressNotFoundError";
NativeError::CommitRevealInvalidPaymentDestination=> PyWalletCommitRevealInvalidPaymentDestinationError, "WalletCommitRevealInvalidPaymentDestinationError";
NativeError::CommitRevealEmptyPaymentOutputs=> PyWalletCommitRevealEmptyPaymentOutputsError, "WalletCommitRevealEmptyPaymentOutputsError";
NativeError::RevealRedeemScriptTemplateError=> PyWalletRevealRedeemScriptTemplateError, "WalletRevealRedeemScriptTemplateError";
NativeError::PSKTGenerationError(_) => PyWalletPSKTGenerationError, "WalletPSKTGenerationError";
NativeError::CommitTransactionSigningError=> PyWalletCommitTransactionSigningError, "WalletCommitTransactionSigningError";
NativeError::PSKTFinalizationError=> PyWalletPSKTFinalizationError, "WalletPSKTFinalizationError";
NativeError::CommitTransactionIdExtractionError => PyWalletCommitTransactionIdExtractionError, "WalletCommitTransactionIdExtractionError";
NativeError::NoQualifiedRevealSignerFound => PyWalletNoQualifiedRevealSignerFoundError, "WalletNoQualifiedRevealSignerFoundError";
NativeError::CommitRevealBundleMergeError => PyWalletCommitRevealBundleMergeError, "WalletCommitRevealBundleMergeError";
}
