use crate::error::{Error, Result};
use crate::output::{print_success, OutputFormat};
use clap::Subcommand;
use serde::Serialize;
use std::sync::Arc;
use tap_agent::did::{
    DIDGenerationOptions, DIDKeyGenerator, KeyType, MultiResolver, SyncDIDResolver,
};
use tap_agent::storage::KeyStorage;

#[derive(Subcommand, Debug)]
pub enum DidCommands {
    /// Generate a new DID
    Generate {
        /// DID method (key or web)
        #[arg(short, long, default_value = "key")]
        method: String,
        /// Key type (ed25519, p256, secp256k1)
        #[arg(short = 't', long, default_value = "ed25519")]
        key_type: String,
        /// Domain for did:web
        #[arg(short, long)]
        domain: Option<String>,
        /// Save to storage
        #[arg(short, long)]
        save: bool,
        /// Set as default key
        #[arg(long)]
        default: bool,
        /// Label for the key
        #[arg(short, long)]
        label: Option<String>,
    },
    /// Resolve a DID to its DID Document
    Lookup {
        /// DID to resolve
        did: String,
    },
    /// Manage stored keys
    Keys {
        #[command(subcommand)]
        cmd: Option<KeysCommands>,
    },
}

#[derive(Subcommand, Debug)]
pub enum KeysCommands {
    /// List all stored keys
    List,
    /// View key details
    View {
        /// DID or label
        did_or_label: String,
    },
    /// Set a key as default
    SetDefault {
        /// DID or label
        did_or_label: String,
    },
    /// Delete a key
    Delete {
        /// DID or label
        did_or_label: String,
    },
    /// Relabel a key
    Relabel {
        /// DID or label
        did_or_label: String,
        /// New label
        new_label: String,
    },
}

#[derive(Debug, Serialize)]
struct GeneratedDidResponse {
    did: String,
    key_type: String,
    public_key: String,
    saved: bool,
    is_default: bool,
}

#[derive(Debug, Serialize)]
struct KeyInfo {
    did: String,
    label: String,
    key_type: String,
    public_key: String,
    is_default: bool,
}

#[derive(Debug, Serialize)]
struct KeyListResponse {
    keys: Vec<KeyInfo>,
    total: usize,
}

pub async fn handle(cmd: &DidCommands, format: OutputFormat) -> Result<()> {
    match cmd {
        DidCommands::Generate {
            method,
            key_type,
            domain,
            save,
            default,
            label,
        } => {
            handle_generate(
                method,
                key_type,
                domain.as_deref(),
                *save,
                *default,
                label.as_deref(),
                format,
            )
            .await
        }
        DidCommands::Lookup { did } => handle_lookup(did, format).await,
        DidCommands::Keys { cmd } => handle_keys(cmd.as_ref(), format).await,
    }
}

async fn handle_generate(
    method: &str,
    key_type_str: &str,
    domain: Option<&str>,
    save: bool,
    set_default: bool,
    label: Option<&str>,
    format: OutputFormat,
) -> Result<()> {
    let key_type = match key_type_str.to_lowercase().as_str() {
        "ed25519" => KeyType::Ed25519,
        _ => KeyType::Ed25519,
    };

    let did_options = DIDGenerationOptions { key_type };
    let generator = DIDKeyGenerator::new();

    let generated_key = match method.to_lowercase().as_str() {
        "key" => generator.generate_did(did_options)?,
        "web" => {
            let domain =
                domain.ok_or_else(|| Error::invalid_parameter("Domain is required for did:web"))?;
            generator.generate_web_did(domain, did_options)?
        }
        _ => generator.generate_did(did_options)?,
    };

    if save {
        let stored_key = if let Some(label) = label {
            KeyStorage::from_generated_key_with_label(&generated_key, label)
        } else {
            KeyStorage::from_generated_key(&generated_key)
        };

        let mut storage = KeyStorage::load_default().unwrap_or_else(|_| KeyStorage::new());
        storage.add_key(stored_key);

        if set_default {
            storage.default_did = Some(generated_key.did.clone());
        }

        storage
            .save_default()
            .map_err(|e| Error::command_failed(format!("Failed to save key: {}", e)))?;
    }

    let response = GeneratedDidResponse {
        did: generated_key.did,
        key_type: format!("{:?}", generated_key.key_type),
        public_key: generated_key
            .public_key
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>(),
        saved: save,
        is_default: set_default,
    };
    print_success(format, &response);
    Ok(())
}

async fn handle_lookup(did: &str, format: OutputFormat) -> Result<()> {
    let resolver = Arc::new(MultiResolver::default());
    let did_doc = resolver.resolve(did).await?;

    match did_doc {
        Some(doc) => {
            print_success(format, &doc);
            Ok(())
        }
        None => Err(Error::command_failed(format!("DID not found: {}", did))),
    }
}

async fn handle_keys(cmd: Option<&KeysCommands>, format: OutputFormat) -> Result<()> {
    let mut storage = KeyStorage::load_default().unwrap_or_else(|_| KeyStorage::new());
    let default_did = storage.default_did.clone();

    match cmd {
        Some(KeysCommands::List) | None => {
            let keys: Vec<KeyInfo> = storage
                .keys
                .iter()
                .map(|(did, key)| KeyInfo {
                    did: did.clone(),
                    label: key.label.clone(),
                    key_type: format!("{:?}", key.key_type),
                    public_key: key.public_key.clone(),
                    is_default: default_did.as_deref() == Some(did.as_str()),
                })
                .collect();

            let response = KeyListResponse {
                total: keys.len(),
                keys,
            };
            print_success(format, &response);
            Ok(())
        }
        Some(KeysCommands::View { did_or_label }) => {
            let key = storage
                .find_by_label(did_or_label)
                .or_else(|| storage.keys.get(did_or_label))
                .ok_or_else(|| {
                    Error::command_failed(format!("Key '{}' not found", did_or_label))
                })?;

            let info = KeyInfo {
                did: key.did.clone(),
                label: key.label.clone(),
                key_type: format!("{:?}", key.key_type),
                public_key: key.public_key.clone(),
                is_default: default_did.as_deref() == Some(key.did.as_str()),
            };
            print_success(format, &info);
            Ok(())
        }
        Some(KeysCommands::SetDefault { did_or_label }) => {
            let did = if let Some(key) = storage.find_by_label(did_or_label) {
                key.did.clone()
            } else if storage.keys.contains_key(did_or_label) {
                did_or_label.to_string()
            } else {
                return Err(Error::command_failed(format!(
                    "Key '{}' not found",
                    did_or_label
                )));
            };

            storage.default_did = Some(did.clone());
            storage
                .save_default()
                .map_err(|e| Error::command_failed(format!("Failed to save: {}", e)))?;

            #[derive(Serialize)]
            struct SetDefaultResponse {
                did: String,
                status: String,
            }
            print_success(
                format,
                &SetDefaultResponse {
                    did,
                    status: "default_set".to_string(),
                },
            );
            Ok(())
        }
        Some(KeysCommands::Delete { did_or_label }) => {
            let did = if let Some(key) = storage.find_by_label(did_or_label) {
                key.did.clone()
            } else if storage.keys.contains_key(did_or_label) {
                did_or_label.to_string()
            } else {
                return Err(Error::command_failed(format!(
                    "Key '{}' not found",
                    did_or_label
                )));
            };

            storage.keys.remove(&did);
            if storage.default_did.as_deref() == Some(&did) {
                storage.default_did = storage.keys.keys().next().cloned();
            }

            storage
                .save_default()
                .map_err(|e| Error::command_failed(format!("Failed to save: {}", e)))?;

            #[derive(Serialize)]
            struct DeleteResponse {
                did: String,
                status: String,
            }
            print_success(
                format,
                &DeleteResponse {
                    did,
                    status: "deleted".to_string(),
                },
            );
            Ok(())
        }
        Some(KeysCommands::Relabel {
            did_or_label,
            new_label,
        }) => {
            let did = if let Some(key) = storage.find_by_label(did_or_label) {
                key.did.clone()
            } else if storage.keys.contains_key(did_or_label) {
                did_or_label.to_string()
            } else {
                return Err(Error::command_failed(format!(
                    "Key '{}' not found",
                    did_or_label
                )));
            };

            storage
                .update_label(&did, new_label)
                .map_err(|e| Error::command_failed(format!("Failed to relabel: {}", e)))?;
            storage
                .save_default()
                .map_err(|e| Error::command_failed(format!("Failed to save: {}", e)))?;

            #[derive(Serialize)]
            struct RelabelResponse {
                did: String,
                new_label: String,
                status: String,
            }
            print_success(
                format,
                &RelabelResponse {
                    did,
                    new_label: new_label.clone(),
                    status: "relabeled".to_string(),
                },
            );
            Ok(())
        }
    }
}
