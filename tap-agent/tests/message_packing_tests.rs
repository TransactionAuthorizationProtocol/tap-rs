use tap_agent::{
    message_packing::{Packable, Unpackable, PackOptions, UnpackOptions, PlainMessage, SecurityMode},
    key_manager::{DefaultKeyManager, KeyManagerBuilder, KeyManagerPacking},
    error::Result,
};

use serde_json::{Value, json};
use std::sync::Arc;
use async_trait::async_trait;

// Simple test message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct TestMessage {
    pub message_type: String,
    pub id: String,
    pub content: String,
}

// Implement Packable for TestMessage
#[async_trait]
impl Packable for TestMessage {
    async fn pack(
        &self,
        key_manager: &impl KeyManagerPacking,
        options: PackOptions,
    ) -> Result<String, tap_agent::error::Error> {
        // Serialize to JSON
        let json_data = serde_json::to_string(self)?;
        
        // Apply security according to options
        match options.security_mode() {
            SecurityMode::Plain => Ok(json_data),
            SecurityMode::Signed(kid) => {
                // Create JWS
                let jws = key_manager.sign_jws(kid, json_data.as_bytes(), None).await?;
                Ok(jws)
            },
            SecurityMode::AuthCrypt(sender_kid, recipient_jwk) => {
                // Encrypt to recipient
                let jwe = key_manager.encrypt_to_jwk(
                    sender_kid,
                    json_data.as_bytes(),
                    recipient_jwk,
                    None
                ).await?;
                Ok(jwe)
            }
        }
    }
}

#[tokio::test]
async fn test_plain_packing_unpacking() -> Result<()> {
    // Create key manager
    let key_manager = KeyManagerBuilder::new()
        .build()
        .await?;
    
    // Create test message
    let message = TestMessage {
        message_type: "test/plain".to_string(),
        id: "msg-123".to_string(),
        content: "Plain message content".to_string(),
    };
    
    // Pack with plain security
    let options = PackOptions::new().with_plain();
    let packed = message.pack(&key_manager, options).await?;
    
    // Should be plain JSON
    assert!(packed.contains("Plain message content"));
    assert!(packed.contains("test/plain"));
    
    // Unpack
    let unpack_options = UnpackOptions::new();
    let unpacked = PlainMessage::unpack(&packed, &key_manager, unpack_options).await?;
    
    // Verify content
    let content: Value = serde_json::from_str(unpacked.message())?;
    assert_eq!(content["message_type"], "test/plain");
    assert_eq!(content["id"], "msg-123");
    assert_eq!(content["content"], "Plain message content");
    
    Ok(())
}

#[tokio::test]
async fn test_signed_packing_unpacking() -> Result<()> {
    // Create key manager with a signing key
    let key_manager = KeyManagerBuilder::new()
        .with_auto_generated_ed25519_key("signer")
        .build()
        .await?;
    
    // Create test message
    let message = TestMessage {
        message_type: "test/signed".to_string(),
        id: "msg-456".to_string(),
        content: "Signed message content".to_string(),
    };
    
    // Pack with signed security
    let options = PackOptions::new().with_sign("signer");
    let packed = message.pack(&key_manager, options).await?;
    
    // Should be a JWS (compact serialization)
    let parts: Vec<&str> = packed.split('.').collect();
    assert_eq!(parts.len(), 3); // Header, payload, signature
    
    // Unpack
    let unpack_options = UnpackOptions::new();
    let unpacked = PlainMessage::unpack(&packed, &key_manager, unpack_options).await?;
    
    // Verify content
    let content: Value = serde_json::from_str(unpacked.message())?;
    assert_eq!(content["message_type"], "test/signed");
    assert_eq!(content["id"], "msg-456");
    assert_eq!(content["content"], "Signed message content");
    
    Ok(())
}

#[tokio::test]
async fn test_auth_crypt_packing_unpacking() -> Result<()> {
    // Create key manager with encryption keys
    let key_manager = KeyManagerBuilder::new()
        .with_auto_generated_p256_key("encrypter")
        .build()
        .await?;
    
    // Get the public key for encryption
    let key = key_manager.get_encryption_key("encrypter").await?;
    let recipient_jwk = key.public_key_jwk()?;
    
    // Create test message
    let message = TestMessage {
        message_type: "test/encrypted".to_string(),
        id: "msg-789".to_string(),
        content: "Encrypted message content".to_string(),
    };
    
    // Pack with auth crypt security
    let options = PackOptions::new().with_auth_crypt("encrypter", &recipient_jwk);
    let packed = message.pack(&key_manager, options).await?;
    
    // Should be a JWE (JSON serialization)
    assert!(packed.contains("encrypted_key"));
    assert!(packed.contains("recipients"));
    
    // Unpack
    let unpack_options = UnpackOptions::new();
    let unpacked = PlainMessage::unpack(&packed, &key_manager, unpack_options).await?;
    
    // Verify content
    let content: Value = serde_json::from_str(unpacked.message())?;
    assert_eq!(content["message_type"], "test/encrypted");
    assert_eq!(content["id"], "msg-789");
    assert_eq!(content["content"], "Encrypted message content");
    
    Ok(())
}

#[tokio::test]
async fn test_unpack_options() -> Result<()> {
    // Create key manager
    let key_manager = KeyManagerBuilder::new()
        .with_auto_generated_ed25519_key("signer")
        .build()
        .await?;
    
    // Create and sign a message
    let message = TestMessage {
        message_type: "test/options".to_string(),
        id: "msg-options".to_string(),
        content: "Testing unpack options".to_string(),
    };
    
    let options = PackOptions::new().with_sign("signer");
    let packed = message.pack(&key_manager, options).await?;
    
    // Test with require_signature = true (should pass)
    let unpack_options = UnpackOptions::new().with_require_signature(true);
    let result = PlainMessage::unpack(&packed, &key_manager, unpack_options).await;
    assert!(result.is_ok());
    
    // Create plain message
    let plain_options = PackOptions::new().with_plain();
    let plain_packed = message.pack(&key_manager, plain_options).await?;
    
    // Test with require_signature = true on plain message (should fail)
    let strict_options = UnpackOptions::new().with_require_signature(true);
    let result = PlainMessage::unpack(&plain_packed, &key_manager, strict_options).await;
    assert!(result.is_err());
    
    Ok(())
}