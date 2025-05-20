use tap_agent::{
    agent_key::{
        AgentKey, DecryptionKey, EncryptionKey, JwsAlgorithm, SigningKey, VerificationKey,
    },
    error::Result,
    key_manager::{DefaultKeyManager, KeyManagerBuilder, KeyManagerPacking},
    local_agent_key::LocalAgentKey,
    message_packing::{PackOptions, Packable, PlainMessage, UnpackOptions, Unpackable},
};

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tap_msg::message::tap_message_trait::TapMessageBody;

// Simple message for testing packing/unpacking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct TestMessage {
    pub message_type: String,
    pub content: String,
}

impl TapMessageBody for TestMessage {
    fn get_type(&self) -> &str {
        &self.message_type
    }
}

#[tokio::test]
async fn test_key_manager_builder() -> Result<()> {
    // Test building with default options
    let key_manager = KeyManagerBuilder::new().build().await?;

    // Key manager should have no keys initially
    assert_eq!(key_manager.list_signing_keys().await?.len(), 0);

    // Test building with automatic key generation
    let key_manager = KeyManagerBuilder::new()
        .with_auto_generated_ed25519_key("default")
        .build()
        .await?;

    // Key manager should have one signing key
    assert_eq!(key_manager.list_signing_keys().await?.len(), 1);

    // Test key retrieval
    let signing_key = key_manager.get_signing_key("default").await?;
    assert_eq!(signing_key.key_id(), "default");
    assert_eq!(signing_key.recommended_jws_alg(), JwsAlgorithm::EdDSA);

    Ok(())
}

#[tokio::test]
async fn test_key_manager_key_operations() -> Result<()> {
    // Create a key manager
    let key_manager = KeyManagerBuilder::new().build().await?;

    // Generate various key types and add them
    let ed25519_key = Arc::new(LocalAgentKey::generate_ed25519("ed25519-key")?);
    let p256_key = Arc::new(LocalAgentKey::generate_p256("p256-key")?);

    key_manager.add_signing_key(ed25519_key.clone()).await?;
    key_manager.add_signing_key(p256_key.clone()).await?;

    // Test listing keys
    let signing_keys = key_manager.list_signing_keys().await?;
    assert_eq!(signing_keys.len(), 2);
    assert!(signing_keys.contains(&"ed25519-key".to_string()));
    assert!(signing_keys.contains(&"p256-key".to_string()));

    // Test signing with specific key
    let test_data = b"test message to sign";
    let signature = key_manager.sign("ed25519-key", test_data).await?;

    // Test verification
    assert!(key_manager
        .verify("ed25519-key", test_data, &signature)
        .await
        .is_ok());

    // Test direct JWS operations
    let payload = b"{\"hello\":\"world\"}";
    let jws = key_manager.sign_jws("p256-key", payload, None).await?;
    let verified = key_manager.verify_jws("p256-key", &jws).await?;
    assert_eq!(verified, payload);

    Ok(())
}

#[tokio::test]
async fn test_message_packing_unpacking() -> Result<()> {
    // Create a key manager with keys
    let key_manager = KeyManagerBuilder::new()
        .with_auto_generated_ed25519_key("sender")
        .build()
        .await?;

    // Create a test message
    let message = TestMessage {
        message_type: "test/message".to_string(),
        content: "Hello, world!".to_string(),
    };

    // Plain packing
    let pack_options = PackOptions::new().with_plain();
    let packed = message.pack(&key_manager, pack_options).await?;

    // Unpack
    let unpack_options = UnpackOptions::new();
    let unpacked = PlainMessage::unpack(&packed, &key_manager, unpack_options).await?;

    // Verify content
    let content = unpacked.message();
    assert!(content.contains("Hello, world!"));

    // Signed packing
    let pack_options = PackOptions::new().with_sign("sender");
    let packed = message.pack(&key_manager, pack_options).await?;

    // Unpack signed
    let unpacked = PlainMessage::unpack(&packed, &key_manager, unpack_options).await?;
    let content = unpacked.message();
    assert!(content.contains("Hello, world!"));

    Ok(())
}

#[tokio::test]
async fn test_key_removal() -> Result<()> {
    // Create a key manager
    let key_manager = KeyManagerBuilder::new()
        .with_auto_generated_ed25519_key("temp-key")
        .build()
        .await?;

    // Verify key exists
    assert_eq!(key_manager.list_signing_keys().await?.len(), 1);

    // Remove key
    key_manager.remove_key("temp-key").await?;

    // Verify key is gone
    assert_eq!(key_manager.list_signing_keys().await?.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_key_rotation() -> Result<()> {
    // Create a key manager
    let key_manager = KeyManagerBuilder::new()
        .with_auto_generated_ed25519_key("old-key")
        .build()
        .await?;

    // Get the original key's DID
    let old_key = key_manager.get_signing_key("old-key").await?;
    let old_did = old_key.did().to_string();

    // Rotate key
    key_manager.rotate_key("old-key", "new-key").await?;

    // Old key ID should be gone
    assert!(key_manager.get_signing_key("old-key").await.is_err());

    // New key should exist
    let new_key = key_manager.get_signing_key("new-key").await?;

    // DIDs should be different
    assert_ne!(new_key.did(), old_did);

    Ok(())
}
