use tap_wasm::{generate_private_key, generate_uuid, generate_uuid_v4};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Test UUID generation
#[wasm_bindgen_test]
fn test_generate_uuid_v4() {
    let uuid1 = generate_uuid_v4();
    let uuid2 = generate_uuid_v4();

    // Should generate different UUIDs
    assert_ne!(uuid1, uuid2, "UUIDs should be unique");

    // Should be valid UUID v4 format
    assert_eq!(uuid1.len(), 36, "UUID should be 36 characters");
    assert!(uuid1.contains('-'), "UUID should contain hyphens");

    // Check UUID v4 format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
    let parts: Vec<&str> = uuid1.split('-').collect();
    assert_eq!(parts.len(), 5, "UUID should have 5 parts");
    assert_eq!(parts[0].len(), 8, "First part should be 8 chars");
    assert_eq!(parts[1].len(), 4, "Second part should be 4 chars");
    assert_eq!(parts[2].len(), 4, "Third part should be 4 chars");
    assert_eq!(parts[3].len(), 4, "Fourth part should be 4 chars");
    assert_eq!(parts[4].len(), 12, "Fifth part should be 12 chars");

    // Third part should start with '4' for UUID v4
    assert!(
        parts[2].starts_with('4'),
        "Third part should start with '4' for UUID v4"
    );
}

/// Test generate_uuid alias
#[wasm_bindgen_test]
fn test_generate_uuid_alias() {
    let uuid1 = generate_uuid();
    let uuid2 = generate_uuid();

    // Should generate different UUIDs
    assert_ne!(uuid1, uuid2, "UUIDs should be unique");

    // Should be valid UUID format
    assert_eq!(uuid1.len(), 36, "UUID should be 36 characters");
    assert!(uuid1.contains('-'), "UUID should contain hyphens");
}

/// Test generating Ed25519 private key
#[wasm_bindgen_test]
fn test_generate_private_key_ed25519() {
    let result = generate_private_key("Ed25519".to_string());

    assert!(result.is_ok(), "Should generate Ed25519 private key");

    let private_key = result.unwrap();

    // Ed25519 private keys are 32 bytes = 64 hex characters
    assert_eq!(
        private_key.len(),
        64,
        "Ed25519 private key should be 64 hex chars"
    );

    // Should be valid hex
    assert!(
        hex::decode(&private_key).is_ok(),
        "Private key should be valid hex"
    );

    // Generate another key and ensure they're different
    let private_key2 = generate_private_key("Ed25519".to_string()).unwrap();
    assert_ne!(
        private_key, private_key2,
        "Generated keys should be different"
    );
}

/// Test generating P256 private key
#[wasm_bindgen_test]
fn test_generate_private_key_p256() {
    let result = generate_private_key("P256".to_string());

    assert!(result.is_ok(), "Should generate P256 private key");

    let private_key = result.unwrap();

    // P256 private keys are 32 bytes = 64 hex characters
    assert_eq!(
        private_key.len(),
        64,
        "P256 private key should be 64 hex chars"
    );

    // Should be valid hex
    assert!(
        hex::decode(&private_key).is_ok(),
        "Private key should be valid hex"
    );

    // Generate another key and ensure they're different
    let private_key2 = generate_private_key("P256".to_string()).unwrap();
    assert_ne!(
        private_key, private_key2,
        "Generated keys should be different"
    );
}

/// Test generating Secp256k1 private key
#[wasm_bindgen_test]
fn test_generate_private_key_secp256k1() {
    let result = generate_private_key("Secp256k1".to_string());

    assert!(result.is_ok(), "Should generate Secp256k1 private key");

    let private_key = result.unwrap();

    // Secp256k1 private keys are 32 bytes = 64 hex characters
    assert_eq!(
        private_key.len(),
        64,
        "Secp256k1 private key should be 64 hex chars"
    );

    // Should be valid hex
    assert!(
        hex::decode(&private_key).is_ok(),
        "Private key should be valid hex"
    );

    // Generate another key and ensure they're different
    let private_key2 = generate_private_key("Secp256k1".to_string()).unwrap();
    assert_ne!(
        private_key, private_key2,
        "Generated keys should be different"
    );
}

/// Test error handling for invalid key type
#[wasm_bindgen_test]
fn test_generate_private_key_invalid_type() {
    let result = generate_private_key("InvalidKeyType".to_string());

    assert!(result.is_err(), "Should fail with invalid key type");

    // Check error message
    if let Err(err) = result {
        let err_str = err.as_string().unwrap_or_default();
        assert!(
            err_str.contains("Invalid key type"),
            "Error should mention invalid key type"
        );
    }
}

/// Test that private keys are cryptographically random
#[wasm_bindgen_test]
fn test_private_key_randomness() {
    // Generate multiple keys and ensure they're all different
    let mut keys = Vec::new();
    for _ in 0..10 {
        let key = generate_private_key("Ed25519".to_string()).unwrap();
        keys.push(key);
    }

    // Check that all keys are unique
    for i in 0..keys.len() {
        for j in (i + 1)..keys.len() {
            assert_ne!(keys[i], keys[j], "All generated keys should be unique");
        }
    }
}

/// Test UUID uniqueness over multiple generations
#[wasm_bindgen_test]
fn test_uuid_uniqueness() {
    let mut uuids = Vec::new();
    for _ in 0..100 {
        let uuid = generate_uuid();
        uuids.push(uuid);
    }

    // Check that all UUIDs are unique
    for i in 0..uuids.len() {
        for j in (i + 1)..uuids.len() {
            assert_ne!(uuids[i], uuids[j], "All generated UUIDs should be unique");
        }
    }
}

/// Test that generated keys can be used to create agents
#[wasm_bindgen_test]
async fn test_generated_key_usability() {
    use tap_wasm::WasmTapAgent;

    // Generate keys for all supported types
    let ed25519_key = generate_private_key("Ed25519".to_string()).unwrap();
    let p256_key = generate_private_key("P256".to_string()).unwrap();
    let secp256k1_key = generate_private_key("Secp256k1".to_string()).unwrap();

    // Try to create agents with each key
    let agent1 = WasmTapAgent::from_private_key(ed25519_key, "Ed25519".to_string()).await;
    assert!(
        agent1.is_ok(),
        "Should create agent with generated Ed25519 key"
    );

    let agent2 = WasmTapAgent::from_private_key(p256_key, "P256".to_string()).await;
    assert!(
        agent2.is_ok(),
        "Should create agent with generated P256 key"
    );

    let agent3 = WasmTapAgent::from_private_key(secp256k1_key, "Secp256k1".to_string()).await;
    assert!(
        agent3.is_ok(),
        "Should create agent with generated Secp256k1 key"
    );
}

/// Test hex encoding correctness
#[wasm_bindgen_test]
fn test_private_key_hex_encoding() {
    let private_key = generate_private_key("Ed25519".to_string()).unwrap();

    // Decode and re-encode to verify correctness
    let bytes = hex::decode(&private_key).expect("Should decode hex");
    let re_encoded = hex::encode(&bytes);

    assert_eq!(private_key, re_encoded, "Hex encoding should be consistent");

    // Verify all characters are valid hex
    for c in private_key.chars() {
        assert!(
            c.is_ascii_hexdigit(),
            "All characters should be valid hex digits"
        );
    }
}
