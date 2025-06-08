//! Tests for key label functionality

use std::env;
use std::fs;
use tap_agent::did::{DIDKeyGenerator, KeyType};
use tap_agent::error::Result;
use tap_agent::storage::KeyStorage;
use tempfile::TempDir;

#[test]
fn test_default_label_generation() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    env::set_var("TAP_HOME", temp_dir.path());
    let mut storage = KeyStorage::new();

    // Generate and add keys without labels
    let generator = DIDKeyGenerator::new();

    for i in 1..=5 {
        let key = generator.generate_ed25519_did()?;
        let stored_key = KeyStorage::from_generated_key(&key);
        storage.add_key(stored_key);

        // Verify the auto-generated label
        let added_key = storage.keys.get(&key.did).unwrap();
        assert_eq!(added_key.label, format!("agent-{}", i));
    }

    // Verify labels are assigned correctly
    assert_eq!(storage.keys.len(), 5);
    for i in 1..=5 {
        let label = format!("agent-{}", i);
        assert!(storage.keys.values().any(|key| key.label == label));
    }

    env::remove_var("TAP_HOME");
    Ok(())
}

#[test]
fn test_custom_label() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    env::set_var("TAP_HOME", temp_dir.path());
    let mut storage = KeyStorage::new();
    let generator = DIDKeyGenerator::new();

    // Add key with custom label
    let key = generator.generate_ed25519_did()?;
    let stored_key = KeyStorage::from_generated_key_with_label(&key, "production-key");
    storage.add_key(stored_key);

    // Verify custom label was preserved
    let added_key = storage.keys.get(&key.did).unwrap();
    assert_eq!(added_key.label, "production-key");

    // Verify label is set correctly
    assert_eq!(storage.keys.get(&key.did).unwrap().label, "production-key");

    env::remove_var("TAP_HOME");
    Ok(())
}

#[test]
fn test_label_uniqueness() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    env::set_var("TAP_HOME", temp_dir.path());
    let mut storage = KeyStorage::new();
    let generator = DIDKeyGenerator::new();

    // Add first key with label "test-key"
    let key1 = generator.generate_ed25519_did()?;
    let stored_key1 = KeyStorage::from_generated_key_with_label(&key1, "test-key");
    storage.add_key(stored_key1);

    // Add second key with same label - should be auto-modified
    let key2 = generator.generate_ed25519_did()?;
    let stored_key2 = KeyStorage::from_generated_key_with_label(&key2, "test-key");
    storage.add_key(stored_key2);

    // Verify labels
    let added_key1 = storage.keys.get(&key1.did).unwrap();
    let added_key2 = storage.keys.get(&key2.did).unwrap();

    assert_eq!(added_key1.label, "test-key");
    assert_eq!(added_key2.label, "test-key-2");

    env::remove_var("TAP_HOME");
    Ok(())
}

#[test]
fn test_find_by_label() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    env::set_var("TAP_HOME", temp_dir.path());
    let mut storage = KeyStorage::new();
    let generator = DIDKeyGenerator::new();

    // Add keys with different labels
    let key1 = generator.generate_ed25519_did()?;
    storage.add_key(KeyStorage::from_generated_key_with_label(
        &key1,
        "signing-key",
    ));

    let key2 = generator.generate_p256_did()?;
    storage.add_key(KeyStorage::from_generated_key_with_label(
        &key2,
        "encryption-key",
    ));

    // Test finding by label
    let found1 = storage.find_by_label("signing-key");
    assert!(found1.is_some());
    assert_eq!(found1.unwrap().did, key1.did);

    let found2 = storage.find_by_label("encryption-key");
    assert!(found2.is_some());
    assert_eq!(found2.unwrap().did, key2.did);

    let not_found = storage.find_by_label("non-existent");
    assert!(not_found.is_none());

    env::remove_var("TAP_HOME");
    Ok(())
}

#[test]
fn test_update_label() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    env::set_var("TAP_HOME", temp_dir.path());
    let mut storage = KeyStorage::new();
    let generator = DIDKeyGenerator::new();

    // Add key with initial label
    let key = generator.generate_ed25519_did()?;
    storage.add_key(KeyStorage::from_generated_key_with_label(&key, "old-label"));

    // Update label
    storage.update_label(&key.did, "new-label")?;

    // Verify update
    let updated_key = storage.keys.get(&key.did).unwrap();
    assert_eq!(updated_key.label, "new-label");

    // Verify label was updated
    assert!(!storage.keys.values().any(|k| k.label == "old-label"));
    assert_eq!(storage.keys.get(&key.did).unwrap().label, "new-label");

    // Verify can still find by new label
    let found = storage.find_by_label("new-label");
    assert!(found.is_some());
    assert_eq!(found.unwrap().did, key.did);

    env::remove_var("TAP_HOME");
    Ok(())
}

#[test]
fn test_storage_persistence_with_labels() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    env::set_var("TAP_HOME", temp_dir.path());
    let path = temp_dir.path().join("keys.json");

    // Create and save storage with labeled keys
    {
        let mut storage = KeyStorage::new();
        let generator = DIDKeyGenerator::new();

        let key1 = generator.generate_ed25519_did()?;
        storage.add_key(KeyStorage::from_generated_key_with_label(
            &key1,
            "test-key-1",
        ));

        let key2 = generator.generate_p256_did()?;
        storage.add_key(KeyStorage::from_generated_key_with_label(
            &key2,
            "test-key-2",
        ));

        storage.save_to_path(&path)?;
    }

    // Load and verify
    {
        let loaded_storage = KeyStorage::load_from_path(&path)?;

        assert_eq!(loaded_storage.keys.len(), 2);

        let key1 = loaded_storage.find_by_label("test-key-1");
        assert!(key1.is_some());
        assert_eq!(key1.unwrap().key_type, KeyType::Ed25519);

        let key2 = loaded_storage.find_by_label("test-key-2");
        assert!(key2.is_some());
        assert_eq!(key2.unwrap().key_type, KeyType::P256);
    }

    // Cleanup happens automatically when temp_dir is dropped
    env::remove_var("TAP_HOME");
    Ok(())
}

#[test]
fn test_backward_compatibility() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    env::set_var("TAP_HOME", temp_dir.path());
    let path = temp_dir.path().join("keys.json");

    // Create old-style storage without labels
    let old_storage_json = r#"{
        "keys": {
            "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK": {
                "did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                "key_type": "Ed25519",
                "private_key": "YmFzZTY0LWVuY29kZWQtcHJpdmF0ZS1rZXk=",
                "public_key": "YmFzZTY0LWVuY29kZWQtcHVibGljLWtleQ==",
                "metadata": {}
            },
            "did:key:z6MkrHKzgsahxBLyNAbLQyB1pcWNYC9GmywiWPgkrvntAZcj": {
                "did": "did:key:z6MkrHKzgsahxBLyNAbLQyB1pcWNYC9GmywiWPgkrvntAZcj",
                "key_type": "P256",
                "private_key": "YmFzZTY0LWVuY29kZWQtcHJpdmF0ZS1rZXk=",
                "public_key": "YmFzZTY0LWVuY29kZWQtcHVibGljLWtleQ==",
                "metadata": {}
            }
        },
        "default_did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z"
    }"#;

    fs::write(&path, old_storage_json)?;

    // Load old storage - should auto-generate labels
    let loaded_storage = KeyStorage::load_from_path(&path)?;

    // Verify labels were auto-generated
    assert_eq!(loaded_storage.keys.len(), 2);

    // Check that all keys have labels
    for (_, key) in &loaded_storage.keys {
        assert!(!key.label.is_empty());
        assert!(key.label.starts_with("agent-"));
    }

    // Verify can find by auto-generated labels
    let key1 = loaded_storage.find_by_label("agent-1");
    assert!(key1.is_some());

    let key2 = loaded_storage.find_by_label("agent-2");
    assert!(key2.is_some());

    // Cleanup happens automatically when temp_dir is dropped
    env::remove_var("TAP_HOME");
    Ok(())
}

#[test]
fn test_mixed_labeled_and_unlabeled_keys() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    env::set_var("TAP_HOME", temp_dir.path());
    let mut storage = KeyStorage::new();
    let generator = DIDKeyGenerator::new();

    // Add labeled key
    let key1 = generator.generate_ed25519_did()?;
    storage.add_key(KeyStorage::from_generated_key_with_label(
        &key1,
        "custom-label",
    ));

    // Add unlabeled key
    let key2 = generator.generate_ed25519_did()?;
    storage.add_key(KeyStorage::from_generated_key(&key2));

    // Add another labeled key
    let key3 = generator.generate_ed25519_did()?;
    storage.add_key(KeyStorage::from_generated_key_with_label(
        &key3,
        "another-custom",
    ));

    // Verify labels
    assert_eq!(storage.keys.get(&key1.did).unwrap().label, "custom-label");
    assert_eq!(storage.keys.get(&key2.did).unwrap().label, "agent-1");
    assert_eq!(storage.keys.get(&key3.did).unwrap().label, "another-custom");

    env::remove_var("TAP_HOME");
    Ok(())
}

#[test]
fn test_label_collision_with_auto_generated() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    env::set_var("TAP_HOME", temp_dir.path());
    let mut storage = KeyStorage::new();
    let generator = DIDKeyGenerator::new();

    // Add a key with label "agent-2"
    let key1 = generator.generate_ed25519_did()?;
    storage.add_key(KeyStorage::from_generated_key_with_label(&key1, "agent-2"));

    // Add unlabeled keys - should skip "agent-2"
    let key2 = generator.generate_ed25519_did()?;
    storage.add_key(KeyStorage::from_generated_key(&key2));

    let key3 = generator.generate_ed25519_did()?;
    storage.add_key(KeyStorage::from_generated_key(&key3));

    // Verify labels
    assert_eq!(storage.keys.get(&key1.did).unwrap().label, "agent-2");
    assert_eq!(storage.keys.get(&key2.did).unwrap().label, "agent-1");
    assert_eq!(storage.keys.get(&key3.did).unwrap().label, "agent-3");

    env::remove_var("TAP_HOME");
    Ok(())
}
