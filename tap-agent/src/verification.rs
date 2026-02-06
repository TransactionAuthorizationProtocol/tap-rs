#![cfg(not(target_arch = "wasm32"))]
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
#[cfg(not(target_arch = "wasm32"))]
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
#[cfg(feature = "crypto-ed25519")]
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
#[cfg(feature = "crypto-p256")]
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
#[cfg(feature = "crypto-secp256k1")]
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

#[cfg(all(not(feature = "crypto-p256"), not(feature = "crypto-secp256k1")))]
fn verify_es256(
    _verification_method: &crate::did::VerificationMethod,
    _signing_input: &str,
    _signature: &[u8],
) -> bool {
    false
}

#[cfg(all(not(feature = "crypto-p256"), not(feature = "crypto-secp256k1")))]
fn verify_es256k(
    _verification_method: &crate::did::VerificationMethod,
    _signing_input: &str,
    _signature: &[u8],
) -> bool {
    false
}
