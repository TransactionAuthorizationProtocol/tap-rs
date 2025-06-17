use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use std::env;
use std::path::{Path, PathBuf};
use tap_msg::didcomm::PlainMessage;
use tracing::{debug, info};

use super::error::StorageError;
use super::models::{
    Customer, CustomerIdentifier, CustomerRelationship, Delivery, DeliveryStatus, DeliveryType,
    IdentifierType, Message, MessageDirection, Received, ReceivedStatus, SchemaType, SourceType,
    Transaction, TransactionStatus, TransactionType,
};

/// Storage backend for TAP transactions and message audit trail
///
/// This struct provides the main interface for storing and retrieving TAP data
/// from a SQLite database. It maintains two separate tables:
/// - `transactions`: For Transfer and Payment messages requiring business logic
/// - `messages`: For complete audit trail of all messages
///
/// It uses sqlx's built-in connection pooling for efficient concurrent access
/// and provides a native async API.
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
/// // Create storage with DID-based path
/// let agent_did = "did:web:example.com";
/// let storage_with_did = Storage::new_with_did(agent_did, None).await?;
///
/// // Create storage with custom TAP root
/// let custom_root = PathBuf::from("/custom/tap/root");
/// let storage_custom = Storage::new_with_did(agent_did, Some(custom_root)).await?;
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
#[derive(Clone, Debug)]
pub struct Storage {
    pool: SqlitePool,
    db_path: PathBuf,
}

impl Storage {
    /// Create a new Storage instance with an agent DID
    ///
    /// This will initialize a SQLite database in the TAP directory structure:
    /// - Default: ~/.tap/{did}/transactions.db
    /// - Custom root: {tap_root}/{did}/transactions.db
    ///
    /// # Arguments
    ///
    /// * `agent_did` - The DID of the agent this storage is for
    /// * `tap_root` - Optional custom root directory (defaults to ~/.tap)
    ///
    /// # Errors
    ///
    /// Returns `StorageError` if:
    /// - Database initialization fails
    /// - Migrations fail to run
    /// - Connection pool cannot be created
    pub async fn new_with_did(
        agent_did: &str,
        tap_root: Option<PathBuf>,
    ) -> Result<Self, StorageError> {
        let root_dir = tap_root.unwrap_or_else(|| {
            // Check TAP_HOME first (for tests)
            if let Ok(tap_home) = env::var("TAP_HOME") {
                PathBuf::from(tap_home)
            } else if let Ok(tap_root) = env::var("TAP_ROOT") {
                PathBuf::from(tap_root)
            } else if let Ok(test_dir) = env::var("TAP_TEST_DIR") {
                PathBuf::from(test_dir).join(".tap")
            } else {
                dirs::home_dir()
                    .expect("Could not find home directory")
                    .join(".tap")
            }
        });

        // Sanitize the DID for use as a directory name
        let sanitized_did = agent_did.replace(':', "_");
        let db_path = root_dir.join(&sanitized_did).join("transactions.db");

        Self::new(Some(db_path)).await
    }

    /// Create a new in-memory storage instance for testing
    /// This provides complete isolation between tests with no file system dependencies
    pub async fn new_in_memory() -> Result<Self, StorageError> {
        info!("Initializing in-memory storage for testing");

        // Use SQLite in-memory database
        let db_url = "sqlite://:memory:";

        // Create connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(1) // In-memory databases don't benefit from multiple connections
            .connect(db_url)
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| StorageError::Migration(e.to_string()))?;

        Ok(Storage {
            pool,
            db_path: PathBuf::from(":memory:"),
        })
    }

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

        // Create connection URL for SQLite with create mode
        let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

        // Create connection pool with optimizations
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(&db_url)
            .await?;

        // Enable WAL mode and other optimizations
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&pool)
            .await?;
        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(&pool)
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| StorageError::Migration(e.to_string()))?;

        Ok(Storage { pool, db_path })
    }

    /// Get the database path
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Get the default logs directory
    ///
    /// Returns the default directory for log files:
    /// - Default: ~/.tap/logs
    /// - Custom root: {tap_root}/logs
    ///
    /// # Arguments
    ///
    /// * `tap_root` - Optional custom root directory (defaults to ~/.tap)
    pub fn default_logs_dir(tap_root: Option<PathBuf>) -> PathBuf {
        let root_dir = tap_root.unwrap_or_else(|| {
            // Check TAP_HOME first (for tests)
            if let Ok(tap_home) = env::var("TAP_HOME") {
                PathBuf::from(tap_home)
            } else if let Ok(tap_root) = env::var("TAP_ROOT") {
                PathBuf::from(tap_root)
            } else if let Ok(test_dir) = env::var("TAP_TEST_DIR") {
                PathBuf::from(test_dir).join(".tap")
            } else {
                dirs::home_dir()
                    .expect("Could not find home directory")
                    .join(".tap")
            }
        });

        root_dir.join("logs")
    }

    /// Update the status of a message in the messages table
    ///
    /// # Arguments
    ///
    /// * `message_id` - The ID of the message to update
    /// * `status` - The new status (accepted, rejected, pending)
    ///
    /// # Errors
    ///
    /// Returns `StorageError` if the database update fails
    pub async fn update_message_status(
        &self,
        message_id: &str,
        status: &str,
    ) -> Result<(), StorageError> {
        debug!("Updating message {} status to {}", message_id, status);

        sqlx::query(
            r#"
            UPDATE messages 
            SET status = ?1 
            WHERE message_id = ?2
            "#,
        )
        .bind(status)
        .bind(message_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update the status of a transaction in the transactions table
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - The reference ID of the transaction to update
    /// * `status` - The new status (pending, confirmed, failed, cancelled, reverted)
    ///
    /// # Errors
    ///
    /// Returns `StorageError` if the database update fails
    pub async fn update_transaction_status(
        &self,
        transaction_id: &str,
        status: &str,
    ) -> Result<(), StorageError> {
        debug!(
            "Updating transaction {} status to {}",
            transaction_id, status
        );

        sqlx::query(
            r#"
            UPDATE transactions 
            SET status = ?1 
            WHERE reference_id = ?2
            "#,
        )
        .bind(status)
        .bind(transaction_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get a transaction by its reference ID
    ///
    /// # Arguments
    ///
    /// * `reference_id` - The reference ID of the transaction
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
        let result = sqlx::query_as::<_, (
            i64,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            String,
            String,
            serde_json::Value,
            String,
            String,
        )>(
            r#"
            SELECT id, type, reference_id, from_did, to_did, thread_id, message_type, status, message_json, created_at, updated_at
            FROM transactions WHERE reference_id = ?1
            "#,
        )
        .bind(reference_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((
            id,
            tx_type,
            reference_id,
            from_did,
            to_did,
            thread_id,
            message_type,
            status,
            message_json,
            created_at,
            updated_at,
        )) = result
        {
            Ok(Some(Transaction {
                id,
                transaction_type: TransactionType::try_from(tx_type.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                reference_id,
                from_did,
                to_did,
                thread_id,
                message_type,
                status: TransactionStatus::try_from(status.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                message_json,
                created_at,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get a transaction by thread ID
    ///
    /// # Arguments
    ///
    /// * `thread_id` - The thread ID to search for
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Transaction))` if found
    /// * `Ok(None)` if not found
    /// * `Err(StorageError)` on database error
    pub async fn get_transaction_by_thread_id(
        &self,
        thread_id: &str,
    ) -> Result<Option<Transaction>, StorageError> {
        let result = sqlx::query_as::<_, (
            i64,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            String,
            String,
            serde_json::Value,
            String,
            String,
        )>(
            r#"
            SELECT id, type, reference_id, from_did, to_did, thread_id, message_type, status, message_json, created_at, updated_at
            FROM transactions WHERE thread_id = ?1
            "#,
        )
        .bind(thread_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((
            id,
            tx_type,
            reference_id,
            from_did,
            to_did,
            thread_id,
            message_type,
            status,
            message_json,
            created_at,
            updated_at,
        )) = result
        {
            Ok(Some(Transaction {
                id,
                transaction_type: TransactionType::try_from(tx_type.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                reference_id,
                from_did,
                to_did,
                thread_id,
                message_type,
                status: TransactionStatus::try_from(status.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                message_json,
                created_at,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// Check if an agent is authorized for a transaction
    ///
    /// This checks the transaction_agents table to see if the given agent
    /// is associated with the transaction.
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - The reference ID of the transaction
    /// * `agent_did` - The DID of the agent to check
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if the agent is authorized
    /// * `Ok(false)` if the agent is not authorized or transaction doesn't exist
    /// * `Err(StorageError)` on database error
    pub async fn is_agent_authorized_for_transaction(
        &self,
        transaction_id: &str,
        agent_did: &str,
    ) -> Result<bool, StorageError> {
        // First get the transaction's internal ID
        let tx_result = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT id FROM transactions WHERE reference_id = ?1
            "#,
        )
        .bind(transaction_id)
        .fetch_optional(&self.pool)
        .await?;

        let tx_internal_id = match tx_result {
            Some(id) => id,
            None => return Ok(false), // Transaction doesn't exist
        };

        // Check if agent is in transaction_agents table
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM transaction_agents 
            WHERE transaction_id = ?1 AND agent_did = ?2
            "#,
        )
        .bind(tx_internal_id)
        .bind(agent_did)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    /// Insert a transaction agent
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - The reference ID of the transaction
    /// * `agent_did` - The DID of the agent
    /// * `agent_role` - The role of the agent (sender, receiver, compliance, other)
    ///
    /// # Returns
    ///
    /// * `Ok(())` on success
    /// * `Err(StorageError)` on database error
    pub async fn insert_transaction_agent(
        &self,
        transaction_id: &str,
        agent_did: &str,
        agent_role: &str,
    ) -> Result<(), StorageError> {
        // First get the transaction's internal ID
        let tx_result = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT id FROM transactions WHERE reference_id = ?1
            "#,
        )
        .bind(transaction_id)
        .fetch_optional(&self.pool)
        .await?;

        let tx_internal_id = match tx_result {
            Some(id) => id,
            None => {
                return Err(StorageError::NotFound(format!(
                    "Transaction {} not found",
                    transaction_id
                )))
            }
        };

        // Insert the agent
        sqlx::query(
            r#"
            INSERT INTO transaction_agents (transaction_id, agent_did, agent_role, status)
            VALUES (?1, ?2, ?3, 'pending')
            ON CONFLICT(transaction_id, agent_did) DO UPDATE SET
                agent_role = excluded.agent_role,
                updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
            "#,
        )
        .bind(tx_internal_id)
        .bind(agent_did)
        .bind(agent_role)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update transaction agent status
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - The reference ID of the transaction
    /// * `agent_did` - The DID of the agent
    /// * `status` - The new status (pending, authorized, rejected, cancelled)
    ///
    /// # Returns
    ///
    /// * `Ok(())` on success
    /// * `Err(StorageError)` on database error
    pub async fn update_transaction_agent_status(
        &self,
        transaction_id: &str,
        agent_did: &str,
        status: &str,
    ) -> Result<(), StorageError> {
        // First get the transaction's internal ID
        let tx_result = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT id FROM transactions WHERE reference_id = ?1
            "#,
        )
        .bind(transaction_id)
        .fetch_optional(&self.pool)
        .await?;

        let tx_internal_id = match tx_result {
            Some(id) => id,
            None => {
                return Err(StorageError::NotFound(format!(
                    "Transaction {} not found",
                    transaction_id
                )))
            }
        };

        // Update the agent status
        let result = sqlx::query(
            r#"
            UPDATE transaction_agents 
            SET status = ?1 
            WHERE transaction_id = ?2 AND agent_did = ?3
            "#,
        )
        .bind(status)
        .bind(tx_internal_id)
        .bind(agent_did)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!(
                "Agent {} not found for transaction {}",
                agent_did, transaction_id
            )));
        }

        Ok(())
    }

    /// Get all agents for a transaction
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - The reference ID of the transaction
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<(agent_did, agent_role, status)>)` on success
    /// * `Err(StorageError)` on database error
    pub async fn get_transaction_agents(
        &self,
        transaction_id: &str,
    ) -> Result<Vec<(String, String, String)>, StorageError> {
        // First get the transaction's internal ID
        let tx_result = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT id FROM transactions WHERE reference_id = ?1
            "#,
        )
        .bind(transaction_id)
        .fetch_optional(&self.pool)
        .await?;

        let tx_internal_id = match tx_result {
            Some(id) => id,
            None => {
                return Err(StorageError::NotFound(format!(
                    "Transaction {} not found",
                    transaction_id
                )))
            }
        };

        // Get all agents
        let agents = sqlx::query_as::<_, (String, String, String)>(
            r#"
            SELECT agent_did, agent_role, status
            FROM transaction_agents
            WHERE transaction_id = ?1
            ORDER BY created_at
            "#,
        )
        .bind(tx_internal_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(agents)
    }

    /// Check if all agents have authorized the transaction
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - The reference ID of the transaction
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if all agents have authorized
    /// * `Ok(false)` if any agent hasn't authorized or has rejected/cancelled
    /// * `Err(StorageError)` on database error
    pub async fn are_all_agents_authorized(
        &self,
        transaction_id: &str,
    ) -> Result<bool, StorageError> {
        // First get the transaction's internal ID
        let tx_result = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT id FROM transactions WHERE reference_id = ?1
            "#,
        )
        .bind(transaction_id)
        .fetch_optional(&self.pool)
        .await?;

        let tx_internal_id = match tx_result {
            Some(id) => id,
            None => return Ok(false), // Transaction doesn't exist
        };

        // Check if there are any agents not in 'authorized' status
        let non_authorized_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM transaction_agents 
            WHERE transaction_id = ?1 AND status != 'authorized'
            "#,
        )
        .bind(tx_internal_id)
        .fetch_one(&self.pool)
        .await?;

        // If there are no agents, transaction is ready to settle
        // If there are agents, all must be authorized
        Ok(non_authorized_count == 0)
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
        let message_json = serde_json::to_value(message)?;

        // Extract transaction type and use message ID as reference
        let message_type_lower = message.type_.to_lowercase();
        let tx_type = if message_type_lower.contains("transfer") {
            TransactionType::Transfer
        } else if message_type_lower.contains("payment") {
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

        debug!("Inserting transaction: {} ({})", reference_id, tx_type);

        let result = sqlx::query(
            r#"
            INSERT INTO transactions (type, reference_id, from_did, to_did, thread_id, message_type, message_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(tx_type.to_string())
        .bind(&reference_id)
        .bind(from_did)
        .bind(to_did)
        .bind(thread_id)
        .bind(message_type.to_string())
        .bind(sqlx::types::Json(message_json))
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!("Successfully inserted transaction: {}", reference_id);
                Ok(())
            }
            Err(sqlx::Error::Database(db_err)) => {
                if db_err.message().contains("UNIQUE") {
                    Err(StorageError::DuplicateTransaction(reference_id))
                } else {
                    Err(StorageError::Database(sqlx::Error::Database(db_err)))
                }
            }
            Err(e) => Err(StorageError::Database(e)),
        }
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
        let rows = sqlx::query_as::<_, (
            i64,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            String,
            String,
            serde_json::Value,
            String,
            String,
        )>(
            r#"
            SELECT id, type, reference_id, from_did, to_did, thread_id, message_type, status, message_json, created_at, updated_at
            FROM transactions
            ORDER BY created_at DESC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let mut transactions = Vec::new();
        for (
            id,
            tx_type,
            reference_id,
            from_did,
            to_did,
            thread_id,
            message_type,
            status,
            message_json,
            created_at,
            updated_at,
        ) in rows
        {
            transactions.push(Transaction {
                id,
                transaction_type: TransactionType::try_from(tx_type.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                reference_id,
                from_did,
                to_did,
                thread_id,
                message_type,
                status: TransactionStatus::try_from(status.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                message_json,
                created_at,
                updated_at,
            });
        }

        Ok(transactions)
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
        let message_json = serde_json::to_value(message)?;
        let message_id = message.id.clone();
        let message_type = message.type_.clone();
        let from_did = message.from.clone();
        let to_did = message.to.first().cloned();
        let thread_id = message.thid.clone();
        let parent_thread_id = message.pthid.clone();

        debug!(
            "Logging {} message: {} ({})",
            direction, message_id, message_type
        );

        let result = sqlx::query(
            r#"
            INSERT INTO messages (message_id, message_type, from_did, to_did, thread_id, parent_thread_id, direction, message_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
        )
        .bind(&message_id)
        .bind(message_type)
        .bind(from_did)
        .bind(to_did)
        .bind(thread_id)
        .bind(parent_thread_id)
        .bind(direction.to_string())
        .bind(sqlx::types::Json(message_json))
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!("Successfully logged message: {}", message_id);
                Ok(())
            }
            Err(sqlx::Error::Database(db_err)) => {
                if db_err.message().contains("UNIQUE") {
                    // Message already logged, this is fine
                    debug!("Message already logged: {}", message_id);
                    Ok(())
                } else {
                    Err(StorageError::Database(sqlx::Error::Database(db_err)))
                }
            }
            Err(e) => Err(StorageError::Database(e)),
        }
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
        let result = sqlx::query_as::<_, (
            i64,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            String,
            serde_json::Value,
            String,
        )>(
            r#"
            SELECT id, message_id, message_type, from_did, to_did, thread_id, parent_thread_id, direction, message_json, created_at
            FROM messages WHERE message_id = ?1
            "#,
        )
        .bind(message_id)
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some((
                id,
                message_id,
                message_type,
                from_did,
                to_did,
                thread_id,
                parent_thread_id,
                direction,
                message_json,
                created_at,
            )) => Ok(Some(Message {
                id,
                message_id,
                message_type,
                from_did,
                to_did,
                thread_id,
                parent_thread_id,
                direction: MessageDirection::try_from(direction.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                message_json,
                created_at,
            })),
            None => Ok(None),
        }
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
        let rows = if let Some(dir) = direction {
            sqlx::query_as::<_, (
                i64,
                String,
                String,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                String,
                serde_json::Value,
                String,
            )>(
                r#"
                SELECT id, message_id, message_type, from_did, to_did, thread_id, parent_thread_id, direction, message_json, created_at
                FROM messages
                WHERE direction = ?1
                ORDER BY created_at DESC
                LIMIT ?2 OFFSET ?3
                "#,
            )
            .bind(dir.to_string())
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, (
                i64,
                String,
                String,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                String,
                serde_json::Value,
                String,
            )>(
                r#"
                SELECT id, message_id, message_type, from_did, to_did, thread_id, parent_thread_id, direction, message_json, created_at
                FROM messages
                ORDER BY created_at DESC
                LIMIT ?1 OFFSET ?2
                "#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };

        let mut messages = Vec::new();
        for (
            id,
            message_id,
            message_type,
            from_did,
            to_did,
            thread_id,
            parent_thread_id,
            direction,
            message_json,
            created_at,
        ) in rows
        {
            messages.push(Message {
                id,
                message_id,
                message_type,
                from_did,
                to_did,
                thread_id,
                parent_thread_id,
                direction: MessageDirection::try_from(direction.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                message_json,
                created_at,
            });
        }

        Ok(messages)
    }

    /// Create a new delivery record
    ///
    /// # Arguments
    ///
    /// * `message_id` - The ID of the message being delivered
    /// * `message_text` - The full message text being delivered
    /// * `recipient_did` - The DID of the recipient
    /// * `delivery_url` - Optional URL where the message is being delivered
    /// * `delivery_type` - The type of delivery (https, internal, return_path, pickup)
    ///
    /// # Returns
    ///
    /// * `Ok(i64)` - The ID of the created delivery record
    /// * `Err(StorageError)` on database error
    pub async fn create_delivery(
        &self,
        message_id: &str,
        message_text: &str,
        recipient_did: &str,
        delivery_url: Option<&str>,
        delivery_type: DeliveryType,
    ) -> Result<i64, StorageError> {
        let result = sqlx::query(
            r#"
            INSERT INTO deliveries (message_id, message_text, recipient_did, delivery_url, delivery_type, status, retry_count)
            VALUES (?1, ?2, ?3, ?4, ?5, 'pending', 0)
            "#,
        )
        .bind(message_id)
        .bind(message_text)
        .bind(recipient_did)
        .bind(delivery_url)
        .bind(delivery_type.to_string())
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Update delivery status
    ///
    /// # Arguments
    ///
    /// * `delivery_id` - The ID of the delivery record
    /// * `status` - The new status (pending, success, failed)
    /// * `http_status_code` - Optional HTTP status code from delivery attempt
    /// * `error_message` - Optional error message if delivery failed
    ///
    /// # Returns
    ///
    /// * `Ok(())` on success
    /// * `Err(StorageError)` on database error
    pub async fn update_delivery_status(
        &self,
        delivery_id: i64,
        status: DeliveryStatus,
        http_status_code: Option<i32>,
        error_message: Option<&str>,
    ) -> Result<(), StorageError> {
        let now = chrono::Utc::now().to_rfc3339();
        let delivered_at = if status == DeliveryStatus::Success {
            Some(now.clone())
        } else {
            None
        };

        sqlx::query(
            r#"
            UPDATE deliveries 
            SET status = ?1, last_http_status_code = ?2, error_message = ?3, updated_at = ?4, delivered_at = ?5
            WHERE id = ?6
            "#,
        )
        .bind(status.to_string())
        .bind(http_status_code)
        .bind(error_message)
        .bind(now)
        .bind(delivered_at)
        .bind(delivery_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Increment retry count for a delivery
    ///
    /// # Arguments
    ///
    /// * `delivery_id` - The ID of the delivery record
    ///
    /// # Returns
    ///
    /// * `Ok(())` on success
    /// * `Err(StorageError)` on database error
    pub async fn increment_delivery_retry_count(
        &self,
        delivery_id: i64,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            UPDATE deliveries 
            SET retry_count = retry_count + 1, updated_at = ?1
            WHERE id = ?2
            "#,
        )
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(delivery_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get delivery record by ID
    ///
    /// # Arguments
    ///
    /// * `delivery_id` - The ID of the delivery record
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Delivery))` if found
    /// * `Ok(None)` if not found
    /// * `Err(StorageError)` on database error
    pub async fn get_delivery_by_id(
        &self,
        delivery_id: i64,
    ) -> Result<Option<Delivery>, StorageError> {
        let result = sqlx::query_as::<
            _,
            (
                i64,
                String,
                String,
                String,
                Option<String>,
                String,
                String,
                i32,
                Option<i32>,
                Option<String>,
                String,
                String,
                Option<String>,
            ),
        >(
            r#"
            SELECT id, message_id, message_text, recipient_did, delivery_url, delivery_type, status, retry_count, 
                   last_http_status_code, error_message, created_at, updated_at, delivered_at
            FROM deliveries WHERE id = ?1
            "#,
        )
        .bind(delivery_id)
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some((
                id,
                message_id,
                message_text,
                recipient_did,
                delivery_url,
                delivery_type,
                status,
                retry_count,
                last_http_status_code,
                error_message,
                created_at,
                updated_at,
                delivered_at,
            )) => Ok(Some(Delivery {
                id,
                message_id,
                message_text,
                recipient_did,
                delivery_url,
                delivery_type: DeliveryType::try_from(delivery_type.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                status: DeliveryStatus::try_from(status.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                retry_count,
                last_http_status_code,
                error_message,
                created_at,
                updated_at,
                delivered_at,
            })),
            None => Ok(None),
        }
    }

    /// Get all deliveries for a message
    ///
    /// # Arguments
    ///
    /// * `message_id` - The ID of the message
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Delivery>)` - List of deliveries for the message
    /// * `Err(StorageError)` on database error
    pub async fn get_deliveries_for_message(
        &self,
        message_id: &str,
    ) -> Result<Vec<Delivery>, StorageError> {
        let rows = sqlx::query_as::<
            _,
            (
                i64,
                String,
                String,
                String,
                Option<String>,
                String,
                String,
                i32,
                Option<i32>,
                Option<String>,
                String,
                String,
                Option<String>,
            ),
        >(
            r#"
            SELECT id, message_id, message_text, recipient_did, delivery_url, delivery_type, status, retry_count, 
                   last_http_status_code, error_message, created_at, updated_at, delivered_at
            FROM deliveries WHERE message_id = ?1
            ORDER BY created_at ASC
            "#,
        )
        .bind(message_id)
        .fetch_all(&self.pool)
        .await?;

        let mut deliveries = Vec::new();
        for (
            id,
            message_id,
            message_text,
            recipient_did,
            delivery_url,
            delivery_type,
            status,
            retry_count,
            last_http_status_code,
            error_message,
            created_at,
            updated_at,
            delivered_at,
        ) in rows
        {
            deliveries.push(Delivery {
                id,
                message_id,
                message_text,
                recipient_did,
                delivery_url,
                delivery_type: DeliveryType::try_from(delivery_type.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                status: DeliveryStatus::try_from(status.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                retry_count,
                last_http_status_code,
                error_message,
                created_at,
                updated_at,
                delivered_at,
            });
        }

        Ok(deliveries)
    }

    /// Get pending deliveries for retry processing
    ///
    /// # Arguments
    ///
    /// * `max_retry_count` - Maximum retry count to include
    /// * `limit` - Maximum number of deliveries to return
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Delivery>)` - List of pending deliveries
    /// * `Err(StorageError)` on database error
    pub async fn get_pending_deliveries(
        &self,
        max_retry_count: i32,
        limit: u32,
    ) -> Result<Vec<Delivery>, StorageError> {
        let rows = sqlx::query_as::<
            _,
            (
                i64,
                String,
                String,
                String,
                Option<String>,
                String,
                String,
                i32,
                Option<i32>,
                Option<String>,
                String,
                String,
                Option<String>,
            ),
        >(
            r#"
            SELECT id, message_id, message_text, recipient_did, delivery_url, delivery_type, status, retry_count, 
                   last_http_status_code, error_message, created_at, updated_at, delivered_at
            FROM deliveries 
            WHERE status = 'pending' AND retry_count < ?1
            ORDER BY created_at ASC
            LIMIT ?2
            "#,
        )
        .bind(max_retry_count)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut deliveries = Vec::new();
        for (
            id,
            message_id,
            message_text,
            recipient_did,
            delivery_url,
            delivery_type,
            status,
            retry_count,
            last_http_status_code,
            error_message,
            created_at,
            updated_at,
            delivered_at,
        ) in rows
        {
            deliveries.push(Delivery {
                id,
                message_id,
                message_text,
                recipient_did,
                delivery_url,
                delivery_type: DeliveryType::try_from(delivery_type.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                status: DeliveryStatus::try_from(status.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                retry_count,
                last_http_status_code,
                error_message,
                created_at,
                updated_at,
                delivered_at,
            });
        }

        Ok(deliveries)
    }

    /// Get failed deliveries for a specific recipient
    ///
    /// # Arguments
    ///
    /// * `recipient_did` - The DID of the recipient
    /// * `limit` - Maximum number of deliveries to return
    /// * `offset` - Number of deliveries to skip (for pagination)
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Delivery>)` - List of failed deliveries
    /// * `Err(StorageError)` on database error
    pub async fn get_failed_deliveries_for_recipient(
        &self,
        recipient_did: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Delivery>, StorageError> {
        let rows = sqlx::query_as::<
            _,
            (
                i64,
                String,
                String,
                String,
                Option<String>,
                String,
                String,
                i32,
                Option<i32>,
                Option<String>,
                String,
                String,
                Option<String>,
            ),
        >(
            r#"
            SELECT id, message_id, message_text, recipient_did, delivery_url, delivery_type, status, retry_count, 
                   last_http_status_code, error_message, created_at, updated_at, delivered_at
            FROM deliveries 
            WHERE recipient_did = ?1 AND status = 'failed'
            ORDER BY updated_at DESC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(recipient_did)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let mut deliveries = Vec::new();
        for (
            id,
            message_id,
            message_text,
            recipient_did,
            delivery_url,
            delivery_type,
            status,
            retry_count,
            last_http_status_code,
            error_message,
            created_at,
            updated_at,
            delivered_at,
        ) in rows
        {
            deliveries.push(Delivery {
                id,
                message_id,
                message_text,
                recipient_did,
                delivery_url,
                delivery_type: DeliveryType::try_from(delivery_type.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                status: DeliveryStatus::try_from(status.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                retry_count,
                last_http_status_code,
                error_message,
                created_at,
                updated_at,
                delivered_at,
            });
        }

        Ok(deliveries)
    }

    /// Get all deliveries for a specific recipient
    ///
    /// # Arguments
    ///
    /// * `recipient_did` - The DID of the recipient
    /// * `limit` - Maximum number of deliveries to return
    /// * `offset` - Number of deliveries to skip (for pagination)
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Delivery>)` - List of deliveries
    /// * `Err(StorageError)` on database error
    pub async fn get_deliveries_by_recipient(
        &self,
        recipient_did: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Delivery>, StorageError> {
        let rows = sqlx::query_as::<
            _,
            (
                i64,
                String,
                String,
                String,
                Option<String>,
                String,
                String,
                i32,
                Option<i32>,
                Option<String>,
                String,
                String,
                Option<String>,
            ),
        >(
            r#"
            SELECT id, message_id, message_text, recipient_did, delivery_url, delivery_type, status, retry_count, 
                   last_http_status_code, error_message, created_at, updated_at, delivered_at
            FROM deliveries 
            WHERE recipient_did = ?1
            ORDER BY created_at DESC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(recipient_did)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let mut deliveries = Vec::new();
        for (
            id,
            message_id,
            message_text,
            recipient_did,
            delivery_url,
            delivery_type,
            status,
            retry_count,
            last_http_status_code,
            error_message,
            created_at,
            updated_at,
            delivered_at,
        ) in rows
        {
            deliveries.push(Delivery {
                id,
                message_id,
                message_text,
                recipient_did,
                delivery_url,
                delivery_type: delivery_type
                    .parse::<DeliveryType>()
                    .unwrap_or(DeliveryType::Internal),
                status: status
                    .parse::<DeliveryStatus>()
                    .unwrap_or(DeliveryStatus::Pending),
                retry_count,
                last_http_status_code,
                error_message,
                created_at,
                updated_at,
                delivered_at,
            });
        }

        Ok(deliveries)
    }

    /// Get all deliveries for messages in a specific thread
    ///
    /// # Arguments
    ///
    /// * `thread_id` - The thread ID to search for
    /// * `limit` - Maximum number of deliveries to return
    /// * `offset` - Number of deliveries to skip (for pagination)
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Delivery>)` - List of deliveries for messages in the thread
    /// * `Err(StorageError)` on database error
    pub async fn get_deliveries_for_thread(
        &self,
        thread_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Delivery>, StorageError> {
        let rows = sqlx::query_as::<
            _,
            (
                i64,
                String,
                String,
                String,
                Option<String>,
                String,
                String,
                i32,
                Option<i32>,
                Option<String>,
                String,
                String,
                Option<String>,
            ),
        >(
            r#"
            SELECT d.id, d.message_id, d.message_text, d.recipient_did, d.delivery_url, 
                   d.delivery_type, d.status, d.retry_count, d.last_http_status_code, 
                   d.error_message, d.created_at, d.updated_at, d.delivered_at
            FROM deliveries d
            INNER JOIN messages m ON d.message_id = m.message_id
            WHERE m.thread_id = ?1
            ORDER BY d.created_at ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(thread_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let mut deliveries = Vec::new();
        for (
            id,
            message_id,
            message_text,
            recipient_did,
            delivery_url,
            delivery_type,
            status,
            retry_count,
            last_http_status_code,
            error_message,
            created_at,
            updated_at,
            delivered_at,
        ) in rows
        {
            deliveries.push(Delivery {
                id,
                message_id,
                message_text,
                recipient_did,
                delivery_url,
                delivery_type: delivery_type
                    .parse::<DeliveryType>()
                    .unwrap_or(DeliveryType::Internal),
                status: status
                    .parse::<DeliveryStatus>()
                    .unwrap_or(DeliveryStatus::Pending),
                retry_count,
                last_http_status_code,
                error_message,
                created_at,
                updated_at,
                delivered_at,
            });
        }

        Ok(deliveries)
    }

    /// Create a new received message record
    ///
    /// This records a raw incoming message (JWE, JWS, or plain JSON) before processing.
    ///
    /// # Arguments
    ///
    /// * `raw_message` - The raw message content as received
    /// * `source_type` - The type of source (https, internal, websocket, etc.)
    /// * `source_identifier` - Optional identifier for the source (URL, agent DID, etc.)
    ///
    /// # Returns
    ///
    /// * `Ok(i64)` - The ID of the created record
    /// * `Err(StorageError)` on database error
    pub async fn create_received(
        &self,
        raw_message: &str,
        source_type: SourceType,
        source_identifier: Option<&str>,
    ) -> Result<i64, StorageError> {
        // Try to extract message ID from the raw message
        let message_id =
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(raw_message) {
                json_value
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            } else {
                None
            };

        let result = sqlx::query(
            r#"
            INSERT INTO received (message_id, raw_message, source_type, source_identifier)
            VALUES (?1, ?2, ?3, ?4)
            "#,
        )
        .bind(message_id)
        .bind(raw_message)
        .bind(source_type.to_string())
        .bind(source_identifier)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Update the status of a received message
    ///
    /// # Arguments
    ///
    /// * `received_id` - The ID of the received record
    /// * `status` - The new status (processed, failed)
    /// * `processed_message_id` - Optional ID of the processed message in the messages table
    /// * `error_message` - Optional error message if processing failed
    ///
    /// # Returns
    ///
    /// * `Ok(())` on success
    /// * `Err(StorageError)` on database error
    pub async fn update_received_status(
        &self,
        received_id: i64,
        status: ReceivedStatus,
        processed_message_id: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<(), StorageError> {
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE received 
            SET status = ?1, processed_at = ?2, processed_message_id = ?3, error_message = ?4
            WHERE id = ?5
            "#,
        )
        .bind(status.to_string())
        .bind(&now)
        .bind(processed_message_id)
        .bind(error_message)
        .bind(received_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get a received message by ID
    ///
    /// # Arguments
    ///
    /// * `received_id` - The ID of the received record
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Received))` if found
    /// * `Ok(None)` if not found
    /// * `Err(StorageError)` on database error
    pub async fn get_received_by_id(
        &self,
        received_id: i64,
    ) -> Result<Option<Received>, StorageError> {
        let result = sqlx::query_as::<
            _,
            (
                i64,
                Option<String>,
                String,
                String,
                Option<String>,
                String,
                Option<String>,
                String,
                Option<String>,
                Option<String>,
            ),
        >(
            r#"
            SELECT id, message_id, raw_message, source_type, source_identifier, 
                   status, error_message, received_at, processed_at, processed_message_id
            FROM received WHERE id = ?1
            "#,
        )
        .bind(received_id)
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some((
                id,
                message_id,
                raw_message,
                source_type,
                source_identifier,
                status,
                error_message,
                received_at,
                processed_at,
                processed_message_id,
            )) => Ok(Some(Received {
                id,
                message_id,
                raw_message,
                source_type: SourceType::try_from(source_type.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                source_identifier,
                status: ReceivedStatus::try_from(status.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                error_message,
                received_at,
                processed_at,
                processed_message_id,
            })),
            None => Ok(None),
        }
    }

    /// Get pending received messages for processing
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of messages to return
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Received>)` - List of pending received messages
    /// * `Err(StorageError)` on database error
    pub async fn get_pending_received(&self, limit: u32) -> Result<Vec<Received>, StorageError> {
        let rows = sqlx::query_as::<
            _,
            (
                i64,
                Option<String>,
                String,
                String,
                Option<String>,
                String,
                Option<String>,
                String,
                Option<String>,
                Option<String>,
            ),
        >(
            r#"
            SELECT id, message_id, raw_message, source_type, source_identifier, 
                   status, error_message, received_at, processed_at, processed_message_id
            FROM received 
            WHERE status = 'pending'
            ORDER BY received_at ASC
            LIMIT ?1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut received_messages = Vec::new();
        for (
            id,
            message_id,
            raw_message,
            source_type,
            source_identifier,
            status,
            error_message,
            received_at,
            processed_at,
            processed_message_id,
        ) in rows
        {
            received_messages.push(Received {
                id,
                message_id,
                raw_message,
                source_type: SourceType::try_from(source_type.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                source_identifier,
                status: ReceivedStatus::try_from(status.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                error_message,
                received_at,
                processed_at,
                processed_message_id,
            });
        }

        Ok(received_messages)
    }

    /// List received messages with optional filtering
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of messages to return
    /// * `offset` - Number of messages to skip (for pagination)
    /// * `source_type` - Optional filter by source type
    /// * `status` - Optional filter by status
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Received>)` - List of received messages
    /// * `Err(StorageError)` on database error
    pub async fn list_received(
        &self,
        limit: u32,
        offset: u32,
        source_type: Option<SourceType>,
        status: Option<ReceivedStatus>,
    ) -> Result<Vec<Received>, StorageError> {
        let mut query = "SELECT id, message_id, raw_message, source_type, source_identifier, status, error_message, received_at, processed_at, processed_message_id FROM received WHERE 1=1".to_string();
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(st) = source_type {
            query.push_str(" AND source_type = ?");
            bind_values.push(st.to_string());
        }

        if let Some(s) = status {
            query.push_str(" AND status = ?");
            bind_values.push(s.to_string());
        }

        query.push_str(" ORDER BY received_at DESC LIMIT ? OFFSET ?");

        // Build the query dynamically based on filters
        let mut sqlx_query = sqlx::query_as::<
            _,
            (
                i64,
                Option<String>,
                String,
                String,
                Option<String>,
                String,
                Option<String>,
                String,
                Option<String>,
                Option<String>,
            ),
        >(&query);

        for value in bind_values {
            sqlx_query = sqlx_query.bind(value);
        }

        let rows = sqlx_query
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let mut received_messages = Vec::new();
        for (
            id,
            message_id,
            raw_message,
            source_type,
            source_identifier,
            status,
            error_message,
            received_at,
            processed_at,
            processed_message_id,
        ) in rows
        {
            received_messages.push(Received {
                id,
                message_id,
                raw_message,
                source_type: SourceType::try_from(source_type.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                source_identifier,
                status: ReceivedStatus::try_from(status.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                error_message,
                received_at,
                processed_at,
                processed_message_id,
            });
        }

        Ok(received_messages)
    }

    // Customer Management Methods

    /// Create or update a customer record
    pub async fn upsert_customer(&self, customer: &Customer) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO customers (
                id, agent_did, schema_type, given_name, family_name, display_name,
                legal_name, lei_code, mcc_code, address_country, address_locality,
                postal_code, street_address, profile, ivms101_data, verified_at,
                created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18
            ) ON CONFLICT(id) DO UPDATE SET
                agent_did = excluded.agent_did,
                schema_type = excluded.schema_type,
                given_name = excluded.given_name,
                family_name = excluded.family_name,
                display_name = excluded.display_name,
                legal_name = excluded.legal_name,
                lei_code = excluded.lei_code,
                mcc_code = excluded.mcc_code,
                address_country = excluded.address_country,
                address_locality = excluded.address_locality,
                postal_code = excluded.postal_code,
                street_address = excluded.street_address,
                profile = excluded.profile,
                ivms101_data = excluded.ivms101_data,
                verified_at = excluded.verified_at,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&customer.id)
        .bind(&customer.agent_did)
        .bind(customer.schema_type.to_string())
        .bind(&customer.given_name)
        .bind(&customer.family_name)
        .bind(&customer.display_name)
        .bind(&customer.legal_name)
        .bind(&customer.lei_code)
        .bind(&customer.mcc_code)
        .bind(&customer.address_country)
        .bind(&customer.address_locality)
        .bind(&customer.postal_code)
        .bind(&customer.street_address)
        .bind(serde_json::to_string(&customer.profile).unwrap())
        .bind(
            customer
                .ivms101_data
                .as_ref()
                .map(|v| serde_json::to_string(v).unwrap()),
        )
        .bind(&customer.verified_at)
        .bind(&customer.created_at)
        .bind(&customer.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get a customer by ID
    pub async fn get_customer(&self, customer_id: &str) -> Result<Option<Customer>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT id, agent_did, schema_type, given_name, family_name, display_name,
                   legal_name, lei_code, mcc_code, address_country, address_locality,
                   postal_code, street_address, profile, ivms101_data, verified_at,
                   created_at, updated_at
            FROM customers
            WHERE id = ?1
            "#,
        )
        .bind(customer_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(Customer {
                id: row.get("id"),
                agent_did: row.get("agent_did"),
                schema_type: SchemaType::try_from(row.get::<String, _>("schema_type").as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                given_name: row.get("given_name"),
                family_name: row.get("family_name"),
                display_name: row.get("display_name"),
                legal_name: row.get("legal_name"),
                lei_code: row.get("lei_code"),
                mcc_code: row.get("mcc_code"),
                address_country: row.get("address_country"),
                address_locality: row.get("address_locality"),
                postal_code: row.get("postal_code"),
                street_address: row.get("street_address"),
                profile: serde_json::from_str(&row.get::<String, _>("profile")).unwrap(),
                ivms101_data: row
                    .get::<Option<String>, _>("ivms101_data")
                    .map(|v| serde_json::from_str(&v).unwrap()),
                verified_at: row.get("verified_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })),
            None => Ok(None),
        }
    }

    /// Get a customer by identifier
    pub async fn get_customer_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<Option<Customer>, StorageError> {
        let row = sqlx::query_as::<_, (String,)>(
            r#"
            SELECT customer_id
            FROM customer_identifiers
            WHERE id = ?1
            "#,
        )
        .bind(identifier)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((customer_id,)) => self.get_customer(&customer_id).await,
            None => Ok(None),
        }
    }

    /// List customers for an agent
    pub async fn list_customers(
        &self,
        agent_did: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Customer>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT id, agent_did, schema_type, given_name, family_name, display_name,
                   legal_name, lei_code, mcc_code, address_country, address_locality,
                   postal_code, street_address, profile, ivms101_data, verified_at,
                   created_at, updated_at
            FROM customers
            WHERE agent_did = ?1
            ORDER BY updated_at DESC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(agent_did)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let mut customers = Vec::new();
        for row in rows {
            customers.push(Customer {
                id: row.get("id"),
                agent_did: row.get("agent_did"),
                schema_type: SchemaType::try_from(row.get::<String, _>("schema_type").as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                given_name: row.get("given_name"),
                family_name: row.get("family_name"),
                display_name: row.get("display_name"),
                legal_name: row.get("legal_name"),
                lei_code: row.get("lei_code"),
                mcc_code: row.get("mcc_code"),
                address_country: row.get("address_country"),
                address_locality: row.get("address_locality"),
                postal_code: row.get("postal_code"),
                street_address: row.get("street_address"),
                profile: serde_json::from_str(&row.get::<String, _>("profile")).unwrap(),
                ivms101_data: row
                    .get::<Option<String>, _>("ivms101_data")
                    .map(|v| serde_json::from_str(&v).unwrap()),
                verified_at: row.get("verified_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(customers)
    }

    /// Add an identifier to a customer
    pub async fn add_customer_identifier(
        &self,
        identifier: &CustomerIdentifier,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO customer_identifiers (
                id, customer_id, identifier_type, verified, verification_method,
                verified_at, created_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7
            ) ON CONFLICT(id, customer_id) DO UPDATE SET
                verified = excluded.verified,
                verification_method = excluded.verification_method,
                verified_at = excluded.verified_at
            "#,
        )
        .bind(&identifier.id)
        .bind(&identifier.customer_id)
        .bind(identifier.identifier_type.to_string())
        .bind(identifier.verified)
        .bind(&identifier.verification_method)
        .bind(&identifier.verified_at)
        .bind(&identifier.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get identifiers for a customer
    pub async fn get_customer_identifiers(
        &self,
        customer_id: &str,
    ) -> Result<Vec<CustomerIdentifier>, StorageError> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                bool,
                Option<String>,
                Option<String>,
                String,
            ),
        >(
            r#"
            SELECT id, customer_id, identifier_type, verified, verification_method,
                   verified_at, created_at
            FROM customer_identifiers
            WHERE customer_id = ?1
            "#,
        )
        .bind(customer_id)
        .fetch_all(&self.pool)
        .await?;

        let mut identifiers = Vec::new();
        for (
            id,
            customer_id,
            identifier_type,
            verified,
            verification_method,
            verified_at,
            created_at,
        ) in rows
        {
            identifiers.push(CustomerIdentifier {
                id,
                customer_id,
                identifier_type: IdentifierType::try_from(identifier_type.as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                verified,
                verification_method,
                verified_at,
                created_at,
            });
        }

        Ok(identifiers)
    }

    /// Add a customer relationship
    pub async fn add_customer_relationship(
        &self,
        relationship: &CustomerRelationship,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO customer_relationships (
                id, customer_id, relationship_type, related_identifier,
                proof, confirmed_at, created_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7
            ) ON CONFLICT(customer_id, relationship_type, related_identifier) DO UPDATE SET
                proof = excluded.proof,
                confirmed_at = excluded.confirmed_at
            "#,
        )
        .bind(&relationship.id)
        .bind(&relationship.customer_id)
        .bind(&relationship.relationship_type)
        .bind(&relationship.related_identifier)
        .bind(
            relationship
                .proof
                .as_ref()
                .map(|v| serde_json::to_string(v).unwrap()),
        )
        .bind(&relationship.confirmed_at)
        .bind(&relationship.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get relationships for a customer
    pub async fn get_customer_relationships(
        &self,
        customer_id: &str,
    ) -> Result<Vec<CustomerRelationship>, StorageError> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                Option<String>,
                Option<String>,
                String,
            ),
        >(
            r#"
            SELECT id, customer_id, relationship_type, related_identifier,
                   proof, confirmed_at, created_at
            FROM customer_relationships
            WHERE customer_id = ?1
            "#,
        )
        .bind(customer_id)
        .fetch_all(&self.pool)
        .await?;

        let mut relationships = Vec::new();
        for (
            id,
            customer_id,
            relationship_type,
            related_identifier,
            proof,
            confirmed_at,
            created_at,
        ) in rows
        {
            relationships.push(CustomerRelationship {
                id,
                customer_id,
                relationship_type,
                related_identifier,
                proof: proof.map(|v| serde_json::from_str(&v).unwrap()),
                confirmed_at,
                created_at,
            });
        }

        Ok(relationships)
    }

    /// Search customers by name or identifier
    pub async fn search_customers(
        &self,
        agent_did: &str,
        query: &str,
        limit: u32,
    ) -> Result<Vec<Customer>, StorageError> {
        let search_pattern = format!("%{}%", query);

        let rows = sqlx::query(
            r#"
            SELECT DISTINCT c.id, c.agent_did, c.schema_type, c.given_name, c.family_name, c.display_name,
                   c.legal_name, c.lei_code, c.mcc_code, c.address_country, c.address_locality,
                   c.postal_code, c.street_address, c.profile, c.ivms101_data, c.verified_at,
                   c.created_at, c.updated_at
            FROM customers c
            LEFT JOIN customer_identifiers ci ON c.id = ci.customer_id
            WHERE c.agent_did = ?1
            AND (
                c.given_name LIKE ?2
                OR c.family_name LIKE ?2
                OR c.display_name LIKE ?2
                OR c.legal_name LIKE ?2
                OR ci.id LIKE ?2
            )
            ORDER BY c.updated_at DESC
            LIMIT ?3
            "#,
        )
        .bind(agent_did)
        .bind(&search_pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut customers = Vec::new();
        for row in rows {
            customers.push(Customer {
                id: row.get("id"),
                agent_did: row.get("agent_did"),
                schema_type: SchemaType::try_from(row.get::<String, _>("schema_type").as_str())
                    .map_err(StorageError::InvalidTransactionType)?,
                given_name: row.get("given_name"),
                family_name: row.get("family_name"),
                display_name: row.get("display_name"),
                legal_name: row.get("legal_name"),
                lei_code: row.get("lei_code"),
                mcc_code: row.get("mcc_code"),
                address_country: row.get("address_country"),
                address_locality: row.get("address_locality"),
                postal_code: row.get("postal_code"),
                street_address: row.get("street_address"),
                profile: serde_json::from_str(&row.get::<String, _>("profile")).unwrap(),
                ivms101_data: row
                    .get::<Option<String>, _>("ivms101_data")
                    .map(|v| serde_json::from_str(&v).unwrap()),
                verified_at: row.get("verified_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(customers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tap_msg::message::transfer::Transfer;
    use tap_msg::message::Party;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_storage_creation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        let _storage = Storage::new(Some(db_path)).await.unwrap();
        // Just verify we can create a storage instance
    }

    #[tokio::test]
    async fn test_storage_with_did() {
        let _ = env_logger::builder().is_test(true).try_init();

        let dir = tempdir().unwrap();
        let tap_root = dir.path().to_path_buf();
        let agent_did = "did:web:example.com";

        let storage = Storage::new_with_did(agent_did, Some(tap_root.clone()))
            .await
            .unwrap();

        // Verify the database was created in the expected location
        let expected_path = tap_root.join("did_web_example.com").join("transactions.db");
        assert!(
            expected_path.exists(),
            "Database file not created at expected path"
        );

        // Test that we can use the storage
        let messages = storage.list_messages(10, 0, None).await.unwrap();
        assert_eq!(messages.len(), 0);
    }

    #[tokio::test]
    async fn test_default_logs_dir() {
        let dir = tempdir().unwrap();
        let tap_root = dir.path().to_path_buf();

        let logs_dir = Storage::default_logs_dir(Some(tap_root.clone()));
        assert_eq!(logs_dir, tap_root.join("logs"));

        // Test with no tap_root (should use home dir)
        let default_logs = Storage::default_logs_dir(None);
        assert!(default_logs.to_string_lossy().contains(".tap/logs"));
    }

    #[tokio::test]
    async fn test_insert_and_retrieve_transaction() {
        let _ = env_logger::builder().is_test(true).try_init();

        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = Storage::new(Some(db_path)).await.unwrap();

        // Create a test transfer message
        let transfer_body = Transfer {
            transaction_id: Some("test_transfer_123".to_string()),
            originator: Some(Party::new("did:example:originator")),
            beneficiary: Some(Party::new("did:example:beneficiary")),
            asset: "eip155:1/erc20:0x0000000000000000000000000000000000000000"
                .parse()
                .unwrap(),
            amount: "1000000000000000000".to_string(),
            agents: vec![],
            memo: None,
            settlement_id: None,
            connection_id: None,
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
