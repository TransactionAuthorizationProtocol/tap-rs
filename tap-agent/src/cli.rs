//! CLI tool for managing DIDs and keys
//!
//! This module provides command-line utilities for creating and managing
//! Decentralized Identifiers (DIDs) and associated cryptographic keys.
//!
//! This module is only available when the `native` feature is enabled.
#![cfg(feature = "native")]

use crate::did::{
    DIDGenerationOptions, DIDKeyGenerator, GeneratedKey, KeyType, MultiResolver, SyncDIDResolver,
    VerificationMaterial,
};
use crate::error::{Error, Result};
use crate::message::SecurityMode;
use crate::message_packing::{PackOptions, Packable, Unpackable};
use crate::storage::{KeyStorage, StoredKey};
use base64::Engine;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tap_msg::didcomm::PlainMessage;

/// TAP Agent CLI Tool for DID and Key Management
#[derive(Parser, Debug)]
#[command(name = "tap-agent-cli")]
#[command(about = "CLI tool for managing DIDs and keys for TAP protocol", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate a new DID
    #[command(name = "generate")]
    Generate {
        /// The DID method to use (key or web)
        #[arg(short, long, default_value = "key")]
        method: String,

        /// The key type to use
        #[arg(short = 't', long, default_value = "ed25519")]
        key_type: String,

        /// Domain for did:web (required if method is 'web')
        #[arg(short, long)]
        domain: Option<String>,

        /// Output file path for the DID document
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output file for private key (if not specified, key is shown only in console)
        #[arg(short = 'k', long)]
        key_output: Option<PathBuf>,

        /// Save key to default location (~/.tap/keys.json)
        #[arg(short = 's', long)]
        save: bool,

        /// Set as default key
        #[arg(long)]
        default: bool,

        /// Label for the key (defaults to agent-{n})
        #[arg(short = 'l', long)]
        label: Option<String>,
    },

    /// Lookup and resolve a DID to its DID Document
    #[command(name = "lookup")]
    Lookup {
        /// The DID to resolve
        #[arg(required = true)]
        did: String,

        /// Output file path for the resolved DID document
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// List all stored keys
    #[command(name = "keys", about = "List, view, and manage stored keys")]
    Keys {
        #[command(subcommand)]
        subcommand: Option<KeysCommands>,
    },

    /// Import an existing key into storage
    #[command(name = "import", about = "Import an existing key into storage")]
    Import {
        /// The JSON file containing the key to import
        #[arg(required = true)]
        key_file: PathBuf,

        /// Set as default key
        #[arg(long)]
        default: bool,

        /// Label for the imported key (defaults to agent-{n})
        #[arg(short = 'l', long)]
        label: Option<String>,
    },

    /// Pack a plaintext DIDComm message
    #[command(name = "pack", about = "Pack a plaintext DIDComm message")]
    Pack {
        /// The input file containing the plaintext message
        #[arg(short, long, required = true)]
        input: PathBuf,

        /// The output file for the packed message
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// The DID of the sender (uses default if not specified)
        #[arg(short, long)]
        sender: Option<String>,

        /// The DID of the recipient
        #[arg(short, long)]
        recipient: Option<String>,

        /// The security mode to use (plain, signed, or authcrypt)
        #[arg(short, long, default_value = "signed")]
        mode: String,
    },

    /// Unpack a signed or encrypted DIDComm message
    #[command(
        name = "unpack",
        about = "Unpack a signed or encrypted DIDComm message"
    )]
    Unpack {
        /// The input file containing the packed message
        #[arg(short, long, required = true)]
        input: PathBuf,

        /// The output file for the unpacked message
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// The DID of the recipient (uses default if not specified)
        #[arg(short, long)]
        recipient: Option<String>,
    },
}

/// Subcommands for key management
#[derive(Subcommand, Debug)]
pub enum KeysCommands {
    /// List all stored keys
    #[command(name = "list")]
    List,

    /// View details of a specific key
    #[command(name = "view")]
    View {
        /// The DID or label of the key to view
        #[arg(required = true)]
        did_or_label: String,
    },

    /// Set a key as the default
    #[command(name = "set-default")]
    SetDefault {
        /// The DID or label of the key to set as default
        #[arg(required = true)]
        did_or_label: String,
    },

    /// Delete a key from storage
    #[command(name = "delete")]
    Delete {
        /// The DID or label of the key to delete
        #[arg(required = true)]
        did_or_label: String,

        /// Force deletion without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Update the label of a key
    #[command(name = "relabel")]
    Relabel {
        /// The DID or label of the key to relabel
        #[arg(required = true)]
        did_or_label: String,

        /// The new label for the key
        #[arg(required = true)]
        new_label: String,
    },
}

/// Run the CLI with the given arguments
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Generate {
            method,
            key_type,
            domain,
            output,
            key_output,
            save,
            default,
            label,
        } => {
            generate_did(GenerateDIDOptions {
                method: &method,
                key_type: &key_type,
                domain: domain.as_deref(),
                output,
                key_output,
                save,
                set_default: default,
                label: label.as_deref(),
            })?;
        }
        Commands::Lookup { did, output } => {
            lookup_did(&did, output)?;
        }
        Commands::Keys { subcommand } => {
            manage_keys(subcommand)?;
        }
        Commands::Import {
            key_file,
            default,
            label,
        } => {
            import_key(&key_file, default, label.as_deref())?;
        }
        Commands::Pack {
            input,
            output,
            sender,
            recipient,
            mode,
        } => {
            pack_message(&input, output, sender, recipient, &mode)?;
        }
        Commands::Unpack {
            input,
            output,
            recipient,
        } => {
            unpack_message(&input, output, recipient)?;
        }
    }

    Ok(())
}

/// Options for DID generation
struct GenerateDIDOptions<'a> {
    method: &'a str,
    key_type: &'a str,
    domain: Option<&'a str>,
    output: Option<PathBuf>,
    key_output: Option<PathBuf>,
    save: bool,
    set_default: bool,
    label: Option<&'a str>,
}

/// Generate a DID of the specified method and key type
fn generate_did(options: GenerateDIDOptions) -> Result<()> {
    // Parse key type
    let key_type = match options.key_type.to_lowercase().as_str() {
        "ed25519" => KeyType::Ed25519,
        "p256" => KeyType::P256,
        "secp256k1" => KeyType::Secp256k1,
        _ => {
            eprintln!(
                "Unsupported key type: {}. Using Ed25519 as default.",
                options.key_type
            );
            KeyType::Ed25519
        }
    };

    // Create DID generation options
    let did_options = DIDGenerationOptions { key_type };

    // Generate DID using the specified method
    let generator = DIDKeyGenerator::new();
    let generated_key = match options.method.to_lowercase().as_str() {
        "key" => generator.generate_did(did_options)?,
        "web" => {
            // For did:web, domain is required
            let domain = options.domain.ok_or_else(|| {
                crate::error::Error::MissingConfig("Domain is required for did:web".to_string())
            })?;
            generator.generate_web_did(domain, did_options)?
        }
        _ => {
            eprintln!(
                "Unsupported DID method: {}. Using did:key as default.",
                options.method
            );
            generator.generate_did(did_options)?
        }
    };

    // Display DID information
    display_generated_did(&generated_key, options.method, options.domain);

    // Save DID document if output path is specified
    if let Some(output_path) = options.output {
        save_did_document(&generated_key, &output_path)?;
    }

    // Save private key if key output path is specified
    if let Some(key_path) = options.key_output {
        save_private_key(&generated_key, &key_path)?;
    }

    // Save key to default storage if requested
    if options.save {
        save_key_to_storage(&generated_key, options.set_default, options.label)?;
    }

    Ok(())
}

/// Display information about the generated DID
fn display_generated_did(generated_key: &GeneratedKey, method: &str, domain: Option<&str>) {
    println!("\n=== Generated DID ===");
    println!("DID: {}", generated_key.did);
    println!("Key Type: {:?}", generated_key.key_type);

    // For did:web, show where to place the DID document
    if method == "web" {
        if let Some(d) = domain {
            println!("\nTo use this did:web, place the DID document at:");
            println!("https://{}/.well-known/did.json", d);
        }
    }

    // Display the private key
    println!("\n=== Private Key (keep this secure!) ===");
    println!(
        "Private Key (Base64): {}",
        base64::engine::general_purpose::STANDARD.encode(&generated_key.private_key)
    );

    println!("\n=== Public Key ===");
    println!(
        "Public Key (Base64): {}",
        base64::engine::general_purpose::STANDARD.encode(&generated_key.public_key)
    );
}

/// Save DID document to a file
fn save_did_document(generated_key: &GeneratedKey, output_path: &PathBuf) -> Result<()> {
    let did_doc_json = serde_json::to_string_pretty(&generated_key.did_doc)
        .map_err(|e| crate::error::Error::Serialization(e.to_string()))?;

    fs::write(output_path, did_doc_json).map_err(crate::error::Error::Io)?;

    println!("\nDID document saved to: {}", output_path.display());
    Ok(())
}

/// Save private key to a file
fn save_private_key(generated_key: &GeneratedKey, key_path: &PathBuf) -> Result<()> {
    // Create a JSON object with key information
    let key_info = serde_json::json!({
        "did": generated_key.did,
        "keyType": format!("{:?}", generated_key.key_type),
        "privateKey": base64::engine::general_purpose::STANDARD.encode(&generated_key.private_key),
        "publicKey": base64::engine::general_purpose::STANDARD.encode(&generated_key.public_key),
    });

    let key_json = serde_json::to_string_pretty(&key_info)
        .map_err(|e| crate::error::Error::Serialization(e.to_string()))?;

    fs::write(key_path, key_json).map_err(crate::error::Error::Io)?;

    println!("Private key saved to: {}", key_path.display());
    Ok(())
}

/// Save a key to the default storage location
fn save_key_to_storage(
    generated_key: &GeneratedKey,
    set_as_default: bool,
    label: Option<&str>,
) -> Result<()> {
    // Convert GeneratedKey to StoredKey
    let stored_key = if let Some(label) = label {
        KeyStorage::from_generated_key_with_label(generated_key, label)
    } else {
        KeyStorage::from_generated_key(generated_key)
    };

    // Load existing storage or create a new one
    let mut storage = match KeyStorage::load_default() {
        Ok(storage) => storage,
        Err(_) => KeyStorage::new(),
    };

    // Add the key to storage
    storage.add_key(stored_key);

    // If requested to set as default, update the default DID
    if set_as_default {
        storage.default_did = Some(generated_key.did.clone());
    }

    // Save the updated storage
    storage.save_default()?;

    println!("Key saved to default storage (~/.tap/keys.json)");
    if set_as_default {
        println!("Key set as default agent key");
    }

    Ok(())
}

/// Import a key from a file into the key storage
fn import_key(key_file: &PathBuf, set_as_default: bool, label: Option<&str>) -> Result<()> {
    // Read and parse the key file
    let key_json = fs::read_to_string(key_file)
        .map_err(|e| Error::Storage(format!("Failed to read key file: {}", e)))?;

    let key_info: serde_json::Value = serde_json::from_str(&key_json)
        .map_err(|e| Error::Storage(format!("Failed to parse key file: {}", e)))?;

    // Extract key information
    let did = key_info["did"]
        .as_str()
        .ok_or_else(|| Error::Storage("Missing 'did' field in key file".to_string()))?;

    let key_type_str = key_info["keyType"]
        .as_str()
        .ok_or_else(|| Error::Storage("Missing 'keyType' field in key file".to_string()))?;

    let private_key = key_info["privateKey"]
        .as_str()
        .ok_or_else(|| Error::Storage("Missing 'privateKey' field in key file".to_string()))?;

    let public_key = key_info["publicKey"]
        .as_str()
        .ok_or_else(|| Error::Storage("Missing 'publicKey' field in key file".to_string()))?;

    // Parse key type
    let key_type = match key_type_str {
        "Ed25519" => KeyType::Ed25519,
        "P256" => KeyType::P256,
        "Secp256k1" => KeyType::Secp256k1,
        _ => {
            return Err(Error::Storage(format!(
                "Unsupported key type: {}",
                key_type_str
            )))
        }
    };

    // Create a StoredKey
    let stored_key = StoredKey {
        did: did.to_string(),
        label: label.unwrap_or("").to_string(),
        key_type,
        private_key: private_key.to_string(),
        public_key: public_key.to_string(),
        metadata: std::collections::HashMap::new(),
    };

    // Load existing storage or create a new one
    let mut storage = match KeyStorage::load_default() {
        Ok(storage) => storage,
        Err(_) => KeyStorage::new(),
    };

    // Add the key to storage
    storage.add_key(stored_key);

    // If requested to set as default, update the default DID
    if set_as_default {
        storage.default_did = Some(did.to_string());
    }

    // Save the updated storage
    storage.save_default()?;

    println!("Key '{}' imported to default storage", did);
    if set_as_default {
        println!("Key set as default agent key");
    }

    Ok(())
}

/// Manage stored keys
fn manage_keys(subcommand: Option<KeysCommands>) -> Result<()> {
    // Load key storage
    let mut storage = match KeyStorage::load_default() {
        Ok(storage) => storage,
        Err(e) => {
            eprintln!("Error loading key storage: {}", e);
            eprintln!("Creating new key storage.");
            KeyStorage::new()
        }
    };

    match subcommand {
        Some(KeysCommands::List) => {
            list_keys(&storage)?;
        }
        Some(KeysCommands::View { did_or_label }) => {
            view_key(&storage, &did_or_label)?;
        }
        Some(KeysCommands::SetDefault { did_or_label }) => {
            set_default_key(&mut storage, &did_or_label)?;
        }
        Some(KeysCommands::Delete {
            did_or_label,
            force,
        }) => {
            delete_key(&mut storage, &did_or_label, force)?;
        }
        Some(KeysCommands::Relabel {
            did_or_label,
            new_label,
        }) => {
            relabel_key(&mut storage, &did_or_label, &new_label)?;
        }
        None => {
            // Default to list if no subcommand is provided
            list_keys(&storage)?;
        }
    }

    Ok(())
}

/// Relabel a key in storage
fn relabel_key(storage: &mut KeyStorage, did_or_label: &str, new_label: &str) -> Result<()> {
    // Try to find by label first, then by DID
    let did = if let Some(key) = storage.find_by_label(did_or_label) {
        key.did.clone()
    } else if storage.keys.contains_key(did_or_label) {
        did_or_label.to_string()
    } else {
        return Err(Error::Storage(format!(
            "Key '{}' not found in storage",
            did_or_label
        )));
    };

    // Update the label
    storage.update_label(&did, new_label)?;

    // Save the updated storage
    storage.save_default()?;

    println!("Key relabeled successfully to '{}'", new_label);

    Ok(())
}

/// List all keys in storage
fn list_keys(storage: &KeyStorage) -> Result<()> {
    // Check if storage is empty
    if storage.keys.is_empty() {
        println!("No keys found in storage.");
        println!("Generate a key with: tap-agent-cli generate --save");
        return Ok(());
    }

    println!("Keys in storage:");
    println!("{:-<60}", "");

    // Get the default DID for marking
    let default_did = storage.default_did.as_deref();

    // Print header
    println!("{:<15} {:<40} {:<10} Default", "Label", "DID", "Key Type");
    println!("{:-<75}", "");

    // Print each key
    for (did, key) in &storage.keys {
        let is_default = if Some(did.as_str()) == default_did {
            "*"
        } else {
            ""
        };
        println!(
            "{:<15} {:<40} {:<10} {}",
            key.label,
            did,
            format!("{:?}", key.key_type),
            is_default
        );
    }

    println!("\nTotal keys: {}", storage.keys.len());

    Ok(())
}

/// View details for a specific key
fn view_key(storage: &KeyStorage, did_or_label: &str) -> Result<()> {
    // Try to find by label first, then by DID
    let key = storage
        .find_by_label(did_or_label)
        .or_else(|| storage.keys.get(did_or_label))
        .ok_or_else(|| Error::Storage(format!("Key '{}' not found in storage", did_or_label)))?;

    // Display key information
    println!("\n=== Key Details ===");
    println!("Label: {}", key.label);
    println!("DID: {}", key.did);
    println!("Key Type: {:?}", key.key_type);
    println!("Public Key (Base64): {}", key.public_key);

    // Check if this is the default key
    if storage.default_did.as_deref() == Some(&key.did) {
        println!("Default: Yes");
    } else {
        println!("Default: No");
    }

    // Print metadata if any
    if !key.metadata.is_empty() {
        println!("\nMetadata:");
        for (k, v) in &key.metadata {
            println!("  {}: {}", k, v);
        }
    }

    Ok(())
}

/// Set a key as the default
fn set_default_key(storage: &mut KeyStorage, did_or_label: &str) -> Result<()> {
    // Try to find by label first, then by DID
    let did = if let Some(key) = storage.find_by_label(did_or_label) {
        key.did.clone()
    } else if storage.keys.contains_key(did_or_label) {
        did_or_label.to_string()
    } else {
        return Err(Error::Storage(format!(
            "Key '{}' not found in storage",
            did_or_label
        )));
    };

    // Set as default
    storage.default_did = Some(did.clone());

    // Save the updated storage
    storage.save_default()?;

    println!("Key '{}' set as default", did);

    Ok(())
}

/// Delete a key from storage
fn delete_key(storage: &mut KeyStorage, did_or_label: &str, force: bool) -> Result<()> {
    // Try to find by label first, then by DID
    let did = if let Some(key) = storage.find_by_label(did_or_label) {
        key.did.clone()
    } else if storage.keys.contains_key(did_or_label) {
        did_or_label.to_string()
    } else {
        return Err(Error::Storage(format!(
            "Key '{}' not found in storage",
            did_or_label
        )));
    };

    // Confirm deletion if not forced
    if !force {
        println!("Are you sure you want to delete key '{}'? (y/N): ", did);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).map_err(Error::Io)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }

    // Remove the key
    storage.keys.remove(&did);

    // If this was the default key, clear the default
    if storage.default_did.as_deref() == Some(&did) {
        storage.default_did = storage.keys.keys().next().cloned();
    }

    // Save the updated storage
    storage.save_default()?;

    println!("Key '{}' deleted from storage", did);

    Ok(())
}

/// Lookup and resolve a DID to its corresponding DID document
fn lookup_did(did: &str, output: Option<PathBuf>) -> Result<()> {
    println!("Looking up DID: {}", did);

    // Create a resolver
    let resolver = Arc::new(MultiResolver::default());

    // Create a Tokio runtime for async resolution
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::DIDResolution(format!("Failed to create runtime: {}", e)))?;

    // Resolve the DID
    let did_doc = rt.block_on(async { resolver.resolve(did).await })?;

    // Check if DID Document was found
    match did_doc {
        Some(doc) => {
            println!("\n=== DID Document ===");

            // Pretty print the DID Document details
            println!("DID: {}", doc.id);

            println!("\nVerification Methods:");
            for (i, vm) in doc.verification_method.iter().enumerate() {
                println!("  [{}] ID: {}", i + 1, vm.id);
                println!("      Type: {:?}", vm.type_);
                println!("      Controller: {}", vm.controller);

                match &vm.verification_material {
                    VerificationMaterial::JWK { public_key_jwk } => {
                        println!("      Material: JWK");
                        if let Some(kty) = public_key_jwk.get("kty") {
                            println!("        Key Type: {}", kty);
                        }
                        if let Some(crv) = public_key_jwk.get("crv") {
                            println!("        Curve: {}", crv);
                        }
                    }
                    VerificationMaterial::Base58 { public_key_base58 } => {
                        println!("      Material: Base58");
                        println!("        Key: {}", public_key_base58);
                    }
                    VerificationMaterial::Multibase {
                        public_key_multibase,
                    } => {
                        println!("      Material: Multibase");
                        println!("        Key: {}", public_key_multibase);
                    }
                }
                println!();
            }

            if !doc.authentication.is_empty() {
                println!("Authentication Methods:");
                for auth in &doc.authentication {
                    println!("  {}", auth);
                }
                println!();
            }

            if !doc.key_agreement.is_empty() {
                println!("Key Agreement Methods:");
                for ka in &doc.key_agreement {
                    println!("  {}", ka);
                }
                println!();
            }

            if !doc.service.is_empty() {
                println!("Services:");
                for (i, svc) in doc.service.iter().enumerate() {
                    println!("  [{}] ID: {}", i + 1, svc.id);
                    println!("      Endpoint: {:?}", svc.service_endpoint);
                    println!();
                }
            }

            // Save DID document if output path is specified
            if let Some(output_path) = output {
                let did_doc_json = serde_json::to_string_pretty(&doc)
                    .map_err(|e| Error::Serialization(e.to_string()))?;

                fs::write(&output_path, did_doc_json).map_err(Error::Io)?;
                println!("DID document saved to: {}", output_path.display());
            }

            Ok(())
        }
        None => {
            println!("No DID Document found for: {}", did);
            println!("The DID may not exist or the resolver might not support this DID method.");

            // Extract method to provide better feedback
            let parts: Vec<&str> = did.split(':').collect();
            if parts.len() >= 2 {
                let method = parts[1];
                println!(
                    "DID method '{}' may not be supported by the default resolver.",
                    method
                );
                println!("Currently, only the following methods are supported:");
                println!("  - did:key");
                println!("  - did:web");

                if method == "web" {
                    println!("\nFor did:web, ensure:");
                    println!("  - The domain is correctly formatted");
                    println!("  - The DID document is hosted at the expected location:");
                    println!(
                        "    - https://example.com/.well-known/did.json for did:web:example.com"
                    );
                    println!("    - https://example.com/path/to/resource/did.json for did:web:example.com:path:to:resource");
                }
            }

            Err(Error::DIDResolution(format!("DID not found: {}", did)))
        }
    }
}

/// Pack a plaintext DIDComm message
async fn pack_message_async(
    input_file: &PathBuf,
    output_file: Option<PathBuf>,
    sender_did: Option<String>,
    recipient_did: Option<String>,
    mode: &str,
) -> Result<()> {
    // Read the plaintext message from the input file
    let plaintext = fs::read_to_string(input_file).map_err(Error::Io)?;

    // Parse the plaintext message
    let plain_message: PlainMessage = serde_json::from_str(&plaintext)
        .map_err(|e| Error::Serialization(format!("Failed to parse plaintext message: {}", e)))?;

    // Load keys from storage
    let storage = KeyStorage::load_default()?;

    // Get the sender DID
    let sender = if let Some(did_or_label) = sender_did {
        // Try to find by label first, then by DID
        let did = if let Some(key) = storage.find_by_label(&did_or_label) {
            key.did.clone()
        } else if storage.keys.contains_key(&did_or_label) {
            did_or_label
        } else {
            return Err(Error::Storage(format!(
                "Sender '{}' not found in storage",
                did_or_label
            )));
        };
        Some(did)
    } else if let Some(default_did) = storage.default_did.clone() {
        // Use default DID if available
        Some(default_did)
    } else if let Some(first_key) = storage.keys.keys().next() {
        // Fallback to first key
        Some(first_key.clone())
    } else {
        // No keys available
        return Err(Error::Storage("No keys found in storage".to_string()));
    };

    if let Some(ref sender_did) = sender {
        println!("Using sender DID: {}", sender_did);
    }

    // Create key manager with the loaded keys
    let key_manager_builder =
        crate::agent_key_manager::AgentKeyManagerBuilder::new().load_from_default_storage();
    let key_manager = Arc::new(key_manager_builder.build()?);

    // Determine security mode
    let security_mode = match mode.to_lowercase().as_str() {
        "plain" => SecurityMode::Plain,
        "signed" => SecurityMode::Signed,
        "authcrypt" | "auth" | "encrypted" => SecurityMode::AuthCrypt,
        _ => {
            eprintln!(
                "Unknown security mode: {}. Using 'signed' as default.",
                mode
            );
            SecurityMode::Signed
        }
    };

    // Create pack options
    let pack_options = PackOptions {
        security_mode,
        sender_kid: sender.as_ref().map(|s| format!("{}#keys-1", s)),
        recipient_kid: recipient_did.map(|did| format!("{}#keys-1", did)),
    };

    // Pack the message directly using the PlainMessage's Packable implementation
    let packed = plain_message.pack(&*key_manager, pack_options).await?;

    // Write the packed message to the output file or display it
    if let Some(output) = output_file {
        fs::write(&output, &packed).map_err(Error::Io)?;
        println!("Packed message saved to: {}", output.display());
    } else {
        // Try to pretty-print if it's valid JSON
        match serde_json::from_str::<serde_json::Value>(&packed) {
            Ok(json) => println!("{}", serde_json::to_string_pretty(&json).unwrap_or(packed)),
            Err(_) => println!("{}", packed),
        }
    }

    Ok(())
}

/// Pack a plaintext DIDComm message (synchronous wrapper)
fn pack_message(
    input_file: &PathBuf,
    output_file: Option<PathBuf>,
    sender_did: Option<String>,
    recipient_did: Option<String>,
    mode: &str,
) -> Result<()> {
    // Create a tokio runtime to run async function
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::Runtime(format!("Failed to create runtime: {}", e)))?;

    // Run the async function in the runtime
    rt.block_on(pack_message_async(
        input_file,
        output_file,
        sender_did,
        recipient_did,
        mode,
    ))
}

/// Unpack a signed or encrypted DIDComm message
async fn unpack_message_async(
    input_file: &PathBuf,
    output_file: Option<PathBuf>,
    recipient_did: Option<String>,
) -> Result<()> {
    // Read the packed message from the input file
    let packed = fs::read_to_string(input_file).map_err(Error::Io)?;

    // Load keys from storage
    let storage = KeyStorage::load_default()?;

    // Get the recipient DID
    let recipient = if let Some(did_or_label) = recipient_did {
        // Try to find by label first, then by DID
        if let Some(key) = storage.find_by_label(&did_or_label) {
            key.did.clone()
        } else if storage.keys.contains_key(&did_or_label) {
            did_or_label
        } else {
            return Err(Error::Storage(format!(
                "Recipient '{}' not found in storage",
                did_or_label
            )));
        }
    } else if let Some(default_did) = storage.default_did.clone() {
        // Use default DID if available
        default_did
    } else if let Some(first_key) = storage.keys.keys().next() {
        // Otherwise use first available DID
        first_key.clone()
    } else {
        // No keys found
        return Err(Error::Storage("No keys found in storage".to_string()));
    };

    println!("Using recipient DID: {}", recipient);

    // Create key manager with the loaded keys
    let key_manager_builder =
        crate::agent_key_manager::AgentKeyManagerBuilder::new().load_from_default_storage();
    let key_manager = Arc::new(key_manager_builder.build()?);

    // Create unpack options
    use crate::message_packing::UnpackOptions;
    let unpack_options = UnpackOptions {
        expected_security_mode: SecurityMode::Any,
        expected_recipient_kid: Some(format!("{}#keys-1", recipient)),
        require_signature: false,
    };

    // Unpack the message using the String's Unpackable implementation
    let unpacked: PlainMessage = String::unpack(&packed, &*key_manager, unpack_options).await?;

    // Convert to pretty JSON
    let unpacked_json = serde_json::to_string_pretty(&unpacked)
        .map_err(|e| Error::Serialization(format!("Failed to format unpacked message: {}", e)))?;

    // Write the unpacked message to the output file or display it
    if let Some(output) = output_file {
        fs::write(&output, &unpacked_json).map_err(Error::Io)?;
        println!("Unpacked message saved to: {}", output.display());
    } else {
        println!("{}", unpacked_json);
    }

    Ok(())
}

/// Unpack a signed or encrypted DIDComm message (synchronous wrapper)
fn unpack_message(
    input_file: &PathBuf,
    output_file: Option<PathBuf>,
    recipient_did: Option<String>,
) -> Result<()> {
    // Create a tokio runtime to run async function
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::Runtime(format!("Failed to create runtime: {}", e)))?;

    // Run the async function in the runtime
    rt.block_on(unpack_message_async(input_file, output_file, recipient_did))
}
