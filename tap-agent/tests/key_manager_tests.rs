use tap_agent::{
    agent_key::{
        AgentKey, JwsAlgorithm, SigningKey,
    },
    error::Result,
    key_manager::{KeyManagerBuilder},
    KeyManager,
};

use std::sync::Arc;

#[tokio::test]
async fn test_key_manager_builder() -> Result<()> {
    // Create a key manager with an auto-generated Ed25519 key
    let key_manager = KeyManagerBuilder::new()
        .with_auto_generated_ed25519_key("test-key")?
        .build()?;

    // Verify that the key was created and stored
    let keys = key_manager.list_keys()?;
    assert_eq!(keys.len(), 1);

    // Get the key by ID
    let kid = "test-key";
    let key = key_manager.get_signing_key(kid).await?;
    
    // Verify the key properties
    assert_eq!(AgentKey::key_id(&*key), kid);
    assert!(key.did().starts_with("did:key:"));
    assert_eq!(key.recommended_jws_alg(), JwsAlgorithm::EdDSA);

    Ok(())
}

#[tokio::test]
async fn test_sign_and_verify() -> Result<()> {
    // Create a key manager with an auto-generated Ed25519 key
    let key_manager = KeyManagerBuilder::new()
        .with_auto_generated_ed25519_key("test-key")?
        .build()?;

    // Get the key for signing
    let kid = "test-key";
    
    // Sign a message
    let message = b"Hello, world!";
    let jws = key_manager.sign_jws(kid, message, None).await?;
    
    // Verify the signature
    let verified_payload = key_manager.verify_jws(&jws, Some(kid)).await?;
    assert_eq!(verified_payload, message);

    Ok(())
}

#[tokio::test]
async fn test_encrypt_and_decrypt() -> Result<()> {
    // Create a key manager with auto-generated keys for sender and recipient
    let sender_manager = KeyManagerBuilder::new()
        .with_auto_generated_ed25519_key("sender-key")?
        .build()?;
        
    let recipient_manager = KeyManagerBuilder::new()
        .with_auto_generated_ed25519_key("recipient-key")?
        .build()?;
    
    // Get the key IDs
    let sender_kid = "sender-key";
    let recipient_kid = "recipient-key";
    
    // Encrypt a message
    let plaintext = b"Secret message";
    let jwe = sender_manager
        .encrypt_jwe(sender_kid, recipient_kid, plaintext, None)
        .await?;
    
    // Decrypt the message
    let decrypted = recipient_manager
        .decrypt_jwe(&jwe, Some(recipient_kid))
        .await?;
        
    assert_eq!(decrypted, plaintext);
    
    Ok(())
}