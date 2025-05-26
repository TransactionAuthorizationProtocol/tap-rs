use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Connection pool error: {0}")]
    Pool(#[from] r2d2::Error),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Transaction not found: {0}")]
    NotFound(String),

    #[error("Invalid transaction type: {0}")]
    InvalidTransactionType(String),

    #[error("Duplicate transaction: {0}")]
    DuplicateTransaction(String),
}
