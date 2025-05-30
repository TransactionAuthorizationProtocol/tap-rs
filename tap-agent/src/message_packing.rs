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
use tap_msg::didcomm::{PlainMessage, PlainMessageExt};
use tap_msg::message::TapMessage;
use uuid::Uuid;

/// Result of unpacking a message containing both the PlainMessage
/// and the parsed TAP message
#[derive(Debug, Clone)]
pub struct UnpackedMessage {
    /// The unpacked PlainMessage
    pub plain_message: PlainMessage,
    /// The parsed TAP message (if it could be parsed)
    pub tap_message: Option<TapMessage>,
}

impl UnpackedMessage {
    /// Create a new UnpackedMessage
    pub fn new(plain_message: PlainMessage) -> Self {
        let tap_message = TapMessage::from_plain_message(&plain_message).ok();
        Self {
            plain_message,
            tap_message,
        }
    }

    /// Try to get the message as a specific typed message
    pub fn as_typed<T: tap_msg::TapMessageBody>(&self) -> Result<PlainMessage<T>> {
        self.plain_message
            .clone()
            .parse_as()
            .map_err(|e| Error::Serialization(e.to_string()))
    }

    /// Convert to a typed message with untyped body
    pub fn into_typed(self) -> PlainMessage<Value> {
        self.plain_message.into_typed()
    }
}

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

impl Default for PackOptions {
    fn default() -> Self {
        Self::new()
    }
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

impl Default for UnpackOptions {
    fn default() -> Self {
        Self::new()
    }
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

                // Create protected header with the sender_kid
                let protected_header = crate::message::JwsProtected {
                    typ: crate::message::DIDCOMM_SIGNED.to_string(),
                    alg: String::new(), // Will be set by create_jws based on key type
                    kid: sender_kid.clone(),
                };

                // Create a JWS
                let jws = signing_key
                    .create_jws(payload.as_bytes(), Some(protected_header))
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

            // Create protected header with the sender_kid
            let protected_header = crate::message::JwsProtected {
                typ: crate::message::DIDCOMM_SIGNED.to_string(),
                alg: String::new(), // Will be set by create_jws based on key type
                kid: sender_kid.clone(),
            };

            // Create a JWS
            let jws = signing_key
                .create_jws(payload.as_bytes(), Some(protected_header))
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

            // Get the key ID from protected header
            let kid = match signature.get_kid() {
                Some(kid) => kid,
                None => continue, // Skip if no kid found
            };

            // Resolve the verification key
            let verification_key = match key_manager.resolve_verification_key(&kid).await {
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

/// Implement Unpackable for String to UnpackedMessage
#[async_trait]
impl Unpackable<String, UnpackedMessage> for String {
    async fn unpack(
        packed_message: &String,
        key_manager: &(impl KeyManagerPacking + ?Sized),
        options: UnpackOptions,
    ) -> Result<UnpackedMessage> {
        // First unpack to PlainMessage
        let plain_message: PlainMessage =
            String::unpack(packed_message, key_manager, options).await?;

        // Then create UnpackedMessage which will try to parse the TAP message
        Ok(UnpackedMessage::new(plain_message))
    }
}

/// Implement Unpackable for JWS to UnpackedMessage
#[async_trait]
impl Unpackable<Jws, UnpackedMessage> for Jws {
    async fn unpack(
        packed_message: &Jws,
        key_manager: &(impl KeyManagerPacking + ?Sized),
        options: UnpackOptions,
    ) -> Result<UnpackedMessage> {
        // First unpack to PlainMessage
        let plain_message: PlainMessage = Jws::unpack(packed_message, key_manager, options).await?;

        // Then create UnpackedMessage which will try to parse the TAP message
        Ok(UnpackedMessage::new(plain_message))
    }
}

/// Implement Unpackable for JWE to UnpackedMessage
#[async_trait]
impl Unpackable<Jwe, UnpackedMessage> for Jwe {
    async fn unpack(
        packed_message: &Jwe,
        key_manager: &(impl KeyManagerPacking + ?Sized),
        options: UnpackOptions,
    ) -> Result<UnpackedMessage> {
        // First unpack to PlainMessage
        let plain_message: PlainMessage = Jwe::unpack(packed_message, key_manager, options).await?;

        // Then create UnpackedMessage which will try to parse the TAP message
        Ok(UnpackedMessage::new(plain_message))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_key_manager::AgentKeyManagerBuilder;
    use crate::did::{DIDGenerationOptions, KeyType};
    use crate::key_manager::KeyManager;
    use std::sync::Arc;
    use tap_msg::didcomm::PlainMessage;

    #[tokio::test]
    async fn test_plain_message_pack_unpack() {
        // Create a key manager with a test key
        let key_manager = Arc::new(AgentKeyManagerBuilder::new().build().unwrap());
        let key = key_manager
            .generate_key(DIDGenerationOptions {
                key_type: KeyType::Ed25519,
            })
            .unwrap();

        // Create a test message
        let message = PlainMessage {
            id: "test-message-1".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: "https://example.org/test".to_string(),
            body: serde_json::json!({
                "content": "Hello, World!"
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

        // Pack in plain mode
        let pack_options = PackOptions::new().with_plain();
        let packed = message.pack(&*key_manager, pack_options).await.unwrap();

        // Unpack
        let unpack_options = UnpackOptions::new();
        let unpacked: PlainMessage = String::unpack(&packed, &*key_manager, unpack_options)
            .await
            .unwrap();

        // Verify
        assert_eq!(unpacked.id, message.id);
        assert_eq!(unpacked.type_, message.type_);
        assert_eq!(unpacked.body, message.body);
        assert_eq!(unpacked.from, message.from);
        assert_eq!(unpacked.to, message.to);
    }

    #[tokio::test]
    async fn test_jws_message_pack_unpack() {
        // Create a key manager with a test key
        let key_manager = Arc::new(AgentKeyManagerBuilder::new().build().unwrap());
        let key = key_manager
            .generate_key(DIDGenerationOptions {
                key_type: KeyType::Ed25519,
            })
            .unwrap();

        let sender_kid = format!("{}#keys-1", key.did);

        // Create a test message
        let message = PlainMessage {
            id: "test-message-2".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: "https://example.org/test".to_string(),
            body: serde_json::json!({
                "content": "Signed message"
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

        // Pack with signing
        let pack_options = PackOptions::new().with_sign(&sender_kid);
        let packed = message.pack(&*key_manager, pack_options).await.unwrap();

        // Verify it's a JWS
        let jws: Jws = serde_json::from_str(&packed).unwrap();
        assert!(!jws.signatures.is_empty());

        // Check the protected header has the correct kid
        let protected_header = jws.signatures[0].get_protected_header().unwrap();
        assert_eq!(protected_header.kid, sender_kid);
        assert_eq!(protected_header.typ, "application/didcomm-signed+json");
        assert_eq!(protected_header.alg, "EdDSA");

        // Unpack
        let unpack_options = UnpackOptions::new();
        let unpacked: PlainMessage = String::unpack(&packed, &*key_manager, unpack_options)
            .await
            .unwrap();

        // Verify
        assert_eq!(unpacked.id, message.id);
        assert_eq!(unpacked.type_, message.type_);
        assert_eq!(unpacked.body, message.body);
        assert_eq!(unpacked.from, message.from);
        assert_eq!(unpacked.to, message.to);
    }

    #[tokio::test]
    async fn test_different_key_types_jws() {
        // Test with different key types
        let key_types = vec![KeyType::Ed25519];

        for key_type in key_types {
            // Create a key manager with a test key
            let key_manager = Arc::new(AgentKeyManagerBuilder::new().build().unwrap());
            let key = key_manager
                .generate_key(DIDGenerationOptions { key_type })
                .unwrap();

            let sender_kid = format!("{}#keys-1", key.did);

            // Create a test message
            let message = PlainMessage {
                id: format!("test-{:?}", key_type),
                typ: "application/didcomm-plain+json".to_string(),
                type_: "https://example.org/test".to_string(),
                body: serde_json::json!({
                    "content": format!("Signed with {:?}", key_type)
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

            // Pack with signing
            let pack_options = PackOptions::new().with_sign(&sender_kid);
            let packed = message.pack(&*key_manager, pack_options).await.unwrap();

            // Verify it's a JWS
            let jws: Jws = serde_json::from_str(&packed).unwrap();
            assert!(!jws.signatures.is_empty());

            // Check the protected header
            let protected_header = jws.signatures[0].get_protected_header().unwrap();
            assert_eq!(protected_header.kid, sender_kid);

            // Check algorithm matches key type
            let expected_alg = match key_type {
                KeyType::Ed25519 => "EdDSA",
                KeyType::P256 => "ES256",
                KeyType::Secp256k1 => "ES256K",
            };
            assert_eq!(protected_header.alg, expected_alg);

            // Unpack and verify
            let unpack_options = UnpackOptions::new();
            let unpacked: PlainMessage = String::unpack(&packed, &*key_manager, unpack_options)
                .await
                .unwrap();

            assert_eq!(unpacked.id, message.id);
            assert_eq!(unpacked.body, message.body);
        }
    }

    #[tokio::test]
    async fn test_unpack_with_wrong_signature() {
        // Create two different key managers
        let key_manager1 = Arc::new(AgentKeyManagerBuilder::new().build().unwrap());
        let key1 = key_manager1
            .generate_key(DIDGenerationOptions {
                key_type: KeyType::Ed25519,
            })
            .unwrap();

        let key_manager2 = Arc::new(AgentKeyManagerBuilder::new().build().unwrap());
        let _key2 = key_manager2
            .generate_key(DIDGenerationOptions {
                key_type: KeyType::Ed25519,
            })
            .unwrap();

        // Create and sign a message with key1
        let message = PlainMessage {
            id: "test-wrong-sig".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: "https://example.org/test".to_string(),
            body: serde_json::json!({
                "content": "Test wrong signature"
            }),
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

        let sender_kid = format!("{}#keys-1", key1.did);
        let pack_options = PackOptions::new().with_sign(&sender_kid);
        let packed = message.pack(&*key_manager1, pack_options).await.unwrap();

        // Try to unpack with key_manager2 (should fail)
        let unpack_options = UnpackOptions::new();
        let result: Result<PlainMessage> =
            String::unpack(&packed, &*key_manager2, unpack_options).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unpack_to_unpacked_message() {
        // Create a key manager with a test key
        let key_manager = Arc::new(AgentKeyManagerBuilder::new().build().unwrap());
        let key = key_manager
            .generate_key(DIDGenerationOptions {
                key_type: KeyType::Ed25519,
            })
            .unwrap();

        // Create a TAP transfer message
        let message = PlainMessage {
            id: "test-transfer-1".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: "https://tap.rsvp/schema/1.0#Transfer".to_string(),
            body: serde_json::json!({
                "@type": "https://tap.rsvp/schema/1.0#Transfer",
                "transaction_id": "test-tx-123",
                "asset": {
                    "chain_id": {
                        "namespace": "eip155",
                        "reference": "1"
                    },
                    "namespace": "slip44",
                    "reference": "60"
                },
                "originator": {
                    "id": key.did.clone(),
                    "name": "Test Originator"
                },
                "amount": "100",
                "agents": [],
                "memo": null,
                "beneficiary": null,
                "settlement_id": null,
                "connection_id": null,
                "metadata": {}
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

        // Pack in plain mode
        let pack_options = PackOptions::new().with_plain();
        let packed = message.pack(&*key_manager, pack_options).await.unwrap();

        // Unpack to UnpackedMessage
        let unpack_options = UnpackOptions::new();
        let unpacked: UnpackedMessage = String::unpack(&packed, &*key_manager, unpack_options)
            .await
            .unwrap();

        // Verify PlainMessage
        assert_eq!(unpacked.plain_message.id, message.id);
        assert_eq!(unpacked.plain_message.type_, message.type_);

        // Verify TAP message was parsed
        assert!(unpacked.tap_message.is_some());
        match unpacked.tap_message.unwrap() {
            TapMessage::Transfer(transfer) => {
                assert_eq!(transfer.amount, "100");
                assert_eq!(transfer.originator.id, key.did);
            }
            _ => panic!("Expected Transfer message"),
        }
    }

    #[tokio::test]
    async fn test_unpack_invalid_tap_message() {
        // Create a key manager with a test key
        let key_manager = Arc::new(AgentKeyManagerBuilder::new().build().unwrap());
        let key = key_manager
            .generate_key(DIDGenerationOptions {
                key_type: KeyType::Ed25519,
            })
            .unwrap();

        // Create a message with an unknown type
        let message = PlainMessage {
            id: "test-unknown-1".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: "https://example.org/unknown#message".to_string(),
            body: serde_json::json!({
                "content": "Unknown message type"
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

        // Pack in plain mode
        let pack_options = PackOptions::new().with_plain();
        let packed = message.pack(&*key_manager, pack_options).await.unwrap();

        // Unpack to UnpackedMessage
        let unpack_options = UnpackOptions::new();
        let unpacked: UnpackedMessage = String::unpack(&packed, &*key_manager, unpack_options)
            .await
            .unwrap();

        // Verify PlainMessage was unpacked
        assert_eq!(unpacked.plain_message.id, message.id);

        // Verify TAP message parsing failed (unknown type)
        assert!(unpacked.tap_message.is_none());
    }
}
