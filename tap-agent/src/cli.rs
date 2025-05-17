//! CLI tool for managing DIDs and keys
//!
//! This module provides command-line utilities for creating and managing
//! Decentralized Identifiers (DIDs) and associated cryptographic keys.

use crate::did::{
    DIDGenerationOptions, DIDKeyGenerator, GeneratedKey, KeyType, MultiResolver, SyncDIDResolver,
};
use crate::error::{Error, Result};
use base64::Engine;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::runtime::Runtime;

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
        } => {
            generate_did(&method, &key_type, domain.as_deref(), output, key_output)?;
        }
        Commands::Lookup { did, output } => {
            lookup_did(&did, output)?;
        }
    }

    Ok(())
}

/// Generate a DID of the specified method and key type
fn generate_did(
    method: &str,
    key_type: &str,
    domain: Option<&str>,
    output: Option<PathBuf>,
    key_output: Option<PathBuf>,
) -> Result<()> {
    // Parse key type
    let key_type = match key_type.to_lowercase().as_str() {
        "ed25519" => KeyType::Ed25519,
        "p256" => KeyType::P256,
        "secp256k1" => KeyType::Secp256k1,
        _ => {
            eprintln!(
                "Unsupported key type: {}. Using Ed25519 as default.",
                key_type
            );
            KeyType::Ed25519
        }
    };

    // Create options
    let options = DIDGenerationOptions { key_type };

    // Generate DID using the specified method
    let generator = DIDKeyGenerator::new();
    let generated_key = match method.to_lowercase().as_str() {
        "key" => generator.generate_did(options)?,
        "web" => {
            // For did:web, domain is required
            let domain = domain.ok_or_else(|| {
                crate::error::Error::MissingConfig("Domain is required for did:web".to_string())
            })?;
            generator.generate_web_did(domain, options)?
        }
        _ => {
            eprintln!(
                "Unsupported DID method: {}. Using did:key as default.",
                method
            );
            generator.generate_did(options)?
        }
    };

    // Display DID information
    display_generated_did(&generated_key, method, domain);

    // Save DID document if output path is specified
    if let Some(output_path) = output {
        save_did_document(&generated_key, &output_path)?;
    }

    // Save private key if key output path is specified
    if let Some(key_path) = key_output {
        save_private_key(&generated_key, &key_path)?;
    }

    Ok(())
}

/// Display information about the generated DID
fn display_generated_did(generated_key: &GeneratedKey, method: &str, domain: Option<&str>) {
    println!("\n=== Generated DID ===");
    println!("DID: {}", generated_key.did);
    println!("Key Type: {:?}", generated_key.key_type);

    // For did:web, show where to place the DID document
    if method == "web" && domain.is_some() {
        println!("\nTo use this did:web, place the DID document at:");
        println!("https://{}/.well-known/did.json", domain.unwrap());
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

/// Lookup and resolve a DID to its corresponding DID document
fn lookup_did(did: &str, output: Option<PathBuf>) -> Result<()> {
    println!("Looking up DID: {}", did);

    // Create a resolver
    let resolver = Arc::new(MultiResolver::default());

    // Create a Tokio runtime for async resolution
    let rt = Runtime::new()
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
                    didcomm::did::VerificationMaterial::JWK { public_key_jwk } => {
                        println!("      Material: JWK");
                        if let Some(kty) = public_key_jwk.get("kty") {
                            println!("        Key Type: {}", kty);
                        }
                        if let Some(crv) = public_key_jwk.get("crv") {
                            println!("        Curve: {}", crv);
                        }
                    }
                    didcomm::did::VerificationMaterial::Base58 { public_key_base58 } => {
                        println!("      Material: Base58");
                        println!("        Key: {}", public_key_base58);
                    }
                    didcomm::did::VerificationMaterial::Multibase {
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
