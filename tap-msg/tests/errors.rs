use thiserror::Error;

/// Errors that can occur during test vector validation
#[derive(Error, Debug)]
#[allow(dead_code)] // Enum used for test error handling
pub enum ValidationError {
    /// Error parsing a DIDComm message
    #[error("Failed to parse DIDComm message: {0}")]
    MessageParseError(String),

    /// Error parsing a date/time string
    #[error("Failed to parse datetime string '{value}': {message}")]
    DateTimeParseError { value: String, message: String },

    /// Error validating a message body
    #[error("Invalid message body: {0}")]
    InvalidBody(String),

    /// Error in message structure (missing fields, wrong types, etc.)
    #[error("Invalid message structure: {0}")]
    StructureError(String),

    /// Error validating required fields
    #[error("Missing required field: {0}")]
    MissingFieldError(String),

    /// Error in attachment validation
    #[error("Attachment validation failed: {0}")]
    AttachmentError(String),

    /// Other, general validation errors
    #[error("Validation error: {0}")]
    Other(String),
}

/// Helper function to convert a string error to a ValidationError::Other
#[allow(dead_code)] // Function used for test error conversion
pub fn to_validation_error<S: AsRef<str>>(error: S) -> ValidationError {
    ValidationError::Other(error.as_ref().to_string())
}
