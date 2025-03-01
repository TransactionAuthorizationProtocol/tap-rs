use thiserror::Error;

/// Error types for the CAIP library
#[derive(Error, Debug)]
pub enum Error {
    /// Error when parsing a ChainId
    #[error("Invalid CAIP-2 Chain ID: {0}")]
    InvalidChainId(String),

    /// Error when parsing a namespace
    #[error("Invalid namespace: {0}")]
    InvalidNamespace(String),

    /// Error when parsing a reference
    #[error("Invalid reference: {0}")]
    InvalidReference(String),

    /// Error when parsing an AccountId
    #[error("Invalid CAIP-10 Account ID: {0}")]
    InvalidAccountId(String),

    /// Error when validating an address format
    #[error("Invalid address format for chain {0}: {1}")]
    InvalidAddressFormat(String, String),

    /// Error when parsing an AssetId
    #[error("Invalid CAIP-19 Asset ID: {0}")]
    InvalidAssetId(String),

    /// Error when parsing an asset namespace
    #[error("Invalid asset namespace: {0}")]
    InvalidAssetNamespace(String),

    /// Error when parsing an asset reference
    #[error("Invalid asset reference: {0}")]
    InvalidAssetReference(String),

    /// Error when parsing an input that doesn't match any CAIP format
    #[error("Unrecognized CAIP format: {0}")]
    UnrecognizedFormat(String),

    /// Specific error for Ethereum address validation
    #[error("Invalid Ethereum address: {0}")]
    InvalidEthereumAddress(String),

    /// Specific error for Bitcoin address validation
    #[error("Invalid Bitcoin address: {0}")]
    InvalidBitcoinAddress(String),
}
