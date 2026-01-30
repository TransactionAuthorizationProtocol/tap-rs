//! Tests for cryptographic primitives
//!
//! These tests verify that proper cryptographic algorithms are used.

use tap_agent::crypto::{derive_key_ecdh_es, unwrap_key_aes_kw, wrap_key_aes_kw};

/// Test that key wrapping uses AES-KW, not XOR
#[test]
fn test_key_wrap_is_not_xor() {
    let kek = [0x00u8; 32]; // All-zeros key encryption key
    let plaintext_key = [0xFFu8; 32]; // All-ones content key

    let wrapped = wrap_key_aes_kw(&kek, &plaintext_key).expect("wrap should succeed");

    // XOR with all-zeros would produce the original plaintext
    // AES-KW produces a different result with authentication
    assert_ne!(
        &wrapped[..32],
        &plaintext_key[..],
        "Key wrapping must not be simple XOR"
    );

    // AES-KW produces 40 bytes for 32-byte key (adds 8-byte integrity check)
    assert_eq!(
        wrapped.len(),
        40,
        "AES-KW output for 256-bit key should be 40 bytes"
    );
}

/// Test round-trip key wrap/unwrap
#[test]
fn test_key_wrap_unwrap_roundtrip() {
    let kek = [0x42u8; 32];
    let plaintext_key = [0xABu8; 32];

    let wrapped = wrap_key_aes_kw(&kek, &plaintext_key).expect("wrap should succeed");
    let unwrapped = unwrap_key_aes_kw(&kek, &wrapped).expect("unwrap should succeed");

    assert_eq!(
        &unwrapped[..],
        &plaintext_key[..],
        "Unwrapped key must match original"
    );
}

/// Test that unwrap fails with wrong KEK
#[test]
fn test_key_unwrap_fails_with_wrong_kek() {
    let kek1 = [0x42u8; 32];
    let kek2 = [0x43u8; 32];
    let plaintext_key = [0xABu8; 32];

    let wrapped = wrap_key_aes_kw(&kek1, &plaintext_key).expect("wrap should succeed");
    let result = unwrap_key_aes_kw(&kek2, &wrapped);

    assert!(result.is_err(), "Unwrap with wrong KEK must fail");
}

/// Test that unwrap fails with tampered ciphertext
#[test]
fn test_key_unwrap_detects_tampering() {
    let kek = [0x42u8; 32];
    let plaintext_key = [0xABu8; 32];

    let mut wrapped = wrap_key_aes_kw(&kek, &plaintext_key).expect("wrap should succeed");

    // Tamper with the wrapped key
    wrapped[0] ^= 0xFF;

    let result = unwrap_key_aes_kw(&kek, &wrapped);
    assert!(result.is_err(), "Unwrap of tampered ciphertext must fail");
}

/// Test ECDH-ES key derivation produces correct length
#[test]
fn test_ecdh_kdf_output_length() {
    let shared_secret = [0x42u8; 32];
    let apu = b"Alice";
    let apv = b"Bob";

    // Derive 256-bit key for AES-256-KW
    let derived =
        derive_key_ecdh_es(&shared_secret, apu, apv, 256).expect("KDF should succeed");

    assert_eq!(
        derived.len(),
        32,
        "Derived key should be 32 bytes for 256 bits"
    );
}

/// Test that different APU/APV produce different keys
#[test]
fn test_ecdh_kdf_is_context_bound() {
    let shared_secret = [0x42u8; 32];

    let key1 =
        derive_key_ecdh_es(&shared_secret, b"Alice", b"Bob", 256).expect("KDF should succeed");
    let key2 = derive_key_ecdh_es(&shared_secret, b"Alice", b"Charlie", 256)
        .expect("KDF should succeed");

    assert_ne!(
        &key1[..],
        &key2[..],
        "Different APV must produce different derived keys"
    );
}

/// Test that the same inputs produce the same key (deterministic)
#[test]
fn test_ecdh_kdf_is_deterministic() {
    let shared_secret = [0x42u8; 32];

    let key1 =
        derive_key_ecdh_es(&shared_secret, b"Alice", b"Bob", 256).expect("KDF should succeed");
    let key2 =
        derive_key_ecdh_es(&shared_secret, b"Alice", b"Bob", 256).expect("KDF should succeed");

    assert_eq!(
        &key1[..],
        &key2[..],
        "Same inputs must produce same derived key"
    );
}

/// Test wrap/unwrap with random data
#[test]
fn test_key_wrap_unwrap_various_keys() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for _ in 0..10 {
        let mut kek = [0u8; 32];
        let mut plaintext = [0u8; 32];
        rng.fill(&mut kek);
        rng.fill(&mut plaintext);

        let wrapped = wrap_key_aes_kw(&kek, &plaintext).expect("wrap should succeed");
        let unwrapped = unwrap_key_aes_kw(&kek, &wrapped).expect("unwrap should succeed");

        assert_eq!(
            &unwrapped[..],
            &plaintext[..],
            "Round-trip must preserve key"
        );
    }
}
