//! Agent Key Abstraction for the TAP Agent
//!
//! This module provides a trait-based abstraction for cryptographic keys used by the TAP Agent.
//! It defines traits for signing, verification, encryption, and decryption operations, allowing
//! for a unified interface that can support both local keys and remote keys (e.g., HSM-backed).

use crate::error::{Error, Result};
use crate::message::{Jwe, JweProtected, JwsProtected};
use async_trait::async_trait;
use serde_json::Value;
use std::fmt::Debug;
use std::sync::Arc;

/// Defines core capabilities of a cryptographic key used by an agent.
///
/// This trait is the foundation for all agent key operations, providing
/// basic properties that all keys should have regardless of their specific
/// cryptographic capabilities.
#[async_trait]
pub trait AgentKey: Send + Sync + Debug {
    /// Returns the unique identifier for this key
    fn key_id(&self) -> &str;

    /// Exports the public key material as a JWK
    fn public_key_jwk(&self) -> Result<Value>;

    /// Returns the DID associated with this key
    fn did(&self) -> &str;

    /// Returns the key type (e.g., Ed25519, P-256, secp256k1)
    fn key_type(&self) -> &str;
}

/// JWS algorithm identifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JwsAlgorithm {
    /// Ed25519 signatures
    EdDSA,
    /// P-256 ECDSA signatures
    ES256,
    /// secp256k1 ECDSA signatures
    ES256K,
}

impl JwsAlgorithm {
    /// Returns the algorithm identifier as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            JwsAlgorithm::EdDSA => "EdDSA",
            JwsAlgorithm::ES256 => "ES256",
            JwsAlgorithm::ES256K => "ES256K",
        }
    }
}

/// JWE algorithm identifier (for key encryption)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JweAlgorithm {
    /// ECDH-ES + AES key wrap with 256-bit key
    EcdhEsA256kw,
}

impl JweAlgorithm {
    /// Returns the algorithm identifier as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            JweAlgorithm::EcdhEsA256kw => "ECDH-ES+A256KW",
        }
    }
}

/// JWE encryption algorithm identifier (for content encryption)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JweEncryption {
    /// AES-GCM with 256-bit key
    A256GCM,
}

impl JweEncryption {
    /// Returns the algorithm identifier as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            JweEncryption::A256GCM => "A256GCM",
        }
    }
}

/// Agent key capable of signing data for JWS creation.
///
/// Implementations of this trait can sign data to create JWS signatures.
#[async_trait]
pub trait SigningKey: AgentKey {
    /// Signs the provided data using this key
    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// Returns the recommended JWS algorithm for this key
    fn recommended_jws_alg(&self) -> JwsAlgorithm;

    /// Signs and creates a JWS with the provided payload
    async fn create_jws(
        &self,
        payload: &[u8],
        protected_header: Option<JwsProtected>,
    ) -> Result<crate::message::Jws>;
}

/// A key (typically public) capable of verifying a JWS signature.
///
/// This trait might be implemented by a struct holding a public JWK,
/// or by an AgentKey that can expose its public verification capabilities.
#[async_trait]
pub trait VerificationKey: Send + Sync + Debug {
    /// The key ID associated with this verification key
    fn key_id(&self) -> &str;

    /// Exports the public key material as a JWK
    fn public_key_jwk(&self) -> Result<Value>;

    /// Verifies the provided signature against the payload and protected header
    async fn verify_signature(
        &self,
        payload: &[u8],
        signature: &[u8],
        protected_header: &JwsProtected,
    ) -> Result<bool>;
}

/// Agent key capable of encrypting data for JWE creation.
///
/// Implementations of this trait can encrypt data for specific
/// recipients to create JWEs.
#[async_trait]
pub trait EncryptionKey: AgentKey {
    /// Encrypts plaintext data for a specific recipient
    async fn encrypt(
        &self,
        plaintext: &[u8],
        aad: Option<&[u8]>,
        recipient_public_key: &dyn VerificationKey,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)>; // (ciphertext, iv, tag)

    /// Returns the recommended JWE algorithm and encryption for this key
    fn recommended_jwe_alg_enc(&self) -> (JweAlgorithm, JweEncryption);

    /// Creates a JWE for multiple recipients
    async fn create_jwe(
        &self,
        plaintext: &[u8],
        recipients: &[Arc<dyn VerificationKey>],
        protected_header: Option<JweProtected>,
    ) -> Result<Jwe>;
}

/// Agent key capable of decrypting JWE data.
///
/// Implementations of this trait can decrypt JWE ciphertexts
/// that were encrypted for this key.
#[async_trait]
pub trait DecryptionKey: AgentKey {
    /// Decrypts the provided ciphertext
    async fn decrypt(
        &self,
        ciphertext: &[u8],
        encrypted_key: &[u8],
        iv: &[u8],
        tag: &[u8],
        aad: Option<&[u8]>,
        sender_key: Option<&dyn VerificationKey>,
    ) -> Result<Vec<u8>>;

    /// Unwraps a JWE to retrieve the plaintext
    async fn unwrap_jwe(&self, jwe: &Jwe) -> Result<Vec<u8>>;
}

/// Error type specific to agent key operations
#[derive(Debug, thiserror::Error)]
pub enum AgentKeyError {
    #[error("Key operation failed: {0}")]
    Operation(String),

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),

    #[error("Verification failed")]
    VerificationFailed,

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

impl From<AgentKeyError> for Error {
    fn from(err: AgentKeyError) -> Self {
        Error::Cryptography(err.to_string())
    }
}
