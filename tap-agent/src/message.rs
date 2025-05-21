//! Message types and utilities for the TAP Agent.
//!
//! This module provides constants and types for working with TAP messages,
//! including security modes and message type identifiers.

use serde::{Deserialize, Serialize};
// Value is not used in this file

/// Security mode for message packing and unpacking.
///
/// Defines the level of protection applied to messages:
/// - `Plain`: No encryption or signing (insecure, only for testing)
/// - `Signed`: Message is signed but not encrypted (integrity protected)
/// - `AuthCrypt`: Message is authenticated and encrypted (confidentiality + integrity)
/// - `Any`: Accept any security mode when unpacking (only used for receiving)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityMode {
    /// Plaintext - no encryption or signatures
    Plain,
    /// Signed - message is signed but not encrypted
    Signed,
    /// Authenticated and Encrypted - message is both signed and encrypted
    AuthCrypt,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Jws {
    pub payload: String,
    pub signatures: Vec<JwsSignature>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JwsSignature {
    pub protected: String,
    pub signature: String,
    pub header: JwsHeader,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JwsHeader {
    pub kid: String,
}

// Structure for decoded JWS protected field
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JwsProtected {
    #[serde(default = "default_didcomm_signed")]
    pub typ: String,
    pub alg: String,
}

// Helper function for JwsProtected typ default
fn default_didcomm_signed() -> String {
    DIDCOMM_SIGNED.to_string()
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
