//! Utilities for examples to use temporary storage instead of production ~/.tap directory

use crate::error::Result;
use std::env;
use std::path::PathBuf;
use tempfile::TempDir;

/// Sets up a temporary TAP directory for examples
/// Returns the temporary directory path
pub fn setup_example_storage() -> Result<PathBuf> {
    let temp_dir = TempDir::new()
        .map_err(|e| crate::error::Error::Storage(format!("Failed to create temp dir: {}", e)))?;
    
    let tap_dir = temp_dir.path().to_path_buf();
    
    // Set TAP_HOME to the temporary directory
    env::set_var("TAP_HOME", &tap_dir);
    
    // Create the keys.json path
    let keys_path = tap_dir.join("keys.json");
    
    println!("Using temporary storage at: {:?}", tap_dir);
    println!("(This protects your production ~/.tap directory)");
    println!();
    
    // Leak the temp_dir to keep it alive for the duration of the example
    std::mem::forget(temp_dir);
    
    Ok(keys_path)
}

/// Creates a temporary TAP root directory for examples that need the full .tap structure
pub fn setup_example_tap_root() -> Result<PathBuf> {
    let temp_dir = TempDir::new()
        .map_err(|e| crate::error::Error::Storage(format!("Failed to create temp dir: {}", e)))?;
    
    let root_dir = temp_dir.path().to_path_buf();
    let tap_dir = root_dir.join(".tap");
    
    // Create the .tap directory
    std::fs::create_dir_all(&tap_dir)
        .map_err(|e| crate::error::Error::Storage(format!("Failed to create .tap directory: {}", e)))?;
    
    // Set TAP_TEST_DIR to the root directory
    env::set_var("TAP_TEST_DIR", &root_dir);
    
    println!("Using temporary TAP root at: {:?}", root_dir);
    println!("TAP directory: {:?}", tap_dir);
    println!("(This protects your production ~/.tap directory)");
    println!();
    
    // Leak the temp_dir to keep it alive for the duration of the example
    std::mem::forget(temp_dir);
    
    Ok(tap_dir)
}

/// Prints a notice that the example used temporary storage
pub fn print_temp_storage_notice() {
    println!();
    println!("Note: This example used temporary storage.");
    println!("Your production ~/.tap directory was not affected.");
}