//! Agent Key Manager for the TAP Agent
//!
//! This module provides an implementation of a key manager that uses the agent key abstraction.
//! It manages keys for signing, verification, encryption, and decryption operations, with support
//! for different key types (Ed25519, P-256, secp256k1).

use crate::agent_key::{AgentKey, DecryptionKey, EncryptionKey, SigningKey, VerificationKey};
use crate::did::{DIDGenerationOptions, DIDKeyGenerator, GeneratedKey, KeyType};
use crate::error::{Error, Result};
use crate::key_manager::{KeyManager, Secret, SecretMaterial};
use crate::local_agent_key::{LocalAgentKey, PublicVerificationKey};
use crate::message::{JweProtected, JwsProtected};
use crate::message_packing::{KeyManagerPacking, MessageError};
use crate::storage::{KeyStorage, StoredKey};

use async_trait::async_trait;
use base64::Engine;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Agent Key Manager implements the KeyManager trait using the agent key abstraction
#[derive(Debug, Clone)]
pub struct AgentKeyManager {
    /// The DID key generator
    generator: DIDKeyGenerator,
    /// The secret storage (legacy)
    secrets: Arc<RwLock<HashMap<String, Secret>>>,
    /// Signing keys
    signing_keys: Arc<RwLock<HashMap<String, Arc<dyn SigningKey + Send + Sync>>>>,
    /// Encryption keys
    encryption_keys: Arc<RwLock<HashMap<String, Arc<dyn EncryptionKey + Send + Sync>>>>,
    /// Decryption keys
    decryption_keys: Arc<RwLock<HashMap<String, Arc<dyn DecryptionKey + Send + Sync>>>>,
    /// Verification keys
    verification_keys: Arc<RwLock<HashMap<String, Arc<dyn VerificationKey + Send + Sync>>>>,
    /// Generated keys with DID documents (for key ID resolution)
    generated_keys: Arc<RwLock<HashMap<String, GeneratedKey>>>,
    /// Storage path
    storage_path: Option<PathBuf>,
}

impl AgentKeyManager {
    /// Create a new key manager
    pub fn new() -> Self {
        Self {
            generator: DIDKeyGenerator::new(),
            secrets: Arc::new(RwLock::new(HashMap::new())),
            signing_keys: Arc::new(RwLock::new(HashMap::new())),
            encryption_keys: Arc::new(RwLock::new(HashMap::new())),
            decryption_keys: Arc::new(RwLock::new(HashMap::new())),
            verification_keys: Arc::new(RwLock::new(HashMap::new())),
            generated_keys: Arc::new(RwLock::new(HashMap::new())),
            storage_path: None,
        }
    }

    /// Get a generated key (with DID document) by DID
    pub fn get_generated_key(&self, did: &str) -> Result<GeneratedKey> {
        if let Ok(generated_keys) = self.generated_keys.read() {
            if let Some(key) = generated_keys.get(did) {
                return Ok(key.clone());
            }
        } else {
            return Err(Error::FailedToAcquireResolverReadLock);
        }

        Err(Error::KeyNotFound(format!(
            "Generated key not found for DID: {}",
            did
        )))
    }

    /// Get the key type for a signing key (for debugging)
    pub async fn get_signing_key_type(&self, did: &str) -> Result<String> {
        // Try to find a signing key for this DID
        if let Ok(signing_keys) = self.signing_keys.read() {
            for (kid, key) in signing_keys.iter() {
                if kid.starts_with(did) {
                    if let Ok(jwk) = key.public_key_jwk() {
                        let kty = jwk.get("kty").and_then(|v| v.as_str());
                        let crv = jwk.get("crv").and_then(|v| v.as_str());
                        return Ok(format!("kty: {:?}, crv: {:?}", kty, crv));
                    }
                }
            }
        }

        Err(Error::KeyNotFound(format!(
            "No signing key found for DID: {}",
            did
        )))
    }

    /// Create a LocalAgentKey from a GeneratedKey
    pub fn agent_key_from_generated(&self, key: &GeneratedKey) -> Result<LocalAgentKey> {
        // Create a secret for the key
        let secret = self.generator.create_secret_from_key(key);

        // Create a LocalAgentKey
        Ok(LocalAgentKey::new(secret, key.key_type))
    }

    /// Store an agent key in all collections
    fn store_agent_key(&self, agent_key: &LocalAgentKey, key_id: &str) -> Result<()> {
        // Store the agent key as signing, encryption, and decryption keys
        if let Ok(mut signing_keys) = self.signing_keys.write() {
            signing_keys.insert(
                key_id.to_string(),
                Arc::new(agent_key.clone()) as Arc<dyn SigningKey + Send + Sync>,
            );
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        if let Ok(mut encryption_keys) = self.encryption_keys.write() {
            encryption_keys.insert(
                key_id.to_string(),
                Arc::new(agent_key.clone()) as Arc<dyn EncryptionKey + Send + Sync>,
            );
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        if let Ok(mut decryption_keys) = self.decryption_keys.write() {
            decryption_keys.insert(
                key_id.to_string(),
                Arc::new(agent_key.clone()) as Arc<dyn DecryptionKey + Send + Sync>,
            );
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        // Also store a reference in verification keys
        if let Ok(mut verification_keys) = self.verification_keys.write() {
            verification_keys.insert(
                key_id.to_string(),
                Arc::new(agent_key.clone()) as Arc<dyn VerificationKey + Send + Sync>,
            );
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        Ok(())
    }

    /// Save keys to storage if a storage path is configured
    pub fn save_to_storage(&self) -> Result<()> {
        // Skip if no storage path is configured
        if self.storage_path.is_none() {
            return Ok(());
        }

        // Create a KeyStorage object from our secrets
        let mut key_storage = KeyStorage::new();

        // Add all secrets
        if let Ok(secrets) = self.secrets.read() {
            for (did, secret) in secrets.iter() {
                // Extract key type from the key
                let key_type = match secret.secret_material {
                    SecretMaterial::JWK {
                        ref private_key_jwk,
                    } => {
                        let kty = private_key_jwk.get("kty").and_then(|v| v.as_str());
                        let crv = private_key_jwk.get("crv").and_then(|v| v.as_str());

                        match (kty, crv) {
                            #[cfg(feature = "crypto-ed25519")]
                            (Some("OKP"), Some("Ed25519")) => KeyType::Ed25519,
                            #[cfg(feature = "crypto-p256")]
                            (Some("EC"), Some("P-256")) => KeyType::P256,
                            #[cfg(feature = "crypto-secp256k1")]
                            (Some("EC"), Some("secp256k1")) => KeyType::Secp256k1,
                            _ => KeyType::Ed25519, // Default
                        }
                    }
                };

                // Get private and public keys from the JWK
                let private_key_b64 = match &secret.secret_material {
                    SecretMaterial::JWK { private_key_jwk } => private_key_jwk
                        .get("d")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                };

                let public_key_b64 = match &secret.secret_material {
                    SecretMaterial::JWK { private_key_jwk } => private_key_jwk
                        .get("x")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                };

                // Create a StoredKey and add to key storage
                let stored_key = StoredKey {
                    did: did.clone(),
                    label: String::new(), // Will be auto-generated when added
                    key_type,
                    private_key: private_key_b64,
                    public_key: public_key_b64,
                    metadata: HashMap::new(),
                };
                key_storage.add_key(stored_key);
            }
        }

        // Save to storage
        if let Some(path) = &self.storage_path {
            key_storage.save_to_path(path)?;
        } else {
            key_storage.save_default()?;
        }

        Ok(())
    }

    /// Load from default storage location
    pub fn load_from_default_storage(mut self) -> Result<Self> {
        self.storage_path = None;
        self.load_keys_from_storage()
    }

    /// Load from a specific storage path
    pub fn load_from_path(mut self, path: PathBuf) -> Result<Self> {
        self.storage_path = Some(path);
        self.load_keys_from_storage()
    }

    /// Load keys from configured storage
    fn load_keys_from_storage(&self) -> Result<Self> {
        // Load storage
        let storage = if let Some(path) = &self.storage_path {
            KeyStorage::load_from_path(path)?
        } else {
            KeyStorage::load_default()?
        };

        // Process each stored key
        for (did, stored_key) in storage.keys {
            // Convert to a legacy secret
            let secret = KeyStorage::to_secret(&stored_key);

            // Add to secrets
            if let Ok(mut secrets) = self.secrets.write() {
                secrets.insert(did.clone(), secret.clone());
            } else {
                return Err(Error::FailedToAcquireResolverWriteLock);
            }

            // Create an agent key
            let key_type = stored_key.key_type;
            let agent_key = LocalAgentKey::new(secret, key_type);
            let key_id = AgentKey::key_id(&agent_key).to_string();

            // Store in all collections
            self.store_agent_key(&agent_key, &key_id)?;
        }

        Ok(self.clone())
    }

    /// Add a key to the key manager with option to save to storage
    fn add_key_internal(&self, key: &GeneratedKey, save_to_storage: bool) -> Result<()> {
        // Create a LocalAgentKey
        let agent_key = self.agent_key_from_generated(key)?;
        let key_id = AgentKey::key_id(&agent_key).to_string();

        // Store the legacy secret
        if let Ok(mut secrets) = self.secrets.write() {
            secrets.insert(key.did.clone(), agent_key.clone().secret);
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        // Store in all collections
        self.store_agent_key(&agent_key, &key_id)?;

        // Save to storage if configured and requested
        if save_to_storage {
            self.save_to_storage()?;
        }

        Ok(())
    }

    /// Add a key to the key manager without saving to storage
    /// This is useful when you plan to save to storage manually later
    pub fn add_key_without_save(&self, key: &GeneratedKey) -> Result<()> {
        self.add_key_internal(key, false)
    }

    /// Generate a new key with the specified options without saving to storage
    /// This is useful when you plan to save to storage manually later
    pub fn generate_key_without_save(&self, options: DIDGenerationOptions) -> Result<GeneratedKey> {
        self.generate_key_internal(options, false)
    }

    /// Internal method to generate a key with optional storage save
    fn generate_key_internal(
        &self,
        options: DIDGenerationOptions,
        save_to_storage: bool,
    ) -> Result<GeneratedKey> {
        // Generate the key
        let key = self.generator.generate_did(options)?;

        // Create a LocalAgentKey
        let agent_key = self.agent_key_from_generated(&key)?;
        let key_id = AgentKey::key_id(&agent_key).to_string();

        // Store the legacy secret
        if let Ok(mut secrets) = self.secrets.write() {
            secrets.insert(key.did.clone(), agent_key.clone().secret);
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        // Store the generated key for DID document access
        if let Ok(mut generated_keys) = self.generated_keys.write() {
            generated_keys.insert(key.did.clone(), key.clone());
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        // Store in all collections
        self.store_agent_key(&agent_key, &key_id)?;

        // Save to storage if configured and requested
        if save_to_storage {
            self.save_to_storage()?;
        }

        Ok(key)
    }

    /// Generate a new web DID with the specified domain and options without saving to storage
    /// This is useful when you plan to save to storage manually later
    pub fn generate_web_did_without_save(
        &self,
        domain: &str,
        options: DIDGenerationOptions,
    ) -> Result<GeneratedKey> {
        self.generate_web_did_internal(domain, options, false)
    }

    /// Internal method to generate a web DID with optional storage save
    fn generate_web_did_internal(
        &self,
        domain: &str,
        options: DIDGenerationOptions,
        save_to_storage: bool,
    ) -> Result<GeneratedKey> {
        // Generate the key
        let key = self.generator.generate_web_did(domain, options)?;

        // Create a LocalAgentKey
        let agent_key = self.agent_key_from_generated(&key)?;
        let key_id = AgentKey::key_id(&agent_key).to_string();

        // Store the legacy secret
        if let Ok(mut secrets) = self.secrets.write() {
            secrets.insert(key.did.clone(), agent_key.clone().secret);
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        // Store the generated key for DID document access
        if let Ok(mut generated_keys) = self.generated_keys.write() {
            generated_keys.insert(key.did.clone(), key.clone());
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        // Store in all collections
        self.store_agent_key(&agent_key, &key_id)?;

        // Save to storage if configured and requested
        if save_to_storage {
            self.save_to_storage()?;
        }

        Ok(key)
    }
}

impl Default for AgentKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl KeyManager for AgentKeyManager {
    /// Get access to the secrets storage
    fn secrets(&self) -> Arc<RwLock<HashMap<String, Secret>>> {
        Arc::clone(&self.secrets)
    }

    /// Generate a new key with the specified options
    fn generate_key(&self, options: DIDGenerationOptions) -> Result<GeneratedKey> {
        self.generate_key_internal(options, true)
    }

    /// Generate a new web DID with the specified domain and options
    fn generate_web_did(
        &self,
        domain: &str,
        options: DIDGenerationOptions,
    ) -> Result<GeneratedKey> {
        self.generate_web_did_internal(domain, options, true)
    }

    /// Add an existing key to the key manager
    fn add_key(&self, key: &GeneratedKey) -> Result<()> {
        self.add_key_internal(key, true)
    }

    /// Remove a key from the key manager
    fn remove_key(&self, did: &str) -> Result<()> {
        // Remove from legacy secrets
        if let Ok(mut secrets) = self.secrets.write() {
            secrets.remove(did);
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        // Remove from signing keys
        if let Ok(mut signing_keys) = self.signing_keys.write() {
            signing_keys.retain(|k, _| !k.starts_with(did));
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        // Remove from encryption keys
        if let Ok(mut encryption_keys) = self.encryption_keys.write() {
            encryption_keys.retain(|k, _| !k.starts_with(did));
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        // Remove from decryption keys
        if let Ok(mut decryption_keys) = self.decryption_keys.write() {
            decryption_keys.retain(|k, _| !k.starts_with(did));
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        // Remove from verification keys
        if let Ok(mut verification_keys) = self.verification_keys.write() {
            verification_keys.retain(|k, _| !k.starts_with(did));
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        // Save to storage if configured
        self.save_to_storage()?;

        Ok(())
    }

    /// Check if the key manager has a key for the given DID
    fn has_key(&self, did: &str) -> Result<bool> {
        // Check legacy secrets first
        if let Ok(secrets) = self.secrets.read() {
            if secrets.contains_key(did) {
                return Ok(true);
            }
        } else {
            return Err(Error::FailedToAcquireResolverReadLock);
        }

        // Check if any signing key has this DID
        if let Ok(signing_keys) = self.signing_keys.read() {
            if signing_keys.values().any(|k| k.did() == did) {
                return Ok(true);
            }
        } else {
            return Err(Error::FailedToAcquireResolverReadLock);
        }

        Ok(false)
    }

    /// Get a list of all DIDs in the key manager
    fn list_keys(&self) -> Result<Vec<String>> {
        // Collect DIDs from both legacy secrets and new keys
        let mut dids = Vec::new();

        // Add DIDs from legacy secrets
        if let Ok(secrets) = self.secrets.read() {
            dids.extend(secrets.keys().cloned());
        } else {
            return Err(Error::FailedToAcquireResolverReadLock);
        }

        // Add DIDs from signing keys
        if let Ok(signing_keys) = self.signing_keys.read() {
            for key in signing_keys.values() {
                if !dids.contains(&key.did().to_string()) {
                    dids.push(key.did().to_string());
                }
            }
        } else {
            return Err(Error::FailedToAcquireResolverReadLock);
        }

        Ok(dids)
    }

    /// Add a signing key to the key manager
    async fn add_signing_key(&self, key: Arc<dyn SigningKey + Send + Sync>) -> Result<()> {
        if let Ok(mut signing_keys) = self.signing_keys.write() {
            signing_keys.insert(key.key_id().to_string(), key);
            Ok(())
        } else {
            Err(Error::FailedToAcquireResolverWriteLock)
        }
    }

    /// Add an encryption key to the key manager
    async fn add_encryption_key(&self, key: Arc<dyn EncryptionKey + Send + Sync>) -> Result<()> {
        if let Ok(mut encryption_keys) = self.encryption_keys.write() {
            encryption_keys.insert(key.key_id().to_string(), key);
            Ok(())
        } else {
            Err(Error::FailedToAcquireResolverWriteLock)
        }
    }

    /// Add a decryption key to the key manager
    async fn add_decryption_key(&self, key: Arc<dyn DecryptionKey + Send + Sync>) -> Result<()> {
        if let Ok(mut decryption_keys) = self.decryption_keys.write() {
            decryption_keys.insert(key.key_id().to_string(), key);
            Ok(())
        } else {
            Err(Error::FailedToAcquireResolverWriteLock)
        }
    }

    /// Get a signing key by ID
    async fn get_signing_key(&self, kid: &str) -> Result<Arc<dyn SigningKey + Send + Sync>> {
        // Check if we have a signing key with this ID
        if let Ok(signing_keys) = self.signing_keys.read() {
            if let Some(key) = signing_keys.get(kid) {
                return Ok(key.clone());
            }
        } else {
            return Err(Error::FailedToAcquireResolverReadLock);
        }

        // If not, check legacy secrets
        if let Ok(secrets) = self.secrets.read() {
            // Try to find a secret with this DID or kid
            let did = kid.split('#').next().unwrap_or(kid);
            if let Some(secret) = secrets.get(did) {
                // Detect key type from the JWK
                let key_type = match &secret.secret_material {
                    SecretMaterial::JWK { private_key_jwk } => {
                        let kty = private_key_jwk.get("kty").and_then(|v| v.as_str());
                        let crv = private_key_jwk.get("crv").and_then(|v| v.as_str());
                        match (kty, crv) {
                            #[cfg(feature = "crypto-ed25519")]
                            (Some("OKP"), Some("Ed25519")) => KeyType::Ed25519,
                            #[cfg(feature = "crypto-p256")]
                            (Some("EC"), Some("P-256")) => KeyType::P256,
                            #[cfg(feature = "crypto-secp256k1")]
                            (Some("EC"), Some("secp256k1")) => KeyType::Secp256k1,
                            _ => KeyType::Ed25519, // Default
                        }
                    }
                };
                // Create a LocalAgentKey
                let agent_key = LocalAgentKey::new(secret.clone(), key_type);

                // Add to signing keys for next time
                if let Ok(mut signing_keys) = self.signing_keys.write() {
                    let arc_key = Arc::new(agent_key.clone()) as Arc<dyn SigningKey + Send + Sync>;
                    signing_keys.insert(AgentKey::key_id(&agent_key).to_string(), arc_key.clone());
                    return Ok(arc_key);
                }
            }
        }

        Err(Error::Cryptography(format!(
            "No signing key found with ID: {}",
            kid
        )))
    }

    /// Get an encryption key by ID
    async fn get_encryption_key(&self, kid: &str) -> Result<Arc<dyn EncryptionKey + Send + Sync>> {
        // Check if we have an encryption key with this ID
        if let Ok(encryption_keys) = self.encryption_keys.read() {
            if let Some(key) = encryption_keys.get(kid) {
                return Ok(key.clone());
            }
        } else {
            return Err(Error::FailedToAcquireResolverReadLock);
        }

        // If not, check legacy secrets
        if let Ok(secrets) = self.secrets.read() {
            // Try to find a secret with this DID or kid
            let did = kid.split('#').next().unwrap_or(kid);
            if let Some(secret) = secrets.get(did) {
                // Detect key type from the JWK
                let key_type = match &secret.secret_material {
                    SecretMaterial::JWK { private_key_jwk } => {
                        let kty = private_key_jwk.get("kty").and_then(|v| v.as_str());
                        let crv = private_key_jwk.get("crv").and_then(|v| v.as_str());
                        match (kty, crv) {
                            #[cfg(feature = "crypto-ed25519")]
                            (Some("OKP"), Some("Ed25519")) => KeyType::Ed25519,
                            #[cfg(feature = "crypto-p256")]
                            (Some("EC"), Some("P-256")) => KeyType::P256,
                            #[cfg(feature = "crypto-secp256k1")]
                            (Some("EC"), Some("secp256k1")) => KeyType::Secp256k1,
                            _ => KeyType::Ed25519, // Default
                        }
                    }
                };
                // Create a LocalAgentKey
                let agent_key = LocalAgentKey::new(secret.clone(), key_type);

                // Add to encryption keys for next time
                if let Ok(mut encryption_keys) = self.encryption_keys.write() {
                    let arc_key =
                        Arc::new(agent_key.clone()) as Arc<dyn EncryptionKey + Send + Sync>;
                    encryption_keys
                        .insert(AgentKey::key_id(&agent_key).to_string(), arc_key.clone());
                    return Ok(arc_key);
                }
            }
        }

        Err(Error::Cryptography(format!(
            "No encryption key found with ID: {}",
            kid
        )))
    }

    /// Get a decryption key by ID
    async fn get_decryption_key(&self, kid: &str) -> Result<Arc<dyn DecryptionKey + Send + Sync>> {
        // Check if we have a decryption key with this ID
        if let Ok(decryption_keys) = self.decryption_keys.read() {
            if let Some(key) = decryption_keys.get(kid) {
                return Ok(key.clone());
            }
        } else {
            return Err(Error::FailedToAcquireResolverReadLock);
        }

        // If not, check legacy secrets
        if let Ok(secrets) = self.secrets.read() {
            // Try to find a secret with this DID or kid
            let did = kid.split('#').next().unwrap_or(kid);
            if let Some(secret) = secrets.get(did) {
                // Detect key type from the JWK
                let key_type = match &secret.secret_material {
                    SecretMaterial::JWK { private_key_jwk } => {
                        let kty = private_key_jwk.get("kty").and_then(|v| v.as_str());
                        let crv = private_key_jwk.get("crv").and_then(|v| v.as_str());
                        match (kty, crv) {
                            #[cfg(feature = "crypto-ed25519")]
                            (Some("OKP"), Some("Ed25519")) => KeyType::Ed25519,
                            #[cfg(feature = "crypto-p256")]
                            (Some("EC"), Some("P-256")) => KeyType::P256,
                            #[cfg(feature = "crypto-secp256k1")]
                            (Some("EC"), Some("secp256k1")) => KeyType::Secp256k1,
                            _ => KeyType::Ed25519, // Default
                        }
                    }
                };
                // Create a LocalAgentKey
                let agent_key = LocalAgentKey::new(secret.clone(), key_type);

                // Add to decryption keys for next time
                if let Ok(mut decryption_keys) = self.decryption_keys.write() {
                    let arc_key =
                        Arc::new(agent_key.clone()) as Arc<dyn DecryptionKey + Send + Sync>;
                    decryption_keys
                        .insert(AgentKey::key_id(&agent_key).to_string(), arc_key.clone());
                    return Ok(arc_key);
                }
            }
        }

        Err(Error::Cryptography(format!(
            "No decryption key found with ID: {}",
            kid
        )))
    }

    /// Resolve a verification key by ID
    async fn resolve_verification_key(
        &self,
        kid: &str,
    ) -> Result<Arc<dyn VerificationKey + Send + Sync>> {
        // Check if we have a verification key with this ID
        if let Ok(verification_keys) = self.verification_keys.read() {
            if let Some(key) = verification_keys.get(kid) {
                return Ok(key.clone());
            }
        } else {
            return Err(Error::FailedToAcquireResolverReadLock);
        }

        // If not found locally, try to derive from a signing key
        let signing_key = KeyManager::get_signing_key(self, kid).await;
        if let Ok(key) = signing_key {
            // Create a verification key from the signing key
            let public_jwk = key.public_key_jwk()?;
            let verification_key = Arc::new(PublicVerificationKey::new(kid.to_string(), public_jwk))
                as Arc<dyn VerificationKey + Send + Sync>;

            // Add to verification keys for next time
            if let Ok(mut verification_keys) = self.verification_keys.write() {
                verification_keys.insert(kid.to_string(), verification_key.clone());
            }

            return Ok(verification_key);
        }

        // In a full implementation, we would use a DID Resolver here
        Err(Error::Cryptography(format!(
            "No verification key found with ID: {}",
            kid
        )))
    }

    /// Sign data with a key
    async fn sign_jws(
        &self,
        kid: &str,
        payload: &[u8],
        protected_header: Option<JwsProtected>,
    ) -> Result<String> {
        // Get the signing key
        let signing_key = KeyManager::get_signing_key(self, kid).await?;

        // Sign the payload
        let jws = signing_key
            .create_jws(payload, protected_header)
            .await
            .map_err(|e| Error::Cryptography(e.to_string()))?;

        // Serialize the JWS
        serde_json::to_string(&jws).map_err(|e| Error::Serialization(e.to_string()))
    }

    /// Verify a JWS
    async fn verify_jws(&self, jws: &str, expected_kid: Option<&str>) -> Result<Vec<u8>> {
        // Parse the JWS
        let jws: crate::message::Jws = serde_json::from_str(jws)
            .map_err(|e| Error::Serialization(format!("Failed to parse JWS: {}", e)))?;

        // Find the signature to verify
        let signature = if let Some(kid) = expected_kid {
            jws.signatures
                .iter()
                .find(|s| s.get_kid().as_deref() == Some(kid))
                .ok_or_else(|| {
                    Error::Cryptography(format!("No signature found with kid: {}", kid))
                })?
        } else {
            // Use the first signature
            jws.signatures
                .first()
                .ok_or_else(|| Error::Cryptography("No signatures in JWS".to_string()))?
        };

        // Get the protected header
        let protected = signature.get_protected_header().map_err(|e| {
            Error::Cryptography(format!("Failed to decode protected header: {}", e))
        })?;

        // Get the verification key using kid from protected header
        let kid = signature
            .get_kid()
            .ok_or_else(|| Error::Cryptography("No kid found in JWS signature".to_string()))?;
        let verification_key = KeyManager::resolve_verification_key(self, &kid).await?;

        // Decode the signature
        let signature_bytes = base64::engine::general_purpose::STANDARD
            .decode(&signature.signature)
            .map_err(|e| Error::Cryptography(format!("Failed to decode signature: {}", e)))?;

        // Create the signing input (protected.payload)
        let signing_input = format!("{}.{}", signature.protected, jws.payload);

        // Verify the signature
        let verified = verification_key
            .verify_signature(signing_input.as_bytes(), &signature_bytes, &protected)
            .await
            .map_err(|e| Error::Cryptography(e.to_string()))?;

        if !verified {
            return Err(Error::Cryptography(
                "Signature verification failed".to_string(),
            ));
        }

        // Decode the payload
        let payload_bytes = base64::engine::general_purpose::STANDARD
            .decode(&jws.payload)
            .map_err(|e| Error::Cryptography(format!("Failed to decode payload: {}", e)))?;

        Ok(payload_bytes)
    }

    /// Encrypt data for a recipient
    async fn encrypt_jwe(
        &self,
        sender_kid: &str,
        recipient_kid: &str,
        plaintext: &[u8],
        protected_header: Option<JweProtected>,
    ) -> Result<String> {
        // Get the encryption key
        let encryption_key = KeyManager::get_encryption_key(self, sender_kid).await?;

        // Resolve the recipient's verification key
        let recipient_key = KeyManager::resolve_verification_key(self, recipient_kid).await?;

        // Encrypt the plaintext
        let jwe = encryption_key
            .create_jwe(plaintext, &[recipient_key], protected_header)
            .await
            .map_err(|e| Error::Cryptography(e.to_string()))?;

        // Serialize the JWE
        serde_json::to_string(&jwe).map_err(|e| Error::Serialization(e.to_string()))
    }

    /// Decrypt a JWE
    async fn decrypt_jwe(&self, jwe: &str, expected_kid: Option<&str>) -> Result<Vec<u8>> {
        // Parse the JWE
        let jwe: crate::message::Jwe = serde_json::from_str(jwe)
            .map_err(|e| Error::Serialization(format!("Failed to parse JWE: {}", e)))?;

        // Find the recipient if expected_kid is provided
        if let Some(kid) = expected_kid {
            // Just verify recipient exists, we don't need the actual instance
            jwe.recipients
                .iter()
                .find(|r| r.header.kid == kid)
                .ok_or_else(|| {
                    Error::Cryptography(format!("No recipient found with kid: {}", kid))
                })?;

            // Get the decryption key
            let decryption_key = KeyManager::get_decryption_key(self, kid).await?;

            // Decrypt the JWE
            decryption_key
                .unwrap_jwe(&jwe)
                .await
                .map_err(|e| Error::Cryptography(e.to_string()))
        } else {
            // Try each recipient
            for recipient in &jwe.recipients {
                // Try to get the decryption key
                if let Ok(decryption_key) =
                    KeyManager::get_decryption_key(self, &recipient.header.kid).await
                {
                    // Try to decrypt
                    if let Ok(plaintext) = decryption_key.unwrap_jwe(&jwe).await {
                        return Ok(plaintext);
                    }
                }
            }

            Err(Error::Cryptography(
                "Failed to decrypt JWE for any recipient".to_string(),
            ))
        }
    }
}

/// A builder for AgentKeyManager
#[derive(Debug, Clone)]
pub struct AgentKeyManagerBuilder {
    /// Legacy secrets
    secrets: HashMap<String, Secret>,
    /// Signing keys
    signing_keys: HashMap<String, Arc<dyn SigningKey + Send + Sync>>,
    /// Encryption keys
    encryption_keys: HashMap<String, Arc<dyn EncryptionKey + Send + Sync>>,
    /// Decryption keys
    decryption_keys: HashMap<String, Arc<dyn DecryptionKey + Send + Sync>>,
    /// Verification keys
    verification_keys: HashMap<String, Arc<dyn VerificationKey + Send + Sync>>,
    /// Load from storage
    load_from_storage: bool,
    /// Storage path
    storage_path: Option<PathBuf>,
}

impl Default for AgentKeyManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentKeyManagerBuilder {
    /// Create a new KeyManagerBuilder
    pub fn new() -> Self {
        Self {
            secrets: HashMap::new(),
            signing_keys: HashMap::new(),
            encryption_keys: HashMap::new(),
            decryption_keys: HashMap::new(),
            verification_keys: HashMap::new(),
            load_from_storage: false,
            storage_path: None,
        }
    }

    /// Load keys from default storage location
    pub fn load_from_default_storage(mut self) -> Self {
        self.load_from_storage = true;
        self.storage_path = None;
        self
    }

    /// Load keys from a specific storage path
    pub fn load_from_path(mut self, path: PathBuf) -> Self {
        self.load_from_storage = true;
        self.storage_path = Some(path);
        self
    }

    /// Add a legacy secret
    pub fn add_secret(mut self, did: String, secret: Secret) -> Self {
        self.secrets.insert(did, secret);
        self
    }

    /// Add a signing key
    pub fn add_signing_key(mut self, key: Arc<dyn SigningKey + Send + Sync>) -> Self {
        self.signing_keys.insert(key.key_id().to_string(), key);
        self
    }

    /// Add an encryption key
    pub fn add_encryption_key(mut self, key: Arc<dyn EncryptionKey + Send + Sync>) -> Self {
        self.encryption_keys.insert(key.key_id().to_string(), key);
        self
    }

    /// Add a decryption key
    pub fn add_decryption_key(mut self, key: Arc<dyn DecryptionKey + Send + Sync>) -> Self {
        self.decryption_keys.insert(key.key_id().to_string(), key);
        self
    }

    /// Add a verification key
    pub fn add_verification_key(mut self, key: Arc<dyn VerificationKey + Send + Sync>) -> Self {
        self.verification_keys.insert(key.key_id().to_string(), key);
        self
    }

    /// Build the KeyManager
    pub fn build(self) -> Result<AgentKeyManager> {
        let mut key_manager = AgentKeyManager {
            generator: DIDKeyGenerator::new(),
            secrets: Arc::new(RwLock::new(self.secrets)),
            signing_keys: Arc::new(RwLock::new(self.signing_keys)),
            encryption_keys: Arc::new(RwLock::new(self.encryption_keys)),
            decryption_keys: Arc::new(RwLock::new(self.decryption_keys)),
            verification_keys: Arc::new(RwLock::new(self.verification_keys)),
            generated_keys: Arc::new(RwLock::new(HashMap::new())),
            storage_path: self.storage_path.clone(),
        };

        // Load keys from storage if requested
        if self.load_from_storage {
            key_manager = if let Some(path) = self.storage_path {
                key_manager.load_from_path(path)?
            } else {
                key_manager.load_from_default_storage()?
            };
        }

        Ok(key_manager)
    }
}

#[async_trait]
impl KeyManagerPacking for AgentKeyManager {
    async fn get_signing_key(&self, kid: &str) -> Result<Arc<dyn SigningKey + Send + Sync>> {
        KeyManager::get_signing_key(self, kid)
            .await
            .map_err(|e| Error::from(MessageError::KeyManager(e.to_string())))
    }

    async fn get_encryption_key(&self, kid: &str) -> Result<Arc<dyn EncryptionKey + Send + Sync>> {
        KeyManager::get_encryption_key(self, kid)
            .await
            .map_err(|e| Error::from(MessageError::KeyManager(e.to_string())))
    }

    async fn get_decryption_key(&self, kid: &str) -> Result<Arc<dyn DecryptionKey + Send + Sync>> {
        KeyManager::get_decryption_key(self, kid)
            .await
            .map_err(|e| Error::from(MessageError::KeyManager(e.to_string())))
    }

    async fn resolve_verification_key(
        &self,
        kid: &str,
    ) -> Result<Arc<dyn VerificationKey + Send + Sync>> {
        KeyManager::resolve_verification_key(self, kid)
            .await
            .map_err(|e| Error::from(MessageError::KeyManager(e.to_string())))
    }
}
