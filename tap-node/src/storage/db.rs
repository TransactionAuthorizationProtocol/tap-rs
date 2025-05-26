use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, OptionalExtension};
use std::env;
use std::path::PathBuf;
use tap_msg::didcomm::PlainMessage;
use tokio::task;
use tracing::{debug, info};

use super::error::StorageError;
use super::migrations::run_migrations;
use super::models::{Message, MessageDirection, Transaction, TransactionStatus, TransactionType};

/// Storage backend for TAP transactions and message audit trail
///
/// This struct provides the main interface for storing and retrieving TAP data
/// from a SQLite database. It maintains two separate tables:
/// - `transactions`: For Transfer and Payment messages requiring business logic
/// - `messages`: For complete audit trail of all messages
///
/// It uses connection pooling for efficient concurrent access and provides
/// an async-friendly API.
///
/// # Example
///
/// ```no_run
/// use tap_node::storage::{Storage, MessageDirection};
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create storage with default path
/// let storage = Storage::new(None).await?;
///
/// // Query transactions
/// let transactions = storage.list_transactions(10, 0).await?;
///
/// // Query audit trail
/// let all_messages = storage.list_messages(20, 0, None).await?;
/// let incoming_only = storage.list_messages(10, 0, Some(MessageDirection::Incoming)).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Storage {
    pool: Pool<SqliteConnectionManager>,
}

impl Storage {
    /// Create a new Storage instance
    ///
    /// This will initialize a SQLite database at the specified path (or default location),
    /// run any pending migrations, and set up a connection pool.
    ///
    /// # Arguments
    ///
    /// * `path` - Optional path to the database file. If None, uses `TAP_NODE_DB_PATH` env var or defaults to `./tap-node.db`
    ///
    /// # Errors
    ///
    /// Returns `StorageError` if:
    /// - Database initialization fails
    /// - Migrations fail to run
    /// - Connection pool cannot be created
    pub async fn new(path: Option<PathBuf>) -> Result<Self, StorageError> {
        let db_path = path.unwrap_or_else(|| {
            env::var("TAP_NODE_DB_PATH")
                .unwrap_or_else(|_| "tap-node.db".to_string())
                .into()
        });

        info!("Initializing storage at: {:?}", db_path);

        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Initialize connection pool
        let manager = SqliteConnectionManager::file(&db_path);
        let pool = Pool::builder().max_size(10).build(manager)?;

        // Run migrations
        {
            let mut conn = pool.get()?;

            // Enable WAL mode for better concurrency
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "synchronous", "NORMAL")?;

            run_migrations(&mut conn)?;
        }

        Ok(Storage { pool })
    }

    /// Insert a new transaction from a TAP message
    ///
    /// This method extracts transaction details from a Transfer or Payment message
    /// and stores them in the database with a 'pending' status.
    ///
    /// # Arguments
    ///
    /// * `message` - The DIDComm PlainMessage containing a Transfer or Payment body
    ///
    /// # Errors
    ///
    /// Returns `StorageError` if:
    /// - The message is not a Transfer or Payment type
    /// - Database insertion fails
    /// - The transaction already exists (duplicate reference_id)
    pub async fn insert_transaction(&self, message: &PlainMessage) -> Result<(), StorageError> {
        let message_type = message.type_.clone();
        let message_json = serde_json::to_string_pretty(message)?;

        // Extract transaction type and use message ID as reference
        let tx_type = if message.type_.contains("transfer") {
            TransactionType::Transfer
        } else if message.type_.contains("payment") {
            TransactionType::Payment
        } else {
            return Err(StorageError::InvalidTransactionType(
                message_type.to_string(),
            ));
        };

        // Use the PlainMessage ID as the reference_id since transaction_id is not serialized
        let reference_id = message.id.clone();
        let from_did = message.from.clone();
        let to_did = message.to.first().cloned();
        let thread_id = message.thid.clone();

        let pool = self.pool.clone();

        // Execute in blocking task for async compatibility
        task::spawn_blocking(move || {
            let conn = pool.get()?;
            
            debug!("Inserting transaction: {} ({})", reference_id, tx_type);
            
            let result = conn.execute(
                "INSERT INTO transactions (type, reference_id, from_did, to_did, thread_id, message_type, message_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    tx_type.to_string(),
                    reference_id.clone(),
                    from_did,
                    to_did,
                    thread_id,
                    message_type.to_string(),
                    message_json
                ],
            );
            
            match result {
                Ok(_) => debug!("Successfully inserted transaction: {}", reference_id),
                Err(e) => {
                    if let rusqlite::Error::SqliteFailure(err, _) = &e {
                        if err.code == rusqlite::ErrorCode::ConstraintViolation {
                            return Err(StorageError::DuplicateTransaction(reference_id));
                        }
                    }
                    return Err(StorageError::Database(e));
                }
            }
            
            Ok::<(), StorageError>(())
        })
        .await
        .map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))??;

        Ok(())
    }

    /// Retrieve a transaction by its reference ID
    ///
    /// # Arguments
    ///
    /// * `reference_id` - The unique message ID of the transaction
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Transaction))` if found
    /// * `Ok(None)` if not found
    /// * `Err(StorageError)` on database error
    pub async fn get_transaction_by_id(
        &self,
        reference_id: &str,
    ) -> Result<Option<Transaction>, StorageError> {
        let pool = self.pool.clone();
        let reference_id = reference_id.to_string();

        task::spawn_blocking(move || {
            let conn = pool.get()?;
            
            let result = conn.query_row(
                "SELECT id, type, reference_id, from_did, to_did, thread_id, message_type, status, message_json, created_at, updated_at
                 FROM transactions WHERE reference_id = ?1",
                params![reference_id],
                |row| {
                    Ok(Transaction {
                        id: row.get(0)?,
                        transaction_type: TransactionType::try_from(row.get::<_, String>(1)?.as_str())
                            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?,
                        reference_id: row.get(2)?,
                        from_did: row.get(3)?,
                        to_did: row.get(4)?,
                        thread_id: row.get(5)?,
                        message_type: row.get(6)?,
                        status: TransactionStatus::try_from(row.get::<_, String>(7)?.as_str())
                            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?,
                        message_json: row.get(8)?,
                        created_at: row.get(9)?,
                        updated_at: row.get(10)?,
                    })
                },
            ).optional()?;
            
            Ok(result)
        })
        .await
        .map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
    }

    /// List transactions with pagination
    ///
    /// Retrieves transactions ordered by creation time (newest first).
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of transactions to return
    /// * `offset` - Number of transactions to skip (for pagination)
    ///
    /// # Returns
    ///
    /// A vector of transactions ordered by creation time descending
    pub async fn list_transactions(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Transaction>, StorageError> {
        let pool = self.pool.clone();

        task::spawn_blocking(move || {
            let conn = pool.get()?;
            
            let mut stmt = conn.prepare(
                "SELECT id, type, reference_id, from_did, to_did, thread_id, message_type, status, message_json, created_at, updated_at
                 FROM transactions
                 ORDER BY created_at DESC
                 LIMIT ?1 OFFSET ?2"
            )?;
            
            let transactions = stmt.query_map(params![limit, offset], |row| {
                Ok(Transaction {
                    id: row.get(0)?,
                    transaction_type: TransactionType::try_from(row.get::<_, String>(1)?.as_str())
                        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?,
                    reference_id: row.get(2)?,
                    from_did: row.get(3)?,
                    to_did: row.get(4)?,
                    thread_id: row.get(5)?,
                    message_type: row.get(6)?,
                    status: TransactionStatus::try_from(row.get::<_, String>(7)?.as_str())
                        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?,
                    message_json: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
            
            Ok(transactions)
        })
        .await
        .map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
    }

    /// Log an incoming or outgoing message to the audit trail
    ///
    /// This method stores any DIDComm message for audit purposes, regardless of type.
    ///
    /// # Arguments
    ///
    /// * `message` - The DIDComm PlainMessage to log
    /// * `direction` - Whether the message is incoming or outgoing
    ///
    /// # Errors
    ///
    /// Returns `StorageError` if:
    /// - Database insertion fails
    /// - The message already exists (duplicate message_id)
    pub async fn log_message(
        &self,
        message: &PlainMessage,
        direction: MessageDirection,
    ) -> Result<(), StorageError> {
        let message_json = serde_json::to_string_pretty(message)?;
        let message_id = message.id.clone();
        let message_type = message.type_.clone();
        let from_did = message.from.clone();
        let to_did = message.to.first().cloned();
        let thread_id = message.thid.clone();
        let parent_thread_id = message.pthid.clone();

        let pool = self.pool.clone();

        task::spawn_blocking(move || {
            let conn = pool.get()?;
            
            debug!("Logging {} message: {} ({})", direction, message_id, message_type);
            
            let result = conn.execute(
                "INSERT INTO messages (message_id, message_type, from_did, to_did, thread_id, parent_thread_id, direction, message_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    message_id.clone(),
                    message_type,
                    from_did,
                    to_did,
                    thread_id,
                    parent_thread_id,
                    direction.to_string(),
                    message_json
                ],
            );
            
            match result {
                Ok(_) => debug!("Successfully logged message: {}", message_id),
                Err(e) => {
                    if let rusqlite::Error::SqliteFailure(err, _) = &e {
                        if err.code == rusqlite::ErrorCode::ConstraintViolation {
                            // Message already logged, this is fine
                            debug!("Message already logged: {}", message_id);
                            return Ok(());
                        }
                    }
                    return Err(StorageError::Database(e));
                }
            }
            
            Ok::<(), StorageError>(())
        })
        .await
        .map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))??;

        Ok(())
    }

    /// Retrieve a message by its ID
    ///
    /// # Arguments
    ///
    /// * `message_id` - The unique message ID
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Message))` if found
    /// * `Ok(None)` if not found
    /// * `Err(StorageError)` on database error
    pub async fn get_message_by_id(
        &self,
        message_id: &str,
    ) -> Result<Option<Message>, StorageError> {
        let pool = self.pool.clone();
        let message_id = message_id.to_string();

        task::spawn_blocking(move || {
            let conn = pool.get()?;
            
            let result = conn.query_row(
                "SELECT id, message_id, message_type, from_did, to_did, thread_id, parent_thread_id, direction, message_json, created_at
                 FROM messages WHERE message_id = ?1",
                params![message_id],
                |row| {
                    Ok(Message {
                        id: row.get(0)?,
                        message_id: row.get(1)?,
                        message_type: row.get(2)?,
                        from_did: row.get(3)?,
                        to_did: row.get(4)?,
                        thread_id: row.get(5)?,
                        parent_thread_id: row.get(6)?,
                        direction: MessageDirection::try_from(row.get::<_, String>(7)?.as_str())
                            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?,
                        message_json: row.get(8)?,
                        created_at: row.get(9)?,
                    })
                },
            ).optional()?;
            
            Ok(result)
        })
        .await
        .map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
    }

    /// List messages with pagination and optional filtering
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of messages to return
    /// * `offset` - Number of messages to skip (for pagination)
    /// * `direction` - Optional filter by message direction
    ///
    /// # Returns
    ///
    /// A vector of messages ordered by creation time descending
    pub async fn list_messages(
        &self,
        limit: u32,
        offset: u32,
        direction: Option<MessageDirection>,
    ) -> Result<Vec<Message>, StorageError> {
        let pool = self.pool.clone();

        task::spawn_blocking(move || {
            let conn = pool.get()?;
            
            let query = if let Some(dir) = direction {
                format!(
                    "SELECT id, message_id, message_type, from_did, to_did, thread_id, parent_thread_id, direction, message_json, created_at
                     FROM messages
                     WHERE direction = '{}'
                     ORDER BY created_at DESC
                     LIMIT {} OFFSET {}",
                    dir, limit, offset
                )
            } else {
                format!(
                    "SELECT id, message_id, message_type, from_did, to_did, thread_id, parent_thread_id, direction, message_json, created_at
                     FROM messages
                     ORDER BY created_at DESC
                     LIMIT {} OFFSET {}",
                    limit, offset
                )
            };
            
            let mut stmt = conn.prepare(&query)?;
            
            let messages = stmt.query_map([], |row| {
                Ok(Message {
                    id: row.get(0)?,
                    message_id: row.get(1)?,
                    message_type: row.get(2)?,
                    from_did: row.get(3)?,
                    to_did: row.get(4)?,
                    thread_id: row.get(5)?,
                    parent_thread_id: row.get(6)?,
                    direction: MessageDirection::try_from(row.get::<_, String>(7)?.as_str())
                        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?,
                    message_json: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
            
            Ok(messages)
        })
        .await
        .map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tap_msg::message::transfer::Transfer;
    use tap_msg::message::Participant;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_storage_creation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        let storage = Storage::new(Some(db_path)).await.unwrap();
        assert!(storage.pool.get().is_ok());
    }

    #[tokio::test]
    async fn test_insert_and_retrieve_transaction() {
        let _ = env_logger::builder().is_test(true).try_init();

        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = Storage::new(Some(db_path)).await.unwrap();

        // Create a test transfer message
        let transfer_body = Transfer {
            transaction_id: "test_transfer_123".to_string(),
            originator: Participant {
                id: "did:example:originator".to_string(),
                name: None,
                role: None,
                policies: None,
                leiCode: None,
            },
            beneficiary: Some(Participant {
                id: "did:example:beneficiary".to_string(),
                name: None,
                role: None,
                policies: None,
                leiCode: None,
            }),
            asset: "eip155:1/erc20:0x0000000000000000000000000000000000000000"
                .parse()
                .unwrap(),
            amount: "1000000000000000000".to_string(),
            agents: vec![],
            memo: None,
            settlement_id: None,
            metadata: Default::default(),
        };

        let message_id = "test_message_123";
        let message = PlainMessage {
            id: message_id.to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: "https://tap-protocol.io/messages/transfer/1.0".to_string(),
            body: serde_json::to_value(&transfer_body).unwrap(),
            from: "did:example:sender".to_string(),
            to: vec!["did:example:receiver".to_string()],
            thid: None,
            pthid: None,
            extra_headers: Default::default(),
            attachments: None,
            created_time: None,
            expires_time: None,
            from_prior: None,
        };

        // Insert transaction
        storage.insert_transaction(&message).await.unwrap();

        // Retrieve transaction
        let retrieved = storage.get_transaction_by_id(message_id).await.unwrap();
        assert!(retrieved.is_some(), "Transaction not found");

        let tx = retrieved.unwrap();
        assert_eq!(tx.reference_id, message_id);
        assert_eq!(tx.transaction_type, TransactionType::Transfer);
        assert_eq!(tx.status, TransactionStatus::Pending);
    }

    #[tokio::test]
    async fn test_log_and_retrieve_messages() {
        let _ = env_logger::builder().is_test(true).try_init();

        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = Storage::new(Some(db_path)).await.unwrap();

        // Create test messages of different types
        let connect_message = PlainMessage {
            id: "msg_connect_123".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: "https://tap-protocol.io/messages/connect/1.0".to_string(),
            body: serde_json::json!({"constraints": ["test"]}),
            from: "did:example:alice".to_string(),
            to: vec!["did:example:bob".to_string()],
            thid: Some("thread_123".to_string()),
            pthid: None,
            extra_headers: Default::default(),
            attachments: None,
            created_time: None,
            expires_time: None,
            from_prior: None,
        };

        let authorize_message = PlainMessage {
            id: "msg_auth_123".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: "https://tap-protocol.io/messages/authorize/1.0".to_string(),
            body: serde_json::json!({"transaction_id": "test_transfer_123"}),
            from: "did:example:bob".to_string(),
            to: vec!["did:example:alice".to_string()],
            thid: Some("thread_123".to_string()),
            pthid: None,
            extra_headers: Default::default(),
            attachments: None,
            created_time: None,
            expires_time: None,
            from_prior: None,
        };

        // Log messages
        storage
            .log_message(&connect_message, MessageDirection::Incoming)
            .await
            .unwrap();
        storage
            .log_message(&authorize_message, MessageDirection::Outgoing)
            .await
            .unwrap();

        // Retrieve specific message
        let retrieved = storage.get_message_by_id("msg_connect_123").await.unwrap();
        assert!(retrieved.is_some());
        let msg = retrieved.unwrap();
        assert_eq!(msg.message_id, "msg_connect_123");
        assert_eq!(msg.direction, MessageDirection::Incoming);

        // List all messages
        let all_messages = storage.list_messages(10, 0, None).await.unwrap();
        assert_eq!(all_messages.len(), 2);

        // List only incoming messages
        let incoming_messages = storage
            .list_messages(10, 0, Some(MessageDirection::Incoming))
            .await
            .unwrap();
        assert_eq!(incoming_messages.len(), 1);
        assert_eq!(incoming_messages[0].message_id, "msg_connect_123");

        // Test duplicate message handling (should not error)
        storage
            .log_message(&connect_message, MessageDirection::Incoming)
            .await
            .unwrap();
        let all_messages_after = storage.list_messages(10, 0, None).await.unwrap();
        assert_eq!(all_messages_after.len(), 2); // Should still be 2, not 3
    }
}
