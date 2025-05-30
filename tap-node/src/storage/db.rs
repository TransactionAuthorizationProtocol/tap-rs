use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::env;
use std::path::PathBuf;
use tap_msg::didcomm::PlainMessage;
use tracing::{debug, info};

use super::error::StorageError;
use super::models::{Message, MessageDirection, Transaction, TransactionStatus, TransactionType};

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
#[derive(Clone)]
pub struct Storage {
    pool: SqlitePool,
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
            env::var("TAP_ROOT").map(PathBuf::from).unwrap_or_else(|_| {
                dirs::home_dir()
                    .expect("Could not find home directory")
                    .join(".tap")
            })
        });

        // Sanitize the DID for use as a directory name
        let sanitized_did = agent_did.replace(':', "_");
        let db_path = root_dir.join(&sanitized_did).join("transactions.db");

        Self::new(Some(db_path)).await
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

        Ok(Storage { pool })
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
            env::var("TAP_ROOT").map(PathBuf::from).unwrap_or_else(|_| {
                dirs::home_dir()
                    .expect("Could not find home directory")
                    .join(".tap")
            })
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
    /// * `raw_message` - Optional raw JWE/JWS message string
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
        raw_message: Option<&str>,
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
            INSERT INTO messages (message_id, message_type, from_did, to_did, thread_id, parent_thread_id, direction, message_json, raw_message)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
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
        .bind(raw_message)
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
            .log_message(&connect_message, MessageDirection::Incoming, None)
            .await
            .unwrap();
        storage
            .log_message(&authorize_message, MessageDirection::Outgoing, None)
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
            .log_message(&connect_message, MessageDirection::Incoming, None)
            .await
            .unwrap();
        let all_messages_after = storage.list_messages(10, 0, None).await.unwrap();
        assert_eq!(all_messages_after.len(), 2); // Should still be 2, not 3
    }
}
