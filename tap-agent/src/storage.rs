//! Key storage functionality for TAP Agent
//!
//! This module provides utilities for persisting agent keys to disk
//! and loading them later. This allows for persistent agent identities
//! across multiple runs.

use crate::did::{GeneratedKey, KeyType};
use crate::error::{Error, Result};
use crate::key_manager::{Secret, SecretMaterial, SecretType};
use base64::Engine;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Default directory for TAP configuration and keys
pub const DEFAULT_TAP_DIR: &str = ".tap";
/// Default filename for the keys file
pub const DEFAULT_KEYS_FILE: &str = "keys.json";

/// A structure representing a stored key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredKey {
    /// The DID for this key
    pub did: String,
    /// The key type (e.g., Ed25519, P256)
    #[serde(with = "key_type_serde")]
    pub key_type: KeyType,
    /// Base64-encoded private key
    pub private_key: String,
    /// Base64-encoded public key
    pub public_key: String,
    /// Optional metadata for this key
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Serialization helper for KeyType
mod key_type_serde {
    use super::KeyType;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(key_type: &KeyType, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match key_type {
            KeyType::Ed25519 => "Ed25519",
            KeyType::P256 => "P256",
            KeyType::Secp256k1 => "Secp256k1",
        };
        serializer.serialize_str(s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<KeyType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Ed25519" => Ok(KeyType::Ed25519),
            "P256" => Ok(KeyType::P256),
            "Secp256k1" => Ok(KeyType::Secp256k1),
            _ => Err(serde::de::Error::custom(format!("Unknown key type: {}", s))),
        }
    }
}

/// A collection of stored keys
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyStorage {
    /// A map of DIDs to their stored keys
    pub keys: HashMap<String, StoredKey>,
    /// The default DID to use when not specified
    pub default_did: Option<String>,
    /// Creation timestamp
    #[serde(default = "chrono::Utc::now")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update timestamp
    #[serde(default = "chrono::Utc::now")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl KeyStorage {
    /// Create a new empty key storage
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a key to the storage
    pub fn add_key(&mut self, key: StoredKey) {
        // If this is the first key, make it the default
        if self.keys.is_empty() {
            self.default_did = Some(key.did.clone());
        }
        
        self.keys.insert(key.did.clone(), key);
        self.updated_at = chrono::Utc::now();
    }

    /// Get the default key path
    pub fn default_key_path() -> Option<PathBuf> {
        home_dir().map(|home| home.join(DEFAULT_TAP_DIR).join(DEFAULT_KEYS_FILE))
    }

    /// Load keys from the default location
    pub fn load_default() -> Result<Self> {
        let path = Self::default_key_path().ok_or_else(|| {
            Error::Storage("Could not determine home directory for default key path".to_string())
        })?;
        Self::load_from_path(&path)
    }

    /// Load keys from a specific path
    pub fn load_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let contents = fs::read_to_string(path).map_err(|e| {
            Error::Storage(format!("Failed to read key storage file: {}", e))
        })?;

        let storage: KeyStorage = serde_json::from_str(&contents).map_err(|e| {
            Error::Storage(format!("Failed to parse key storage file: {}", e))
        })?;

        Ok(storage)
    }

    /// Save keys to the default location
    pub fn save_default(&self) -> Result<()> {
        let path = Self::default_key_path().ok_or_else(|| {
            Error::Storage("Could not determine home directory for default key path".to_string())
        })?;
        
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                Error::Storage(format!("Failed to create key storage directory: {}", e))
            })?;
        }
        
        self.save_to_path(&path)
    }

    /// Save keys to a specific path
    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        let contents = serde_json::to_string_pretty(self).map_err(|e| {
            Error::Storage(format!("Failed to serialize key storage: {}", e))
        })?;

        fs::write(path, contents).map_err(|e| {
            Error::Storage(format!("Failed to write key storage file: {}", e))
        })?;

        Ok(())
    }

    /// Convert a GeneratedKey to a StoredKey
    pub fn from_generated_key(key: &GeneratedKey) -> StoredKey {
        StoredKey {
            did: key.did.clone(),
            key_type: key.key_type.clone(),
            private_key: base64::engine::general_purpose::STANDARD.encode(&key.private_key),
            public_key: base64::engine::general_purpose::STANDARD.encode(&key.public_key),
            metadata: HashMap::new(),
        }
    }

    /// Convert a StoredKey to a Secret
    pub fn to_secret(key: &StoredKey) -> Secret {
        Secret {
            id: key.did.clone(),
            type_: SecretType::JsonWebKey2020,
            secret_material: SecretMaterial::JWK {
                private_key_jwk: generate_jwk_for_key(key),
            },
        }
    }
}

/// Generate a JWK for a stored key
fn generate_jwk_for_key(key: &StoredKey) -> serde_json::Value {
    match key.key_type {
        KeyType::Ed25519 => {
            serde_json::json!({
                "kty": "OKP",
                "crv": "Ed25519",
                "x": key.public_key,
                "d": key.private_key,
                "kid": format!("{}#keys-1", key.did)
            })
        }
        KeyType::P256 => {
            serde_json::json!({
                "kty": "EC",
                "crv": "P-256",
                "x": key.public_key,
                "d": key.private_key,
                "kid": format!("{}#keys-1", key.did)
            })
        }
        KeyType::Secp256k1 => {
            serde_json::json!({
                "kty": "EC",
                "crv": "secp256k1",
                "x": key.public_key,
                "d": key.private_key,
                "kid": format!("{}#keys-1", key.did)
            })
        }
    }
}