//! Test utilities for TAP Agent
//!
//! This module provides utilities for testing that use temporary directories
//! instead of the production ~/.tap directory.

use crate::error::Result;
use crate::storage::KeyStorage;
use std::env;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test storage wrapper that uses a temporary directory
pub struct TestStorage {
    /// The temporary directory (kept alive for the duration of the test)
    _temp_dir: TempDir,
    /// Path to the storage file within the temp directory
    pub storage_path: PathBuf,
}

impl TestStorage {
    /// Create a new test storage in a temporary directory
    pub fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir().map_err(|e| {
            crate::error::Error::Storage(format!("Failed to create temp dir: {}", e))
        })?;

        // Set TAP_HOME to the temp directory to ensure all TAP operations use it
        env::set_var("TAP_HOME", temp_dir.path());

        let storage_path = temp_dir.path().join("keys.json");

        Ok(Self {
            _temp_dir: temp_dir,
            storage_path,
        })
    }

    /// Load key storage from the test directory
    pub fn load(&self) -> Result<KeyStorage> {
        KeyStorage::load_from_path(&self.storage_path)
    }

    /// Save key storage to the test directory
    pub fn save(&self, storage: &KeyStorage) -> Result<()> {
        storage.save_to_path(&self.storage_path)
    }

    /// Get the path to the storage file
    pub fn path(&self) -> &PathBuf {
        &self.storage_path
    }

    /// Get the directory path (for agent subdirectories)
    pub fn directory(&self) -> PathBuf {
        self._temp_dir.path().to_path_buf()
    }
}

impl Default for TestStorage {
    fn default() -> Self {
        Self::new().expect("Failed to create test storage")
    }
}

/// Creates a temporary storage path for testing
/// This creates the storage file in a temporary directory
pub fn temp_storage_path() -> PathBuf {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // Set TAP_HOME to the temp directory
    env::set_var("TAP_HOME", temp_dir.path());

    let path = temp_dir.path().join("keys.json");

    // We need to leak the temp_dir to keep it alive for the test
    // This is acceptable for tests as they're short-lived
    std::mem::forget(temp_dir);

    path
}

/// Create a temporary directory path for testing
/// Creates a temporary .tap directory for testing
/// This creates the .tap equivalent in a temporary directory
pub fn temp_tap_directory() -> PathBuf {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let tap_dir = temp_dir.path().join(".tap");

    // Create the .tap directory
    std::fs::create_dir_all(&tap_dir).expect("Failed to create .tap directory");

    // Set TAP_HOME to point to the .tap directory
    env::set_var("TAP_HOME", &tap_dir);

    // We need to leak the temp_dir to keep it alive for the test
    std::mem::forget(temp_dir);

    tap_dir
}

/// Setup test environment with isolated TAP directory
/// This should be called at the beginning of each test to ensure isolation
pub fn setup_test_environment() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    env::set_var("TAP_TEST_DIR", temp_dir.path());
    std::mem::forget(temp_dir);
}

/// Reset test environment (removes TAP_TEST_DIR and TAP_HOME)
pub fn reset_test_environment() {
    env::remove_var("TAP_TEST_DIR");
    env::remove_var("TAP_HOME");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::did::KeyType;
    use crate::storage::{KeyStorage, StoredKey};
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_storage_can_be_created() {
        setup_test_environment();
        let test_storage = TestStorage::new().unwrap();
        assert!(test_storage.storage_path.ends_with("keys.json"));
        reset_test_environment();
    }

    #[test]
    #[serial]
    fn test_storage_save_and_load() {
        setup_test_environment();
        let test_storage = TestStorage::new().unwrap();

        // Create a test key
        let stored_key = StoredKey {
            did: "did:test:example".to_string(),
            label: "test-key".to_string(),
            key_type: KeyType::Ed25519,
            private_key: "test-private".to_string(),
            public_key: "test-public".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        // Save to test storage
        let mut storage = KeyStorage::new();
        storage.add_key(stored_key);
        test_storage.save(&storage).unwrap();

        // Load from test storage
        let loaded_storage = test_storage.load().unwrap();
        assert!(loaded_storage.keys.contains_key("did:test:example"));
        reset_test_environment();
    }

    #[test]
    #[serial]
    fn test_temp_storage_path() {
        setup_test_environment();
        let path = temp_storage_path();
        assert!(path.ends_with("keys.json"));
        reset_test_environment();
    }

    #[test]
    #[serial]
    fn test_temp_tap_directory() {
        setup_test_environment();
        let dir = temp_tap_directory();
        assert!(dir.ends_with(".tap"));
        assert!(dir.exists());
        reset_test_environment();
    }
}
