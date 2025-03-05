//! Error handling for the TAP HTTP server.

use thiserror::Error;

/// Result type for tap-http operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for the TAP HTTP server.
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid DIDComm message format.
    #[error("Invalid DIDComm message: {0}")]
    DIDComm(String),

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

    /// Unknown error.
    #[error("Unknown error: {0}")]
    Unknown(String),
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
