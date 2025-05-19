//! Key management functionality for the TAP Agent.
//!
//! This module provides a key manager for storing and retrieving
//! cryptographic keys used by the TAP Agent for DID operations.
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::did::{DIDGenerationOptions, DIDKeyGenerator, GeneratedKey};
use crate::error::{Error, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// Secret key material types

/// Secret key material type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SecretType {
    /// JSON Web Key 2020
    JsonWebKey2020,
}

/// Secret key material
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SecretMaterial {
    /// JSON Web Key
    JWK {
        /// Private key in JWK format
        private_key_jwk: Value,
    },
}

/// Secret for cryptographic operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Secret {
    /// Secret ID
    pub id: String,

    /// Secret type
    pub type_: SecretType,

    /// Secret material
    pub secret_material: SecretMaterial,
}

/// Trait defining the interface for a key manager component
pub trait KeyManager: Send + Sync + std::fmt::Debug + 'static {
    /// Generate a new key with the specified options
    fn generate_key(&self, options: DIDGenerationOptions) -> Result<GeneratedKey>;

    /// Generate a new web DID with the specified domain and options
    fn generate_web_did(&self, domain: &str, options: DIDGenerationOptions)
        -> Result<GeneratedKey>;

    /// Add an existing key to the key manager
    fn add_key(&self, key: &GeneratedKey) -> Result<()>;

    /// Remove a key from the key manager
    fn remove_key(&self, did: &str) -> Result<()>;

    /// Check if the key manager has a key for the given DID
    fn has_key(&self, did: &str) -> Result<bool>;

    /// Get a list of all DIDs in the key manager
    fn list_keys(&self) -> Result<Vec<String>>;

    /// Get access to the secrets storage
    fn get_secrets(&self) -> Arc<RwLock<HashMap<String, Secret>>>;

    /// Get a secret resolver for use with cryptographic operations
    fn secret_resolver(&self) -> KeyManagerSecretResolver;
}

/// A default implementation of the KeyManager trait.
#[derive(Debug, Clone)]
pub struct DefaultKeyManager {
    /// The DID key generator
    pub generator: DIDKeyGenerator,
    /// The secret storage
    pub secrets: Arc<RwLock<HashMap<String, Secret>>>,
}

impl DefaultKeyManager {
    /// Create a new key manager
    pub fn new() -> Self {
        Self {
            generator: DIDKeyGenerator::new(),
            secrets: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for DefaultKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyManager for DefaultKeyManager {
    /// Generate a new key with the specified options
    fn generate_key(&self, options: DIDGenerationOptions) -> Result<GeneratedKey> {
        // Generate the key
        let key = self.generator.generate_did(options)?;

        // Create a secret for the key
        let secret = self.generator.create_secret_from_key(&key);

        // Store the secret
        if let Ok(mut secrets) = self.secrets.write() {
            secrets.insert(key.did.clone(), secret);
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        Ok(key)
    }

    /// Generate a new web DID with the specified domain and options
    fn generate_web_did(
        &self,
        domain: &str,
        options: DIDGenerationOptions,
    ) -> Result<GeneratedKey> {
        // Generate the key
        let key = self.generator.generate_web_did(domain, options)?;

        // Create a secret for the key
        let secret = self.generator.create_secret_from_key(&key);

        // Store the secret
        if let Ok(mut secrets) = self.secrets.write() {
            secrets.insert(key.did.clone(), secret);
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        Ok(key)
    }

    /// Add an existing key to the key manager
    fn add_key(&self, key: &GeneratedKey) -> Result<()> {
        // Create a secret for the key
        let secret = self.generator.create_secret_from_key(key);

        // Store the secret
        if let Ok(mut secrets) = self.secrets.write() {
            secrets.insert(key.did.clone(), secret);
            Ok(())
        } else {
            Err(Error::FailedToAcquireResolverWriteLock)
        }
    }

    /// Remove a key from the key manager
    fn remove_key(&self, did: &str) -> Result<()> {
        // Remove the secret
        if let Ok(mut secrets) = self.secrets.write() {
            secrets.remove(did);
            Ok(())
        } else {
            Err(Error::FailedToAcquireResolverWriteLock)
        }
    }

    /// Check if the key manager has a key for the given DID
    fn has_key(&self, did: &str) -> Result<bool> {
        // Check if the secret exists
        if let Ok(secrets) = self.secrets.read() {
            Ok(secrets.contains_key(did))
        } else {
            Err(Error::FailedToAcquireResolverReadLock)
        }
    }

    /// Get a list of all DIDs in the key manager
    fn list_keys(&self) -> Result<Vec<String>> {
        // Get all DIDs
        if let Ok(secrets) = self.secrets.read() {
            Ok(secrets.keys().cloned().collect())
        } else {
            Err(Error::FailedToAcquireResolverReadLock)
        }
    }

    /// Get access to the secrets storage
    fn get_secrets(&self) -> Arc<RwLock<HashMap<String, Secret>>> {
        Arc::clone(&self.secrets)
    }

    /// Get a secret resolver implementation for use with cryptographic operations
    fn secret_resolver(&self) -> KeyManagerSecretResolver {
        KeyManagerSecretResolver {
            secrets: Arc::clone(&self.secrets),
        }
    }
}

/// A secret resolver implementation that uses the key manager's secrets
#[derive(Debug, Clone)]
pub struct KeyManagerSecretResolver {
    /// The secret storage
    secrets: Arc<RwLock<HashMap<String, Secret>>>,
}

impl KeyManagerSecretResolver {
    /// Create a new KeyManagerSecretResolver
    pub fn new(key_manager: Arc<dyn KeyManager>) -> Self {
        Self {
            secrets: Arc::clone(&key_manager.get_secrets()),
        }
    }
}

// Import the DebugSecretsResolver trait
use crate::crypto::DebugSecretsResolver;

impl DebugSecretsResolver for KeyManagerSecretResolver {
    /// Get a reference to the secrets map for debugging purposes
    fn get_secrets_map(&self) -> &std::collections::HashMap<String, Secret> {
        // This is not a suitable pattern for a shared reference return type
        // Instead, we'll use a static empty HashMap with a 'static lifetime

        // NOTE: This implementation is only for debugging and diagnostics purposes
        // It does not provide access to the actual secrets, which should be accessed
        // using the get_secret_by_id method instead

        // Return a reference to a static empty HashMap
        static EMPTY_MAP: once_cell::sync::Lazy<std::collections::HashMap<String, Secret>> =
            once_cell::sync::Lazy::new(std::collections::HashMap::new);

        &EMPTY_MAP
    }

    /// Get a secret by ID
    fn get_secret_by_id(&self, secret_id: &str) -> Option<Secret> {
        if let Ok(secrets) = self.secrets.read() {
            if let Some(secret) = secrets.get(secret_id) {
                return Some(secret.clone());
            }
        }
        None
    }
}

// The KeyManagerSecretResolver already implements AsAny through the blanket implementation

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_manager() {
        let manager = DefaultKeyManager::new();

        // Generate an Ed25519 key
        let options = DIDGenerationOptions {
            key_type: crate::did::KeyType::Ed25519,
        };

        let key = manager.generate_key(options).unwrap();

        // Check that the key is stored
        assert!(manager.has_key(&key.did).unwrap());

        // List keys
        let keys = manager.list_keys().unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], key.did);

        // Remove the key
        manager.remove_key(&key.did).unwrap();
        assert!(!manager.has_key(&key.did).unwrap());

        // Add the key back
        manager.add_key(&key).unwrap();
        assert!(manager.has_key(&key.did).unwrap());
    }

    #[test]
    fn test_secret_resolver() {
        let manager = DefaultKeyManager::new();

        // Generate keys of different types
        let ed25519_key = manager
            .generate_key(DIDGenerationOptions {
                key_type: crate::did::KeyType::Ed25519,
            })
            .unwrap();
        let p256_key = manager
            .generate_key(DIDGenerationOptions {
                key_type: crate::did::KeyType::P256,
            })
            .unwrap();
        let secp256k1_key = manager
            .generate_key(DIDGenerationOptions {
                key_type: crate::did::KeyType::Secp256k1,
            })
            .unwrap();

        // Create a secret resolver
        let resolver = manager.secret_resolver();

        // Test get_secret_by_id for each key
        let ed25519_secret = resolver.get_secret_by_id(&ed25519_key.did);
        let p256_secret = resolver.get_secret_by_id(&p256_key.did);
        let secp256k1_secret = resolver.get_secret_by_id(&secp256k1_key.did);

        assert!(ed25519_secret.is_some());
        assert!(p256_secret.is_some());
        assert!(secp256k1_secret.is_some());

        // Test non-existent secret
        let non_existent = resolver.get_secret_by_id("did:key:non_existent");
        assert!(non_existent.is_none());
    }

    #[test]
    fn test_web_did_generation() {
        let manager = DefaultKeyManager::new();

        // Generate a web DID
        let domain = "example.com";
        let options = DIDGenerationOptions {
            key_type: crate::did::KeyType::Ed25519,
        };

        let key = manager.generate_web_did(domain, options).unwrap();

        // Check that the key is stored
        assert!(manager.has_key(&key.did).unwrap());

        // Verify the DID format
        assert_eq!(key.did, format!("did:web:{}", domain));
    }
}
