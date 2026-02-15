use tap_agent::{
    agent_key::JwsAlgorithm, error::Result, key_manager::KeyManagerBuilder, KeyManager,
};

#[tokio::test]
async fn test_key_manager_builder() -> Result<()> {
    // Create a key manager with an auto-generated Ed25519 key
    let key_manager = KeyManagerBuilder::new()
        .with_auto_generated_ed25519_key("test-key")?
        .build()?;

    // Verify that the key was created and stored
    let keys = key_manager.list_keys()?;
    assert_eq!(keys.len(), 1);

    // Get the actual kid (generate_ed25519 creates did:key:z...#z..., ignoring the label)
    let did = &keys[0];
    let key_part = did.strip_prefix("did:key:").unwrap();
    let kid = format!("{}#{}", did, key_part);
    let key = key_manager.get_signing_key(&kid).await?;

    // Verify the key properties
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

    // Get the actual kid (did:key:z...#z...)
    let keys = key_manager.list_keys()?;
    let did = &keys[0];
    let key_part = did.strip_prefix("did:key:").unwrap();
    let kid = format!("{}#{}", did, key_part);

    // Sign a message
    let message = b"Hello, world!";
    let jws = key_manager.sign_jws(&kid, message, None).await?;

    // Verify the signature
    let verified_payload = key_manager.verify_jws(&jws, Some(&kid)).await?;
    assert_eq!(verified_payload, message);

    // Verify without specifying kid
    let verified_payload = key_manager.verify_jws(&jws, None).await?;
    assert_eq!(verified_payload, message);

    Ok(())
}

#[tokio::test]
#[cfg(feature = "crypto-p256")]
async fn test_encrypt_and_decrypt() -> Result<()> {
    // Encryption requires P-256 keys for ECDH-ES+A256KW
    use tap_agent::did::{DIDGenerationOptions, KeyType};
    use tap_agent::key_manager::DefaultKeyManager;

    let manager = DefaultKeyManager::new();

    // Generate P-256 keys for sender and recipient
    let sender_key = manager
        .generate_key(DIDGenerationOptions {
            key_type: KeyType::P256,
        })
        .unwrap();
    let sender_part = &sender_key.did["did:key:".len()..];
    let sender_kid = format!("{}#{}", sender_key.did, sender_part);

    let recipient_key = manager
        .generate_key(DIDGenerationOptions {
            key_type: KeyType::P256,
        })
        .unwrap();
    let recipient_part = &recipient_key.did["did:key:".len()..];
    let recipient_kid = format!("{}#{}", recipient_key.did, recipient_part);

    // Encrypt a message
    let plaintext = b"Secret message";
    let jwe = manager
        .encrypt_jwe(&sender_kid, &recipient_kid, plaintext, None)
        .await?;

    // Decrypt the message
    let decrypted = manager.decrypt_jwe(&jwe, Some(&recipient_kid)).await?;

    assert_eq!(decrypted, plaintext);

    Ok(())
}
