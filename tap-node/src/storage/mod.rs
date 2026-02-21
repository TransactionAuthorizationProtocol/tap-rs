//! Storage module for persisting TAP messages and transactions
//!
//! This module provides comprehensive persistent storage capabilities for the TAP Node,
//! maintaining both business transaction records and a complete message audit trail
//! in a SQLite database for compliance, debugging, and operational purposes.
//!
//! # Architecture
//!
//! The storage system uses a dual-table design:
//! - **transactions**: Stores Transfer and Payment messages for business logic
//! - **messages**: Stores all messages (incoming/outgoing) for audit trail
//!
//! # Features
//!
//! - **Automatic Schema Migration**: Database schema is created and migrated automatically
//! - **Connection Pooling**: Uses sqlx's built-in async connection pool
//! - **Async API**: Native async operations without blocking threads
//! - **WASM Compatibility**: Storage is automatically disabled in WASM builds
//! - **Idempotent Operations**: Duplicate messages are silently ignored
//! - **Direction Tracking**: Messages are tagged as incoming or outgoing
//! - **Thread Tracking**: Full support for DIDComm thread and parent thread IDs
//!
//! # Usage
//!
//! Storage is automatically initialized when creating a TapNode with the storage feature enabled:
//!
//! ```no_run
//! use tap_node::{NodeConfig, TapNode};
//! use tap_node::storage::MessageDirection;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = NodeConfig {
//!     #[cfg(feature = "storage")]
//!     storage_path: Some(PathBuf::from("./my-database.db")),
//!     ..Default::default()
//! };
//!
//! let node = TapNode::new(config);
//!
//! // Access storage functionality
//! if let Some(storage) = node.storage() {
//!     // Query transactions
//!     let txs = storage.list_transactions(10, 0).await?;
//!     
//!     // Query message audit trail
//!     let messages = storage.list_messages(20, 0, Some(MessageDirection::Incoming)).await?;
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Environment Variables
//!
//! - `TAP_NODE_DB_PATH`: Override the default database path
//!
//! # Automatic Message Logging
//!
//! The TapNode automatically logs all messages:
//! - Incoming messages are logged when `receive_message()` is called
//! - Outgoing messages are logged when `send_message()` is called
//! - Transfer and Payment messages are additionally stored in the transactions table

#[cfg(feature = "storage")]
pub mod agent_storage_manager;
#[cfg(feature = "storage")]
pub mod db;
#[cfg(feature = "storage")]
pub mod error;
#[cfg(feature = "storage")]
pub mod models;

#[cfg(feature = "storage")]
pub use agent_storage_manager::AgentStorageManager;
#[cfg(feature = "storage")]
pub use db::Storage;
#[cfg(feature = "storage")]
pub use error::StorageError;
#[cfg(feature = "storage")]
pub use models::{
    Customer, CustomerIdentifier, CustomerRelationship, DecisionLogEntry, DecisionStatus,
    DecisionType, Delivery, DeliveryStatus, DeliveryType, IdentifierType, Message,
    MessageDirection, Received, ReceivedStatus, SchemaType, SourceType, Transaction,
    TransactionStatus, TransactionType,
};

#[cfg(not(feature = "storage"))]
pub use mock::*;

#[cfg(not(feature = "storage"))]
mod mock {
    use serde::{Deserialize, Serialize};
    use tap_msg::didcomm::PlainMessage;

    #[derive(Debug, Clone)]
    pub struct Storage;

    #[derive(Debug, thiserror::Error)]
    #[error("Storage is not available in this build")]
    pub struct StorageError;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum MessageDirection {
        Incoming,
        Outgoing,
    }

    impl Storage {
        pub async fn new(_path: Option<std::path::PathBuf>) -> Result<Self, StorageError> {
            Ok(Storage)
        }

        pub async fn insert_transaction(
            &self,
            _message: &PlainMessage,
        ) -> Result<(), StorageError> {
            Ok(())
        }

        pub async fn log_message(
            &self,
            _message: &PlainMessage,
            _direction: MessageDirection,
        ) -> Result<(), StorageError> {
            Ok(())
        }
    }
}
