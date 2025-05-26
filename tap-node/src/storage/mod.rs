//! Storage module for persisting TAP transactions
//!
//! This module provides persistent storage capabilities for the TAP Node,
//! allowing Transfer and Payment transactions to be stored in a SQLite database
//! for audit trails, querying, and compliance purposes.
//!
//! # Features
//!
//! - **Automatic Schema Migration**: Database schema is created and migrated automatically
//! - **Connection Pooling**: Uses r2d2 for efficient concurrent database access
//! - **Async API**: All operations are async-friendly using tokio's spawn_blocking
//! - **WASM Compatibility**: Storage is automatically disabled in WASM builds
//!
//! # Usage
//!
//! Storage is automatically initialized when creating a TapNode with the storage feature enabled:
//!
//! ```no_run
//! use tap_node::{NodeConfig, TapNode};
//! use std::path::PathBuf;
//!
//! let config = NodeConfig {
//!     #[cfg(feature = "storage")]
//!     storage_path: Some(PathBuf::from("./my-database.db")),
//!     ..Default::default()
//! };
//!
//! let node = TapNode::new(config);
//! ```
//!
//! # Environment Variables
//!
//! - `TAP_NODE_DB_PATH`: Override the default database path

#[cfg(feature = "storage")]
pub mod db;
#[cfg(feature = "storage")]
pub mod error;
#[cfg(feature = "storage")]
pub mod migrations;
#[cfg(feature = "storage")]
pub mod models;

#[cfg(feature = "storage")]
pub use db::Storage;
#[cfg(feature = "storage")]
pub use error::StorageError;
#[cfg(feature = "storage")]
pub use models::{Transaction, TransactionStatus, TransactionType};

#[cfg(not(feature = "storage"))]
pub use mock::*;

#[cfg(not(feature = "storage"))]
mod mock {
    use tap_msg::didcomm::PlainMessage;
    
    #[derive(Debug, Clone)]
    pub struct Storage;
    
    #[derive(Debug, thiserror::Error)]
    #[error("Storage is not available in this build")]
    pub struct StorageError;
    
    impl Storage {
        pub async fn new(_path: Option<std::path::PathBuf>) -> Result<Self, StorageError> {
            Ok(Storage)
        }
        
        pub async fn insert_transaction(&self, _message: &PlainMessage) -> Result<(), StorageError> {
            Ok(())
        }
    }
}