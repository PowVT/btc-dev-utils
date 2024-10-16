use std::{error::Error, fmt};

use bitcoincore_rpc::Error as RpcError;
use bitcoin::consensus::encode::Error as EncodeError;

/// Bitcoind Errors

#[derive(Debug)]
pub enum BitcoindError {
    RpcError(RpcError),
    ClientError(ClientError),
    InvalidTxId,
    InsufficientConfirmations,
    TxOutNotFound,
    IncompletePsbt,
    NoHexInFinalizedPsbt,
    Other(String),
}

impl fmt::Display for BitcoindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BitcoindError::RpcError(e) => write!(f, "RPC error: {}", e),
            BitcoindError::ClientError(e) => write!(f, "Client error: {}", e),
            BitcoindError::InvalidTxId => write!(f, "Invalid transaction ID"),
            BitcoindError::InsufficientConfirmations => write!(f, "Insufficient confirmations"),
            BitcoindError::TxOutNotFound => write!(f, "TxOut not found"),
            BitcoindError::IncompletePsbt => write!(f, "PSBT is not complete"),
            BitcoindError::NoHexInFinalizedPsbt => write!(f, "No hex found in FinalizePsbtResult"),
            BitcoindError::Other(s) => write!(f, "Other error: {}", s),
        }
    }
}

impl Error for BitcoindError {}

impl From<RpcError> for BitcoindError {
    fn from(err: RpcError) -> Self {
        BitcoindError::RpcError(err)
    }
}

impl From<ClientError> for BitcoindError {
    fn from(err: ClientError) -> Self {
        BitcoindError::ClientError(err)
    }
}

/// Client Errors

#[derive(Debug)]
pub enum ClientError {
    CannotConnect(RpcError),
    UnsupportedNetwork,
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::CannotConnect(err) => write!(f, "Cannot connect to Bitcoin Core: {}", err),
            ClientError::UnsupportedNetwork => write!(f, "Unsupported network"),
        }
    }
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ClientError::CannotConnect(err) => Some(err),
            ClientError::UnsupportedNetwork => None,
        }
    }
}

impl From<RpcError> for ClientError {
    fn from(err: RpcError) -> Self {
        ClientError::CannotConnect(err)
    }
}

/// Verification Errors

#[derive(Debug)]
pub enum VerificationError {
    HexDecodeError(hex::FromHexError),
    DeserializationError(EncodeError),
    UTXOAlreadySpent(usize),
    UTXOCheckError(usize, String),
    TransactionVerificationFailed(String),
    UTXOError(String),
}

impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerificationError::HexDecodeError(e) => write!(f, "Failed to decode transaction hex: {}", e),
            VerificationError::DeserializationError(e) => write!(f, "Failed to deserialize transaction: {}", e),
            VerificationError::UTXOAlreadySpent(index) => write!(f, "UTXO for input {} has already been spent", index),
            VerificationError::UTXOCheckError(index, e) => write!(f, "Error checking UTXO for input {}: {}", index, e),
            VerificationError::TransactionVerificationFailed(e) => write!(f, "Transaction verification failed: {}", e),
            VerificationError::UTXOError(e) => write!(f, "Error checking UTXO: {}", e),
        }
    }
}

impl std::error::Error for VerificationError {}

impl From<hex::FromHexError> for VerificationError {
    fn from(err: hex::FromHexError) -> Self {
        VerificationError::HexDecodeError(err)
    }
}

impl From<bitcoin::consensus::encode::Error> for VerificationError {
    fn from(err: bitcoin::consensus::encode::Error) -> Self {
        VerificationError::DeserializationError(err)
    }
}

// Wallet Ops Errors

#[derive(Debug)]
pub enum WalletOpsError {
    WalletError(WalletError),
    ClientError(ClientError),
    RpcError(RpcError),
    BitcoindError(BitcoindError),
    UtilsError(UtilsError),
    InsufficientBalance,
    NoUnspentTransactions,
    NotMultisigWallet,
    DescriptorError(miniscript::Error),
    JsonError(serde_json::Error),
    Other(String),
}

impl fmt::Display for WalletOpsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WalletOpsError::WalletError(err) => write!(f, "Wallet error: {}", err),
            WalletOpsError::ClientError(err) => write!(f, "Client error: {}", err),
            WalletOpsError::RpcError(err) => write!(f, "RPC error: {}", err),
            WalletOpsError::BitcoindError(err) => write!(f, "Bitcoind error: {}", err),
            WalletOpsError::UtilsError(e) => write!(f, "Utils error: {}", e),
            WalletOpsError::InsufficientBalance => write!(f, "Insufficient balance"),
            WalletOpsError::NoUnspentTransactions => write!(f, "No unspent transactions"),
            WalletOpsError::NotMultisigWallet => write!(f, "Wallet is not a multisig wallet"),
            WalletOpsError::DescriptorError(err) => write!(f, "Descriptor error: {}", err),
            WalletOpsError::JsonError(err) => write!(f, "JSON error: {}", err),
            WalletOpsError::Other(err) => write!(f, "Other error: {}", err),
        }
    }
}

impl Error for WalletOpsError {}

impl From<WalletError> for WalletOpsError {
    fn from(err: WalletError) -> Self {
        WalletOpsError::WalletError(err)
    }
}

impl From<ClientError> for WalletOpsError {
    fn from(err: ClientError) -> Self {
        WalletOpsError::ClientError(err)
    }
}

impl From<RpcError> for WalletOpsError {
    fn from(err: RpcError) -> Self {
        WalletOpsError::RpcError(err)
    }
}

impl From<BitcoindError> for WalletOpsError {
    fn from(err: BitcoindError) -> Self {
        WalletOpsError::BitcoindError(err)
    }
}

impl From<UtilsError> for WalletOpsError {
    fn from(err: UtilsError) -> Self {
        WalletOpsError::UtilsError(err)
    }
}

impl From<miniscript::Error> for WalletOpsError {
    fn from(err: miniscript::Error) -> Self {
        WalletOpsError::DescriptorError(err)
    }
}

impl From<serde_json::Error> for WalletOpsError {
    fn from(err: serde_json::Error) -> Self {
        WalletOpsError::JsonError(err)
    }
}

/// Wallet Errors

#[derive(Debug)]
pub enum WalletError {
    ClientError(ClientError),
    WalletCreationDisabled(String),
    AddressNetworkMismatch,
    SigningFailed(String),
    RpcError(RpcError),
    AddressNotFound,
}

impl fmt::Display for WalletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WalletError::ClientError(err) => write!(f, "Client error: {}", err),
            WalletError::WalletCreationDisabled(name) => write!(f, "Wallet creation disabled: {}", name),
            WalletError::AddressNetworkMismatch => write!(f, "Address network mismatch"),
            WalletError::SigningFailed(err) => write!(f, "Signing failed: {}", err),
            WalletError::RpcError(err) => write!(f, "RPC error: {}", err),
            WalletError::AddressNotFound => write!(f, "Address not found in transaction details"),
        }
    }
}

impl Error for WalletError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            WalletError::ClientError(err) => Some(err),
            WalletError::RpcError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<ClientError> for WalletError {
    fn from(err: ClientError) -> Self {
        WalletError::ClientError(err)
    }
}

impl From<RpcError> for WalletError {
    fn from(err: RpcError) -> Self {
        WalletError::RpcError(err)
    }
}

/// Utils Errors

#[derive(Debug)]
pub enum UtilsError {
    ExternalXpubNotFound,
    InternalXpubNotFound,
    InsufficientUTXOs,
    JsonParsingError(serde_json::Error),
}

impl fmt::Display for UtilsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UtilsError::ExternalXpubNotFound => write!(f, "External xpub descriptor not found"),
            UtilsError::InternalXpubNotFound => write!(f, "Internal xpub descriptor not found"),
            UtilsError::InsufficientUTXOs => write!(f, "Insufficient UTXOs to meet target amount"),
            UtilsError::JsonParsingError(e) => write!(f, "JSON parsing error: {}", e),
        }
    }
}

impl Error for UtilsError {}

impl From<serde_json::Error> for UtilsError {
    fn from(err: serde_json::Error) -> Self {
        UtilsError::JsonParsingError(err)
    }
}