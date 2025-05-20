//! Message Packing and Unpacking Utilities
//!
//! This module provides traits and implementations for standardizing
//! how messages are prepared for transmission (packed) and processed
//! upon receipt (unpacked).

use crate::agent_key::VerificationKey;
use crate::error::{Error, Result};
use crate::message::{Jwe, Jws, SecurityMode};
use async_trait::async_trait;
use base64::Engine;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use tap_msg::didcomm::PlainMessage;
use uuid::Uuid;

/// Error type specific to message packing and unpacking
#[derive(Debug, thiserror::Error)]
pub enum MessageError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Key manager error: {0}")]
    KeyManager(String),

    #[error("Crypto operation failed: {0}")]
    Crypto(String),

    #[error("Invalid message format: {0}")]
    InvalidFormat(String),

    #[error("Unsupported security mode: {0:?}")]
    UnsupportedSecurityMode(SecurityMode),

    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Verification failed")]
    VerificationFailed,

    #[error("Decryption failed")]
    DecryptionFailed,
}

impl From<MessageError> for Error {
    fn from(err: MessageError) -> Self {
        match err {
            MessageError::Serialization(e) => Error::Serialization(e.to_string()),
            MessageError::KeyManager(e) => Error::Cryptography(e),
            MessageError::Crypto(e) => Error::Cryptography(e),
            MessageError::InvalidFormat(e) => Error::Validation(e),
            MessageError::UnsupportedSecurityMode(mode) => {
                Error::Validation(format!("Unsupported security mode: {:?}", mode))
            }
            MessageError::MissingParameter(e) => {
                Error::Validation(format!("Missing parameter: {}", e))
            }
            MessageError::KeyNotFound(e) => Error::Cryptography(format!("Key not found: {}", e)),
            MessageError::VerificationFailed => {
                Error::Cryptography("Verification failed".to_string())
            }
            MessageError::DecryptionFailed => Error::Cryptography("Decryption failed".to_string()),
        }
    }
}

/// Options for packing a message
#[derive(Debug, Clone)]
pub struct PackOptions {
    /// Security mode to use
    pub security_mode: SecurityMode,
    /// Key ID of the recipient (for JWE)
    pub recipient_kid: Option<String>,
    /// Key ID of the sender (for JWS and JWE)
    pub sender_kid: Option<String>,
}

impl PackOptions {
    /// Create new default packing options
    pub fn new() -> Self {
        Self {
            security_mode: SecurityMode::Plain,
            recipient_kid: None,
            sender_kid: None,
        }
    }

    /// Set to use plain mode (no security)
    pub fn with_plain(mut self) -> Self {
        self.security_mode = SecurityMode::Plain;
        self
    }

    /// Set to use signed mode with the given sender key ID
    pub fn with_sign(mut self, sender_kid: &str) -> Self {
        self.security_mode = SecurityMode::Signed;
        self.sender_kid = Some(sender_kid.to_string());
        self
    }

    /// Set to use auth-crypt mode with the given sender and recipient key IDs
    pub fn with_auth_crypt(mut self, sender_kid: &str, recipient_jwk: &serde_json::Value) -> Self {
        self.security_mode = SecurityMode::AuthCrypt;
        self.sender_kid = Some(sender_kid.to_string());

        // Extract kid from JWK if available
        if let Some(kid) = recipient_jwk.get("kid").and_then(|k| k.as_str()) {
            self.recipient_kid = Some(kid.to_string());
        }

        self
    }

    /// Get the security mode
    pub fn security_mode(&self) -> SecurityMode {
        self.security_mode
    }
}

/// Options for unpacking a message
#[derive(Debug, Clone)]
pub struct UnpackOptions {
    /// Expected security mode, or Any to try all modes
    pub expected_security_mode: SecurityMode,
    /// Expected recipient key ID
    pub expected_recipient_kid: Option<String>,
    /// Whether to require a valid signature
    pub require_signature: bool,
}

impl UnpackOptions {
    /// Create new default unpacking options
    pub fn new() -> Self {
        Self {
            expected_security_mode: SecurityMode::Any,
            expected_recipient_kid: None,
            require_signature: false,
        }
    }

    /// Set whether to require a valid signature
    pub fn with_require_signature(mut self, require: bool) -> Self {
        self.require_signature = require;
        self
    }
}

/// Trait for objects that can be packed for secure transmission
#[async_trait]
pub trait Packable<Output = String>: Sized {
    /// Pack the object for secure transmission
    async fn pack(
        &self,
        key_manager: &(impl KeyManagerPacking + ?Sized),
        options: PackOptions,
    ) -> Result<Output>;
}

/// Trait for objects that can be unpacked from a secure format
#[async_trait]
pub trait Unpackable<Input, Output = PlainMessage>: Sized {
    /// Unpack the object from its secure format
    async fn unpack(
        packed_message: &Input,
        key_manager: &(impl KeyManagerPacking + ?Sized),
        options: UnpackOptions,
    ) -> Result<Output>;
}

/// Interface required for key managers to support packing/unpacking
#[async_trait]
pub trait KeyManagerPacking: Send + Sync + Debug {
    /// Get a signing key by ID
    async fn get_signing_key(
        &self,
        kid: &str,
    ) -> Result<Arc<dyn crate::agent_key::SigningKey + Send + Sync>>;

    /// Get an encryption key by ID
    async fn get_encryption_key(
        &self,
        kid: &str,
    ) -> Result<Arc<dyn crate::agent_key::EncryptionKey + Send + Sync>>;

    /// Get a decryption key by ID
    async fn get_decryption_key(
        &self,
        kid: &str,
    ) -> Result<Arc<dyn crate::agent_key::DecryptionKey + Send + Sync>>;

    /// Resolve a verification key
    async fn resolve_verification_key(
        &self,
        kid: &str,
    ) -> Result<Arc<dyn VerificationKey + Send + Sync>>;
}

/// Implement Packable for PlainMessage
#[async_trait]
impl Packable for PlainMessage {
    async fn pack(
        &self,
        key_manager: &(impl KeyManagerPacking + ?Sized),
        options: PackOptions,
    ) -> Result<String> {
        match options.security_mode {
            SecurityMode::Plain => {
                // For plain mode, just serialize the PlainMessage
                serde_json::to_string(self).map_err(|e| Error::Serialization(e.to_string()))
            }
            SecurityMode::Signed => {
                // Signed mode requires a sender KID
                let sender_kid = options.sender_kid.clone().ok_or_else(|| {
                    Error::Validation("Signed mode requires sender_kid".to_string())
                })?;

                // Get the signing key
                let signing_key = key_manager.get_signing_key(&sender_kid).await?;

                // Prepare the message payload to sign
                let payload =
                    serde_json::to_string(self).map_err(|e| Error::Serialization(e.to_string()))?;

                // Create a JWS
                let jws = signing_key
                    .create_jws(payload.as_bytes(), None)
                    .await
                    .map_err(|e| Error::Cryptography(format!("Failed to create JWS: {}", e)))?;

                // Serialize the JWS
                serde_json::to_string(&jws).map_err(|e| Error::Serialization(e.to_string()))
            }
            SecurityMode::AuthCrypt => {
                // AuthCrypt mode requires both sender and recipient KIDs
                let sender_kid = options.sender_kid.clone().ok_or_else(|| {
                    Error::Validation("AuthCrypt mode requires sender_kid".to_string())
                })?;

                let recipient_kid = options.recipient_kid.clone().ok_or_else(|| {
                    Error::Validation("AuthCrypt mode requires recipient_kid".to_string())
                })?;

                // Get the encryption key
                let encryption_key = key_manager.get_encryption_key(&sender_kid).await?;

                // Get the recipient's verification key
                let recipient_key = key_manager.resolve_verification_key(&recipient_kid).await?;

                // Serialize the message
                let plaintext =
                    serde_json::to_string(self).map_err(|e| Error::Serialization(e.to_string()))?;

                // Create a JWE for the recipient
                let jwe = encryption_key
                    .create_jwe(plaintext.as_bytes(), &[recipient_key], None)
                    .await
                    .map_err(|e| Error::Cryptography(format!("Failed to create JWE: {}", e)))?;

                // Serialize the JWE
                serde_json::to_string(&jwe).map_err(|e| Error::Serialization(e.to_string()))
            }
            SecurityMode::Any => {
                // Any mode is not valid for packing, only for unpacking
                Err(Error::Validation(
                    "SecurityMode::Any is not valid for packing".to_string(),
                ))
            }
        }
    }
}

/// We can't implement Packable for all types due to the conflict with PlainMessage
/// Instead, let's create a helper function:
pub async fn pack_any<T>(
    obj: &T,
    key_manager: &(impl KeyManagerPacking + ?Sized),
    options: PackOptions,
) -> Result<String>
where
    T: Serialize + Send + Sync + std::fmt::Debug + 'static + Sized,
{
    // Skip attempt to implement Packable for generic types and use a helper function instead

    // If the object is a PlainMessage, use PlainMessage's implementation
    if obj.type_id() == std::any::TypeId::of::<PlainMessage>() {
        // In this case, we can't easily downcast, so we'll serialize and deserialize
        let value = serde_json::to_value(obj).map_err(|e| Error::Serialization(e.to_string()))?;
        let plain_msg: PlainMessage =
            serde_json::from_value(value).map_err(|e| Error::Serialization(e.to_string()))?;
        return plain_msg.pack(key_manager, options).await;
    }

    // Otherwise, implement the same logic here as in the PlainMessage implementation
    match options.security_mode {
        SecurityMode::Plain => {
            // For plain mode, just serialize the object to JSON
            serde_json::to_string(obj).map_err(|e| Error::Serialization(e.to_string()))
        }
        SecurityMode::Signed => {
            // Signed mode requires a sender KID
            let sender_kid = options
                .sender_kid
                .clone()
                .ok_or_else(|| Error::Validation("Signed mode requires sender_kid".to_string()))?;

            // Get the signing key
            let signing_key = key_manager.get_signing_key(&sender_kid).await?;

            // Convert to a Value first
            let value =
                serde_json::to_value(obj).map_err(|e| Error::Serialization(e.to_string()))?;

            // Ensure it's an object
            let obj = value
                .as_object()
                .ok_or_else(|| Error::Validation("Message is not a JSON object".to_string()))?;

            // Extract ID, or generate one if missing
            let id_string = obj
                .get("id")
                .map(|v| v.as_str().unwrap_or_default().to_string())
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            let id = id_string.as_str();

            // Extract type, or use default
            let msg_type = obj
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("https://tap.rsvp/schema/1.0/message");

            // Create sender/recipient lists
            let from = options.sender_kid.as_ref().map(|kid| {
                // Extract DID part from kid (assuming format is did#key-1)
                kid.split('#').next().unwrap_or(kid).to_string()
            });

            let to = if let Some(kid) = &options.recipient_kid {
                // Extract DID part from kid
                let did = kid.split('#').next().unwrap_or(kid).to_string();
                vec![did]
            } else {
                vec![]
            };

            // Create a PlainMessage
            let plain_message = PlainMessage {
                id: id.to_string(),
                typ: "application/didcomm-plain+json".to_string(),
                type_: msg_type.to_string(),
                body: value,
                from: from.unwrap_or_default(),
                to,
                thid: None,
                pthid: None,
                created_time: Some(chrono::Utc::now().timestamp() as u64),
                expires_time: None,
                from_prior: None,
                attachments: None,
                extra_headers: std::collections::HashMap::new(),
            };

            // Prepare the message payload to sign
            let payload = serde_json::to_string(&plain_message)
                .map_err(|e| Error::Serialization(e.to_string()))?;

            // Create a JWS
            let jws = signing_key
                .create_jws(payload.as_bytes(), None)
                .await
                .map_err(|e| Error::Cryptography(format!("Failed to create JWS: {}", e)))?;

            // Serialize the JWS
            serde_json::to_string(&jws).map_err(|e| Error::Serialization(e.to_string()))
        }
        SecurityMode::AuthCrypt => {
            // AuthCrypt mode requires both sender and recipient KIDs
            let sender_kid = options.sender_kid.clone().ok_or_else(|| {
                Error::Validation("AuthCrypt mode requires sender_kid".to_string())
            })?;

            let recipient_kid = options.recipient_kid.clone().ok_or_else(|| {
                Error::Validation("AuthCrypt mode requires recipient_kid".to_string())
            })?;

            // Get the encryption key
            let encryption_key = key_manager.get_encryption_key(&sender_kid).await?;

            // Get the recipient's verification key
            let recipient_key = key_manager.resolve_verification_key(&recipient_kid).await?;

            // Convert to a Value first
            let value =
                serde_json::to_value(obj).map_err(|e| Error::Serialization(e.to_string()))?;

            // Ensure it's an object
            let obj = value
                .as_object()
                .ok_or_else(|| Error::Validation("Message is not a JSON object".to_string()))?;

            // Extract ID, or generate one if missing
            let id_string = obj
                .get("id")
                .map(|v| v.as_str().unwrap_or_default().to_string())
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            let id = id_string.as_str();

            // Extract type, or use default
            let msg_type = obj
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("https://tap.rsvp/schema/1.0/message");

            // Create sender/recipient lists
            let from = options.sender_kid.as_ref().map(|kid| {
                // Extract DID part from kid (assuming format is did#key-1)
                kid.split('#').next().unwrap_or(kid).to_string()
            });

            let to = if let Some(kid) = &options.recipient_kid {
                // Extract DID part from kid
                let did = kid.split('#').next().unwrap_or(kid).to_string();
                vec![did]
            } else {
                vec![]
            };

            // Create a PlainMessage
            let plain_message = PlainMessage {
                id: id.to_string(),
                typ: "application/didcomm-plain+json".to_string(),
                type_: msg_type.to_string(),
                body: value,
                from: from.unwrap_or_default(),
                to,
                thid: None,
                pthid: None,
                created_time: Some(chrono::Utc::now().timestamp() as u64),
                expires_time: None,
                from_prior: None,
                attachments: None,
                extra_headers: std::collections::HashMap::new(),
            };

            // Serialize the message
            let plaintext = serde_json::to_string(&plain_message)
                .map_err(|e| Error::Serialization(e.to_string()))?;

            // Create a JWE for the recipient
            let jwe = encryption_key
                .create_jwe(plaintext.as_bytes(), &[recipient_key], None)
                .await
                .map_err(|e| Error::Cryptography(format!("Failed to create JWE: {}", e)))?;

            // Serialize the JWE
            serde_json::to_string(&jwe).map_err(|e| Error::Serialization(e.to_string()))
        }
        SecurityMode::Any => {
            // Any mode is not valid for packing, only for unpacking
            Err(Error::Validation(
                "SecurityMode::Any is not valid for packing".to_string(),
            ))
        }
    }
}

/// Implement Unpackable for JWS
#[async_trait]
impl<T: DeserializeOwned + Send + 'static> Unpackable<Jws, T> for Jws {
    async fn unpack(
        packed_message: &Jws,
        key_manager: &(impl KeyManagerPacking + ?Sized),
        _options: UnpackOptions,
    ) -> Result<T> {
        // Decode the payload
        let payload_bytes = base64::engine::general_purpose::STANDARD
            .decode(&packed_message.payload)
            .map_err(|e| Error::Cryptography(format!("Failed to decode JWS payload: {}", e)))?;

        // Convert to string
        let payload_str = String::from_utf8(payload_bytes)
            .map_err(|e| Error::Validation(format!("Invalid UTF-8 in payload: {}", e)))?;

        // Parse as PlainMessage first
        let plain_message: PlainMessage =
            serde_json::from_str(&payload_str).map_err(|e| Error::Serialization(e.to_string()))?;

        // Verify signatures
        let mut verified = false;

        for signature in &packed_message.signatures {
            // Decode the protected header
            let protected_bytes = base64::engine::general_purpose::STANDARD
                .decode(&signature.protected)
                .map_err(|e| {
                    Error::Cryptography(format!("Failed to decode protected header: {}", e))
                })?;

            // Parse the protected header
            let protected: crate::message::JwsProtected = serde_json::from_slice(&protected_bytes)
                .map_err(|e| {
                    Error::Serialization(format!("Failed to parse protected header: {}", e))
                })?;

            // Get the key ID
            let kid = &signature.header.kid;

            // Resolve the verification key
            let verification_key = match key_manager.resolve_verification_key(kid).await {
                Ok(key) => key,
                Err(_) => continue, // Skip key if we can't resolve it
            };

            // Decode the signature
            let signature_bytes = base64::engine::general_purpose::STANDARD
                .decode(&signature.signature)
                .map_err(|e| Error::Cryptography(format!("Failed to decode signature: {}", e)))?;

            // Create the signing input (protected.payload)
            let signing_input = format!("{}.{}", signature.protected, packed_message.payload);

            // Verify the signature
            match verification_key
                .verify_signature(signing_input.as_bytes(), &signature_bytes, &protected)
                .await
            {
                Ok(true) => {
                    verified = true;
                    break;
                }
                _ => continue,
            }
        }

        if !verified {
            return Err(Error::Cryptography(
                "Signature verification failed".to_string(),
            ));
        }

        // If we want the PlainMessage itself, return it
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<PlainMessage>() {
            // This is safe because we've verified that T is PlainMessage
            let result = serde_json::to_value(plain_message).unwrap();
            return serde_json::from_value(result).map_err(|e| Error::Serialization(e.to_string()));
        }

        // Otherwise deserialize the body to the requested type
        serde_json::from_value(plain_message.body).map_err(|e| Error::Serialization(e.to_string()))
    }
}

/// Implement Unpackable for JWE
#[async_trait]
impl<T: DeserializeOwned + Send + 'static> Unpackable<Jwe, T> for Jwe {
    async fn unpack(
        packed_message: &Jwe,
        key_manager: &(impl KeyManagerPacking + ?Sized),
        options: UnpackOptions,
    ) -> Result<T> {
        // Find a recipient that matches our expected key, if any
        let recipients = if let Some(kid) = &options.expected_recipient_kid {
            // Filter to just the matching recipient
            packed_message
                .recipients
                .iter()
                .filter(|r| r.header.kid == *kid)
                .collect::<Vec<_>>()
        } else {
            // Try all recipients
            packed_message.recipients.iter().collect::<Vec<_>>()
        };

        // Try each recipient until we find one we can decrypt
        for recipient in recipients {
            // Get the recipient's key ID
            let kid = &recipient.header.kid;

            // Get the decryption key
            let decryption_key = match key_manager.get_decryption_key(kid).await {
                Ok(key) => key,
                Err(_) => continue, // Skip if we don't have the key
            };

            // Try to decrypt
            match decryption_key.unwrap_jwe(packed_message).await {
                Ok(plaintext) => {
                    // Convert to string
                    let plaintext_str = String::from_utf8(plaintext).map_err(|e| {
                        Error::Validation(format!("Invalid UTF-8 in plaintext: {}", e))
                    })?;

                    // Parse as PlainMessage
                    let plain_message: PlainMessage = match serde_json::from_str(&plaintext_str) {
                        Ok(msg) => msg,
                        Err(e) => {
                            return Err(Error::Serialization(e.to_string()));
                        }
                    };

                    // If we want the PlainMessage itself, return it
                    if std::any::TypeId::of::<T>() == std::any::TypeId::of::<PlainMessage>() {
                        // This is safe because we've verified that T is PlainMessage
                        let result = serde_json::to_value(plain_message).unwrap();
                        return serde_json::from_value(result)
                            .map_err(|e| Error::Serialization(e.to_string()));
                    }

                    // Otherwise deserialize the body to the requested type
                    return serde_json::from_value(plain_message.body)
                        .map_err(|e| Error::Serialization(e.to_string()));
                }
                Err(_) => continue, // Try next recipient
            }
        }

        // If we get here, we couldn't decrypt for any recipient
        Err(Error::Cryptography("Failed to decrypt message".to_string()))
    }
}

/// Implement Unpackable for String (to handle any packed format)
#[async_trait]
impl<T: DeserializeOwned + Send + 'static> Unpackable<String, T> for String {
    async fn unpack(
        packed_message: &String,
        key_manager: &(impl KeyManagerPacking + ?Sized),
        options: UnpackOptions,
    ) -> Result<T> {
        // Try to parse as JSON first
        if let Ok(value) = serde_json::from_str::<Value>(packed_message) {
            // Check if it's a JWS (has payload and signatures fields)
            if value.get("payload").is_some() && value.get("signatures").is_some() {
                // Parse as JWS
                let jws: Jws = serde_json::from_str(packed_message)
                    .map_err(|e| Error::Serialization(e.to_string()))?;

                return Jws::unpack(&jws, key_manager, options).await;
            }

            // Check if it's a JWE (has ciphertext, protected, and recipients fields)
            if value.get("ciphertext").is_some()
                && value.get("protected").is_some()
                && value.get("recipients").is_some()
            {
                // Parse as JWE
                let jwe: Jwe = serde_json::from_str(packed_message)
                    .map_err(|e| Error::Serialization(e.to_string()))?;

                return Jwe::unpack(&jwe, key_manager, options).await;
            }

            // Check if it's a PlainMessage (has body and type fields)
            if value.get("body").is_some() && value.get("type").is_some() {
                // Parse as PlainMessage
                let plain: PlainMessage = serde_json::from_str(packed_message)
                    .map_err(|e| Error::Serialization(e.to_string()))?;

                // If we want the PlainMessage itself, return it
                if std::any::TypeId::of::<T>() == std::any::TypeId::of::<PlainMessage>() {
                    // This is safe because we've verified that T is PlainMessage
                    let result = serde_json::to_value(plain).unwrap();
                    return serde_json::from_value(result)
                        .map_err(|e| Error::Serialization(e.to_string()));
                }

                // Otherwise get the body
                return serde_json::from_value(plain.body)
                    .map_err(|e| Error::Serialization(e.to_string()));
            }

            // If it doesn't match any known format but is a valid JSON, try to parse directly
            return serde_json::from_value(value).map_err(|e| Error::Serialization(e.to_string()));
        }

        // If not valid JSON, return an error
        Err(Error::Validation("Message is not valid JSON".to_string()))
    }
}
