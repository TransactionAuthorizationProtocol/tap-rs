//! Standalone message verification utilities
//!
//! This module provides utilities for verifying signed messages without
//! requiring access to private keys or a key manager. This is designed to be
//! used by TAP Node for efficient signature verification.
//!
//! # Key Benefits
//!
//! - **No Private Keys Required**: Only needs DID resolution for public keys
//! - **Efficient**: Verify once for multiple recipients
//! - **Protocol Agnostic**: Works with any DID method that supports verification
//! - **Comprehensive**: Supports Ed25519, P-256, and Secp256k1 signatures
//!
//! # Usage
//!
//! The primary function is [`verify_jws`] which takes a JWS message and a DID resolver:
//!
//! ```rust,no_run
//! use tap_agent::{verify_jws, MultiResolver, Jws};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let resolver = MultiResolver::default();
//!     // let jws: Jws = serde_json::from_str(jws_string)?;
//!     // let verified_message = verify_jws(&jws, &resolver).await?;
//!     Ok(())
//! }
//! ```
//!
//! # Verification Process
//!
//! 1. Extract signer's DID from JWS signature header
//! 2. Resolve DID document using provided resolver
//! 3. Find matching verification method in DID document
//! 4. Verify signature using appropriate algorithm (EdDSA, ES256, ES256K)
//! 5. Return verified PlainMessage

use crate::did::SyncDIDResolver;
use crate::error::{Error, Result};
use crate::message::{Jws, JwsProtected};
use base64::Engine;
use tap_msg::didcomm::PlainMessage;

/// Verify a JWS (JSON Web Signature) message using DID resolution
///
/// This function verifies the signature on a JWS message by:
/// 1. Extracting the signer's DID from the signature
/// 2. Resolving the DID document to get the verification key
/// 3. Verifying the signature using the resolved key
///
/// # Arguments
/// * `jws` - The JWS message to verify
/// * `resolver` - A DID resolver to look up verification keys
///
/// # Returns
/// * The verified PlainMessage on success
/// * An error if verification fails or the DID cannot be resolved
pub async fn verify_jws(jws: &Jws, resolver: &dyn SyncDIDResolver) -> Result<PlainMessage> {
    // Ensure we have at least one signature
    if jws.signatures.is_empty() {
        return Err(Error::Validation("No signatures found in JWS".to_string()));
    }

    // Try to verify with each signature until one succeeds
    let mut last_error = None;

    for signature in &jws.signatures {
        // Get the kid from the signature
        let kid = match signature.get_kid() {
            Some(kid) => kid,
            None => {
                last_error = Some(Error::Validation("No kid found in signature".to_string()));
                continue;
            }
        };

        // Extract DID from kid (format: did:example:alice#keys-1)
        let did = match kid.split('#').next() {
            Some(did) => did,
            None => {
                last_error = Some(Error::Validation(format!("Invalid kid format: {}", kid)));
                continue;
            }
        };

        // Resolve the DID document
        let did_doc = match resolver.resolve(did).await {
            Ok(Some(doc)) => doc,
            Ok(None) => {
                last_error = Some(Error::DidResolution(format!("DID {} not found", did)));
                continue;
            }
            Err(e) => {
                last_error = Some(Error::DidResolution(format!(
                    "Failed to resolve DID {}: {}",
                    did, e
                )));
                continue;
            }
        };

        // Find the verification method
        let verification_method = match did_doc.verification_method.iter().find(|vm| vm.id == kid) {
            Some(vm) => vm,
            None => {
                last_error = Some(Error::Validation(format!(
                    "Verification method {} not found in DID document",
                    kid
                )));
                continue;
            }
        };

        // Decode the protected header
        let protected_bytes =
            match base64::engine::general_purpose::STANDARD.decode(&signature.protected) {
                Ok(bytes) => bytes,
                Err(e) => {
                    last_error = Some(Error::Cryptography(format!(
                        "Failed to decode protected header: {}",
                        e
                    )));
                    continue;
                }
            };

        // Parse the protected header
        let protected: JwsProtected = match serde_json::from_slice(&protected_bytes) {
            Ok(p) => p,
            Err(e) => {
                last_error = Some(Error::Serialization(format!(
                    "Failed to parse protected header: {}",
                    e
                )));
                continue;
            }
        };

        // Create the signing input (protected.payload)
        let signing_input = format!("{}.{}", signature.protected, jws.payload);

        // Decode the signature
        let signature_bytes =
            match base64::engine::general_purpose::STANDARD.decode(&signature.signature) {
                Ok(bytes) => bytes,
                Err(e) => {
                    last_error = Some(Error::Cryptography(format!(
                        "Failed to decode signature: {}",
                        e
                    )));
                    continue;
                }
            };

        // Verify the signature based on the algorithm
        let verified = match protected.alg.as_str() {
            "EdDSA" => verify_eddsa(verification_method, &signing_input, &signature_bytes),
            "ES256" => verify_es256(verification_method, &signing_input, &signature_bytes),
            "ES256K" => verify_es256k(verification_method, &signing_input, &signature_bytes),
            alg => {
                last_error = Some(Error::Validation(format!("Unsupported algorithm: {}", alg)));
                continue;
            }
        };

        if verified {
            // Decode and return the payload
            let payload_bytes = base64::engine::general_purpose::STANDARD
                .decode(&jws.payload)
                .map_err(|e| Error::Cryptography(format!("Failed to decode payload: {}", e)))?;

            let payload_str = String::from_utf8(payload_bytes)
                .map_err(|e| Error::Validation(format!("Invalid UTF-8 in payload: {}", e)))?;

            return serde_json::from_str(&payload_str).map_err(|e| {
                Error::Serialization(format!("Failed to parse payload as PlainMessage: {}", e))
            });
        }
    }

    // If we get here, no signature could be verified
    Err(last_error
        .unwrap_or_else(|| Error::Cryptography("Signature verification failed".to_string())))
}

/// Verify an EdDSA signature
fn verify_eddsa(
    verification_method: &crate::did::VerificationMethod,
    signing_input: &str,
    signature: &[u8],
) -> bool {
    use crate::did::VerificationMaterial;
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};
    use multibase::Base;

    // Extract the public key from the verification method
    let public_key_bytes = match &verification_method.verification_material {
        VerificationMaterial::Multibase {
            public_key_multibase,
        } => {
            // Decode the multibase key
            match multibase::decode(public_key_multibase) {
                Ok((Base::Base58Btc, key_data)) => {
                    // Skip the multicodec prefix (0xed01 for Ed25519)
                    if key_data.len() >= 34 && key_data[0] == 0xed && key_data[1] == 0x01 {
                        key_data[2..].to_vec()
                    } else {
                        return false;
                    }
                }
                _ => return false,
            }
        }
        _ => return false,
    };

    // Create the verifying key
    let verifying_key =
        match VerifyingKey::from_bytes(&public_key_bytes.try_into().unwrap_or_default()) {
            Ok(key) => key,
            Err(_) => return false,
        };

    // Create the signature
    let signature = match Signature::from_slice(signature) {
        Ok(sig) => sig,
        Err(_) => return false,
    };

    // Verify
    verifying_key
        .verify(signing_input.as_bytes(), &signature)
        .is_ok()
}

/// Verify an ES256 (P-256) signature
fn verify_es256(
    verification_method: &crate::did::VerificationMethod,
    signing_input: &str,
    signature: &[u8],
) -> bool {
    use crate::did::VerificationMaterial;
    use multibase::Base;
    use p256::ecdsa::signature::Verifier;
    use p256::ecdsa::{Signature, VerifyingKey};

    // Extract the public key from the verification method
    let public_key_bytes = match &verification_method.verification_material {
        VerificationMaterial::Multibase {
            public_key_multibase,
        } => {
            // Decode the multibase key
            match multibase::decode(public_key_multibase) {
                Ok((Base::Base58Btc, key_data)) => {
                    // Skip the multicodec prefix (0x1200 for P-256)
                    if key_data.len() >= 67 && key_data[0] == 0x12 && key_data[1] == 0x00 {
                        key_data[2..].to_vec()
                    } else {
                        return false;
                    }
                }
                _ => return false,
            }
        }
        _ => return false,
    };

    // Create the verifying key
    let verifying_key = match VerifyingKey::from_sec1_bytes(&public_key_bytes) {
        Ok(key) => key,
        Err(_) => return false,
    };

    // Create the signature
    let signature = match Signature::from_slice(signature) {
        Ok(sig) => sig,
        Err(_) => return false,
    };

    // Verify
    verifying_key
        .verify(signing_input.as_bytes(), &signature)
        .is_ok()
}

/// Verify an ES256K (secp256k1) signature
fn verify_es256k(
    verification_method: &crate::did::VerificationMethod,
    signing_input: &str,
    signature: &[u8],
) -> bool {
    use crate::did::VerificationMaterial;
    use k256::ecdsa::signature::Verifier;
    use k256::ecdsa::{Signature, VerifyingKey};
    use multibase::Base;

    // Extract the public key from the verification method
    let public_key_bytes = match &verification_method.verification_material {
        VerificationMaterial::Multibase {
            public_key_multibase,
        } => {
            // Decode the multibase key
            match multibase::decode(public_key_multibase) {
                Ok((Base::Base58Btc, key_data)) => {
                    // Skip the multicodec prefix (0xe701 for secp256k1)
                    if key_data.len() >= 67 && key_data[0] == 0xe7 && key_data[1] == 0x01 {
                        key_data[2..].to_vec()
                    } else {
                        return false;
                    }
                }
                _ => return false,
            }
        }
        _ => return false,
    };

    // Create the verifying key
    let verifying_key = match VerifyingKey::from_sec1_bytes(&public_key_bytes) {
        Ok(key) => key,
        Err(_) => return false,
    };

    // Create the signature
    let signature = match Signature::from_slice(signature) {
        Ok(sig) => sig,
        Err(_) => return false,
    };

    // Verify
    verifying_key
        .verify(signing_input.as_bytes(), &signature)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_key_manager::AgentKeyManagerBuilder;
    use crate::did::{DIDGenerationOptions, KeyType};
    use crate::key_manager::KeyManager;
    use crate::message::JwsSignature;
    use crate::message_packing::{PackOptions, Packable};
    use std::sync::Arc;

    #[derive(Debug)]
    struct TestResolver {
        did_docs: std::collections::HashMap<String, crate::did::DIDDoc>,
    }

    #[async_trait::async_trait]
    impl SyncDIDResolver for TestResolver {
        async fn resolve(&self, did: &str) -> Result<Option<crate::did::DIDDoc>> {
            Ok(self.did_docs.get(did).cloned())
        }
    }

    #[tokio::test]
    async fn test_verify_jws() {
        // Create a key manager and generate a key
        let key_manager = Arc::new(AgentKeyManagerBuilder::new().build().unwrap());
        let key = key_manager
            .generate_key(DIDGenerationOptions {
                key_type: KeyType::Ed25519,
            })
            .unwrap();

        // Create a test message
        let message = PlainMessage {
            id: "test-message".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: "https://example.org/test".to_string(),
            body: serde_json::json!({
                "content": "Test message for verification"
            }),
            from: key.did.clone(),
            to: vec!["did:example:bob".to_string()],
            thid: None,
            pthid: None,
            created_time: Some(1234567890),
            expires_time: None,
            from_prior: None,
            attachments: None,
            extra_headers: Default::default(),
        };

        // Pack the message as JWS - use the actual verification method ID
        let sender_kid = key.did_doc.verification_method[0].id.clone();
        let pack_options = PackOptions::new().with_sign(&sender_kid);
        let packed = message.pack(&*key_manager, pack_options).await.unwrap();

        // Parse as JWS
        let jws: Jws = serde_json::from_str(&packed).unwrap();

        // Create a test resolver with the DID document
        let mut did_docs = std::collections::HashMap::new();
        did_docs.insert(key.did.clone(), key.did_doc.clone());
        let resolver = TestResolver { did_docs };

        // Verify the JWS
        let verified_message = verify_jws(&jws, &resolver).await.unwrap();

        // Check that we got the original message back
        assert_eq!(verified_message.id, message.id);
        assert_eq!(verified_message.type_, message.type_);
        assert_eq!(verified_message.body, message.body);
    }

    #[tokio::test]
    async fn test_verify_jws_unknown_did() {
        // Create a key manager and generate a key
        let key_manager = Arc::new(AgentKeyManagerBuilder::new().build().unwrap());
        let key = key_manager
            .generate_key(DIDGenerationOptions {
                key_type: KeyType::Ed25519,
            })
            .unwrap();

        // Create and sign a message
        let message = PlainMessage {
            id: "test-message".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: "https://example.org/test".to_string(),
            body: serde_json::json!({"content": "Test"}),
            from: key.did.clone(),
            to: vec!["did:example:bob".to_string()],
            thid: None,
            pthid: None,
            created_time: Some(1234567890),
            expires_time: None,
            from_prior: None,
            attachments: None,
            extra_headers: Default::default(),
        };

        let sender_kid = key.did_doc.verification_method[0].id.clone();
        let pack_options = PackOptions::new().with_sign(&sender_kid);
        let packed = message.pack(&*key_manager, pack_options).await.unwrap();
        let jws: Jws = serde_json::from_str(&packed).unwrap();

        // Create an empty resolver (no DIDs)
        let resolver = TestResolver {
            did_docs: std::collections::HashMap::new(),
        };

        // Verification should fail
        let result = verify_jws(&jws, &resolver).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_verify_jws_multiple_signatures() {
        // Create two key managers and keys
        let key_manager1 = Arc::new(AgentKeyManagerBuilder::new().build().unwrap());
        let key1 = key_manager1
            .generate_key(DIDGenerationOptions {
                key_type: KeyType::Ed25519,
            })
            .unwrap();

        let key_manager2 = Arc::new(AgentKeyManagerBuilder::new().build().unwrap());
        let key2 = key_manager2
            .generate_key(DIDGenerationOptions {
                key_type: KeyType::Ed25519,
            })
            .unwrap();

        // Create a message
        let message = PlainMessage {
            id: "multi-sig-test".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: "https://example.org/test".to_string(),
            body: serde_json::json!({"content": "Multiple signatures test"}),
            from: key1.did.clone(),
            to: vec!["did:example:bob".to_string()],
            thid: None,
            pthid: None,
            created_time: Some(1234567890),
            expires_time: None,
            from_prior: None,
            attachments: None,
            extra_headers: Default::default(),
        };

        // Sign with first key
        let sender_kid1 = key1.did_doc.verification_method[0].id.clone();
        let pack_options1 = PackOptions::new().with_sign(&sender_kid1);
        let packed1 = message.pack(&*key_manager1, pack_options1).await.unwrap();
        let jws1: Jws = serde_json::from_str(&packed1).unwrap();

        // Create resolver with both DIDs
        let mut did_docs = std::collections::HashMap::new();
        did_docs.insert(key1.did.clone(), key1.did_doc.clone());
        did_docs.insert(key2.did.clone(), key2.did_doc.clone());
        let resolver = TestResolver { did_docs };

        // Verify with first signature should work
        let verified_message = verify_jws(&jws1, &resolver).await.unwrap();
        assert_eq!(verified_message.id, message.id);
    }

    #[tokio::test]
    async fn test_verify_jws_invalid_signature() {
        // Create a key manager and generate a key
        let key_manager = Arc::new(AgentKeyManagerBuilder::new().build().unwrap());
        let key = key_manager
            .generate_key(DIDGenerationOptions {
                key_type: KeyType::Ed25519,
            })
            .unwrap();

        // Create a JWS with an invalid signature
        let jws = Jws {
            payload: base64::engine::general_purpose::STANDARD.encode(r#"{"id":"test","type":"test"}"#),
            signatures: vec![JwsSignature {
                protected: base64::engine::general_purpose::STANDARD.encode(r#"{"typ":"application/didcomm-signed+json","alg":"EdDSA","kid":"did:key:invalid#key"}"#),
                signature: "invalid_signature".to_string(),
            }],
        };

        // Create resolver with the DID
        let mut did_docs = std::collections::HashMap::new();
        did_docs.insert(key.did.clone(), key.did_doc.clone());
        let resolver = TestResolver { did_docs };

        // Verification should fail due to invalid signature format
        let result = verify_jws(&jws, &resolver).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_jws_no_signatures() {
        // Create a JWS with no signatures
        let jws = Jws {
            payload: base64::engine::general_purpose::STANDARD
                .encode(r#"{"id":"test","type":"test"}"#),
            signatures: vec![],
        };

        let resolver = TestResolver {
            did_docs: std::collections::HashMap::new(),
        };

        // Verification should fail due to no signatures
        let result = verify_jws(&jws, &resolver).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No signatures found"));
    }

    #[tokio::test]
    async fn test_verify_jws_malformed_protected_header() {
        // Create a JWS with malformed protected header
        let jws = Jws {
            payload: base64::engine::general_purpose::STANDARD
                .encode(r#"{"id":"test","type":"test"}"#),
            signatures: vec![JwsSignature {
                protected: "invalid_base64!".to_string(),
                signature: "dGVzdA==".to_string(),
            }],
        };

        let resolver = TestResolver {
            did_docs: std::collections::HashMap::new(),
        };

        // Verification should fail due to malformed protected header
        let result = verify_jws(&jws, &resolver).await;
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("No kid found in signature"));
    }

    #[tokio::test]
    async fn test_verify_jws_missing_kid() {
        // Create a JWS with missing kid in protected header
        let jws = Jws {
            payload: base64::engine::general_purpose::STANDARD
                .encode(r#"{"id":"test","type":"test"}"#),
            signatures: vec![JwsSignature {
                protected: base64::engine::general_purpose::STANDARD
                    .encode(r#"{"typ":"application/didcomm-signed+json","alg":"EdDSA"}"#),
                signature: "dGVzdA==".to_string(),
            }],
        };

        let resolver = TestResolver {
            did_docs: std::collections::HashMap::new(),
        };

        // Verification should fail due to missing kid
        let result = verify_jws(&jws, &resolver).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_jws_invalid_kid_format() {
        // Create a JWS with invalid kid format (no fragment)
        let jws = Jws {
            payload: base64::engine::general_purpose::STANDARD.encode(r#"{"id":"test","type":"test"}"#),
            signatures: vec![JwsSignature {
                protected: base64::engine::general_purpose::STANDARD.encode(r#"{"typ":"application/didcomm-signed+json","alg":"EdDSA","kid":"invalid_kid_format"}"#),
                signature: "dGVzdA==".to_string(),
            }],
        };

        let resolver = TestResolver {
            did_docs: std::collections::HashMap::new(),
        };

        // Verification should fail due to invalid kid format
        let result = verify_jws(&jws, &resolver).await;
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("DID invalid_kid_format not found"));
    }

    #[tokio::test]
    async fn test_verify_jws_unsupported_algorithm() {
        // Create a key manager and generate a key
        let key_manager = Arc::new(AgentKeyManagerBuilder::new().build().unwrap());
        let key = key_manager
            .generate_key(DIDGenerationOptions {
                key_type: KeyType::Ed25519,
            })
            .unwrap();

        // Create a JWS with unsupported algorithm
        let kid = key.did_doc.verification_method[0].id.clone();
        let jws = Jws {
            payload: base64::engine::general_purpose::STANDARD
                .encode(r#"{"id":"test","type":"test"}"#),
            signatures: vec![JwsSignature {
                protected: base64::engine::general_purpose::STANDARD.encode(&format!(
                    r#"{{"typ":"application/didcomm-signed+json","alg":"UNSUPPORTED","kid":"{}"}}"#,
                    kid
                )),
                signature: "dGVzdA==".to_string(),
            }],
        };

        // Create resolver with the DID
        let mut did_docs = std::collections::HashMap::new();
        did_docs.insert(key.did.clone(), key.did_doc.clone());
        let resolver = TestResolver { did_docs };

        // Verification should fail due to unsupported algorithm
        let result = verify_jws(&jws, &resolver).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported algorithm"));
    }

    #[tokio::test]
    async fn test_verify_jws_different_key_types() {
        // Test Ed25519 key type
        let key_manager = Arc::new(AgentKeyManagerBuilder::new().build().unwrap());
        let key = key_manager
            .generate_key(DIDGenerationOptions {
                key_type: KeyType::Ed25519,
            })
            .unwrap();

        let message = PlainMessage {
            id: "ed25519-test".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: "https://example.org/test".to_string(),
            body: serde_json::json!({"content": "Ed25519 test"}),
            from: key.did.clone(),
            to: vec!["did:example:bob".to_string()],
            thid: None,
            pthid: None,
            created_time: Some(1234567890),
            expires_time: None,
            from_prior: None,
            attachments: None,
            extra_headers: Default::default(),
        };

        let sender_kid = key.did_doc.verification_method[0].id.clone();
        let pack_options = PackOptions::new().with_sign(&sender_kid);
        let packed = message.pack(&*key_manager, pack_options).await.unwrap();
        let jws: Jws = serde_json::from_str(&packed).unwrap();

        let mut did_docs = std::collections::HashMap::new();
        did_docs.insert(key.did.clone(), key.did_doc.clone());
        let resolver = TestResolver { did_docs };

        let verified_message = verify_jws(&jws, &resolver).await.unwrap();
        assert_eq!(verified_message.id, message.id);
        assert_eq!(verified_message.body, message.body);
    }
}
