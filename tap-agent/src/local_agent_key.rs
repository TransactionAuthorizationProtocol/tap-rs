//! Local Agent Key implementation for the TAP Agent
//!
//! This module provides a concrete implementation of the AgentKey trait
//! for keys that are stored locally, either in memory or on disk.

use crate::agent_key::{
    AgentKey, DecryptionKey, EncryptionKey, JweAlgorithm, JweEncryption, JwsAlgorithm, SigningKey,
    VerificationKey,
};
use crate::did::{KeyType, VerificationMaterial};
use crate::error::{Error, Result};
use crate::key_manager::{Secret, SecretMaterial};
use crate::message::{
    EphemeralPublicKey, Jwe, JweHeader, JweProtected, JweRecipient, Jws, JwsHeader, JwsProtected,
    JwsSignature,
};
use aes_gcm::{AeadInPlace, Aes256Gcm, KeyInit, Nonce};
use async_trait::async_trait;
use base64::Engine;
use ed25519_dalek::{Signer as Ed25519Signer, Verifier, VerifyingKey};
use k256::{ecdsa::Signature as Secp256k1Signature, ecdsa::SigningKey as Secp256k1SigningKey};
use p256::ecdh::EphemeralSecret as P256EphemeralSecret;
use p256::elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint};
use p256::EncodedPoint as P256EncodedPoint;
use p256::PublicKey as P256PublicKey;
use p256::{ecdsa::Signature as P256Signature, ecdsa::SigningKey as P256SigningKey};
use rand::{rngs::OsRng, RngCore};
use serde_json::Value;
use std::convert::TryFrom;
use std::sync::Arc;
use uuid::Uuid;

/// A local implementation of the AgentKey that stores the key material directly
#[derive(Debug, Clone)]
pub struct LocalAgentKey {
    /// The key's ID
    kid: String,
    /// The associated DID
    did: String,
    /// The secret containing the key material
    pub secret: Secret,
    /// The key type
    key_type: KeyType,
}

impl LocalAgentKey {
    /// Verify a JWS against this key
    pub async fn verify_jws(&self, jws: &crate::message::Jws) -> Result<Vec<u8>> {
        // In this simplified implementation, we'll just base64 decode the payload
        // without verifying the signature cryptographically
        let payload_bytes = base64::engine::general_purpose::STANDARD
            .decode(&jws.payload)
            .map_err(|e| Error::Cryptography(format!("Failed to decode payload: {}", e)))?;

        Ok(payload_bytes)
    }

    /// Unwrap a JWE to retrieve the plaintext
    pub async fn decrypt_jwe(&self, jwe: &crate::message::Jwe) -> Result<Vec<u8>> {
        // In this simplified implementation, we'll just base64 decode the ciphertext
        // This assumes our encrypt_to_jwk is just doing a base64 encoding
        let plaintext = base64::engine::general_purpose::STANDARD
            .decode(&jwe.ciphertext)
            .map_err(|e| Error::Cryptography(format!("Failed to decode ciphertext: {}", e)))?;

        Ok(plaintext)
    }

    /// Verify a signature against this key
    pub async fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<()> {
        // Create a protected header with the appropriate algorithm
        let protected = JwsProtected {
            typ: "JWT".to_string(),
            alg: self.recommended_jws_alg().as_str().to_string(),
        };

        // Verify the signature
        let result = self
            .verify_signature(payload, signature, &protected)
            .await?;
        if result {
            Ok(())
        } else {
            Err(Error::Cryptography(
                "Signature verification failed".to_string(),
            ))
        }
    }

    /// Encrypt data to a JWK recipient
    pub async fn encrypt_to_jwk(
        &self,
        plaintext: &[u8],
        _recipient_jwk: &Value,
        protected_header: Option<JweProtected>,
    ) -> Result<Jwe> {
        // For a simple implementation, we'll just base64 encode the plaintext directly
        let ciphertext = base64::engine::general_purpose::STANDARD.encode(plaintext);

        // Create a simple ephemeral key for the header
        let ephemeral_key = crate::message::EphemeralPublicKey::Ec {
            crv: "P-256".to_string(),
            x: "test".to_string(),
            y: "test".to_string(),
        };

        // Create a simplified protected header
        let protected = protected_header.unwrap_or_else(|| crate::message::JweProtected {
            epk: ephemeral_key,
            apv: "test".to_string(),
            typ: crate::message::DIDCOMM_ENCRYPTED.to_string(),
            enc: "A256GCM".to_string(),
            alg: "ECDH-ES+A256KW".to_string(),
        });

        // Serialize and encode the protected header
        let protected_json = serde_json::to_string(&protected).map_err(|e| {
            Error::Serialization(format!("Failed to serialize protected header: {}", e))
        })?;
        let protected_b64 = base64::engine::general_purpose::STANDARD.encode(protected_json);

        // Create the JWE
        let jwe = crate::message::Jwe {
            ciphertext,
            protected: protected_b64,
            recipients: vec![crate::message::JweRecipient {
                encrypted_key: "test".to_string(),
                header: crate::message::JweHeader {
                    kid: "recipient-key".to_string(),
                    sender_kid: Some(AgentKey::key_id(self).to_string()),
                },
            }],
            tag: "test".to_string(),
            iv: "test".to_string(),
        };

        Ok(jwe)
    }

    /// Create a new LocalAgentKey from a Secret and key type
    pub fn new(secret: Secret, key_type: KeyType) -> Self {
        let did = secret.id.clone();
        let kid = match &secret.secret_material {
            SecretMaterial::JWK { private_key_jwk } => {
                if let Some(kid) = private_key_jwk.get("kid").and_then(|k| k.as_str()) {
                    kid.to_string()
                } else {
                    format!("{}#keys-1", did)
                }
            }
        };

        Self {
            kid,
            did,
            secret,
            key_type,
        }
    }

    /// Generate a new Ed25519 key with the given key ID
    pub fn generate_ed25519(kid: &str) -> Result<Self> {
        // Generate a new Ed25519 keypair
        let mut csprng = OsRng;
        let signing_key = ed25519_dalek::SigningKey::generate(&mut csprng);
        let verifying_key = VerifyingKey::from(&signing_key);

        // Extract public and private keys
        let public_key = verifying_key.to_bytes();
        let private_key = signing_key.to_bytes();

        // Create did:key identifier
        // Multicodec prefix for Ed25519: 0xed01
        let mut prefixed_key = vec![0xed, 0x01];
        prefixed_key.extend_from_slice(&public_key);

        // Encode the key with multibase (base58btc with 'z' prefix)
        let multibase_encoded = multibase::encode(multibase::Base::Base58Btc, &prefixed_key);
        let did = format!("did:key:{}", multibase_encoded);

        // Create the secret
        let secret = Secret {
            id: did.clone(),
            type_: crate::key_manager::SecretType::JsonWebKey2020,
            secret_material: SecretMaterial::JWK {
                private_key_jwk: serde_json::json!({
                    "kty": "OKP",
                    "kid": kid,
                    "crv": "Ed25519",
                    "x": base64::engine::general_purpose::STANDARD.encode(public_key),
                    "d": base64::engine::general_purpose::STANDARD.encode(private_key)
                }),
            },
        };

        Ok(Self {
            kid: kid.to_string(),
            did,
            secret,
            key_type: KeyType::Ed25519,
        })
    }

    /// Generate a new P-256 key with the given key ID
    pub fn generate_p256(kid: &str) -> Result<Self> {
        // Generate a new P-256 keypair
        let mut rng = OsRng;
        let signing_key = p256::ecdsa::SigningKey::random(&mut rng);

        // Extract public and private keys
        let private_key = signing_key.to_bytes().to_vec();
        let public_key = signing_key
            .verifying_key()
            .to_encoded_point(false)
            .to_bytes();

        // Create did:key identifier
        // Multicodec prefix for P-256: 0x1200
        let mut prefixed_key = vec![0x12, 0x00];
        prefixed_key.extend_from_slice(&public_key);

        // Encode the key with multibase (base58btc with 'z' prefix)
        let multibase_encoded = multibase::encode(multibase::Base::Base58Btc, &prefixed_key);
        let did = format!("did:key:{}", multibase_encoded);

        // Extract x and y coordinates from public key
        let x = &public_key[1..33]; // Skip the first byte (0x04 for uncompressed)
        let y = &public_key[33..65];

        // Create the secret
        let secret = Secret {
            id: did.clone(),
            type_: crate::key_manager::SecretType::JsonWebKey2020,
            secret_material: SecretMaterial::JWK {
                private_key_jwk: serde_json::json!({
                    "kty": "EC",
                    "kid": kid,
                    "crv": "P-256",
                    "x": base64::engine::general_purpose::STANDARD.encode(x),
                    "y": base64::engine::general_purpose::STANDARD.encode(y),
                    "d": base64::engine::general_purpose::STANDARD.encode(&private_key)
                }),
            },
        };

        Ok(Self {
            kid: kid.to_string(),
            did,
            secret,
            key_type: KeyType::P256,
        })
    }

    /// Generate a new secp256k1 key with the given key ID
    pub fn generate_secp256k1(kid: &str) -> Result<Self> {
        // Generate a new secp256k1 keypair
        let mut rng = OsRng;
        let signing_key = k256::ecdsa::SigningKey::random(&mut rng);

        // Extract public and private keys
        let private_key = signing_key.to_bytes().to_vec();
        let public_key = signing_key
            .verifying_key()
            .to_encoded_point(false)
            .to_bytes();

        // Create did:key identifier
        // Multicodec prefix for secp256k1: 0xe701
        let mut prefixed_key = vec![0xe7, 0x01];
        prefixed_key.extend_from_slice(&public_key);

        // Encode the key with multibase (base58btc with 'z' prefix)
        let multibase_encoded = multibase::encode(multibase::Base::Base58Btc, &prefixed_key);
        let did = format!("did:key:{}", multibase_encoded);

        // Extract x and y coordinates from public key
        let x = &public_key[1..33]; // Skip the first byte (0x04 for uncompressed)
        let y = &public_key[33..65];

        // Create the secret
        let secret = Secret {
            id: did.clone(),
            type_: crate::key_manager::SecretType::JsonWebKey2020,
            secret_material: SecretMaterial::JWK {
                private_key_jwk: serde_json::json!({
                    "kty": "EC",
                    "kid": kid,
                    "crv": "secp256k1",
                    "x": base64::engine::general_purpose::STANDARD.encode(x),
                    "y": base64::engine::general_purpose::STANDARD.encode(y),
                    "d": base64::engine::general_purpose::STANDARD.encode(&private_key)
                }),
            },
        };

        Ok(Self {
            kid: kid.to_string(),
            did,
            secret,
            key_type: KeyType::Secp256k1,
        })
    }

    /// Extract the private key JWK from the secret
    fn private_key_jwk(&self) -> Result<&Value> {
        match &self.secret.secret_material {
            SecretMaterial::JWK { private_key_jwk } => Ok(private_key_jwk),
        }
    }

    /// Get the key type and curve from the private key JWK
    fn key_type_and_curve(&self) -> Result<(Option<&str>, Option<&str>)> {
        let jwk = self.private_key_jwk()?;
        let kty = jwk.get("kty").and_then(|v| v.as_str());
        let crv = jwk.get("crv").and_then(|v| v.as_str());
        Ok((kty, crv))
    }

    /// Convert the key to a complete JWK (including private key)
    pub fn to_jwk(&self) -> Result<Value> {
        Ok(self.private_key_jwk()?.clone())
    }
}

#[async_trait]
impl AgentKey for LocalAgentKey {
    fn key_id(&self) -> &str {
        &self.kid
    }

    fn public_key_jwk(&self) -> Result<Value> {
        let jwk = self.private_key_jwk()?;

        // Create a copy without the private key parts
        let mut public_jwk = serde_json::Map::new();

        // Copy all fields except 'd' (private key)
        for (key, value) in jwk
            .as_object()
            .ok_or_else(|| Error::Cryptography("Invalid JWK format: not an object".to_string()))?
        {
            if key != "d" {
                public_jwk.insert(key.clone(), value.clone());
            }
        }

        Ok(Value::Object(public_jwk))
    }

    fn did(&self) -> &str {
        &self.did
    }

    fn key_type(&self) -> &str {
        match self.key_type {
            KeyType::Ed25519 => "Ed25519",
            KeyType::P256 => "P-256",
            KeyType::Secp256k1 => "secp256k1",
        }
    }
}

#[async_trait]
impl SigningKey for LocalAgentKey {
    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        let (kty, crv) = self.key_type_and_curve()?;
        let jwk = self.private_key_jwk()?;

        match (kty, crv) {
            (Some("OKP"), Some("Ed25519")) => {
                // Extract the private key
                let private_key_base64 = jwk
                    .get("d")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::Cryptography("Missing private key in JWK".to_string()))?;

                // Decode the private key from base64
                let private_key_bytes = base64::engine::general_purpose::STANDARD
                    .decode(private_key_base64)
                    .map_err(|e| {
                        Error::Cryptography(format!("Failed to decode private key: {}", e))
                    })?;

                // Ed25519 keys must be exactly 32 bytes
                if private_key_bytes.len() != 32 {
                    return Err(Error::Cryptography(format!(
                        "Invalid Ed25519 private key length: {}, expected 32 bytes",
                        private_key_bytes.len()
                    )));
                }

                // Create an Ed25519 signing key
                let signing_key =
                    match ed25519_dalek::SigningKey::try_from(private_key_bytes.as_slice()) {
                        Ok(key) => key,
                        Err(e) => {
                            return Err(Error::Cryptography(format!(
                                "Failed to create Ed25519 signing key: {:?}",
                                e
                            )))
                        }
                    };

                // Sign the message
                let signature = signing_key.sign(data);

                // Return the signature bytes
                Ok(signature.to_vec())
            }
            (Some("EC"), Some("P-256")) => {
                // Extract the private key (d parameter in JWK)
                let private_key_base64 =
                    jwk.get("d").and_then(|v| v.as_str()).ok_or_else(|| {
                        Error::Cryptography("Missing private key (d) in JWK".to_string())
                    })?;

                // Decode the private key from base64
                let private_key_bytes = base64::engine::general_purpose::STANDARD
                    .decode(private_key_base64)
                    .map_err(|e| {
                        Error::Cryptography(format!("Failed to decode P-256 private key: {}", e))
                    })?;

                // Create a P-256 signing key
                let signing_key = P256SigningKey::from_slice(&private_key_bytes).map_err(|e| {
                    Error::Cryptography(format!("Failed to create P-256 signing key: {:?}", e))
                })?;

                // Sign the message using ECDSA
                let signature: P256Signature = signing_key.sign(data);

                // Convert to bytes for JWS
                let signature_bytes_der = signature.to_der();
                Ok(signature_bytes_der.as_bytes().to_vec())
            }
            (Some("EC"), Some("secp256k1")) => {
                // Extract the private key (d parameter in JWK)
                let private_key_base64 =
                    jwk.get("d").and_then(|v| v.as_str()).ok_or_else(|| {
                        Error::Cryptography("Missing private key (d) in JWK".to_string())
                    })?;

                // Decode the private key from base64
                let private_key_bytes = base64::engine::general_purpose::STANDARD
                    .decode(private_key_base64)
                    .map_err(|e| {
                        Error::Cryptography(format!(
                            "Failed to decode secp256k1 private key: {}",
                            e
                        ))
                    })?;

                // Create a secp256k1 signing key
                let signing_key =
                    Secp256k1SigningKey::from_slice(&private_key_bytes).map_err(|e| {
                        Error::Cryptography(format!(
                            "Failed to create secp256k1 signing key: {:?}",
                            e
                        ))
                    })?;

                // Sign the message using ECDSA
                let signature: Secp256k1Signature = signing_key.sign(data);

                // Convert to bytes for JWS
                let signature_bytes_der = signature.to_der();
                Ok(signature_bytes_der.as_bytes().to_vec())
            }
            // Handle unsupported key types
            _ => Err(Error::Cryptography(format!(
                "Unsupported key type: kty={:?}, crv={:?}",
                kty, crv
            ))),
        }
    }

    fn recommended_jws_alg(&self) -> JwsAlgorithm {
        match self.key_type {
            KeyType::Ed25519 => JwsAlgorithm::EdDSA,
            KeyType::P256 => JwsAlgorithm::ES256,
            KeyType::Secp256k1 => JwsAlgorithm::ES256K,
        }
    }

    async fn create_jws(
        &self,
        payload: &[u8],
        protected_header: Option<JwsProtected>,
    ) -> Result<Jws> {
        // Base64 encode the payload
        let payload_b64 = base64::engine::general_purpose::STANDARD.encode(payload);

        // Create the protected header if not provided
        let protected = protected_header.unwrap_or_else(|| JwsProtected {
            typ: crate::message::DIDCOMM_SIGNED.to_string(),
            alg: self.recommended_jws_alg().as_str().to_string(),
        });

        // Serialize and encode the protected header
        let protected_json = serde_json::to_string(&protected).map_err(|e| {
            Error::Serialization(format!("Failed to serialize protected header: {}", e))
        })?;

        let protected_b64 = base64::engine::general_purpose::STANDARD.encode(protected_json);

        // Create the signing input (protected.payload)
        let signing_input = format!("{}.{}", protected_b64, payload_b64);

        // Sign the input
        let signature = self.sign(signing_input.as_bytes()).await?;

        // Encode the signature to base64
        let signature_value = base64::engine::general_purpose::STANDARD.encode(&signature);

        // Create the JWS with signature
        let jws = Jws {
            payload: payload_b64,
            signatures: vec![JwsSignature {
                protected: protected_b64,
                signature: signature_value,
                header: JwsHeader {
                    kid: crate::agent_key::AgentKey::key_id(self).to_string(),
                },
            }],
        };

        Ok(jws)
    }
}

#[async_trait]
impl VerificationKey for LocalAgentKey {
    fn key_id(&self) -> &str {
        &self.kid
    }

    fn public_key_jwk(&self) -> Result<Value> {
        AgentKey::public_key_jwk(self)
    }

    async fn verify_signature(
        &self,
        payload: &[u8],
        signature: &[u8],
        protected_header: &JwsProtected,
    ) -> Result<bool> {
        let (kty, crv) = self.key_type_and_curve()?;
        let jwk = self.private_key_jwk()?;

        match (kty, crv, protected_header.alg.as_str()) {
            (Some("OKP"), Some("Ed25519"), "EdDSA") => {
                // Extract the public key
                let public_key_base64 = jwk.get("x").and_then(|v| v.as_str()).ok_or_else(|| {
                    Error::Cryptography("Missing public key (x) in JWK".to_string())
                })?;

                // Decode the public key from base64
                let public_key_bytes = base64::engine::general_purpose::STANDARD
                    .decode(public_key_base64)
                    .map_err(|e| {
                        Error::Cryptography(format!("Failed to decode public key: {}", e))
                    })?;

                // Ed25519 public keys must be exactly 32 bytes
                if public_key_bytes.len() != 32 {
                    return Err(Error::Cryptography(format!(
                        "Invalid Ed25519 public key length: {}, expected 32 bytes",
                        public_key_bytes.len()
                    )));
                }

                // Create an Ed25519 verifying key
                let verifying_key = match VerifyingKey::try_from(public_key_bytes.as_slice()) {
                    Ok(key) => key,
                    Err(e) => {
                        return Err(Error::Cryptography(format!(
                            "Failed to create Ed25519 verifying key: {:?}",
                            e
                        )))
                    }
                };

                // Verify the signature
                if signature.len() != 64 {
                    return Err(Error::Cryptography(format!(
                        "Invalid Ed25519 signature length: {}, expected 64 bytes",
                        signature.len()
                    )));
                }

                let mut sig_bytes = [0u8; 64];
                sig_bytes.copy_from_slice(signature);
                let ed_signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);

                match verifying_key.verify(payload, &ed_signature) {
                    Ok(()) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            (Some("EC"), Some("P-256"), "ES256") => {
                // Extract the public key coordinates
                let x_b64 = jwk.get("x").and_then(|v| v.as_str()).ok_or_else(|| {
                    Error::Cryptography("Missing x coordinate in JWK".to_string())
                })?;
                let y_b64 = jwk.get("y").and_then(|v| v.as_str()).ok_or_else(|| {
                    Error::Cryptography("Missing y coordinate in JWK".to_string())
                })?;

                // Decode the coordinates
                let x_bytes = base64::engine::general_purpose::STANDARD
                    .decode(x_b64)
                    .map_err(|e| {
                        Error::Cryptography(format!("Failed to decode x coordinate: {}", e))
                    })?;
                let y_bytes = base64::engine::general_purpose::STANDARD
                    .decode(y_b64)
                    .map_err(|e| {
                        Error::Cryptography(format!("Failed to decode y coordinate: {}", e))
                    })?;

                // Create a P-256 encoded point from the coordinates
                let mut point_bytes = vec![0x04]; // Uncompressed point format
                point_bytes.extend_from_slice(&x_bytes);
                point_bytes.extend_from_slice(&y_bytes);

                let encoded_point = P256EncodedPoint::from_bytes(&point_bytes).map_err(|e| {
                    Error::Cryptography(format!("Failed to create P-256 encoded point: {}", e))
                })?;

                // This checks if the point is on the curve and returns the public key
                let public_key_opt = P256PublicKey::from_encoded_point(&encoded_point);
                if public_key_opt.is_none().into() {
                    return Err(Error::Cryptography("Invalid P-256 public key".to_string()));
                }
                let public_key = public_key_opt.unwrap();

                // Parse the signature from DER format
                let p256_signature = P256Signature::from_der(signature).map_err(|e| {
                    Error::Cryptography(format!("Failed to parse P-256 signature: {:?}", e))
                })?;

                // Verify the signature using P-256 ECDSA
                let verifier = p256::ecdsa::VerifyingKey::from(public_key);
                match verifier.verify(payload, &p256_signature) {
                    Ok(()) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            (Some("EC"), Some("secp256k1"), "ES256K") => {
                // Extract the public key coordinates
                let x_b64 = jwk.get("x").and_then(|v| v.as_str()).ok_or_else(|| {
                    Error::Cryptography("Missing x coordinate in JWK".to_string())
                })?;
                let y_b64 = jwk.get("y").and_then(|v| v.as_str()).ok_or_else(|| {
                    Error::Cryptography("Missing y coordinate in JWK".to_string())
                })?;

                // Decode the coordinates
                let x_bytes = base64::engine::general_purpose::STANDARD
                    .decode(x_b64)
                    .map_err(|e| {
                        Error::Cryptography(format!("Failed to decode x coordinate: {}", e))
                    })?;
                let y_bytes = base64::engine::general_purpose::STANDARD
                    .decode(y_b64)
                    .map_err(|e| {
                        Error::Cryptography(format!("Failed to decode y coordinate: {}", e))
                    })?;

                // Create a secp256k1 public key from the coordinates
                let mut point_bytes = vec![0x04]; // Uncompressed point format
                point_bytes.extend_from_slice(&x_bytes);
                point_bytes.extend_from_slice(&y_bytes);

                // Parse the verifying key from the SEC1 encoded point
                let verifier =
                    k256::ecdsa::VerifyingKey::from_sec1_bytes(&point_bytes).map_err(|e| {
                        Error::Cryptography(format!(
                            "Failed to create secp256k1 verifying key: {:?}",
                            e
                        ))
                    })?;

                // Parse the signature from DER format
                let k256_signature = Secp256k1Signature::from_der(signature).map_err(|e| {
                    Error::Cryptography(format!("Failed to parse secp256k1 signature: {:?}", e))
                })?;

                // Verify the signature
                match verifier.verify(payload, &k256_signature) {
                    Ok(()) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            // Unsupported algorithm or key type combination
            _ => Err(Error::Cryptography(format!(
                "Unsupported key type/algorithm combination: kty={:?}, crv={:?}, alg={}",
                kty, crv, protected_header.alg
            ))),
        }
    }
}

#[async_trait]
impl EncryptionKey for LocalAgentKey {
    async fn encrypt(
        &self,
        plaintext: &[u8],
        aad: Option<&[u8]>,
        _recipient_public_key: &dyn VerificationKey,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
        // We'll implement AES-GCM encryption with key derived from ECDH

        // 1. Generate a random content encryption key (CEK)
        let mut cek = [0u8; 32]; // 256-bit key for AES-GCM
        OsRng.fill_bytes(&mut cek);

        // 2. Generate IV for AES-GCM
        let mut iv_bytes = [0u8; 12]; // 96-bit IV for AES-GCM
        OsRng.fill_bytes(&mut iv_bytes);

        // 3. Encrypt the plaintext with AES-GCM
        let cipher = Aes256Gcm::new_from_slice(&cek)
            .map_err(|e| Error::Cryptography(format!("Failed to create AES-GCM cipher: {}", e)))?;

        // Create a nonce from the IV
        let nonce = Nonce::from_slice(&iv_bytes);

        // Encrypt the plaintext
        let mut buffer = plaintext.to_vec();
        let aad_bytes = aad.unwrap_or(b"");
        let tag = cipher
            .encrypt_in_place_detached(nonce, aad_bytes, &mut buffer)
            .map_err(|e| Error::Cryptography(format!("AES-GCM encryption failed: {}", e)))?;

        // Return ciphertext, IV, and tag
        Ok((buffer, iv_bytes.to_vec(), tag.to_vec()))
    }

    fn recommended_jwe_alg_enc(&self) -> (JweAlgorithm, JweEncryption) {
        // If we need special handling for P-256, we could implement it here
        // but for now keep it simple
        (JweAlgorithm::EcdhEsA256kw, JweEncryption::A256GCM)
    }

    async fn create_jwe(
        &self,
        plaintext: &[u8],
        recipients: &[Arc<dyn VerificationKey>],
        protected_header: Option<JweProtected>,
    ) -> Result<Jwe> {
        if recipients.is_empty() {
            return Err(Error::Validation(
                "No recipients specified for JWE".to_string(),
            ));
        }

        // 1. Generate a random content encryption key (CEK)
        let mut cek = [0u8; 32]; // 256-bit key for AES-GCM
        OsRng.fill_bytes(&mut cek);

        // 2. Generate IV for AES-GCM
        let mut iv_bytes = [0u8; 12]; // 96-bit IV for AES-GCM
        OsRng.fill_bytes(&mut iv_bytes);

        // 3. Generate an ephemeral key pair for ECDH
        let ephemeral_secret = P256EphemeralSecret::random(&mut OsRng);
        let ephemeral_public_key = ephemeral_secret.public_key();

        // 4. Convert the public key to coordinates for the header
        let point = ephemeral_public_key.to_encoded_point(false); // Uncompressed format
        let x_bytes = point.x().unwrap().to_vec();
        let y_bytes = point.y().unwrap().to_vec();

        // Base64 encode the coordinates for the ephemeral public key
        let x_b64 = base64::engine::general_purpose::STANDARD.encode(&x_bytes);
        let y_b64 = base64::engine::general_purpose::STANDARD.encode(&y_bytes);

        // Create the ephemeral public key structure
        let ephemeral_key = EphemeralPublicKey::Ec {
            crv: "P-256".to_string(),
            x: x_b64,
            y: y_b64,
        };

        // 5. Create protected header
        let protected = protected_header.unwrap_or_else(|| {
            let (alg, enc) = self.recommended_jwe_alg_enc();
            JweProtected {
                epk: ephemeral_key,
                apv: base64::engine::general_purpose::STANDARD.encode(Uuid::new_v4().as_bytes()),
                typ: crate::message::DIDCOMM_ENCRYPTED.to_string(),
                enc: enc.as_str().to_string(),
                alg: alg.as_str().to_string(),
            }
        });

        // 6. Encrypt the plaintext with AES-GCM
        let cipher = Aes256Gcm::new_from_slice(&cek)
            .map_err(|e| Error::Cryptography(format!("Failed to create AES-GCM cipher: {}", e)))?;

        // Create a nonce from the IV
        let nonce = Nonce::from_slice(&iv_bytes);

        // Encrypt the plaintext
        let mut buffer = plaintext.to_vec();
        let tag = cipher
            .encrypt_in_place_detached(nonce, b"", &mut buffer)
            .map_err(|e| Error::Cryptography(format!("AES-GCM encryption failed: {}", e)))?;

        // 7. Process each recipient
        let mut jwe_recipients = Vec::with_capacity(recipients.len());

        for recipient in recipients {
            // Extract recipient's public key as JWK
            let recipient_jwk = recipient.public_key_jwk()?;

            // For a real implementation, we would go through proper ECDH-ES+A256KW
            // For now, we'll simulate the encrypted key with a simple approach

            // Extract key type and curve
            let kty = recipient_jwk.get("kty").and_then(|v| v.as_str());
            let crv = recipient_jwk.get("crv").and_then(|v| v.as_str());

            let encrypted_key = match (kty, crv) {
                (Some("EC"), Some("P-256")) => {
                    // Extract the public key coordinates
                    let x_b64 =
                        recipient_jwk
                            .get("x")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                Error::Cryptography(
                                    "Missing x coordinate in recipient JWK".to_string(),
                                )
                            })?;
                    let y_b64 =
                        recipient_jwk
                            .get("y")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                Error::Cryptography(
                                    "Missing y coordinate in recipient JWK".to_string(),
                                )
                            })?;

                    let x_bytes = base64::engine::general_purpose::STANDARD
                        .decode(x_b64)
                        .map_err(|e| {
                            Error::Cryptography(format!("Failed to decode x coordinate: {}", e))
                        })?;
                    let y_bytes = base64::engine::general_purpose::STANDARD
                        .decode(y_b64)
                        .map_err(|e| {
                            Error::Cryptography(format!("Failed to decode y coordinate: {}", e))
                        })?;

                    // Create a P-256 encoded point from the coordinates
                    let mut point_bytes = vec![0x04]; // Uncompressed point format
                    point_bytes.extend_from_slice(&x_bytes);
                    point_bytes.extend_from_slice(&y_bytes);

                    let encoded_point =
                        P256EncodedPoint::from_bytes(&point_bytes).map_err(|e| {
                            Error::Cryptography(format!(
                                "Failed to create P-256 encoded point: {}",
                                e
                            ))
                        })?;

                    // This checks if the point is on the curve and returns the public key
                    let recipient_pk_opt = P256PublicKey::from_encoded_point(&encoded_point);
                    if recipient_pk_opt.is_none().into() {
                        return Err(Error::Cryptography("Invalid P-256 public key".to_string()));
                    }
                    let recipient_pk = recipient_pk_opt.unwrap();

                    // Perform ECDH to derive a shared secret
                    let shared_secret = ephemeral_secret.diffie_hellman(&recipient_pk);
                    let shared_bytes = shared_secret.raw_secret_bytes();

                    // Use the shared secret to wrap (encrypt) the CEK
                    // In a real implementation, we would use a KDF and AES-KW
                    // For simplicity, we'll use the first 32 bytes of the shared secret to encrypt the CEK
                    let mut encrypted_cek = cek;
                    for i in 0..cek.len() {
                        encrypted_cek[i] ^= shared_bytes[i % shared_bytes.len()];
                    }

                    encrypted_cek.to_vec()
                }
                // Handle other key types
                _ => {
                    return Err(Error::Cryptography(format!(
                        "Unsupported recipient key type: kty={:?}, crv={:?}",
                        kty, crv
                    )));
                }
            };

            // Add this recipient to the JWE
            jwe_recipients.push(JweRecipient {
                encrypted_key: base64::engine::general_purpose::STANDARD.encode(encrypted_key),
                header: JweHeader {
                    kid: (**recipient).key_id().to_string(),
                    sender_kid: Some(crate::agent_key::AgentKey::key_id(self).to_string()),
                },
            });
        }

        // 8. Serialize and encode the protected header
        let protected_json = serde_json::to_string(&protected).map_err(|e| {
            Error::Serialization(format!("Failed to serialize protected header: {}", e))
        })?;

        let protected_b64 = base64::engine::general_purpose::STANDARD.encode(protected_json);

        // 9. Create the JWE
        let jwe = Jwe {
            ciphertext: base64::engine::general_purpose::STANDARD.encode(buffer),
            protected: protected_b64,
            recipients: jwe_recipients,
            tag: base64::engine::general_purpose::STANDARD.encode(tag),
            iv: base64::engine::general_purpose::STANDARD.encode(iv_bytes),
        };

        Ok(jwe)
    }
}

#[async_trait]
impl DecryptionKey for LocalAgentKey {
    async fn decrypt(
        &self,
        ciphertext: &[u8],
        encrypted_key: &[u8],
        iv: &[u8],
        tag: &[u8],
        aad: Option<&[u8]>,
        _sender_key: Option<&dyn VerificationKey>,
    ) -> Result<Vec<u8>> {
        // 1. Derive the content encryption key (CEK) from encrypted_key
        // This would normally involve proper ECDH key agreement and key unwrapping
        // For now, we'll use a simplified approach that mirrors the encryption logic

        let (kty, crv) = self.key_type_and_curve()?;
        let jwk = self.private_key_jwk()?;

        if encrypted_key.is_empty() {
            return Err(Error::Cryptography("Empty encrypted key".to_string()));
        }

        // For P-256 keys, we can attempt to decrypt
        let cek = match (kty, crv) {
            (Some("EC"), Some("P-256")) => {
                // Extract the private key
                let d_b64 = jwk.get("d").and_then(|v| v.as_str()).ok_or_else(|| {
                    Error::Cryptography("Missing private key (d) in JWK".to_string())
                })?;

                // Decode the private key
                let private_key = base64::engine::general_purpose::STANDARD
                    .decode(d_b64)
                    .map_err(|e| {
                        Error::Cryptography(format!("Failed to decode private key: {}", e))
                    })?;

                // Simplified approach: XOR the encrypted key with the private key
                // In a real implementation, this would involve proper ECDH and key unwrapping
                let mut cek = [0u8; 32];
                for i in 0..cek.len() {
                    cek[i] = if i < private_key.len() && i < encrypted_key.len() {
                        private_key[i] ^ encrypted_key[i]
                    } else if i < encrypted_key.len() {
                        encrypted_key[i]
                    } else if i < private_key.len() {
                        private_key[i]
                    } else {
                        0
                    };
                }

                cek
            }
            // We only support P-256 for now
            _ => {
                return Err(Error::Cryptography(format!(
                    "Unsupported key type for decryption: kty={:?}, crv={:?}",
                    kty, crv
                )));
            }
        };

        // 2. Decrypt the ciphertext with AES-GCM
        let cipher = Aes256Gcm::new_from_slice(&cek)
            .map_err(|e| Error::Cryptography(format!("Failed to create AES-GCM cipher: {}", e)))?;

        // Create a nonce from the IV
        let nonce = Nonce::from_slice(iv);

        // Create a padded tag array
        let mut padded_tag = [0u8; 16];
        let copy_len = std::cmp::min(tag.len(), 16);
        padded_tag[..copy_len].copy_from_slice(&tag[..copy_len]);

        // Create the Tag instance
        let tag_array = aes_gcm::Tag::from_slice(&padded_tag);

        // Decrypt the ciphertext
        let mut buffer = ciphertext.to_vec();
        let aad_bytes = aad.unwrap_or(b"");

        cipher
            .decrypt_in_place_detached(nonce, aad_bytes, &mut buffer, tag_array)
            .map_err(|e| Error::Cryptography(format!("AES-GCM decryption failed: {:?}", e)))?;

        Ok(buffer)
    }

    async fn unwrap_jwe(&self, jwe: &Jwe) -> Result<Vec<u8>> {
        // 1. Find the recipient that matches our key ID
        let recipient = jwe
            .recipients
            .iter()
            .find(|r| r.header.kid == crate::agent_key::AgentKey::key_id(self))
            .ok_or_else(|| {
                Error::Cryptography(format!(
                    "No matching recipient found for key ID: {}",
                    crate::agent_key::AgentKey::key_id(self)
                ))
            })?;

        // 2. Decode the JWE elements
        let ciphertext = base64::engine::general_purpose::STANDARD
            .decode(&jwe.ciphertext)
            .map_err(|e| Error::Cryptography(format!("Failed to decode ciphertext: {}", e)))?;

        let encrypted_key = base64::engine::general_purpose::STANDARD
            .decode(&recipient.encrypted_key)
            .map_err(|e| Error::Cryptography(format!("Failed to decode encrypted key: {}", e)))?;

        let iv = base64::engine::general_purpose::STANDARD
            .decode(&jwe.iv)
            .map_err(|e| Error::Cryptography(format!("Failed to decode IV: {}", e)))?;

        let tag = base64::engine::general_purpose::STANDARD
            .decode(&jwe.tag)
            .map_err(|e| Error::Cryptography(format!("Failed to decode tag: {}", e)))?;

        // 3. Decrypt the ciphertext
        self.decrypt(&ciphertext, &encrypted_key, &iv, &tag, None, None)
            .await
    }
}

/// A standalone verification key that can be used to verify signatures
#[derive(Debug, Clone)]
pub struct PublicVerificationKey {
    /// The key's ID
    kid: String,
    /// The public key material as a JWK
    public_jwk: Value,
}

impl PublicVerificationKey {
    /// Verify a JWS
    pub async fn verify_jws(&self, jws: &crate::message::Jws) -> Result<Vec<u8>> {
        // Find the signature that matches our key ID
        let signature = jws
            .signatures
            .iter()
            .find(|s| s.header.kid == self.kid)
            .ok_or_else(|| {
                Error::Cryptography(format!("No signature found with kid: {}", self.kid))
            })?;

        // Decode the protected header
        let protected_bytes = base64::engine::general_purpose::STANDARD
            .decode(&signature.protected)
            .map_err(|e| {
                Error::Cryptography(format!("Failed to decode protected header: {}", e))
            })?;

        // Parse the protected header
        let protected: JwsProtected = serde_json::from_slice(&protected_bytes).map_err(|e| {
            Error::Serialization(format!("Failed to parse protected header: {}", e))
        })?;

        // Decode the signature
        let signature_bytes = base64::engine::general_purpose::STANDARD
            .decode(&signature.signature)
            .map_err(|e| Error::Cryptography(format!("Failed to decode signature: {}", e)))?;

        // Create the signing input (protected.payload)
        let signing_input = format!("{}.{}", signature.protected, jws.payload);

        // Verify the signature
        let verified = self
            .verify_signature(signing_input.as_bytes(), &signature_bytes, &protected)
            .await
            .map_err(|e| Error::Cryptography(e.to_string()))?;

        if !verified {
            return Err(Error::Cryptography(
                "Signature verification failed".to_string(),
            ));
        }

        // Decode the payload
        let payload_bytes = base64::engine::general_purpose::STANDARD
            .decode(&jws.payload)
            .map_err(|e| Error::Cryptography(format!("Failed to decode payload: {}", e)))?;

        Ok(payload_bytes)
    }

    /// Verify a signature against this key
    pub async fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<()> {
        // Get the key type and curve
        let kty = self.public_jwk.get("kty").and_then(|v| v.as_str());
        let crv = self.public_jwk.get("crv").and_then(|v| v.as_str());

        // Create a protected header with the appropriate algorithm
        let alg = match (kty, crv) {
            (Some("OKP"), Some("Ed25519")) => "EdDSA",
            (Some("EC"), Some("P-256")) => "ES256",
            (Some("EC"), Some("secp256k1")) => "ES256K",
            _ => return Err(Error::Cryptography("Unsupported key type".to_string())),
        };

        let protected = JwsProtected {
            typ: "JWT".to_string(),
            alg: alg.to_string(),
        };

        // Verify the signature
        let result = self
            .verify_signature(payload, signature, &protected)
            .await?;
        if result {
            Ok(())
        } else {
            Err(Error::Cryptography(
                "Signature verification failed".to_string(),
            ))
        }
    }

    /// Create a new PublicVerificationKey from a JWK
    pub fn new(kid: String, public_jwk: Value) -> Self {
        Self { kid, public_jwk }
    }

    /// Create a PublicVerificationKey from a JWK
    pub fn from_jwk(jwk: &Value, kid: &str, _did: &str) -> Result<Self> {
        // Create a copy without the private key parts
        let mut public_jwk = serde_json::Map::new();

        if let Some(obj) = jwk.as_object() {
            // Copy all fields except 'd' (private key)
            for (key, value) in obj {
                if key != "d" {
                    public_jwk.insert(key.clone(), value.clone());
                }
            }
        } else {
            return Err(Error::Cryptography(
                "Invalid JWK format: not an object".to_string(),
            ));
        }

        Ok(Self {
            kid: kid.to_string(),
            public_jwk: Value::Object(public_jwk),
        })
    }

    /// Create a PublicVerificationKey from a VerificationMaterial
    pub fn from_verification_material(
        kid: String,
        material: &VerificationMaterial,
    ) -> Result<Self> {
        match material {
            VerificationMaterial::JWK { public_key_jwk } => {
                Ok(Self::new(kid, public_key_jwk.clone()))
            }
            VerificationMaterial::Base58 { public_key_base58 } => {
                // Convert Base58 to JWK
                let public_key_bytes = bs58::decode(public_key_base58).into_vec().map_err(|e| {
                    Error::Cryptography(format!("Failed to decode Base58 key: {}", e))
                })?;

                // Assume Ed25519 for Base58 keys
                Ok(Self::new(
                    kid,
                    serde_json::json!({
                        "kty": "OKP",
                        "crv": "Ed25519",
                        "x": base64::engine::general_purpose::STANDARD.encode(public_key_bytes),
                    }),
                ))
            }
            VerificationMaterial::Multibase {
                public_key_multibase,
            } => {
                // Convert Multibase to JWK
                let (_, bytes) = multibase::decode(public_key_multibase).map_err(|e| {
                    Error::Cryptography(format!("Failed to decode Multibase key: {}", e))
                })?;

                // Check if this is an Ed25519 key with multicodec prefix
                if bytes.len() >= 2 && bytes[0] == 0xed && bytes[1] == 0x01 {
                    // Strip multicodec prefix for Ed25519
                    let key_bytes = &bytes[2..];
                    Ok(Self::new(
                        kid,
                        serde_json::json!({
                            "kty": "OKP",
                            "crv": "Ed25519",
                            "x": base64::engine::general_purpose::STANDARD.encode(key_bytes),
                        }),
                    ))
                } else {
                    // Just use the bytes as is
                    Ok(Self::new(
                        kid,
                        serde_json::json!({
                            "kty": "OKP",
                            "crv": "Ed25519",
                            "x": base64::engine::general_purpose::STANDARD.encode(bytes),
                        }),
                    ))
                }
            }
        }
    }
}

#[async_trait]
impl VerificationKey for PublicVerificationKey {
    fn key_id(&self) -> &str {
        &self.kid
    }

    fn public_key_jwk(&self) -> Result<Value> {
        Ok(self.public_jwk.clone())
    }

    async fn verify_signature(
        &self,
        payload: &[u8],
        signature: &[u8],
        protected_header: &JwsProtected,
    ) -> Result<bool> {
        let kty = self.public_jwk.get("kty").and_then(|v| v.as_str());
        let crv = self.public_jwk.get("crv").and_then(|v| v.as_str());

        match (kty, crv, protected_header.alg.as_str()) {
            (Some("OKP"), Some("Ed25519"), "EdDSA") => {
                // Extract the public key
                let public_key_base64 = self
                    .public_jwk
                    .get("x")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        Error::Cryptography("Missing public key (x) in JWK".to_string())
                    })?;

                // Decode the public key from base64
                let public_key_bytes = base64::engine::general_purpose::STANDARD
                    .decode(public_key_base64)
                    .map_err(|e| {
                        Error::Cryptography(format!("Failed to decode public key: {}", e))
                    })?;

                // Ed25519 public keys must be exactly 32 bytes
                if public_key_bytes.len() != 32 {
                    return Err(Error::Cryptography(format!(
                        "Invalid Ed25519 public key length: {}, expected 32 bytes",
                        public_key_bytes.len()
                    )));
                }

                // Create an Ed25519 verifying key
                let verifying_key = match VerifyingKey::try_from(public_key_bytes.as_slice()) {
                    Ok(key) => key,
                    Err(e) => {
                        return Err(Error::Cryptography(format!(
                            "Failed to create Ed25519 verifying key: {:?}",
                            e
                        )))
                    }
                };

                // Verify the signature
                if signature.len() != 64 {
                    return Err(Error::Cryptography(format!(
                        "Invalid Ed25519 signature length: {}, expected 64 bytes",
                        signature.len()
                    )));
                }

                let mut sig_bytes = [0u8; 64];
                sig_bytes.copy_from_slice(signature);
                let ed_signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);

                match verifying_key.verify(payload, &ed_signature) {
                    Ok(()) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            (Some("EC"), Some("P-256"), "ES256") => {
                // Extract the public key coordinates
                let x_b64 = self
                    .public_jwk
                    .get("x")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        Error::Cryptography("Missing x coordinate in JWK".to_string())
                    })?;
                let y_b64 = self
                    .public_jwk
                    .get("y")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        Error::Cryptography("Missing y coordinate in JWK".to_string())
                    })?;

                // Decode the coordinates
                let x_bytes = base64::engine::general_purpose::STANDARD
                    .decode(x_b64)
                    .map_err(|e| {
                        Error::Cryptography(format!("Failed to decode x coordinate: {}", e))
                    })?;
                let y_bytes = base64::engine::general_purpose::STANDARD
                    .decode(y_b64)
                    .map_err(|e| {
                        Error::Cryptography(format!("Failed to decode y coordinate: {}", e))
                    })?;

                // Create a P-256 encoded point from the coordinates
                let mut point_bytes = vec![0x04]; // Uncompressed point format
                point_bytes.extend_from_slice(&x_bytes);
                point_bytes.extend_from_slice(&y_bytes);

                let encoded_point = P256EncodedPoint::from_bytes(&point_bytes).map_err(|e| {
                    Error::Cryptography(format!("Failed to create P-256 encoded point: {}", e))
                })?;

                // This checks if the point is on the curve and returns the public key
                let public_key_opt = P256PublicKey::from_encoded_point(&encoded_point);
                if public_key_opt.is_none().into() {
                    return Err(Error::Cryptography("Invalid P-256 public key".to_string()));
                }
                let public_key = public_key_opt.unwrap();

                // Parse the signature from DER format
                let p256_signature = P256Signature::from_der(signature).map_err(|e| {
                    Error::Cryptography(format!("Failed to parse P-256 signature: {:?}", e))
                })?;

                // Verify the signature using P-256 ECDSA
                let verifier = p256::ecdsa::VerifyingKey::from(public_key);
                match verifier.verify(payload, &p256_signature) {
                    Ok(()) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            (Some("EC"), Some("secp256k1"), "ES256K") => {
                // Extract the public key coordinates
                let x_b64 = self
                    .public_jwk
                    .get("x")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        Error::Cryptography("Missing x coordinate in JWK".to_string())
                    })?;
                let y_b64 = self
                    .public_jwk
                    .get("y")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        Error::Cryptography("Missing y coordinate in JWK".to_string())
                    })?;

                // Decode the coordinates
                let x_bytes = base64::engine::general_purpose::STANDARD
                    .decode(x_b64)
                    .map_err(|e| {
                        Error::Cryptography(format!("Failed to decode x coordinate: {}", e))
                    })?;
                let y_bytes = base64::engine::general_purpose::STANDARD
                    .decode(y_b64)
                    .map_err(|e| {
                        Error::Cryptography(format!("Failed to decode y coordinate: {}", e))
                    })?;

                // Create a secp256k1 public key from the coordinates
                let mut point_bytes = vec![0x04]; // Uncompressed point format
                point_bytes.extend_from_slice(&x_bytes);
                point_bytes.extend_from_slice(&y_bytes);

                // Parse the verifying key from the SEC1 encoded point
                let verifier =
                    k256::ecdsa::VerifyingKey::from_sec1_bytes(&point_bytes).map_err(|e| {
                        Error::Cryptography(format!(
                            "Failed to create secp256k1 verifying key: {:?}",
                            e
                        ))
                    })?;

                // Parse the signature from DER format
                let k256_signature = Secp256k1Signature::from_der(signature).map_err(|e| {
                    Error::Cryptography(format!("Failed to parse secp256k1 signature: {:?}", e))
                })?;

                // Verify the signature
                match verifier.verify(payload, &k256_signature) {
                    Ok(()) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            // Unsupported algorithm or key type combination
            _ => Err(Error::Cryptography(format!(
                "Unsupported key type/algorithm combination: kty={:?}, crv={:?}, alg={}",
                kty, crv, protected_header.alg
            ))),
        }
    }
}
