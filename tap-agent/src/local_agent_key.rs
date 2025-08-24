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
    EphemeralPublicKey, Jwe, JweHeader, JweProtected, JweRecipient, Jws, JwsProtected, JwsSignature,
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
        // Find the signature that matches our key ID
        let our_kid = crate::agent_key::AgentKey::key_id(self).to_string();
        let signature = jws
            .signatures
            .iter()
            .find(|s| s.get_kid() == Some(our_kid.clone()))
            .ok_or_else(|| {
                Error::Cryptography(format!("No signature found with kid: {}", our_kid))
            })?;

        // Parse the protected header to get the algorithm
        let protected = signature.get_protected_header().map_err(|e| {
            Error::Cryptography(format!("Failed to decode protected header: {}", e))
        })?;

        // Decode the signature from base64
        let signature_bytes = base64::engine::general_purpose::STANDARD
            .decode(&signature.signature)
            .map_err(|e| Error::Cryptography(format!("Failed to decode signature: {}", e)))?;

        // Construct the signing input: {protected}.{payload}
        let signing_input = format!("{}.{}", signature.protected, jws.payload);

        // Verify the signature using the actual cryptographic verification
        let verified = self
            .verify_signature(signing_input.as_bytes(), &signature_bytes, &protected)
            .await?;

        if !verified {
            return Err(Error::Cryptography(
                "Signature verification failed".to_string(),
            ));
        }

        // Decode and return the payload
        let payload_bytes = base64::engine::general_purpose::STANDARD
            .decode(&jws.payload)
            .map_err(|e| Error::Cryptography(format!("Failed to decode payload: {}", e)))?;

        Ok(payload_bytes)
    }

    /// Unwrap a JWE to retrieve the plaintext using proper ECDH-ES+A256KW decryption
    pub async fn decrypt_jwe(&self, jwe: &crate::message::Jwe) -> Result<Vec<u8>> {
        // Use the proper JWE unwrapping implementation from DecryptionKey trait
        DecryptionKey::unwrap_jwe(self, jwe).await
    }

    /// Verify a signature against this key
    pub async fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<()> {
        // Create a protected header with the appropriate algorithm
        let protected = JwsProtected {
            typ: "JWT".to_string(),
            alg: self.recommended_jws_alg().as_str().to_string(),
            kid: crate::agent_key::AgentKey::key_id(self).to_string(),
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

    /// Encrypt data to a JWK recipient using proper ECDH-ES+A256KW
    pub async fn encrypt_to_jwk(
        &self,
        plaintext: &[u8],
        recipient_jwk: &Value,
        protected_header: Option<JweProtected>,
    ) -> Result<Jwe> {
        use p256::elliptic_curve::sec1::FromEncodedPoint;
        use p256::{EncodedPoint as P256EncodedPoint, PublicKey as P256PublicKey};

        // 1. Generate random CEK (Content Encryption Key)
        let mut cek = [0u8; 32];
        OsRng.fill_bytes(&mut cek);

        // 2. Generate random IV for AES-GCM
        let mut iv_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut iv_bytes);

        // 3. Generate ephemeral key pair for ECDH
        let ephemeral_secret = P256EphemeralSecret::random(&mut OsRng);
        let ephemeral_public_key = ephemeral_secret.public_key();

        // Convert ephemeral public key to JWK coordinates
        let point = ephemeral_public_key.to_encoded_point(false);
        let x_bytes = point.x().unwrap().to_vec();
        let y_bytes = point.y().unwrap().to_vec();
        let x_b64 = base64::engine::general_purpose::STANDARD.encode(&x_bytes);
        let y_b64 = base64::engine::general_purpose::STANDARD.encode(&y_bytes);

        let ephemeral_key = crate::message::EphemeralPublicKey::Ec {
            crv: "P-256".to_string(),
            x: x_b64,
            y: y_b64,
        };

        // 4. Extract recipient public key from JWK
        let recipient_x_b64 = recipient_jwk
            .get("x")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Error::Cryptography("Missing x coordinate in recipient JWK".to_string())
            })?;
        let recipient_y_b64 = recipient_jwk
            .get("y")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Error::Cryptography("Missing y coordinate in recipient JWK".to_string())
            })?;

        let recipient_x = base64::engine::general_purpose::STANDARD
            .decode(recipient_x_b64)
            .map_err(|e| Error::Cryptography(format!("Failed to decode x: {}", e)))?;
        let recipient_y = base64::engine::general_purpose::STANDARD
            .decode(recipient_y_b64)
            .map_err(|e| Error::Cryptography(format!("Failed to decode y: {}", e)))?;

        // Reconstruct recipient public key
        let mut recipient_point_bytes = vec![0x04]; // Uncompressed
        recipient_point_bytes.extend_from_slice(&recipient_x);
        recipient_point_bytes.extend_from_slice(&recipient_y);

        let recipient_encoded_point = P256EncodedPoint::from_bytes(&recipient_point_bytes)
            .map_err(|e| Error::Cryptography(format!("Invalid recipient point: {}", e)))?;

        let recipient_pk = P256PublicKey::from_encoded_point(&recipient_encoded_point);
        if recipient_pk.is_none().into() {
            return Err(Error::Cryptography(
                "Invalid recipient public key".to_string(),
            ));
        }
        let recipient_pk = recipient_pk.unwrap();

        // 5. Perform ECDH
        let shared_secret = ephemeral_secret.diffie_hellman(&recipient_pk);
        let shared_bytes = shared_secret.raw_secret_bytes();

        // 6. Create protected header
        let apv_raw = Uuid::new_v4();
        let apv_b64 = base64::engine::general_purpose::STANDARD.encode(apv_raw.as_bytes());

        let protected = protected_header.unwrap_or_else(|| crate::message::JweProtected {
            epk: ephemeral_key,
            apv: apv_b64.clone(),
            typ: crate::message::DIDCOMM_ENCRYPTED.to_string(),
            enc: "A256GCM".to_string(),
            alg: "ECDH-ES+A256KW".to_string(),
        });

        // 7. Derive KEK using Concat KDF
        let apv_bytes = base64::engine::general_purpose::STANDARD
            .decode(&protected.apv)
            .unwrap_or_default();
        let kek = crate::crypto::derive_key_ecdh_es(shared_bytes.as_slice(), b"", &apv_bytes, 256)?;

        // 8. Wrap CEK with AES-KW
        let mut kek_array = [0u8; 32];
        kek_array.copy_from_slice(&kek);
        let wrapped_cek = crate::crypto::wrap_key_aes_kw(&kek_array, &cek)?;

        // 9. Encrypt plaintext with AES-GCM
        let cipher = Aes256Gcm::new_from_slice(&cek)
            .map_err(|e| Error::Cryptography(format!("Failed to create cipher: {}", e)))?;

        let nonce = Nonce::from_slice(&iv_bytes);
        let mut buffer = plaintext.to_vec();
        let tag = cipher
            .encrypt_in_place_detached(nonce, b"", &mut buffer)
            .map_err(|e| Error::Cryptography(format!("Encryption failed: {:?}", e)))?;

        // 10. Serialize protected header
        let protected_json = serde_json::to_string(&protected).map_err(|e| {
            Error::Serialization(format!("Failed to serialize protected header: {}", e))
        })?;
        let protected_b64 = base64::engine::general_purpose::STANDARD.encode(protected_json);

        // Get recipient kid from JWK if available
        let recipient_kid = recipient_jwk
            .get("kid")
            .and_then(|v| v.as_str())
            .unwrap_or("recipient-key")
            .to_string();

        // 11. Build JWE
        let jwe = crate::message::Jwe {
            ciphertext: base64::engine::general_purpose::STANDARD.encode(&buffer),
            protected: protected_b64,
            recipients: vec![crate::message::JweRecipient {
                encrypted_key: base64::engine::general_purpose::STANDARD.encode(&wrapped_cek),
                header: crate::message::JweHeader {
                    kid: recipient_kid,
                    sender_kid: Some(AgentKey::key_id(self).to_string()),
                },
            }],
            tag: base64::engine::general_purpose::STANDARD.encode(tag),
            iv: base64::engine::general_purpose::STANDARD.encode(iv_bytes),
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
                    // Generate default key ID based on DID method
                    if did.starts_with("did:key:") {
                        // For did:key, extract the multibase key from the DID and use it as fragment
                        // did:key:z6Mk... -> did:key:z6Mk...#z6Mk...
                        let key_part = &did[8..]; // Skip "did:key:"
                        format!("{}#{}", did, key_part)
                    } else if did.starts_with("did:web:") {
                        format!("{}#keys-1", did)
                    } else {
                        format!("{}#key-1", did)
                    }
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
    pub fn generate_ed25519(_kid: &str) -> Result<Self> {
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
        
        // Generate the proper verification method ID (did:key:z...#z...)
        let kid = format!("{}#{}", did, multibase_encoded);

        // Create the secret
        let secret = Secret {
            id: did.clone(),
            type_: crate::key_manager::SecretType::JsonWebKey2020,
            secret_material: SecretMaterial::JWK {
                private_key_jwk: serde_json::json!({
                    "kty": "OKP",
                    "kid": kid.clone(),
                    "crv": "Ed25519",
                    "x": base64::engine::general_purpose::STANDARD.encode(public_key),
                    "d": base64::engine::general_purpose::STANDARD.encode(private_key)
                }),
            },
        };

        Ok(Self {
            kid,
            did,
            secret,
            key_type: KeyType::Ed25519,
        })
    }

    /// Generate a new P-256 key with the given key ID
    pub fn generate_p256(_kid: &str) -> Result<Self> {
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
        
        // Generate the proper verification method ID (did:key:z...#z...)
        let kid = format!("{}#{}", did, multibase_encoded);

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
                    "kid": kid.clone(),
                    "crv": "P-256",
                    "x": base64::engine::general_purpose::STANDARD.encode(x),
                    "y": base64::engine::general_purpose::STANDARD.encode(y),
                    "d": base64::engine::general_purpose::STANDARD.encode(&private_key)
                }),
            },
        };

        Ok(Self {
            kid,
            did,
            secret,
            key_type: KeyType::P256,
        })
    }

    /// Generate a new secp256k1 key with the given key ID
    pub fn generate_secp256k1(_kid: &str) -> Result<Self> {
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
        
        // Generate the proper verification method ID (did:key:z...#z...)
        let kid = format!("{}#{}", did, multibase_encoded);

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
                    "kid": kid.clone(),
                    "crv": "secp256k1",
                    "x": base64::engine::general_purpose::STANDARD.encode(x),
                    "y": base64::engine::general_purpose::STANDARD.encode(y),
                    "d": base64::engine::general_purpose::STANDARD.encode(&private_key)
                }),
            },
        };

        Ok(Self {
            kid,
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

                // Convert to raw bytes for JWS (R||S format, 64 bytes total)
                let signature_bytes = signature.to_bytes();
                Ok(signature_bytes.to_vec())
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

                // Convert to raw bytes for JWS (R||S format)
                let signature_bytes = signature.to_bytes();
                Ok(signature_bytes.to_vec())
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

        // Create the protected header if not provided, respecting the key type
        let protected = if let Some(mut header) = protected_header {
            // Override the algorithm to match the key type
            header.alg = self.recommended_jws_alg().as_str().to_string();
            // Only set kid if not already provided in the header
            if header.kid.is_empty() {
                header.kid = crate::agent_key::AgentKey::key_id(self).to_string();
            }
            header
        } else {
            JwsProtected {
                typ: crate::message::DIDCOMM_SIGNED.to_string(),
                alg: self.recommended_jws_alg().as_str().to_string(),
                kid: crate::agent_key::AgentKey::key_id(self).to_string(),
            }
        };

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

                // Parse the signature from raw bytes (R||S format)
                let p256_signature = P256Signature::from_slice(signature).map_err(|e| {
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

                // Parse the signature from raw bytes (R||S format)
                let k256_signature = Secp256k1Signature::from_slice(signature).map_err(|e| {
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

                    // Derive KEK using Concat KDF per RFC 7518
                    // Use APV from protected header for consistency with decryption
                    let apv_bytes = base64::engine::general_purpose::STANDARD
                        .decode(&protected.apv)
                        .unwrap_or_default();
                    let kek = crate::crypto::derive_key_ecdh_es(
                        shared_bytes.as_slice(),
                        b"", // apu - empty for anonymous sender
                        &apv_bytes,
                        256, // 256 bits for AES-256-KW
                    )?;

                    // Wrap CEK with AES-KW per RFC 3394
                    let mut kek_array = [0u8; 32];
                    kek_array.copy_from_slice(&kek);

                    crate::crypto::wrap_key_aes_kw(&kek_array, &cek)?
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
        use p256::elliptic_curve::sec1::FromEncodedPoint;
        use p256::{EncodedPoint as P256EncodedPoint, PublicKey as P256PublicKey};

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

        // 2. Decode and parse the protected header to get the EPK
        let protected_bytes = base64::engine::general_purpose::STANDARD
            .decode(&jwe.protected)
            .map_err(|e| {
                Error::Cryptography(format!("Failed to decode protected header: {}", e))
            })?;

        let protected: JweProtected = serde_json::from_slice(&protected_bytes)
            .map_err(|e| Error::Cryptography(format!("Failed to parse protected header: {}", e)))?;

        // 3. Decode the JWE elements
        let ciphertext = base64::engine::general_purpose::STANDARD
            .decode(&jwe.ciphertext)
            .map_err(|e| Error::Cryptography(format!("Failed to decode ciphertext: {}", e)))?;

        let wrapped_cek = base64::engine::general_purpose::STANDARD
            .decode(&recipient.encrypted_key)
            .map_err(|e| Error::Cryptography(format!("Failed to decode encrypted key: {}", e)))?;

        let iv = base64::engine::general_purpose::STANDARD
            .decode(&jwe.iv)
            .map_err(|e| Error::Cryptography(format!("Failed to decode IV: {}", e)))?;

        let tag = base64::engine::general_purpose::STANDARD
            .decode(&jwe.tag)
            .map_err(|e| Error::Cryptography(format!("Failed to decode tag: {}", e)))?;

        // 4. Extract EPK coordinates and reconstruct the public key
        let (epk_x_b64, epk_y_b64) = match &protected.epk {
            EphemeralPublicKey::Ec { x, y, .. } => (x.clone(), y.clone()),
            _ => {
                return Err(Error::Cryptography(
                    "Unsupported EPK type for P-256 decryption".to_string(),
                ))
            }
        };

        let epk_x = base64::engine::general_purpose::STANDARD
            .decode(&epk_x_b64)
            .map_err(|e| Error::Cryptography(format!("Failed to decode EPK x: {}", e)))?;
        let epk_y = base64::engine::general_purpose::STANDARD
            .decode(&epk_y_b64)
            .map_err(|e| Error::Cryptography(format!("Failed to decode EPK y: {}", e)))?;

        // Reconstruct the EPK as an uncompressed point
        let mut epk_point_bytes = vec![0x04]; // Uncompressed point format
        epk_point_bytes.extend_from_slice(&epk_x);
        epk_point_bytes.extend_from_slice(&epk_y);

        let epk_encoded_point = P256EncodedPoint::from_bytes(&epk_point_bytes)
            .map_err(|e| Error::Cryptography(format!("Invalid EPK point: {}", e)))?;

        let epk_public_key = P256PublicKey::from_encoded_point(&epk_encoded_point);
        if epk_public_key.is_none().into() {
            return Err(Error::Cryptography("Invalid EPK public key".to_string()));
        }
        let epk_public_key = epk_public_key.unwrap();

        // 5. Get our private key and perform ECDH
        let jwk = self.private_key_jwk()?;
        let d_b64 = jwk
            .get("d")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Cryptography("Missing private key (d) in JWK".to_string()))?;

        let d_bytes = base64::engine::general_purpose::STANDARD
            .decode(d_b64)
            .map_err(|e| Error::Cryptography(format!("Failed to decode private key: {}", e)))?;

        // Create our secret key from the d value
        let secret_key = p256::SecretKey::from_bytes((&d_bytes[..]).into())
            .map_err(|e| Error::Cryptography(format!("Invalid private key: {}", e)))?;

        // Perform ECDH
        let shared_secret =
            p256::ecdh::diffie_hellman(secret_key.to_nonzero_scalar(), epk_public_key.as_affine());
        let shared_bytes = shared_secret.raw_secret_bytes();

        // 6. Derive KEK using Concat KDF
        let apv = base64::engine::general_purpose::STANDARD
            .decode(&protected.apv)
            .unwrap_or_default();

        let kek = crate::crypto::derive_key_ecdh_es(
            shared_bytes.as_slice(),
            b"", // apu - empty for anonymous sender
            &apv,
            256,
        )?;

        // 7. Unwrap CEK using AES-KW
        let mut kek_array = [0u8; 32];
        kek_array.copy_from_slice(&kek);
        let cek = crate::crypto::unwrap_key_aes_kw(&kek_array, &wrapped_cek)?;

        // 8. Decrypt ciphertext with AES-GCM using the CEK
        let cipher = Aes256Gcm::new_from_slice(&cek)
            .map_err(|e| Error::Cryptography(format!("Failed to create AES-GCM cipher: {}", e)))?;

        let nonce = Nonce::from_slice(&iv);

        let mut padded_tag = [0u8; 16];
        let copy_len = std::cmp::min(tag.len(), 16);
        padded_tag[..copy_len].copy_from_slice(&tag[..copy_len]);
        let tag_array = aes_gcm::Tag::from_slice(&padded_tag);

        let mut buffer = ciphertext.to_vec();
        cipher
            .decrypt_in_place_detached(nonce, b"", &mut buffer, tag_array)
            .map_err(|e| Error::Cryptography(format!("AES-GCM decryption failed: {:?}", e)))?;

        Ok(buffer)
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
            .find(|s| s.get_kid().as_ref() == Some(&self.kid))
            .ok_or_else(|| {
                Error::Cryptography(format!("No signature found with kid: {}", self.kid))
            })?;

        // Get the protected header
        let protected = signature.get_protected_header().map_err(|e| {
            Error::Cryptography(format!("Failed to decode protected header: {}", e))
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
            kid: self.kid.clone(),
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
