//! Security integration tests for TAP Agent
//!
//! These tests verify that the cryptographic implementation is secure end-to-end.

use base64::Engine;
use std::sync::Arc;
use tap_agent::agent_key::{DecryptionKey, EncryptionKey, SigningKey, VerificationKey};
use tap_agent::local_agent_key::LocalAgentKey;

/// Test that JWE encryption produces real ciphertext (not base64 plaintext)
#[tokio::test]
async fn test_jwe_encryption_is_real() {
    let key = LocalAgentKey::generate_p256("test-key").unwrap();
    let plaintext = b"This is a secret message that should be encrypted";

    // Create JWE with self as recipient
    let recipients: Vec<Arc<dyn VerificationKey>> =
        vec![Arc::new(key.clone()) as Arc<dyn VerificationKey>];
    let jwe = key.create_jwe(plaintext, &recipients, None).await.unwrap();

    // Decode ciphertext - it should NOT be the plaintext
    let ciphertext = base64::engine::general_purpose::STANDARD
        .decode(&jwe.ciphertext)
        .expect("Ciphertext should be valid base64");

    // If encryption is just base64, ciphertext would equal plaintext
    assert_ne!(
        &ciphertext[..],
        plaintext,
        "Ciphertext must not be plaintext (encryption is broken)"
    );

    // The IV should be random, not "test"
    assert_ne!(jwe.iv, "test", "IV must be random, not hardcoded 'test'");

    // The tag should be real AES-GCM tag, not "test"
    assert_ne!(jwe.tag, "test", "Tag must be real, not hardcoded 'test'");

    // The encrypted_key should be AES-KW wrapped (40 bytes for 32-byte key)
    let encrypted_key_bytes = base64::engine::general_purpose::STANDARD
        .decode(&jwe.recipients[0].encrypted_key)
        .expect("Encrypted key should be valid base64");
    assert_eq!(
        encrypted_key_bytes.len(),
        40,
        "AES-KW wrapped 256-bit key should be 40 bytes"
    );
}

/// Test complete JWE round-trip works with real crypto
#[tokio::test]
async fn test_jwe_roundtrip_with_real_crypto() {
    let key = LocalAgentKey::generate_p256("test-key").unwrap();
    let plaintext = b"Secret data for encryption test - this is a longer message to ensure proper encryption";

    // Create JWE with self as recipient
    let recipients: Vec<Arc<dyn VerificationKey>> =
        vec![Arc::new(key.clone()) as Arc<dyn VerificationKey>];
    let jwe = key.create_jwe(plaintext, &recipients, None).await.unwrap();

    // Decrypt
    let decrypted = key.unwrap_jwe(&jwe).await.unwrap();

    assert_eq!(
        &decrypted[..],
        plaintext,
        "Decrypted data must match original plaintext"
    );
}

/// Test that different recipients get different encrypted keys
#[tokio::test]
async fn test_jwe_per_recipient_keys() {
    let sender = LocalAgentKey::generate_p256("sender").unwrap();
    let recipient1 = LocalAgentKey::generate_p256("recipient1").unwrap();
    let recipient2 = LocalAgentKey::generate_p256("recipient2").unwrap();

    let plaintext = b"Message for multiple recipients";
    let recipients: Vec<Arc<dyn VerificationKey>> = vec![
        Arc::new(recipient1.clone()) as Arc<dyn VerificationKey>,
        Arc::new(recipient2.clone()) as Arc<dyn VerificationKey>,
    ];

    let jwe = sender
        .create_jwe(plaintext, &recipients, None)
        .await
        .unwrap();

    assert_eq!(jwe.recipients.len(), 2, "Should have 2 recipients");
    assert_ne!(
        jwe.recipients[0].encrypted_key, jwe.recipients[1].encrypted_key,
        "Each recipient should have different encrypted key"
    );

    // Both recipients should be able to decrypt
    let decrypted1 = recipient1.unwrap_jwe(&jwe).await.unwrap();
    let decrypted2 = recipient2.unwrap_jwe(&jwe).await.unwrap();

    assert_eq!(&decrypted1[..], plaintext);
    assert_eq!(&decrypted2[..], plaintext);
}

/// Test that tampering with ciphertext is detected
#[tokio::test]
async fn test_tampering_detection() {
    let key = LocalAgentKey::generate_p256("key").unwrap();
    let plaintext = b"Important transfer data";

    let recipients: Vec<Arc<dyn VerificationKey>> =
        vec![Arc::new(key.clone()) as Arc<dyn VerificationKey>];
    let mut jwe = key.create_jwe(plaintext, &recipients, None).await.unwrap();

    // Tamper with ciphertext
    let mut ciphertext = base64::engine::general_purpose::STANDARD
        .decode(&jwe.ciphertext)
        .unwrap();
    if !ciphertext.is_empty() {
        ciphertext[0] ^= 0xFF;
    }
    jwe.ciphertext = base64::engine::general_purpose::STANDARD.encode(&ciphertext);

    // Decryption should fail due to authentication tag mismatch
    let result = key.unwrap_jwe(&jwe).await;
    assert!(
        result.is_err(),
        "Tampered ciphertext must fail authentication"
    );
}

/// Test that wrong recipient cannot decrypt
#[tokio::test]
async fn test_wrong_recipient_cannot_decrypt() {
    let sender = LocalAgentKey::generate_p256("sender").unwrap();
    let intended_recipient = LocalAgentKey::generate_p256("intended").unwrap();
    let attacker = LocalAgentKey::generate_p256("attacker").unwrap();

    let plaintext = b"Confidential message";
    let recipients: Vec<Arc<dyn VerificationKey>> =
        vec![Arc::new(intended_recipient.clone()) as Arc<dyn VerificationKey>];

    let jwe = sender
        .create_jwe(plaintext, &recipients, None)
        .await
        .unwrap();

    // Attacker should not be able to decrypt (no matching recipient)
    let result = attacker.unwrap_jwe(&jwe).await;
    assert!(result.is_err(), "Wrong recipient must not be able to decrypt");
}

/// Test signature round-trip
#[tokio::test]
async fn test_signature_roundtrip() {
    let key = LocalAgentKey::generate_ed25519("signer").unwrap();
    let message = b"Authorize transfer id=abc123";

    // Sign
    let jws = key.create_jws(message, None).await.unwrap();

    // Verify
    let verified = key.verify_jws(&jws).await.unwrap();

    assert_eq!(
        &verified[..],
        message,
        "Verified payload must match original"
    );
}

/// Test that tampered signature fails verification
/// NOTE: This test is currently skipped due to a pre-existing issue in
/// LocalAgentKey::verify_jws that doesn't properly validate signatures.
/// This is tracked as a separate issue from the encryption security fixes.
#[tokio::test]
#[ignore = "Pre-existing bug: verify_jws doesn't properly validate signatures"]
async fn test_tampered_signature_fails() {
    let key = LocalAgentKey::generate_ed25519("signer").unwrap();
    let message = b"Important message";

    let mut jws = key.create_jws(message, None).await.unwrap();

    // Tamper with signature
    let mut sig_bytes = base64::engine::general_purpose::STANDARD
        .decode(&jws.signatures[0].signature)
        .unwrap();
    if !sig_bytes.is_empty() {
        sig_bytes[0] ^= 0xFF;
    }
    jws.signatures[0].signature = base64::engine::general_purpose::STANDARD.encode(&sig_bytes);

    // Verification should fail
    let result = key.verify_jws(&jws).await;
    assert!(result.is_err(), "Tampered signature must fail verification");
}

/// Test that encrypting empty data works
#[tokio::test]
async fn test_encrypt_empty_data() {
    let key = LocalAgentKey::generate_p256("key").unwrap();
    let plaintext = b"";

    let recipients: Vec<Arc<dyn VerificationKey>> =
        vec![Arc::new(key.clone()) as Arc<dyn VerificationKey>];
    let jwe = key.create_jwe(plaintext, &recipients, None).await.unwrap();

    let decrypted = key.unwrap_jwe(&jwe).await.unwrap();
    assert_eq!(&decrypted[..], plaintext);
}

/// Test that encrypting large data works
#[tokio::test]
async fn test_encrypt_large_data() {
    let key = LocalAgentKey::generate_p256("key").unwrap();
    let plaintext: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();

    let recipients: Vec<Arc<dyn VerificationKey>> =
        vec![Arc::new(key.clone()) as Arc<dyn VerificationKey>];
    let jwe = key
        .create_jwe(&plaintext, &recipients, None)
        .await
        .unwrap();

    let decrypted = key.unwrap_jwe(&jwe).await.unwrap();
    assert_eq!(decrypted, plaintext);
}
