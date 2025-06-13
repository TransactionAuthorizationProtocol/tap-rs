//! Error types for IVMS 101 validation and processing

use thiserror::Error;

/// Result type alias for IVMS operations
pub type Result<T> = std::result::Result<T, Error>;

/// IVMS 101 error types
#[derive(Debug, Error)]
pub enum Error {
    /// Name validation error
    #[error("Invalid name: {0}")]
    InvalidName(String),

    /// Address validation error
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Identifier validation error
    #[error("Invalid identifier: {0}")]
    InvalidIdentifier(String),

    /// Date validation error
    #[error("Invalid date: {0}")]
    InvalidDate(String),

    /// National identification validation error
    #[error("Invalid national identification: {0}")]
    InvalidNationalId(String),

    /// Customer identification validation error
    #[error("Invalid customer identification: {0}")]
    InvalidCustomerId(String),

    /// Registration authority validation error
    #[error("Invalid registration authority: {0}")]
    InvalidRegistrationAuthority(String),

    /// Country code validation error
    #[error("Invalid country code: {0}")]
    InvalidCountryCode(String),

    /// Currency code validation error
    #[error("Invalid currency code: {0}")]
    InvalidCurrencyCode(String),

    /// LEI validation error
    #[error("Invalid LEI: {0}")]
    InvalidLei(String),

    /// BIC validation error
    #[error("Invalid BIC: {0}")]
    InvalidBic(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingRequiredField(String),

    /// Invalid enum value
    #[error("Invalid {field} value: {value}")]
    InvalidEnumValue { field: String, value: String },

    /// Validation error with multiple issues
    #[error("Validation failed: {issues:?}")]
    ValidationFailed { issues: Vec<String> },

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
