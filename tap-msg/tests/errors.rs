use thiserror::Error;

/// Errors that can occur during test vector validation
#[derive(Error, Debug)]
pub enum ValidationError {
    /// Error parsing a DIDComm message
    #[error("Failed to parse DIDComm message: {0}")]
    MessageParseError(String),
    
    /// Error parsing a date/time string
    #[error("Failed to parse datetime string '{value}': {message}")]
    DateTimeParseError {
        value: String,
        message: String,
    },
    
    /// Error validating a message body
    #[error("Body validation failed: {0}")]
    BodyValidationError(String),
    
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
pub fn to_validation_error<S: AsRef<str>>(error: S) -> ValidationError {
    ValidationError::Other(error.as_ref().to_string())
}
