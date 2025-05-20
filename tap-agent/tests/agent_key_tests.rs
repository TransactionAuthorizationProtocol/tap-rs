use tap_agent::{
    agent_key::{AgentKey, SigningKey, VerificationKey, EncryptionKey, DecryptionKey, JwsAlgorithm, JweAlgorithm, JweEncryption},
    local_agent_key::{LocalAgentKey, PublicVerificationKey},
    error::Result,
    KeyManager,
};
use serde_json::{Value, json};
use async_trait::async_trait;
use std::sync::Arc;

#[tokio::test]
async fn test_local_agent_key_creation() -> Result<()> {
    // Test creating keys for different types
    let ed25519_key = LocalAgentKey::generate_ed25519("test-ed25519")?;
    let p256_key = LocalAgentKey::generate_p256("test-p256")?;
    let secp256k1_key = LocalAgentKey::generate_secp256k1("test-secp256k1")?;
    
    // Verify correct key types
    assert_eq!(ed25519_key.key_type(), "Ed25519");
    assert_eq!(p256_key.key_type(), "P-256");
    assert_eq!(secp256k1_key.key_type(), "secp256k1");
    
    // Verify DID format
    assert!(ed25519_key.did().starts_with("did:key:"));
    assert!(p256_key.did().starts_with("did:key:"));
    assert!(secp256k1_key.did().starts_with("did:key:"));
    
    // Verify key IDs
    assert_eq!(ed25519_key.key_id(), "test-ed25519");
    assert_eq!(p256_key.key_id(), "test-p256");
    assert_eq!(secp256k1_key.key_id(), "test-secp256k1");
    
    Ok(())
}

#[tokio::test]
async fn test_sign_and_verify() -> Result<()> {
    // Test each key type
    let test_data = b"test message to sign";
    
    // Ed25519
    let ed25519_key = LocalAgentKey::generate_ed25519("test-ed25519")?;
    let signature = ed25519_key.sign(test_data).await?;
    assert!(ed25519_key.verify(test_data, &signature).await.is_ok());
    
    // Extract public verification key and test
    let public_jwk = ed25519_key.public_key_jwk()?;
    let verification_key = PublicVerificationKey::from_jwk(
        &public_jwk, 
        ed25519_key.key_id(),
        ed25519_key.did()
    )?;
    assert!(verification_key.verify(test_data, &signature).await.is_ok());
    
    // Test with corrupted signature
    let mut corrupted = signature.clone();
    if !corrupted.is_empty() {
        corrupted[0] = corrupted[0].wrapping_add(1);
        assert!(verification_key.verify(test_data, &corrupted).await.is_err());
    }
    
    // P-256
    let p256_key = LocalAgentKey::generate_p256("test-p256")?;
    let signature = p256_key.sign(test_data).await?;
    assert!(p256_key.verify(test_data, &signature).await.is_ok());
    
    // secp256k1
    let secp256k1_key = LocalAgentKey::generate_secp256k1("test-secp256k1")?;
    let signature = secp256k1_key.sign(test_data).await?;
    assert!(secp256k1_key.verify(test_data, &signature).await.is_ok());
    
    Ok(())
}

#[tokio::test]
async fn test_jws_creation_and_verification() -> Result<()> {
    let key = LocalAgentKey::generate_ed25519("test-jws-key")?;
    let payload = b"{\"hello\":\"world\"}";
    
    // Create JWS
    let jws = key.create_jws(payload, None).await?;
    
    // Verify JWS
    let verified = key.verify_jws(&jws).await?;
    assert_eq!(verified, payload);
    
    // Test with public verification key
    let public_jwk = key.public_key_jwk()?;
    let verification_key = PublicVerificationKey::from_jwk(
        &public_jwk, 
        key.key_id(),
        key.did()
    )?;
    let verified = verification_key.verify_jws(&jws).await?;
    assert_eq!(verified, payload);
    
    Ok(())
}

#[tokio::test]
async fn test_encrypt_and_decrypt() -> Result<()> {
    let key = LocalAgentKey::generate_p256("test-enc-key")?;
    let plaintext = b"secret message for encryption";
    
    // Create recipient JWK
    let recipient_jwk = key.public_key_jwk()?;
    
    // Encrypt
    let jwe = key.encrypt_to_jwk(plaintext, &recipient_jwk, None).await?;
    
    // Decrypt
    let decrypted = key.decrypt_jwe(&jwe).await?;
    assert_eq!(decrypted, plaintext);
    
    Ok(())
}

#[tokio::test]
async fn test_recommended_algorithms() -> Result<()> {
    // Ed25519 should recommend EdDSA
    let ed25519_key = LocalAgentKey::generate_ed25519("test-ed25519")?;
    assert_eq!(ed25519_key.recommended_jws_alg(), JwsAlgorithm::EdDSA);
    
    // P-256 should recommend ES256
    let p256_key = LocalAgentKey::generate_p256("test-p256")?;
    assert_eq!(p256_key.recommended_jws_alg(), JwsAlgorithm::ES256);
    
    // secp256k1 should recommend ES256K
    let secp256k1_key = LocalAgentKey::generate_secp256k1("test-secp256k1")?;
    assert_eq!(secp256k1_key.recommended_jws_alg(), JwsAlgorithm::ES256K);
    
    // For JWE, check P-256
    assert_eq!(p256_key.recommended_jwe_alg(), JweAlgorithm::ECDH_ES);
    assert_eq!(p256_key.recommended_jwe_enc(), JweEncryption::A256GCM);
    
    Ok(())
}

#[tokio::test]
async fn test_serialization() -> Result<()> {
    // Create a key
    let key = LocalAgentKey::generate_ed25519("test-serialization")?;
    
    // Serialize to JWK
    let jwk = key.to_jwk()?;
    assert!(jwk.contains_key("d")); // Private key should be present
    
    // Serialize public only
    let public_jwk = key.public_key_jwk()?;
    assert!(!public_jwk.contains_key("d")); // Private key should not be present
    
    Ok(())
}