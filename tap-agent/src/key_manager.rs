//! Key management functionality for the TAP Agent.
//!
//! This module provides a key manager for storing and retrieving
//! cryptographic keys used by the TAP Agent for DID operations.
use crate::agent_key::{
    AgentKey, DecryptionKey, EncryptionKey, SigningKey, VerificationKey,
};
use crate::did::{DIDGenerationOptions, DIDKeyGenerator, GeneratedKey};
use crate::error::{Error, Result};
use crate::local_agent_key::{LocalAgentKey, PublicVerificationKey};
use crate::message_packing::{KeyManagerPacking, MessageError};

use async_trait::async_trait;
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
#[async_trait]
pub trait KeyManager: Send + Sync + std::fmt::Debug + 'static {
    /// Get access to the secrets storage for this key manager
    fn secrets(&self) -> Arc<RwLock<HashMap<String, Secret>>>;
    
    /// Get a secret resolver for this key manager
    fn secret_resolver(&self) -> KeyManagerSecretResolver {
        KeyManagerSecretResolver::new_from_secrets(self.secrets())
    }
    
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

    /// Add a signing key to the key manager
    async fn add_signing_key(&self, key: Arc<dyn SigningKey + Send + Sync>) -> Result<()>;

    /// Add an encryption key to the key manager
    async fn add_encryption_key(&self, key: Arc<dyn EncryptionKey + Send + Sync>) -> Result<()>;

    /// Add a decryption key to the key manager
    async fn add_decryption_key(&self, key: Arc<dyn DecryptionKey + Send + Sync>) -> Result<()>;

    /// Get a signing key by ID
    async fn get_signing_key(&self, kid: &str) -> Result<Arc<dyn SigningKey + Send + Sync>>;

    /// Get an encryption key by ID
    async fn get_encryption_key(&self, kid: &str) -> Result<Arc<dyn EncryptionKey + Send + Sync>>;

    /// Get a decryption key by ID
    async fn get_decryption_key(&self, kid: &str) -> Result<Arc<dyn DecryptionKey + Send + Sync>>;

    /// Resolve a verification key by ID
    async fn resolve_verification_key(
        &self,
        kid: &str,
    ) -> Result<Arc<dyn VerificationKey + Send + Sync>>;

    /// Sign data with a key
    async fn sign_jws(
        &self,
        kid: &str,
        payload: &[u8],
        protected_header: Option<crate::message::JwsProtected>,
    ) -> Result<String>;

    /// Verify a JWS
    async fn verify_jws(&self, jws: &str, expected_kid: Option<&str>) -> Result<Vec<u8>>;

    /// Encrypt data for a recipient
    async fn encrypt_jwe(
        &self,
        sender_kid: &str,
        recipient_kid: &str,
        plaintext: &[u8],
        protected_header: Option<crate::message::JweProtected>,
    ) -> Result<String>;

    /// Decrypt a JWE
    async fn decrypt_jwe(&self, jwe: &str, expected_kid: Option<&str>) -> Result<Vec<u8>>;
}

/// A default implementation of the KeyManager trait.
#[derive(Debug, Clone)]
pub struct DefaultKeyManager {
    /// The DID key generator
    pub generator: DIDKeyGenerator,
    /// The secret storage (legacy)
    pub secrets: Arc<RwLock<HashMap<String, Secret>>>,
    /// Signing keys
    signing_keys: Arc<RwLock<HashMap<String, Arc<dyn SigningKey + Send + Sync>>>>,
    /// Encryption keys
    encryption_keys: Arc<RwLock<HashMap<String, Arc<dyn EncryptionKey + Send + Sync>>>>,
    /// Decryption keys
    decryption_keys: Arc<RwLock<HashMap<String, Arc<dyn DecryptionKey + Send + Sync>>>>,
    /// Verification keys
    verification_keys: Arc<RwLock<HashMap<String, Arc<dyn VerificationKey + Send + Sync>>>>,
}

impl DefaultKeyManager {
    /// Create a new key manager
    pub fn new() -> Self {
        Self {
            generator: DIDKeyGenerator::new(),
            secrets: Arc::new(RwLock::new(HashMap::new())),
            signing_keys: Arc::new(RwLock::new(HashMap::new())),
            encryption_keys: Arc::new(RwLock::new(HashMap::new())),
            decryption_keys: Arc::new(RwLock::new(HashMap::new())),
            verification_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a LocalAgentKey from a GeneratedKey
    pub fn agent_key_from_generated(&self, key: &GeneratedKey) -> Result<LocalAgentKey> {
        // Create a secret for the key
        let secret = self.generator.create_secret_from_key(key);

        // Create a LocalAgentKey
        Ok(LocalAgentKey::new(secret, key.key_type))
    }
}

impl Default for DefaultKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl KeyManager for DefaultKeyManager {
    /// Get access to the secrets storage
    fn secrets(&self) -> Arc<RwLock<HashMap<String, Secret>>> {
        Arc::clone(&self.secrets)
    }

    /// Generate a new key with the specified options
    fn generate_key(&self, options: DIDGenerationOptions) -> Result<GeneratedKey> {
        // Generate the key
        let key = self.generator.generate_did(options)?;

        // Create a LocalAgentKey
        let agent_key = self.agent_key_from_generated(&key)?;

        // Store the legacy secret
        if let Ok(mut secrets) = self.secrets.write() {
            secrets.insert(key.did.clone(), agent_key.clone().secret);
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        // Store the agent key as signing, encryption, and decryption keys
        if let Ok(mut signing_keys) = self.signing_keys.write() {
            signing_keys.insert(
                AgentKey::key_id(&agent_key).to_string(),
                Arc::new(agent_key.clone()) as Arc<dyn SigningKey + Send + Sync>,
            );
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        if let Ok(mut encryption_keys) = self.encryption_keys.write() {
            encryption_keys.insert(
                AgentKey::key_id(&agent_key).to_string(),
                Arc::new(agent_key.clone()) as Arc<dyn EncryptionKey + Send + Sync>,
            );
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        if let Ok(mut decryption_keys) = self.decryption_keys.write() {
            decryption_keys.insert(
                AgentKey::key_id(&agent_key).to_string(),
                Arc::new(agent_key.clone()) as Arc<dyn DecryptionKey + Send + Sync>,
            );
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        // Also store a reference in verification keys
        if let Ok(mut verification_keys) = self.verification_keys.write() {
            verification_keys.insert(
                AgentKey::key_id(&agent_key).to_string(),
                Arc::new(agent_key.clone()) as Arc<dyn VerificationKey + Send + Sync>,
            );
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

        // Create a LocalAgentKey
        let agent_key = self.agent_key_from_generated(&key)?;

        // Store the legacy secret
        if let Ok(mut secrets) = self.secrets.write() {
            secrets.insert(key.did.clone(), agent_key.clone().secret);
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        // Store the agent key as signing, encryption, and decryption keys
        if let Ok(mut signing_keys) = self.signing_keys.write() {
            signing_keys.insert(
                AgentKey::key_id(&agent_key).to_string(),
                Arc::new(agent_key.clone()) as Arc<dyn SigningKey + Send + Sync>,
            );
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        if let Ok(mut encryption_keys) = self.encryption_keys.write() {
            encryption_keys.insert(
                AgentKey::key_id(&agent_key).to_string(),
                Arc::new(agent_key.clone()) as Arc<dyn EncryptionKey + Send + Sync>,
            );
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        if let Ok(mut decryption_keys) = self.decryption_keys.write() {
            decryption_keys.insert(
                AgentKey::key_id(&agent_key).to_string(),
                Arc::new(agent_key.clone()) as Arc<dyn DecryptionKey + Send + Sync>,
            );
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        // Also store a reference in verification keys
        if let Ok(mut verification_keys) = self.verification_keys.write() {
            verification_keys.insert(
                AgentKey::key_id(&agent_key).to_string(),
                Arc::new(agent_key.clone()) as Arc<dyn VerificationKey + Send + Sync>,
            );
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        Ok(key)
    }

    /// Add an existing key to the key manager
    fn add_key(&self, key: &GeneratedKey) -> Result<()> {
        // Create a LocalAgentKey
        let agent_key = self.agent_key_from_generated(key)?;

        // Store the legacy secret
        if let Ok(mut secrets) = self.secrets.write() {
            secrets.insert(key.did.clone(), agent_key.clone().secret);
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        // Store the agent key as signing, encryption, and decryption keys
        if let Ok(mut signing_keys) = self.signing_keys.write() {
            signing_keys.insert(
                AgentKey::key_id(&agent_key).to_string(),
                Arc::new(agent_key.clone()) as Arc<dyn SigningKey + Send + Sync>,
            );
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        if let Ok(mut encryption_keys) = self.encryption_keys.write() {
            encryption_keys.insert(
                AgentKey::key_id(&agent_key).to_string(),
                Arc::new(agent_key.clone()) as Arc<dyn EncryptionKey + Send + Sync>,
            );
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        if let Ok(mut decryption_keys) = self.decryption_keys.write() {
            decryption_keys.insert(
                AgentKey::key_id(&agent_key).to_string(),
                Arc::new(agent_key.clone()) as Arc<dyn DecryptionKey + Send + Sync>,
            );
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        // Also store a reference in verification keys
        if let Ok(mut verification_keys) = self.verification_keys.write() {
            verification_keys.insert(
                AgentKey::key_id(&agent_key).to_string(),
                Arc::new(agent_key.clone()) as Arc<dyn VerificationKey + Send + Sync>,
            );
        } else {
            return Err(Error::FailedToAcquireResolverWriteLock);
        }

        Ok(())
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
                // Create a LocalAgentKey
                let key_type = crate::did::KeyType::Ed25519; // Default to Ed25519
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
                // Create a LocalAgentKey
                let key_type = crate::did::KeyType::Ed25519; // Default to Ed25519
                let agent_key = LocalAgentKey::new(secret.clone(), key_type);

                // Add to encryption keys for next time
                if let Ok(mut encryption_keys) = self.encryption_keys.write() {
                    let arc_key =
                        Arc::new(agent_key.clone()) as Arc<dyn EncryptionKey + Send + Sync>;
                    encryption_keys.insert(AgentKey::key_id(&agent_key).to_string(), arc_key.clone());
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
                // Create a LocalAgentKey
                let key_type = crate::did::KeyType::Ed25519; // Default to Ed25519
                let agent_key = LocalAgentKey::new(secret.clone(), key_type);

                // Add to decryption keys for next time
                if let Ok(mut decryption_keys) = self.decryption_keys.write() {
                    let arc_key =
                        Arc::new(agent_key.clone()) as Arc<dyn DecryptionKey + Send + Sync>;
                    decryption_keys.insert(AgentKey::key_id(&agent_key).to_string(), arc_key.clone());
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

        // TODO: If not found locally, use DID Resolver to look up the public key
        // For now, we'll just check our signing keys and create verification keys from them

        // Check if we can resolve from a signing key
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

        // In a real implementation, we would use a DID Resolver here
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
        protected_header: Option<crate::message::JwsProtected>,
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
                .find(|s| s.header.kid == kid)
                .ok_or_else(|| {
                    Error::Cryptography(format!("No signature found with kid: {}", kid))
                })?
        } else {
            // Use the first signature
            jws.signatures
                .first()
                .ok_or_else(|| Error::Cryptography("No signatures in JWS".to_string()))?
        };

        // Decode the protected header
        let protected_bytes = base64::engine::general_purpose::STANDARD
            .decode(&signature.protected)
            .map_err(|e| {
                Error::Cryptography(format!("Failed to decode protected header: {}", e))
            })?;

        // Parse the protected header
        let protected: crate::message::JwsProtected = serde_json::from_slice(&protected_bytes)
            .map_err(|e| {
                Error::Serialization(format!("Failed to parse protected header: {}", e))
            })?;

        // Resolve the verification key
        let verification_key = KeyManager::resolve_verification_key(self, &signature.header.kid).await?;

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
        protected_header: Option<crate::message::JweProtected>,
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
            let recipient = jwe
                .recipients
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
                if let Ok(decryption_key) = KeyManager::get_decryption_key(self, &recipient.header.kid).await {
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

#[async_trait]
impl KeyManagerPacking for DefaultKeyManager {
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

/// A builder for KeyManager
#[derive(Debug, Clone)]
pub struct KeyManagerBuilder {
    /// The DID key generator
    generator: DIDKeyGenerator,
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
    storage_path: Option<std::path::PathBuf>,
}

impl KeyManagerBuilder {
    /// Create a new KeyManagerBuilder
    pub fn new() -> Self {
        Self {
            generator: DIDKeyGenerator::new(),
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
    pub fn load_from_path(mut self, path: std::path::PathBuf) -> Self {
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
    pub fn build(self) -> Result<DefaultKeyManager> {
        let mut key_manager = DefaultKeyManager {
            generator: self.generator,
            secrets: Arc::new(RwLock::new(self.secrets)),
            signing_keys: Arc::new(RwLock::new(self.signing_keys)),
            encryption_keys: Arc::new(RwLock::new(self.encryption_keys)),
            decryption_keys: Arc::new(RwLock::new(self.decryption_keys)),
            verification_keys: Arc::new(RwLock::new(self.verification_keys)),
        };

        // Load keys from storage if requested
        if self.load_from_storage {
            use crate::storage::KeyStorage;

            let storage = if let Some(path) = self.storage_path {
                KeyStorage::load_from_path(&path)?
            } else {
                KeyStorage::load_default()?
            };

            // Process each stored key
            for (did, stored_key) in storage.keys {
                // Convert to a legacy secret
                let secret = KeyStorage::to_secret(&stored_key);

                // Add to secrets
                if let Ok(mut secrets) = key_manager.secrets.write() {
                    secrets.insert(did.clone(), secret.clone());
                } else {
                    return Err(Error::FailedToAcquireResolverWriteLock);
                }

                // Create an agent key
                let key_type = stored_key.key_type;
                let agent_key = LocalAgentKey::new(secret, key_type);

                // Add to signing keys
                if let Ok(mut signing_keys) = key_manager.signing_keys.write() {
                    signing_keys.insert(
                        AgentKey::key_id(&agent_key).to_string(),
                        Arc::new(agent_key.clone()) as Arc<dyn SigningKey + Send + Sync>,
                    );
                } else {
                    return Err(Error::FailedToAcquireResolverWriteLock);
                }

                // Add to encryption keys
                if let Ok(mut encryption_keys) = key_manager.encryption_keys.write() {
                    encryption_keys.insert(
                        AgentKey::key_id(&agent_key).to_string(),
                        Arc::new(agent_key.clone()) as Arc<dyn EncryptionKey + Send + Sync>,
                    );
                } else {
                    return Err(Error::FailedToAcquireResolverWriteLock);
                }

                // Add to decryption keys
                if let Ok(mut decryption_keys) = key_manager.decryption_keys.write() {
                    decryption_keys.insert(
                        AgentKey::key_id(&agent_key).to_string(),
                        Arc::new(agent_key.clone()) as Arc<dyn DecryptionKey + Send + Sync>,
                    );
                } else {
                    return Err(Error::FailedToAcquireResolverWriteLock);
                }

                // Add to verification keys
                if let Ok(mut verification_keys) = key_manager.verification_keys.write() {
                    verification_keys.insert(
                        AgentKey::key_id(&agent_key).to_string(),
                        Arc::new(agent_key.clone()) as Arc<dyn VerificationKey + Send + Sync>,
                    );
                } else {
                    return Err(Error::FailedToAcquireResolverWriteLock);
                }
            }
        }

        Ok(key_manager)
    }
}

// Legacy Secret Resolver

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
            secrets: key_manager.secrets(),
        }
    }
    
    /// Create a new KeyManagerSecretResolver directly from secrets
    pub fn new_from_secrets(secrets: Arc<RwLock<HashMap<String, Secret>>>) -> Self {
        Self {
            secrets,
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
        let options = crate::did::DIDGenerationOptions {
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

    #[tokio::test]
    async fn test_agent_key_operations() {
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

        // Test signing and verification
        let test_data = b"Hello, world!";

        // Ed25519
        let ed25519_kid = format!("{}#keys-1", ed25519_key.did);
        let signing_key = KeyManager::get_signing_key(&manager, &ed25519_kid).await.unwrap();
        let signature = signing_key.sign(test_data).await.unwrap();

        let verification_key = KeyManager::resolve_verification_key(&manager, &ed25519_kid).await.unwrap();
        let protected = crate::message::JwsProtected {
            typ: "application/didcomm-signed+json".to_string(),
            alg: "EdDSA".to_string(),
        };

        let verified = verification_key
            .verify_signature(test_data, &signature, &protected)
            .await
            .unwrap();
        assert!(verified);

        // P-256
        let p256_kid = format!("{}#keys-1", p256_key.did);
        let signing_key = KeyManager::get_signing_key(&manager, &p256_kid).await.unwrap();
        let signature = signing_key.sign(test_data).await.unwrap();

        let verification_key = KeyManager::resolve_verification_key(&manager, &p256_kid).await.unwrap();
        let protected = crate::message::JwsProtected {
            typ: "application/didcomm-signed+json".to_string(),
            alg: "ES256".to_string(),
        };

        let verified = verification_key
            .verify_signature(test_data, &signature, &protected)
            .await
            .unwrap();
        assert!(verified);

        // secp256k1
        let secp256k1_kid = format!("{}#keys-1", secp256k1_key.did);
        let signing_key = KeyManager::get_signing_key(&manager, &secp256k1_kid).await.unwrap();
        let signature = signing_key.sign(test_data).await.unwrap();

        let verification_key = KeyManager::resolve_verification_key(&manager, &secp256k1_kid).await.unwrap();
        let protected = crate::message::JwsProtected {
            typ: "application/didcomm-signed+json".to_string(),
            alg: "ES256K".to_string(),
        };

        let verified = verification_key
            .verify_signature(test_data, &signature, &protected)
            .await
            .unwrap();
        assert!(verified);
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

    #[tokio::test]
    async fn test_jws_operations() {
        let manager = DefaultKeyManager::new();

        // Generate a key
        let options = crate::did::DIDGenerationOptions {
            key_type: crate::did::KeyType::Ed25519,
        };

        let key = manager.generate_key(options).unwrap();
        let kid = format!("{}#keys-1", key.did);

        // Test data
        let test_data = b"Hello, world!";

        // Sign
        let jws = manager.sign_jws(&kid, test_data, None).await.unwrap();

        // Verify
        let payload = manager.verify_jws(&jws, Some(&kid)).await.unwrap();
        assert_eq!(payload, test_data);
    }

    #[tokio::test]
    async fn test_jwe_operations() {
        let manager = DefaultKeyManager::new();

        // Generate two keys (sender and recipient)
        let options = crate::did::DIDGenerationOptions {
            key_type: crate::did::KeyType::P256,
        };

        let sender_key = manager.generate_key(options.clone()).unwrap();
        let sender_kid = format!("{}#keys-1", sender_key.did);

        let recipient_key = manager.generate_key(options).unwrap();
        let recipient_kid = format!("{}#keys-1", recipient_key.did);

        // Test data
        let test_data = b"Hello, world!";

        // Encrypt
        let jwe = manager
            .encrypt_jwe(&sender_kid, &recipient_kid, test_data, None)
            .await
            .unwrap();

        // Decrypt
        let plaintext = manager
            .decrypt_jwe(&jwe, Some(&recipient_kid))
            .await
            .unwrap();
        assert_eq!(plaintext, test_data);
    }
}
