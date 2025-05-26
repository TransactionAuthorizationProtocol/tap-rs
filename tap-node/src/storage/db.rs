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
use super::models::{Transaction, TransactionStatus, TransactionType};

/// Storage backend for TAP transactions
///
/// This struct provides the main interface for storing and retrieving TAP transactions
/// from a SQLite database. It uses connection pooling for efficient concurrent access
/// and provides an async-friendly API.
///
/// # Example
///
/// ```no_run
/// use tap_node::storage::Storage;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create storage with default path
/// let storage = Storage::new(None).await?;
///
/// // Or with custom path
/// let storage = Storage::new(Some(PathBuf::from("./my-db.db"))).await?;
/// # Ok(())
/// # }
/// ```
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
        let pool = Pool::builder()
            .max_size(10)
            .build(manager)?;
        
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
            return Err(StorageError::InvalidTransactionType(message_type.to_string()));
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
    pub async fn get_transaction_by_id(&self, reference_id: &str) -> Result<Option<Transaction>, StorageError> {
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
    pub async fn list_transactions(&self, limit: u32, offset: u32) -> Result<Vec<Transaction>, StorageError> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tap_msg::message::transfer::Transfer;
    use tap_msg::message::Participant;
    
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
            asset: "eip155:1/erc20:0x0000000000000000000000000000000000000000".parse().unwrap(),
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
}