//! Example of TAP key generation and management
//!
//! This example demonstrates how to:
//! - Generate cryptographic keys
//! - Save keys to a temporary directory (for demonstration)
//! - Load existing keys
//! - Manage multiple keys
//!
//! Note: This example uses temporary storage to avoid affecting your production ~/.tap directory

use std::collections::HashMap;
use std::path::PathBuf;
use tap_agent::{
    did::{DIDKeyGenerator, DIDGenerationOptions, KeyType},
    storage::{KeyStorage, StoredKey},
    error::Result,
};
use tempfile::TempDir;

fn main() -> Result<()> {
    println!("TAP Key Management Example\n");

    // Create a temporary directory for this example to avoid affecting production ~/.tap
    let temp_dir = TempDir::new().map_err(|e| tap_agent::error::Error::Storage(e.to_string()))?;
    let storage_path = temp_dir.path().join("keys.json");
    
    println!("Using temporary storage at: {:?}\n", storage_path);
    println!("(This avoids affecting your production ~/.tap directory)\n");

    // Example 1: Generate a new Ed25519 key
    generate_new_key(&storage_path)?;
    
    // Example 2: Load existing keys
    load_and_display_keys(&storage_path)?;
    
    // Example 3: Generate multiple key types
    generate_multiple_key_types(&storage_path)?;
    
    // Example 4: Key rotation example
    demonstrate_key_rotation(&storage_path)?;
    
    println!("Example completed! All keys were stored in temporary directory.");
    println!("Your production ~/.tap directory was not affected.");

    Ok(())
}

/// Generate a new Ed25519 key and save it
fn generate_new_key(storage_path: &PathBuf) -> Result<()> {
    println!("=== Generating New Ed25519 Key ===");
    
    // Create key generator
    let generator = DIDKeyGenerator::new();
    
    // Generate Ed25519 key (default)
    let options = DIDGenerationOptions {
        key_type: KeyType::Ed25519,
    };
    let generated_key = generator.generate_did(options)?;
    
    println!("Generated DID: {}", generated_key.did);
    println!("Key Type: {:?}", generated_key.key_type);
    
    // Convert to storage format
    let stored_key = StoredKey {
        did: generated_key.did.clone(),
        key_type: generated_key.key_type,
        private_key: base64::encode(&generated_key.private_key),
        public_key: base64::encode(&generated_key.public_key),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("created_by".to_string(), "key_management_example".to_string());
            meta.insert("purpose".to_string(), "demonstration".to_string());
            meta
        },
    };
    
    // Load existing storage or create new
    let mut storage = KeyStorage::load_from_path(storage_path)
        .unwrap_or_else(|_| {
            println!("No existing key storage found, creating new one");
            KeyStorage::new()
        });
    
    // Add key to storage
    storage.add_key(stored_key);
    
    // Save to temporary location
    storage.save_to_path(storage_path)?;
    println!("Key saved to {:?}\n", storage_path);
    
    Ok(())
}

/// Load and display existing keys
fn load_and_display_keys(storage_path: &PathBuf) -> Result<()> {
    println!("=== Loading Existing Keys ===");
    
    match KeyStorage::load_from_path(storage_path) {
        Ok(storage) => {
            println!("Found {} key(s) in storage", storage.keys.len());
            
            if let Some(default_did) = &storage.default_did {
                println!("Default DID: {}", default_did);
            }
            
            println!("\nStored keys:");
            for (did, key) in &storage.keys {
                println!("\n  DID: {}", did);
                println!("  Key Type: {:?}", key.key_type);
                println!("  Public Key (first 32 chars): {}...", 
                    &key.public_key[..32.min(key.public_key.len())]);
                
                if !key.metadata.is_empty() {
                    println!("  Metadata:");
                    for (k, v) in &key.metadata {
                        println!("    {}: {}", k, v);
                    }
                }
            }
        }
        Err(e) => {
            println!("No key storage found or error loading: {}", e);
            println!("Run 'generate_new_key' first to create keys");
        }
    }
    
    println!();
    Ok(())
}

/// Generate keys of different types
fn generate_multiple_key_types(storage_path: &PathBuf) -> Result<()> {
    println!("=== Generating Multiple Key Types ===");
    
    let generator = DIDKeyGenerator::new();
    let mut storage = KeyStorage::load_from_path(storage_path)
        .unwrap_or_else(|_| KeyStorage::new());
    
    // Generate one key of each type
    let key_types = vec![
        (KeyType::Ed25519, "Ed25519 - Fast and secure"),
        (KeyType::P256, "P256 - NIST standard"),
        (KeyType::Secp256k1, "Secp256k1 - Bitcoin/Ethereum compatible"),
    ];
    
    for (key_type, description) in key_types {
        println!("\nGenerating {} key...", description);
        
        let options = DIDGenerationOptions { key_type };
        let generated_key = generator.generate_did(options)?;
        
        let stored_key = StoredKey {
            did: generated_key.did.clone(),
            key_type: generated_key.key_type,
            private_key: base64::encode(&generated_key.private_key),
            public_key: base64::encode(&generated_key.public_key),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("key_type_description".to_string(), description.to_string());
                meta
            },
        };
        
        storage.add_key(stored_key);
        println!("  Generated: {}", &generated_key.did[..50.min(generated_key.did.len())] );
    }
    
    storage.save_to_path(storage_path)?;
    println!("\nAll keys saved to {:?}\n", storage_path);
    
    Ok(())
}

/// Demonstrate key rotation
fn demonstrate_key_rotation(storage_path: &PathBuf) -> Result<()> {
    println!("=== Key Rotation Example ===");
    
    let mut storage = KeyStorage::load_from_path(storage_path)
        .unwrap_or_else(|_| KeyStorage::new());
    
    if storage.keys.is_empty() {
        println!("No keys found. Please run other examples first.");
        return Ok(());
    }
    
    // Get current default
    let old_default = storage.default_did.clone();
    println!("Current default DID: {:?}", old_default);
    
    // Generate a new key for rotation
    let generator = DIDKeyGenerator::new();
    let new_key = generator.generate_ed25519_did()?;
    
    let stored_key = StoredKey {
        did: new_key.did.clone(),
        key_type: new_key.key_type,
        private_key: base64::encode(&new_key.private_key),
        public_key: base64::encode(&new_key.public_key),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("purpose".to_string(), "key_rotation".to_string());
            meta.insert("rotated_from".to_string(), 
                old_default.unwrap_or_else(|| "none".to_string()));
            meta.insert("rotation_date".to_string(), 
                chrono::Utc::now().to_rfc3339());
            meta
        },
    };
    
    // Add new key and set as default
    storage.add_key(stored_key);
    storage.default_did = Some(new_key.did.clone());
    
    // Save updated storage
    storage.save_to_path(storage_path)?;
    
    println!("Key rotation complete!");
    println!("New default DID: {}", new_key.did);
    println!("\nOld keys are preserved for decrypting historical data.\n");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    
    #[test]
    fn test_key_generation() {
        let generator = DIDKeyGenerator::new();
        let key = generator.generate_ed25519_did().unwrap();
        
        assert!(key.did.starts_with("did:key:"));
        assert_eq!(key.private_key.len(), 32); // Ed25519 private key size
        assert_eq!(key.public_key.len(), 32);  // Ed25519 public key size
    }
    
    #[test]
    fn test_key_storage_roundtrip() {
        let generator = DIDKeyGenerator::new();
        let key = generator.generate_ed25519_did().unwrap();
        
        // Create stored key
        let stored_key = StoredKey {
            did: key.did.clone(),
            key_type: key.key_type,
            private_key: base64::encode(&key.private_key),
            public_key: base64::encode(&key.public_key),
            metadata: HashMap::new(),
        };
        
        // Create storage and add key
        let mut storage = KeyStorage::new();
        storage.add_key(stored_key.clone());
        
        // Verify storage
        assert_eq!(storage.keys.len(), 1);
        assert_eq!(storage.default_did, Some(key.did.clone()));
        
        let retrieved = &storage.keys[&key.did];
        assert_eq!(retrieved.did, stored_key.did);
        assert_eq!(retrieved.private_key, stored_key.private_key);
    }
}