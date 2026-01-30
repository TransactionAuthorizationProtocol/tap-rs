//! Key Storage Abstraction Layer
//!
//! This module defines the [`KeyStore`] trait for future integration with
//! external key management systems.
//!
//! # Current Implementation
//!
//! Currently, keys are stored as plaintext JSON in `~/.tap/keys.json`.
//! This is suitable for development and testing but **NOT recommended for production**
//! deployments with high-value keys.
//!
//! The current storage flow:
//! 1. Keys are generated or imported into [`AgentKeyManager`](crate::AgentKeyManager)
//! 2. Keys are serialized to [`StoredKey`](crate::StoredKey) format (base64-encoded key material)
//! 3. [`KeyStorage`](crate::KeyStorage) writes JSON to disk at the configured path
//!
//! # Security Considerations
//!
//! The plaintext storage has the following security properties:
//! - **No encryption at rest**: Key material is base64-encoded but not encrypted
//! - **File permissions**: Relies on OS file permissions for access control
//! - **Portable**: Keys can be easily backed up or transferred
//!
//! For production deployments, consider implementing a [`KeyStore`] backend that provides:
//! - Encryption at rest (e.g., envelope encryption with master key)
//! - Hardware security module (HSM) integration
//! - Cloud key management service (KMS) integration
//! - Platform keychain integration
//!
//! # Future External Key Management
//!
//! The [`KeyStore`] trait provides an abstraction that can be implemented for various backends:
//!
//! ## Hardware Security Modules (HSMs)
//! - AWS CloudHSM
//! - Azure Dedicated HSM
//! - Thales Luna HSM
//!
//! ## Cloud Key Management Services
//! - AWS KMS
//! - Google Cloud KMS
//! - Azure Key Vault
//! - HashiCorp Vault
//!
//! ## Platform Keychains
//! - macOS Keychain (Security.framework)
//! - Windows DPAPI / Credential Manager
//! - Linux Secret Service (libsecret)
//!
//! # Implementation Guide
//!
//! To implement a custom key store backend:
//!
//! 1. Implement the [`KeyStore`] trait for your backend
//! 2. Handle key material serialization appropriate for your backend
//! 3. Implement proper error handling for network/hardware failures
//! 4. Consider caching strategies for performance
//!
//! ## Example: HashiCorp Vault Integration
//!
//! ```rust,ignore
//! use tap_agent::key_store::{KeyStore, KeyStoreError};
//! use async_trait::async_trait;
//!
//! pub struct VaultKeyStore {
//!     client: vault::Client,
//!     mount_path: String,
//! }
//!
//! impl VaultKeyStore {
//!     pub fn new(addr: &str, token: &str) -> Result<Self, Box<dyn std::error::Error>> {
//!         let client = vault::Client::new(addr, token)?;
//!         Ok(Self {
//!             client,
//!             mount_path: "secret/tap-keys".to_string(),
//!         })
//!     }
//! }
//!
//! #[async_trait]
//! impl KeyStore for VaultKeyStore {
//!     async fn store_key(&self, id: &str, material: &[u8]) -> Result<(), KeyStoreError> {
//!         let path = format!("{}/{}", self.mount_path, id);
//!         let data = base64::encode(material);
//!         self.client.secrets().kv2().set(&path, &[("key", &data)]).await
//!             .map_err(|e| KeyStoreError::Storage(e.to_string()))?;
//!         Ok(())
//!     }
//!
//!     async fn load_key(&self, id: &str) -> Result<Vec<u8>, KeyStoreError> {
//!         let path = format!("{}/{}", self.mount_path, id);
//!         let secret = self.client.secrets().kv2().get(&path).await
//!             .map_err(|e| KeyStoreError::NotFound(id.to_string()))?;
//!         let data = secret.data.get("key")
//!             .ok_or_else(|| KeyStoreError::InvalidFormat("Missing key field".to_string()))?;
//!         base64::decode(data)
//!             .map_err(|e| KeyStoreError::InvalidFormat(e.to_string()))
//!     }
//!
//!     async fn delete_key(&self, id: &str) -> Result<(), KeyStoreError> {
//!         let path = format!("{}/{}", self.mount_path, id);
//!         self.client.secrets().kv2().delete(&path).await
//!             .map_err(|e| KeyStoreError::Storage(e.to_string()))?;
//!         Ok(())
//!     }
//!
//!     async fn key_exists(&self, id: &str) -> Result<bool, KeyStoreError> {
//!         let path = format!("{}/{}", self.mount_path, id);
//!         match self.client.secrets().kv2().get(&path).await {
//!             Ok(_) => Ok(true),
//!             Err(_) => Ok(false),
//!         }
//!     }
//!
//!     async fn list_keys(&self) -> Result<Vec<String>, KeyStoreError> {
//!         self.client.secrets().kv2().list(&self.mount_path).await
//!             .map_err(|e| KeyStoreError::Storage(e.to_string()))
//!     }
//! }
//! ```
//!
//! ## Integration with AgentKeyManager
//!
//! Future versions will support passing a custom [`KeyStore`] to the
//! [`AgentKeyManagerBuilder`](crate::AgentKeyManagerBuilder):
//!
//! ```rust,ignore
//! let vault_store = VaultKeyStore::new("https://vault.example.com", token)?;
//!
//! let key_manager = AgentKeyManagerBuilder::new()
//!     .with_key_store(Box::new(vault_store))
//!     .build()?;
//! ```

use async_trait::async_trait;

/// Error types for key storage operations
#[derive(Debug, thiserror::Error)]
pub enum KeyStoreError {
    /// The requested key was not found
    #[error("Key not found: {0}")]
    NotFound(String),

    /// A storage backend error occurred
    #[error("Storage error: {0}")]
    Storage(String),

    /// Access to the key was denied
    #[error("Access denied: {0}")]
    AccessDenied(String),

    /// The key material format is invalid
    #[error("Invalid key format: {0}")]
    InvalidFormat(String),

    /// The storage backend is unavailable
    #[error("Backend unavailable: {0}")]
    Unavailable(String),
}

/// Trait for key storage backends
///
/// Implement this trait to integrate with external key management systems.
/// All operations are async to support network-based backends (HSMs, cloud KMS, etc.).
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to allow use across async tasks.
///
/// # Error Handling
///
/// Implementations should:
/// - Return `KeyStoreError::NotFound` for missing keys (not a general error)
/// - Return `KeyStoreError::AccessDenied` for permission issues
/// - Return `KeyStoreError::Unavailable` for transient failures (enable retry logic)
/// - Return `KeyStoreError::Storage` for other backend errors
#[async_trait]
pub trait KeyStore: Send + Sync {
    /// Store key material with the given identifier
    ///
    /// If a key with the same ID already exists, it should be overwritten.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the key (typically a DID or key ID)
    /// * `material` - Raw key material (private key bytes)
    async fn store_key(&self, id: &str, material: &[u8]) -> Result<(), KeyStoreError>;

    /// Load key material by identifier
    ///
    /// # Arguments
    /// * `id` - The key identifier
    ///
    /// # Returns
    /// The raw key material, or `KeyStoreError::NotFound` if the key doesn't exist
    async fn load_key(&self, id: &str) -> Result<Vec<u8>, KeyStoreError>;

    /// Delete a key by identifier
    ///
    /// # Arguments
    /// * `id` - The key identifier
    ///
    /// # Returns
    /// `Ok(())` if the key was deleted or didn't exist
    async fn delete_key(&self, id: &str) -> Result<(), KeyStoreError>;

    /// Check if a key exists
    ///
    /// # Arguments
    /// * `id` - The key identifier
    ///
    /// # Returns
    /// `true` if the key exists, `false` otherwise
    async fn key_exists(&self, id: &str) -> Result<bool, KeyStoreError>;

    /// List all key identifiers
    ///
    /// # Returns
    /// A vector of all key IDs in the store
    async fn list_keys(&self) -> Result<Vec<String>, KeyStoreError>;
}

/// Plaintext file-based key store
///
/// **WARNING**: This implementation stores keys as plaintext JSON.
/// It wraps the existing [`KeyStorage`](crate::KeyStorage) implementation
/// for backwards compatibility.
///
/// Use only for development and testing. For production deployments,
/// implement a secure [`KeyStore`] backend with encryption at rest.
#[derive(Debug, Default)]
pub struct PlaintextFileKeyStore {
    /// Path to the key storage file
    path: Option<std::path::PathBuf>,
}

impl PlaintextFileKeyStore {
    /// Create a new plaintext key store using the default path (~/.tap/keys.json)
    pub fn new() -> Self {
        Self { path: None }
    }

    /// Create a new plaintext key store at a specific path
    pub fn with_path(path: std::path::PathBuf) -> Self {
        Self { path: Some(path) }
    }
}

// Note: Full KeyStore implementation for PlaintextFileKeyStore would wrap
// the existing KeyStorage functionality. This is left as a TODO for when
// the KeyStore trait is fully integrated into AgentKeyManager.
