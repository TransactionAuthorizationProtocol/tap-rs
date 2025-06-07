//! Example demonstrating key label functionality in TAP Agent
//!
//! This example shows how to:
//! - Generate keys with custom labels
//! - Access keys by their labels
//! - Manage and update labels
//! - Use labels in CLI commands

use tap_agent::{
    did::DIDKeyGenerator,
    storage::KeyStorage,
    error::Result,
};
use std::env;
use tempfile::TempDir;

fn main() -> Result<()> {
    println!("TAP Key Labels Demo\n");

    // Create temporary directory for this demo
    let temp_dir = TempDir::new().map_err(|e| tap_agent::error::Error::Storage(e.to_string()))?;
    
    // Set TAP_HOME to temporary directory to protect production ~/.tap
    env::set_var("TAP_HOME", temp_dir.path());
    
    println!("Using temporary storage at: {:?}", temp_dir.path());
    println!("(This protects your production ~/.tap directory)\n");

    // Example 1: Generate keys with custom labels
    generate_keys_with_labels()?;
    
    // Example 2: Access keys by labels
    access_keys_by_labels()?;
    
    // Example 3: Update labels
    update_key_labels()?;
    
    // Example 4: Show CLI usage examples
    show_cli_examples();

    println!("\nDemo completed! All operations used temporary storage.");
    println!("Your production ~/.tap directory was not affected.");

    Ok(())
}

/// Generate keys with custom labels
fn generate_keys_with_labels() -> Result<()> {
    println!("=== Generating Keys with Custom Labels ===\n");
    
    let generator = DIDKeyGenerator::new();
    let mut storage = KeyStorage::new();
    
    // Generate production signing key
    let signing_key = generator.generate_ed25519_did()?;
    let stored_signing = KeyStorage::from_generated_key_with_label(&signing_key, "production-signing");
    storage.add_key(stored_signing);
    println!("✓ Generated key with label: 'production-signing'");
    
    // Generate development key
    let dev_key = generator.generate_ed25519_did()?;
    let stored_dev = KeyStorage::from_generated_key_with_label(&dev_key, "development");
    storage.add_key(stored_dev);
    println!("✓ Generated key with label: 'development'");
    
    // Generate key without label (auto-generates)
    let auto_key = generator.generate_ed25519_did()?;
    let stored_auto = KeyStorage::from_generated_key(&auto_key);
    storage.add_key(stored_auto);
    let auto_label = storage.keys.get(&auto_key.did).unwrap().label.clone();
    println!("✓ Generated key with auto-label: '{}'", auto_label);
    
    // Save to storage
    storage.save_default()?;
    println!("\n✓ Keys saved to temporary storage\n");
    
    Ok(())
}

/// Access keys by their labels
fn access_keys_by_labels() -> Result<()> {
    println!("=== Accessing Keys by Labels ===\n");
    
    let storage = KeyStorage::load_default()?;
    
    // Find by label
    if let Some(prod_key) = storage.find_by_label("production-signing") {
        println!("Found production-signing key:");
        println!("  DID: {}", &prod_key.did[..50]);
        println!("  Type: {:?}", prod_key.key_type);
    }
    
    if let Some(dev_key) = storage.find_by_label("development") {
        println!("\nFound development key:");
        println!("  DID: {}", &dev_key.did[..50]);
        println!("  Type: {:?}", dev_key.key_type);
    }
    
    // Show all keys with their labels
    println!("\nAll stored keys:");
    println!("{:<20} {:<50} {:<10}", "Label", "DID", "Type");
    println!("{:-<80}", "");
    for (did, key) in &storage.keys {
        println!(
            "{:<20} {:<50} {:<10}",
            key.label,
            &did[..50.min(did.len())],
            format!("{:?}", key.key_type)
        );
    }
    
    println!();
    Ok(())
}

/// Update key labels
fn update_key_labels() -> Result<()> {
    println!("=== Updating Key Labels ===\n");
    
    let mut storage = KeyStorage::load_default()?;
    
    // Find a key to relabel
    if let Some(key) = storage.find_by_label("agent-1") {
        let did = key.did.clone();
        println!("Found key with label 'agent-1'");
        
        // Update the label
        storage.update_label(&did, "test-key")?;
        storage.save_default()?;
        
        println!("✓ Relabeled 'agent-1' to 'test-key'");
        
        // Verify the update
        if let Some(updated) = storage.find_by_label("test-key") {
            println!("✓ Verified: Key is now accessible as 'test-key'");
            println!("  DID: {}", &updated.did[..50]);
        }
    }
    
    println!();
    Ok(())
}

/// Show CLI usage examples
fn show_cli_examples() {
    println!("=== CLI Usage Examples ===\n");
    
    println!("Generate keys with labels:");
    println!("  tap-agent-cli generate --save --label \"production-key\"");
    println!("  tap-agent-cli generate --save --label \"backup-key\" --key-type p256");
    println!();
    
    println!("Access keys by label:");
    println!("  tap-agent-cli keys view \"production-key\"");
    println!("  tap-agent-cli keys set-default \"production-key\"");
    println!();
    
    println!("Use labels in other commands:");
    println!("  tap-agent-cli pack -i message.json -s \"production-key\" -r \"recipient-did\"");
    println!("  tap-agent-cli unpack -i packed.json -r \"production-key\"");
    println!();
    
    println!("Manage labels:");
    println!("  tap-agent-cli keys relabel \"agent-1\" \"my-signing-key\"");
    println!("  tap-agent-cli keys delete \"old-key\"");
    println!();
    
    println!("Import with label:");
    println!("  tap-agent-cli import key.json --label \"imported-key\"");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;
    
    #[test]
    fn test_label_examples() -> Result<()> {
        // Set up temporary storage
        let temp_dir = TempDir::new().unwrap();
        env::set_var("TAP_HOME", temp_dir.path());
        
        // Create temporary storage
        let temp_path = temp_dir.path().join("tap_label_demo_test.json");
        
        // Test generating with labels
        let generator = DIDKeyGenerator::new();
        let mut storage = KeyStorage::new();
        
        let key = generator.generate_ed25519_did()?;
        let stored = KeyStorage::from_generated_key_with_label(&key, "test-label");
        storage.add_key(stored);
        
        // Verify label was set
        assert_eq!(storage.keys.get(&key.did).unwrap().label, "test-label");
        
        // Test finding by label
        let found = storage.find_by_label("test-label");
        assert!(found.is_some());
        assert_eq!(found.unwrap().did, key.did);
        
        // Clean up
        fs::remove_file(&temp_path).ok();
        
        Ok(())
    }
}