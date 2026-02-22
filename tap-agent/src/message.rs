//! Message types and utilities for the TAP Agent.
//!
//! This module provides constants and types for working with TAP messages,
//! including security modes and message type identifiers.

use base64::{engine::general_purpose, Engine};
use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Decode a base64-encoded string, accepting both standard base64 and base64url (with or without padding).
pub fn base64_decode_flexible(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
    general_purpose::URL_SAFE_NO_PAD
        .decode(input)
        .or_else(|_| general_purpose::URL_SAFE.decode(input))
        .or_else(|_| general_purpose::STANDARD.decode(input))
        .or_else(|_| general_purpose::STANDARD_NO_PAD.decode(input))
}

/// Security mode for message packing and unpacking.
///
/// Defines the level of protection applied to messages:
/// - `Plain`: No encryption or signing (insecure, only for testing)
/// - `Signed`: Message is signed but not encrypted (integrity protected)
/// - `AuthCrypt`: Message is authenticated and encrypted (confidentiality + integrity, sender revealed)
/// - `AnonCrypt`: Message is anonymously encrypted (confidentiality only, sender hidden)
/// - `Any`: Accept any security mode when unpacking (only used for receiving)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityMode {
    /// Plaintext - no encryption or signatures
    Plain,
    /// Signed - message is signed but not encrypted
    Signed,
    /// Authenticated and Encrypted - message is both signed and encrypted (sender revealed)
    AuthCrypt,
    /// Anonymous Encrypted - message is encrypted but not signed (sender hidden)
    AnonCrypt,
    /// Any security mode - used for unpacking when any mode is acceptable
    Any,
}

/// Message type identifiers used by the TAP Protocol
/// These constant strings are used to identify different message types
/// in the TAP protocol communications.
/// Type identifier for Presentation messages
pub const PRESENTATION_MESSAGE_TYPE: &str = "https://tap.rsvp/schema/1.0#Presentation";

pub const DIDCOMM_SIGNED: &str = "application/didcomm-signed+json";
pub const DIDCOMM_ENCRYPTED: &str = "application/didcomm-encrypted+json";

// JWS-related types

/// JWS (JSON Web Signature) supporting both General and Flattened serializations per RFC 7515.
///
/// When serializing:
/// - Single signature: uses Flattened JWS format (`protected`, `payload`, `signature` at top level)
/// - Multiple signatures: uses General JWS format (`payload`, `signatures` array)
///
/// When deserializing: accepts both formats.
#[derive(Debug)]
pub struct Jws {
    pub payload: String,
    pub signatures: Vec<JwsSignature>,
}

impl Serialize for Jws {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        if self.signatures.len() == 1 {
            // Flattened JWS: { "protected", "payload", "signature" }
            let sig = &self.signatures[0];
            let mut map = serializer.serialize_map(Some(3))?;
            map.serialize_entry("payload", &self.payload)?;
            map.serialize_entry("protected", &sig.protected)?;
            map.serialize_entry("signature", &sig.signature)?;
            map.end()
        } else {
            // General JWS: { "payload", "signatures": [...] }
            let mut map = serializer.serialize_map(Some(2))?;
            map.serialize_entry("payload", &self.payload)?;
            map.serialize_entry("signatures", &self.signatures)?;
            map.end()
        }
    }
}

impl<'de> Deserialize<'de> for Jws {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        struct JwsVisitor;

        impl<'de> Visitor<'de> for JwsVisitor {
            type Value = Jws;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a JWS in General or Flattened serialization")
            }

            fn visit_map<M: MapAccess<'de>>(
                self,
                mut map: M,
            ) -> std::result::Result<Jws, M::Error> {
                let mut payload: Option<String> = None;
                let mut signatures: Option<Vec<JwsSignature>> = None;
                // Flattened fields
                let mut protected: Option<String> = None;
                let mut signature: Option<String> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "payload" => payload = Some(map.next_value()?),
                        "signatures" => signatures = Some(map.next_value()?),
                        "protected" => protected = Some(map.next_value()?),
                        "signature" => signature = Some(map.next_value()?),
                        _ => {
                            let _: serde_json::Value = map.next_value()?;
                        }
                    }
                }

                let payload = payload.ok_or_else(|| de::Error::missing_field("payload"))?;

                // Prefer General format if "signatures" is present
                if let Some(sigs) = signatures {
                    Ok(Jws {
                        payload,
                        signatures: sigs,
                    })
                } else if let (Some(prot), Some(sig)) = (protected, signature) {
                    // Flattened format
                    Ok(Jws {
                        payload,
                        signatures: vec![JwsSignature {
                            protected: prot,
                            signature: sig,
                        }],
                    })
                } else {
                    Err(de::Error::custom(
                        "JWS must have either 'signatures' array or 'protected'+'signature' fields",
                    ))
                }
            }
        }

        deserializer.deserialize_map(JwsVisitor)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JwsSignature {
    pub protected: String,
    pub signature: String,
}

// Structure for decoded JWS protected field
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JwsProtected {
    #[serde(default = "default_didcomm_signed")]
    pub typ: String,
    pub alg: String,
    pub kid: String,
}

// Helper function for JwsProtected typ default
fn default_didcomm_signed() -> String {
    DIDCOMM_SIGNED.to_string()
}

impl JwsSignature {
    /// Extracts the kid (key identifier) from the protected header
    pub fn get_kid(&self) -> Option<String> {
        if let Ok(protected_bytes) = base64_decode_flexible(&self.protected) {
            if let Ok(protected) = serde_json::from_slice::<JwsProtected>(&protected_bytes) {
                return Some(protected.kid);
            }
        }
        None
    }

    /// Decodes and returns the protected header
    pub fn get_protected_header(&self) -> Result<JwsProtected, Box<dyn std::error::Error>> {
        let protected_bytes = base64_decode_flexible(&self.protected)?;
        let protected = serde_json::from_slice::<JwsProtected>(&protected_bytes)?;
        Ok(protected)
    }
}
// JWE-related types

#[derive(Serialize, Deserialize, Debug)]
pub struct Jwe {
    pub ciphertext: String,
    pub protected: String,
    pub recipients: Vec<JweRecipient>,
    pub tag: String,
    pub iv: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JweRecipient {
    pub encrypted_key: String,
    pub header: JweHeader,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JweHeader {
    pub kid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_kid: Option<String>,
}

// Structure for decoded JWE protected field
#[derive(Serialize, Deserialize, Debug)]
pub struct JweProtected {
    pub epk: EphemeralPublicKey,
    pub apv: String,
    #[serde(default = "default_didcomm_encrypted")]
    pub typ: String,
    pub enc: String,
    pub alg: String,
}

// Helper function for JweProtected typ default
fn default_didcomm_encrypted() -> String {
    DIDCOMM_ENCRYPTED.to_string()
}

// Enum to handle different ephemeral public key types
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kty")]
pub enum EphemeralPublicKey {
    #[serde(rename = "EC")]
    Ec { crv: String, x: String, y: String },
    #[serde(rename = "OKP")]
    Okp { crv: String, x: String },
}
