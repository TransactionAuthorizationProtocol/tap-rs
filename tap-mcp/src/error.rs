use thiserror::Error;

/// Result type for TAP-MCP operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for TAP-MCP
#[derive(Error, Debug)]
pub enum Error {
    #[error("TAP Node error: {0}")]
    TapNode(#[from] tap_node::error::Error),

    #[error("TAP Storage error: {0}")]
    TapStorage(#[from] tap_node::storage::error::StorageError),

    #[error("TAP Agent error: {0}")]
    TapAgent(#[from] tap_agent::error::Error),

    #[error("TAP Message error: {0}")]
    TapMessage(#[from] tap_msg::error::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid tool parameter: {0}")]
    InvalidParameter(String),

    #[error("Tool execution failed: {0}")]
    ToolExecution(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Configuration error: {0}")]
    Configuration(String),
}

impl Error {
    pub fn invalid_parameter(msg: impl Into<String>) -> Self {
        Self::InvalidParameter(msg.into())
    }

    pub fn tool_execution(msg: impl Into<String>) -> Self {
        Self::ToolExecution(msg.into())
    }

    pub fn resource_not_found(msg: impl Into<String>) -> Self {
        Self::ResourceNotFound(msg.into())
    }

    pub fn configuration(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }
}
