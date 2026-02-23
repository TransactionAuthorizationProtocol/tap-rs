//! Secret helper for external key management integration
//!
//! Provides a git-like secret helper pattern that allows TAP agents to retrieve
//! private keys from external secret stores (HashiCorp Vault, AWS KMS, 1Password, etc).
//!
//! ## Protocol
//!
//! The secret helper is invoked as:
//! ```text
//! <command> [args...] <did>
//! ```
//!
//! It outputs JSON to stdout:
//! ```json
//! {"private_key": "abcdef...", "key_type": "Ed25519", "encoding": "hex"}
//! ```
//!
//! - `private_key` (required): key material
//! - `key_type` (required): `Ed25519` | `P256` | `Secp256k1`
//! - `encoding` (optional, default `hex`): `hex` | `base64`

use crate::did::KeyType;
use crate::error::{Error, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Output format from a secret helper command
#[derive(Debug, Deserialize)]
pub struct SecretHelperOutput {
    /// Private key material (hex or base64 encoded)
    pub private_key: String,
    /// Key type string
    pub key_type: String,
    /// Encoding format (defaults to "hex")
    #[serde(default = "default_encoding")]
    pub encoding: String,
}

fn default_encoding() -> String {
    "hex".to_string()
}

impl SecretHelperOutput {
    /// Decode the private key bytes and parse the key type
    pub fn decode(&self) -> Result<(Vec<u8>, KeyType)> {
        let bytes = match self.encoding.as_str() {
            "hex" => hex::decode(&self.private_key).map_err(|e| {
                Error::Cryptography(format!("Failed to decode hex private key: {}", e))
            })?,
            "base64" => {
                use base64::Engine;
                base64::engine::general_purpose::STANDARD
                    .decode(&self.private_key)
                    .map_err(|e| {
                        Error::Cryptography(format!("Failed to decode base64 private key: {}", e))
                    })?
            }
            other => {
                return Err(Error::Validation(format!(
                    "Unsupported encoding: {}",
                    other
                )))
            }
        };

        let key_type = match self.key_type.as_str() {
            "Ed25519" => {
                #[cfg(feature = "crypto-ed25519")]
                {
                    KeyType::Ed25519
                }
                #[cfg(not(feature = "crypto-ed25519"))]
                {
                    return Err(Error::Validation("Ed25519 support not enabled".to_string()));
                }
            }
            "P256" => {
                #[cfg(feature = "crypto-p256")]
                {
                    KeyType::P256
                }
                #[cfg(not(feature = "crypto-p256"))]
                {
                    return Err(Error::Validation("P256 support not enabled".to_string()));
                }
            }
            "Secp256k1" => {
                #[cfg(feature = "crypto-secp256k1")]
                {
                    KeyType::Secp256k1
                }
                #[cfg(not(feature = "crypto-secp256k1"))]
                {
                    return Err(Error::Validation(
                        "Secp256k1 support not enabled".to_string(),
                    ));
                }
            }
            other => return Err(Error::Validation(format!("Unknown key type: {}", other))),
        };

        Ok((bytes, key_type))
    }
}

/// Configuration for a secret helper command
#[derive(Debug, Clone)]
pub struct SecretHelperConfig {
    /// The command to execute
    pub command: String,
    /// Arguments to pass before the DID
    pub args: Vec<String>,
}

impl SecretHelperConfig {
    /// Parse a command string into a SecretHelperConfig
    ///
    /// Splits on whitespace. The first token is the command, the rest are arguments.
    /// The DID will be appended as the final argument at invocation time.
    pub fn from_command_string(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.is_empty() {
            return Err(Error::Validation(
                "Secret helper command string is empty".to_string(),
            ));
        }

        Ok(Self {
            command: parts[0].to_string(),
            args: parts[1..].iter().map(|s| s.to_string()).collect(),
        })
    }

    /// Invoke the secret helper for a given DID and return the decoded key
    pub fn get_key(&self, did: &str) -> Result<(Vec<u8>, KeyType)> {
        let mut cmd = Command::new(&self.command);
        for arg in &self.args {
            cmd.arg(arg);
        }
        cmd.arg(did);

        // Inherit stderr so the user sees errors from the helper
        cmd.stderr(std::process::Stdio::inherit());

        let output = cmd.output().map_err(|e| {
            Error::Storage(format!(
                "Failed to run secret helper '{}': {}",
                self.command, e
            ))
        })?;

        if !output.status.success() {
            let code = output.status.code().unwrap_or(-1);
            return Err(Error::Storage(format!(
                "Secret helper '{}' exited with code {}",
                self.command, code
            )));
        }

        let stdout = String::from_utf8(output.stdout).map_err(|e| {
            Error::Storage(format!(
                "Secret helper produced invalid UTF-8 output: {}",
                e
            ))
        })?;

        let helper_output: SecretHelperOutput = serde_json::from_str(&stdout).map_err(|e| {
            Error::Storage(format!("Failed to parse secret helper JSON output: {}", e))
        })?;

        helper_output.decode()
    }
}

/// Discover agent DIDs by scanning TAP home directory for `did_*` subdirectories
///
/// Each agent creates a directory like `did_key_z6Mk...` when `KeyStorage::create_agent_directory`
/// is called. This function reverses the sanitization (`_` -> `:`) to recover the DID.
pub fn discover_agent_dids(tap_root: Option<&Path>) -> Result<Vec<String>> {
    let tap_dir = if let Some(root) = tap_root {
        root.to_path_buf()
    } else if let Ok(tap_home) = std::env::var("TAP_HOME") {
        PathBuf::from(tap_home)
    } else if let Ok(test_dir) = std::env::var("TAP_TEST_DIR") {
        PathBuf::from(test_dir).join(crate::storage::DEFAULT_TAP_DIR)
    } else {
        dirs::home_dir()
            .ok_or_else(|| Error::Storage("Could not determine home directory".to_string()))?
            .join(crate::storage::DEFAULT_TAP_DIR)
    };

    if !tap_dir.exists() {
        return Ok(Vec::new());
    }

    let mut dids = Vec::new();
    for entry in std::fs::read_dir(&tap_dir)
        .map_err(|e| Error::Storage(format!("Failed to read TAP directory: {}", e)))?
    {
        let entry =
            entry.map_err(|e| Error::Storage(format!("Failed to read directory entry: {}", e)))?;
        let path = entry.path();
        if path.is_dir() {
            let dir_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };
            // Only consider directories that look like sanitized DIDs (contain "did_")
            if dir_name.starts_with("did_") {
                let did = dir_name.replace('_', ":");
                dids.push(did);
            }
        }
    }

    dids.sort();
    Ok(dids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_manager::KeyManager;
    use tempfile::TempDir;

    /// Write script content to a file, set it executable, and rename to final path.
    /// The write-then-rename avoids ETXTBSY ("Text file busy") on Linux, which can
    /// occur when exec races with the kernel releasing a write file descriptor.
    #[cfg(unix)]
    fn write_test_script(dir: &std::path::Path, name: &str, content: &str) -> std::path::PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let tmp_path = dir.join(format!("{}.tmp", name));
        let final_path = dir.join(name);
        std::fs::write(&tmp_path, content).unwrap();
        std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::rename(&tmp_path, &final_path).unwrap();
        final_path
    }

    #[test]
    fn test_from_command_string_simple() {
        let config = SecretHelperConfig::from_command_string("my-helper").unwrap();
        assert_eq!(config.command, "my-helper");
        assert!(config.args.is_empty());
    }

    #[test]
    fn test_from_command_string_with_args() {
        let config = SecretHelperConfig::from_command_string(
            "vault-helper --vault-addr https://vault.example.com",
        )
        .unwrap();
        assert_eq!(config.command, "vault-helper");
        assert_eq!(
            config.args,
            vec!["--vault-addr", "https://vault.example.com"]
        );
    }

    #[test]
    fn test_from_command_string_empty() {
        let result = SecretHelperConfig::from_command_string("");
        assert!(result.is_err());
    }

    #[test]
    fn test_secret_helper_output_hex() {
        let json = r#"{"private_key": "abcdef0123456789", "key_type": "Ed25519"}"#;
        let output: SecretHelperOutput = serde_json::from_str(json).unwrap();
        assert_eq!(output.encoding, "hex"); // default
        let (bytes, key_type) = output.decode().unwrap();
        assert_eq!(bytes, hex::decode("abcdef0123456789").unwrap());
        assert_eq!(key_type, KeyType::Ed25519);
    }

    #[test]
    fn test_secret_helper_output_base64() {
        use base64::Engine;
        let key_bytes = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let b64 = base64::engine::general_purpose::STANDARD.encode(&key_bytes);
        let json = format!(
            r#"{{"private_key": "{}", "key_type": "Ed25519", "encoding": "base64"}}"#,
            b64
        );
        let output: SecretHelperOutput = serde_json::from_str(&json).unwrap();
        let (bytes, _) = output.decode().unwrap();
        assert_eq!(bytes, key_bytes);
    }

    #[test]
    fn test_secret_helper_output_explicit_hex() {
        let json = r#"{"private_key": "deadbeef", "key_type": "Ed25519", "encoding": "hex"}"#;
        let output: SecretHelperOutput = serde_json::from_str(json).unwrap();
        let (bytes, _) = output.decode().unwrap();
        assert_eq!(bytes, vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn test_secret_helper_output_unknown_key_type() {
        let json = r#"{"private_key": "abcd", "key_type": "RSA"}"#;
        let output: SecretHelperOutput = serde_json::from_str(json).unwrap();
        let result = output.decode();
        assert!(result.is_err());
    }

    #[test]
    fn test_secret_helper_output_unsupported_encoding() {
        let json = r#"{"private_key": "abcd", "key_type": "Ed25519", "encoding": "raw"}"#;
        let output: SecretHelperOutput = serde_json::from_str(json).unwrap();
        let result = output.decode();
        assert!(result.is_err());
    }

    #[test]
    fn test_get_key_with_mock_script() {
        let temp_dir = TempDir::new().unwrap();

        // Generate a real key to test with
        let km = crate::agent_key_manager::AgentKeyManager::new();
        let key = km
            .generate_key(crate::did::DIDGenerationOptions {
                key_type: KeyType::Ed25519,
            })
            .unwrap();
        let hex_key = hex::encode(&key.private_key);

        let script_path = write_test_script(
            temp_dir.path(),
            "helper.sh",
            &format!(
                "#!/bin/sh\necho '{{\"private_key\": \"{}\", \"key_type\": \"Ed25519\"}}'",
                hex_key
            ),
        );

        let config = SecretHelperConfig {
            command: script_path.to_str().unwrap().to_string(),
            args: vec![],
        };

        let (bytes, key_type) = config.get_key(&key.did).unwrap();
        assert_eq!(bytes, key.private_key);
        assert_eq!(key_type, KeyType::Ed25519);
    }

    #[tokio::test]
    async fn test_secret_helper_roundtrip() {
        let km = crate::agent_key_manager::AgentKeyManager::new();
        let key = km
            .generate_key(crate::did::DIDGenerationOptions {
                key_type: KeyType::Ed25519,
            })
            .unwrap();
        let hex_key = hex::encode(&key.private_key);

        let temp_dir = TempDir::new().unwrap();
        let script_path = write_test_script(
            temp_dir.path(),
            "helper.sh",
            &format!(
                "#!/bin/sh\necho '{{\"private_key\": \"{}\", \"key_type\": \"Ed25519\"}}'",
                hex_key
            ),
        );

        let config = SecretHelperConfig {
            command: script_path.to_str().unwrap().to_string(),
            args: vec![],
        };

        let (bytes, key_type) = config.get_key(&key.did).unwrap();
        let (_agent, new_did) = crate::agent::TapAgent::from_private_key(&bytes, key_type, false)
            .await
            .unwrap();
        assert_eq!(new_did, key.did);
    }

    #[test]
    fn test_get_key_command_not_found() {
        let config = SecretHelperConfig {
            command: "/nonexistent/helper".to_string(),
            args: vec![],
        };
        let result = config.get_key("did:key:test");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_key_non_zero_exit() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = write_test_script(temp_dir.path(), "fail.sh", "#!/bin/sh\nexit 1");

        let config = SecretHelperConfig {
            command: script_path.to_str().unwrap().to_string(),
            args: vec![],
        };
        let result = config.get_key("did:key:test");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_key_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let script_path =
            write_test_script(temp_dir.path(), "bad-json.sh", "#!/bin/sh\necho 'not json'");

        let config = SecretHelperConfig {
            command: script_path.to_str().unwrap().to_string(),
            args: vec![],
        };
        let result = config.get_key("did:key:test");
        assert!(result.is_err());
    }

    #[test]
    fn test_discover_agent_dids() {
        let temp_dir = TempDir::new().unwrap();
        let tap_dir = temp_dir.path();

        // Create some agent directories
        std::fs::create_dir(tap_dir.join("did_key_z6Mk1234")).unwrap();
        std::fs::create_dir(tap_dir.join("did_web_example.com")).unwrap();
        // Not a DID directory - should be ignored
        std::fs::create_dir(tap_dir.join("logs")).unwrap();
        // Create a file - should be ignored
        std::fs::write(tap_dir.join("keys.json"), "{}").unwrap();

        let dids = discover_agent_dids(Some(tap_dir)).unwrap();
        assert_eq!(dids.len(), 2);
        assert!(dids.contains(&"did:key:z6Mk1234".to_string()));
        assert!(dids.contains(&"did:web:example.com".to_string()));
    }

    #[test]
    fn test_discover_agent_dids_empty() {
        let temp_dir = TempDir::new().unwrap();
        let dids = discover_agent_dids(Some(temp_dir.path())).unwrap();
        assert!(dids.is_empty());
    }

    #[test]
    fn test_discover_agent_dids_nonexistent() {
        let dids = discover_agent_dids(Some(Path::new("/nonexistent/path"))).unwrap();
        assert!(dids.is_empty());
    }
}
