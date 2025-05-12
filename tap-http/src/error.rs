//! Error handling for the TAP HTTP server.
//!
//! This module provides a comprehensive error handling system for the TAP HTTP server.
//! It defines error types for various failure scenarios and provides conversions
//! from common error types to the tap-http error type.

use thiserror::Error;
use warp::Reply;

/// Result type for tap-http operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for the TAP HTTP server.
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid DIDComm message format.
    #[error("Invalid DIDComm message: {0}")]
    DIDComm(String),

    /// Message validation error.
    #[error("Message validation failed: {0}")]
    Validation(String),

    /// Message authentication error.
    #[error("Message authentication failed: {0}")]
    Authentication(String),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(String),

    /// HTTP server error.
    #[error("HTTP error: {0}")]
    Http(String),

    /// TAP Node error.
    #[error("TAP Node error: {0}")]
    Node(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Rate limiting error.
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// TLS error.
    #[error("TLS error: {0}")]
    Tls(String),

    /// Unknown error.
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Error severity for logging and reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Informational errors (e.g., validation failures)
    Info,
    /// Warning-level errors (e.g., rate limiting)
    Warning,
    /// Critical errors (e.g., server failures)
    Critical,
}

impl Error {
    /// Returns the HTTP status code that should be used for this error.
    pub fn status_code(&self) -> warp::http::StatusCode {
        use warp::http::StatusCode;

        match self {
            Error::DIDComm(_) | Error::Validation(_) | Error::Json(_) => StatusCode::BAD_REQUEST,
            Error::Authentication(_) => StatusCode::UNAUTHORIZED,
            Error::RateLimit(_) => StatusCode::TOO_MANY_REQUESTS,
            Error::Node(_) | Error::Unknown(_) | Error::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Config(_) => StatusCode::SERVICE_UNAVAILABLE,
            Error::Http(_) => StatusCode::BAD_GATEWAY,
            Error::Tls(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Returns the severity level of this error.
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Error::DIDComm(_) | Error::Validation(_) | Error::Json(_) => ErrorSeverity::Info,
            Error::RateLimit(_) | Error::Authentication(_) => ErrorSeverity::Warning,
            Error::Node(_)
            | Error::Http(_)
            | Error::Config(_)
            | Error::Tls(_)
            | Error::Unknown(_)
            | Error::Io(_) => ErrorSeverity::Critical,
        }
    }

    /// Creates an error response for this error.
    pub fn to_response(&self) -> warp::reply::Response {
        let status = self.status_code();
        let message = self.to_string();
        let error_type = match self {
            Error::DIDComm(_) => "didcomm_error",
            Error::Validation(_) => "validation_error",
            Error::Authentication(_) => "authentication_error",
            Error::Json(_) => "json_error",
            Error::Http(_) => "http_error",
            Error::Node(_) => "node_error",
            Error::Config(_) => "configuration_error",
            Error::Io(_) => "io_error",
            Error::RateLimit(_) => "rate_limit_error",
            Error::Tls(_) => "tls_error",
            Error::Unknown(_) => "unknown_error",
        };

        warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "status": "error",
                "error": {
                    "type": error_type,
                    "message": message,
                }
            })),
            status,
        )
        .into_response()
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err.to_string())
    }
}

impl From<tap_msg::error::Error> for Error {
    fn from(err: tap_msg::error::Error) -> Self {
        Error::DIDComm(err.to_string())
    }
}

impl From<tap_node::error::Error> for Error {
    fn from(err: tap_node::error::Error) -> Self {
        Error::Node(err.to_string())
    }
}

impl warp::reject::Reject for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = Error::DIDComm("test error".to_string());
        assert_eq!(error.to_string(), "Invalid DIDComm message: test error");

        let error = Error::Node("node error".to_string());
        assert_eq!(error.to_string(), "TAP Node error: node error");

        let error = Error::Http("server error".to_string());
        assert_eq!(error.to_string(), "HTTP error: server error");

        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = Error::from(io_error);
        assert!(error.to_string().contains("file not found"));
    }

    #[test]
    fn test_error_conversions() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let error = Error::from(json_error);
        assert!(matches!(error, Error::Json(_)));
    }
}
