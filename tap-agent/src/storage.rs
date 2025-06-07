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
use std::env;
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
    /// Human-friendly label for this key
    #[serde(default)]
    pub label: String,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use tempfile::TempDir;

    #[test]
    #[serial]
    fn test_tap_home_environment_variable() {
        // Save current env vars
        let old_home = env::var("TAP_HOME").ok();
        let old_test = env::var("TAP_TEST_DIR").ok();

        // Clear env vars
        env::remove_var("TAP_HOME");
        env::remove_var("TAP_TEST_DIR");

        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        // Set TAP_HOME
        env::set_var("TAP_HOME", &temp_path);

        // Get the default key path
        let key_path = KeyStorage::default_key_path().unwrap();

        // Verify it uses TAP_HOME
        assert_eq!(key_path, temp_path.join(DEFAULT_KEYS_FILE));

        // Restore env vars
        env::remove_var("TAP_HOME");
        if let Some(val) = old_home {
            env::set_var("TAP_HOME", val);
        }
        if let Some(val) = old_test {
            env::set_var("TAP_TEST_DIR", val);
        }
    }

    #[test]
    #[serial]
    fn test_tap_test_dir_environment_variable() {
        
        // Save current env vars
        let old_home = env::var("TAP_HOME").ok();
        let old_test = env::var("TAP_TEST_DIR").ok();
        
        // Clear env vars
        env::remove_var("TAP_HOME");
        env::remove_var("TAP_TEST_DIR");
        
        // Create a temporary directory and keep it alive
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        
        // Set TAP_TEST_DIR
        env::set_var("TAP_TEST_DIR", &temp_path);
        
        // Get the default key path
        let key_path = KeyStorage::default_key_path().unwrap();
        
        // Verify it uses TAP_TEST_DIR/.tap
        let expected_path = temp_path.join(DEFAULT_TAP_DIR).join(DEFAULT_KEYS_FILE);
        assert_eq!(key_path, expected_path);
        
        // Restore env vars
        env::remove_var("TAP_TEST_DIR");
        if let Some(val) = old_home {
            env::set_var("TAP_HOME", val);
        }
        if let Some(val) = old_test {
            env::set_var("TAP_TEST_DIR", val);
        }
        
        // Keep temp_dir alive until the end of the test
        drop(temp_dir);
    }

    #[test]
    #[serial]
    fn test_environment_variable_priority() {
        // Save current env vars
        let old_home = env::var("TAP_HOME").ok();
        let old_test = env::var("TAP_TEST_DIR").ok();

        // Create temporary directories
        let home_dir = TempDir::new().unwrap();
        let test_dir = TempDir::new().unwrap();

        let home_path = home_dir.path().to_path_buf();
        let test_path = test_dir.path().to_path_buf();

        // Set both TAP_HOME and TAP_TEST_DIR
        env::set_var("TAP_HOME", &home_path);
        env::set_var("TAP_TEST_DIR", &test_path);

        // Get the default key path
        let key_path = KeyStorage::default_key_path().unwrap();

        // Verify TAP_HOME takes priority
        assert_eq!(key_path, home_path.join(DEFAULT_KEYS_FILE));

        // Restore env vars
        env::remove_var("TAP_HOME");
        env::remove_var("TAP_TEST_DIR");
        if let Some(val) = old_home {
            env::set_var("TAP_HOME", val);
        }
        if let Some(val) = old_test {
            env::set_var("TAP_TEST_DIR", val);
        }
    }

    #[test]
    #[serial]
    fn test_agent_directory_with_tap_home() {
        // Save current env vars
        let old_home = env::var("TAP_HOME").ok();
        let old_test = env::var("TAP_TEST_DIR").ok();

        // Clear env vars
        env::remove_var("TAP_HOME");
        env::remove_var("TAP_TEST_DIR");

        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        // Set TAP_HOME
        env::set_var("TAP_HOME", &temp_path);

        // Create a storage instance
        let storage = KeyStorage::new();

        // Get agent directory
        let agent_dir = storage.get_agent_directory("did:key:test123").unwrap();

        // Verify it uses TAP_HOME with sanitized DID
        assert_eq!(agent_dir, temp_path.join("did:key:test123"));

        // Restore env vars
        env::remove_var("TAP_HOME");
        if let Some(val) = old_home {
            env::set_var("TAP_HOME", val);
        }
        if let Some(val) = old_test {
            env::set_var("TAP_TEST_DIR", val);
        }
    }

    #[test]
    #[serial]
    fn test_storage_persistence_with_temp_dir() {
        // Save current env vars
        let old_home = env::var("TAP_HOME").ok();
        let old_test = env::var("TAP_TEST_DIR").ok();

        // Clear env vars
        env::remove_var("TAP_HOME");
        env::remove_var("TAP_TEST_DIR");

        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        // Set TAP_HOME
        env::set_var("TAP_HOME", &temp_path);

        // Create and save storage
        let mut storage = KeyStorage::new();
        storage.add_key(StoredKey {
            did: "did:key:test".to_string(),
            label: "test-key".to_string(),
            key_type: KeyType::Ed25519,
            private_key: "test-private".to_string(),
            public_key: "test-public".to_string(),
            metadata: HashMap::new(),
        });

        // Save to default location
        storage.save_default().unwrap();

        // Verify file was created in temp directory
        let expected_path = temp_path.join(DEFAULT_KEYS_FILE);
        assert!(expected_path.exists());

        // Load it back
        let loaded = KeyStorage::load_default().unwrap();
        assert_eq!(loaded.keys.len(), 1);
        assert!(loaded.keys.contains_key("did:key:test"));

        // Restore env vars
        env::remove_var("TAP_HOME");
        if let Some(val) = old_home {
            env::set_var("TAP_HOME", val);
        }
        if let Some(val) = old_test {
            env::set_var("TAP_TEST_DIR", val);
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
    /// Base directory for agent subdirectories (not serialized, runtime only)
    #[serde(skip)]
    base_directory: Option<PathBuf>,
}

impl KeyStorage {
    /// Create a new empty key storage
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a key to the storage
    pub fn add_key(&mut self, mut key: StoredKey) {
        // Generate default label if not provided
        if key.label.is_empty() {
            key.label = self.generate_default_label();
        }

        // Ensure label is unique
        let final_label = self.ensure_unique_label(&key.label, Some(&key.did));
        key.label = final_label.clone();

        // If this is the first key, make it the default
        if self.keys.is_empty() {
            self.default_did = Some(key.did.clone());
        }

        self.keys.insert(key.did.clone(), key);
        self.updated_at = chrono::Utc::now();
    }

    /// Generate a default label in the format agent-{n}
    fn generate_default_label(&self) -> String {
        let mut counter = 1;
        loop {
            let label = format!("agent-{}", counter);
            if !self.keys.values().any(|key| key.label == label) {
                return label;
            }
            counter += 1;
        }
    }

    /// Ensure a label is unique, modifying it if necessary
    fn ensure_unique_label(&self, desired_label: &str, exclude_did: Option<&str>) -> String {
        // Check if the label exists in any key
        if let Some(existing_key) = self.keys.values().find(|key| key.label == desired_label) {
            // If it belongs to the same DID we're updating, it's fine
            if exclude_did.is_some() && existing_key.did == exclude_did.unwrap() {
                return desired_label.to_string();
            }
        } else {
            // Label doesn't exist, so it's available
            return desired_label.to_string();
        }

        // Generate a unique label by appending a number
        let mut counter = 2;
        loop {
            let new_label = format!("{}-{}", desired_label, counter);
            if !self.keys.values().any(|key| key.label == new_label) {
                return new_label;
            }
            counter += 1;
        }
    }

    /// Find a key by label
    pub fn find_by_label(&self, label: &str) -> Option<&StoredKey> {
        self.keys.values().find(|key| key.label == label)
    }

    /// Update a key's label
    pub fn update_label(&mut self, did: &str, new_label: &str) -> Result<()> {
        // First ensure the key exists
        if !self.keys.contains_key(did) {
            return Err(Error::Storage(format!("Key with DID '{}' not found", did)));
        }

        // Ensure new label is unique
        let final_label = self.ensure_unique_label(new_label, Some(did));

        // Update the key's label
        if let Some(key) = self.keys.get_mut(did) {
            key.label = final_label;
        }

        self.updated_at = chrono::Utc::now();
        Ok(())
    }

    /// Get the default key path
    pub fn default_key_path() -> Option<PathBuf> {
        // Check for TAP_HOME environment variable first (useful for tests)
        if let Ok(tap_home) = env::var("TAP_HOME") {
            return Some(PathBuf::from(tap_home).join(DEFAULT_KEYS_FILE));
        }

        // Check for TAP_TEST_DIR environment variable (for tests/examples)
        if let Ok(test_dir) = env::var("TAP_TEST_DIR") {
            return Some(
                PathBuf::from(test_dir)
                    .join(DEFAULT_TAP_DIR)
                    .join(DEFAULT_KEYS_FILE),
            );
        }

        // Default to home directory
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
        let mut storage = if !path.exists() {
            Self::new()
        } else {
            let contents = fs::read_to_string(path)
                .map_err(|e| Error::Storage(format!("Failed to read key storage file: {}", e)))?;

            let mut storage: KeyStorage = serde_json::from_str(&contents)
                .map_err(|e| Error::Storage(format!("Failed to parse key storage file: {}", e)))?;

            // Ensure all keys have labels (for backward compatibility)
            storage.ensure_all_keys_have_labels();

            storage
        };

        // Set base directory from the path for agent subdirectories
        if let Some(parent) = path.parent() {
            storage.base_directory = Some(parent.to_path_buf());
        }

        Ok(storage)
    }

    /// Ensure all keys have labels (for backward compatibility)
    fn ensure_all_keys_have_labels(&mut self) {
        let mut keys_to_update = Vec::new();

        for (did, key) in &self.keys {
            if key.label.is_empty() {
                keys_to_update.push(did.clone());
            }
        }

        for did in keys_to_update {
            let new_label = self.generate_default_label();
            if let Some(key) = self.keys.get_mut(&did) {
                key.label = new_label;
            }
        }
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
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| Error::Storage(format!("Failed to serialize key storage: {}", e)))?;

        fs::write(path, contents)
            .map_err(|e| Error::Storage(format!("Failed to write key storage file: {}", e)))?;

        Ok(())
    }

    /// Convert a GeneratedKey to a StoredKey
    pub fn from_generated_key(key: &GeneratedKey) -> StoredKey {
        StoredKey {
            did: key.did.clone(),
            label: String::new(), // Will be set when added to storage
            key_type: key.key_type,
            private_key: base64::engine::general_purpose::STANDARD.encode(&key.private_key),
            public_key: base64::engine::general_purpose::STANDARD.encode(&key.public_key),
            metadata: HashMap::new(),
        }
    }

    /// Convert a GeneratedKey to a StoredKey with a specific label
    pub fn from_generated_key_with_label(key: &GeneratedKey, label: &str) -> StoredKey {
        StoredKey {
            did: key.did.clone(),
            label: label.to_string(),
            key_type: key.key_type,
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
    /// Create agent directory and save policies/metadata files
    pub fn create_agent_directory(
        &self,
        did: &str,
        policies: &[String],
        metadata: &HashMap<String, String>,
    ) -> Result<()> {
        let sanitized_did = sanitize_did(did);
        let agent_dir = self.get_agent_directory(&sanitized_did)?;

        // Create the agent directory
        fs::create_dir_all(&agent_dir).map_err(|e| {
            Error::Storage(format!(
                "Failed to create agent directory {}: {}",
                agent_dir.display(),
                e
            ))
        })?;

        // Save policies.json
        let policies_file = agent_dir.join("policies.json");
        let policies_json = serde_json::to_string_pretty(policies)
            .map_err(|e| Error::Storage(format!("Failed to serialize policies: {}", e)))?;
        fs::write(&policies_file, policies_json)
            .map_err(|e| Error::Storage(format!("Failed to write policies file: {}", e)))?;

        // Save metadata.json
        let metadata_file = agent_dir.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(metadata)
            .map_err(|e| Error::Storage(format!("Failed to serialize metadata: {}", e)))?;
        fs::write(&metadata_file, metadata_json)
            .map_err(|e| Error::Storage(format!("Failed to write metadata file: {}", e)))?;

        Ok(())
    }

    /// Load policies from agent directory
    pub fn load_agent_policies(&self, did: &str) -> Result<Vec<String>> {
        let sanitized_did = sanitize_did(did);
        let agent_dir = self.get_agent_directory(&sanitized_did)?;
        let policies_file = agent_dir.join("policies.json");

        if !policies_file.exists() {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(&policies_file)
            .map_err(|e| Error::Storage(format!("Failed to read policies file: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| Error::Storage(format!("Failed to parse policies file: {}", e)))
    }

    /// Load metadata from agent directory
    pub fn load_agent_metadata(&self, did: &str) -> Result<HashMap<String, String>> {
        let sanitized_did = sanitize_did(did);
        let agent_dir = self.get_agent_directory(&sanitized_did)?;
        let metadata_file = agent_dir.join("metadata.json");

        if !metadata_file.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&metadata_file)
            .map_err(|e| Error::Storage(format!("Failed to read metadata file: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| Error::Storage(format!("Failed to parse metadata file: {}", e)))
    }

    /// Get the agent directory path for a given DID
    fn get_agent_directory(&self, sanitized_did: &str) -> Result<PathBuf> {
        let base_dir = if let Some(ref base) = self.base_directory {
            base.clone()
        } else {
            // Check for TAP_HOME environment variable first
            if let Ok(tap_home) = env::var("TAP_HOME") {
                PathBuf::from(tap_home)
            } else if let Ok(test_dir) = env::var("TAP_TEST_DIR") {
                // For tests, use TAP_TEST_DIR/.tap
                PathBuf::from(test_dir).join(DEFAULT_TAP_DIR)
            } else {
                let home = home_dir().ok_or_else(|| {
                    Error::Storage("Could not determine home directory".to_string())
                })?;
                home.join(DEFAULT_TAP_DIR)
            }
        };
        Ok(base_dir.join(sanitized_did))
    }
}

/// Sanitize a DID for use as a directory name (same as TAP Node)
fn sanitize_did(did: &str) -> String {
    did.replace(':', "_")
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
