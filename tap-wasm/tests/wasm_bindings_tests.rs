use js_sys::{Object, Reflect};
use tap_wasm::{generate_private_key, generate_uuid, WasmTapAgent};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Test creating a new WasmTapAgent with default config
#[wasm_bindgen_test]
async fn test_new_agent_default() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into());

    assert!(agent.is_ok(), "Should create agent with default config");

    let agent = agent.unwrap();
    let did = agent.get_did();

    // Verify DID format (should be did:key:...)
    assert!(
        did.starts_with("did:key:"),
        "DID should start with 'did:key:'"
    );
    assert!(did.len() > 10, "DID should have reasonable length");
}

/// Test creating a new WasmTapAgent with debug enabled
#[wasm_bindgen_test]
async fn test_new_agent_with_debug() {
    let config = Object::new();
    Reflect::set(&config, &JsValue::from_str("debug"), &JsValue::TRUE).unwrap();

    let agent = WasmTapAgent::new(config.into());
    assert!(agent.is_ok(), "Should create agent with debug config");
}

/// Test creating a new WasmTapAgent with nickname
#[wasm_bindgen_test]
async fn test_new_agent_with_nickname() {
    let config = Object::new();
    Reflect::set(
        &config,
        &JsValue::from_str("nickname"),
        &JsValue::from_str("TestAgent"),
    )
    .unwrap();

    let agent = WasmTapAgent::new(config.into()).expect("Should create agent");
    let nickname = agent.nickname();

    assert_eq!(
        nickname,
        Some("TestAgent".to_string()),
        "Nickname should be set"
    );
}

/// Test creating an agent from a private key
#[wasm_bindgen_test]
async fn test_from_private_key_ed25519() {
    // Generate a private key first
    let private_key =
        generate_private_key("Ed25519".to_string()).expect("Should generate private key");

    // Create agent from the private key
    let agent_future = WasmTapAgent::from_private_key(private_key.clone(), "Ed25519".to_string());
    let agent_result = agent_future.await;

    assert!(agent_result.is_ok(), "Should create agent from private key");

    let agent = agent_result.unwrap();
    let did = agent.get_did();

    // Verify DID format
    assert!(
        did.starts_with("did:key:z6Mk"),
        "Ed25519 DID should start with 'did:key:z6Mk'"
    );

    // Export the private key and verify it matches
    let exported_key = agent
        .export_private_key()
        .expect("Should export private key");

    assert_eq!(
        exported_key, private_key,
        "Exported key should match original"
    );
}

/// Test creating an agent from a P256 private key
#[wasm_bindgen_test]
async fn test_from_private_key_p256() {
    // Generate a P256 private key
    let private_key =
        generate_private_key("P256".to_string()).expect("Should generate P256 private key");

    // Create agent from the private key
    let agent_future = WasmTapAgent::from_private_key(private_key, "P256".to_string());
    let agent_result = agent_future.await;

    assert!(
        agent_result.is_ok(),
        "Should create agent from P256 private key"
    );

    let agent = agent_result.unwrap();
    let did = agent.get_did();

    // P256 DIDs have a different prefix
    assert!(
        did.starts_with("did:key:"),
        "P256 DID should start with 'did:key:'"
    );
}

/// Test creating an agent from a Secp256k1 private key
#[wasm_bindgen_test]
async fn test_from_private_key_secp256k1() {
    // Generate a Secp256k1 private key
    let private_key = generate_private_key("Secp256k1".to_string())
        .expect("Should generate Secp256k1 private key");

    // Create agent from the private key
    let agent_future = WasmTapAgent::from_private_key(private_key, "Secp256k1".to_string());
    let agent_result = agent_future.await;

    assert!(
        agent_result.is_ok(),
        "Should create agent from Secp256k1 private key"
    );

    let agent = agent_result.unwrap();
    let did = agent.get_did();

    // Secp256k1 DIDs have a different prefix
    assert!(
        did.starts_with("did:key:"),
        "Secp256k1 DID should start with 'did:key:'"
    );
}

/// Test error handling for invalid private key
#[wasm_bindgen_test]
async fn test_from_private_key_invalid_hex() {
    let invalid_key = "not-a-hex-string".to_string();

    let agent_future = WasmTapAgent::from_private_key(invalid_key, "Ed25519".to_string());
    let agent_result = agent_future.await;

    assert!(agent_result.is_err(), "Should fail with invalid hex string");
}

/// Test error handling for invalid key type
#[wasm_bindgen_test]
async fn test_from_private_key_invalid_type() {
    let private_key =
        generate_private_key("Ed25519".to_string()).expect("Should generate private key");

    let agent_future = WasmTapAgent::from_private_key(private_key, "InvalidType".to_string());
    let agent_result = agent_future.await;

    assert!(agent_result.is_err(), "Should fail with invalid key type");
}

/// Test get_did returns consistent value
#[wasm_bindgen_test]
async fn test_get_did_consistency() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Should create agent");

    let did1 = agent.get_did();
    let did2 = agent.get_did();
    let did3 = agent.get_did();

    assert_eq!(did1, did2, "DID should be consistent");
    assert_eq!(did2, did3, "DID should be consistent");
}

/// Test exporting public key
#[wasm_bindgen_test]
async fn test_export_public_key() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Should create agent");

    let public_key = agent.export_public_key();

    assert!(public_key.is_ok(), "Should export public key");

    let key_hex = public_key.unwrap();
    // Ed25519 public keys are 32 bytes = 64 hex chars
    assert!(
        key_hex.len() >= 64,
        "Public key should have reasonable length"
    );

    // Verify it's valid hex
    assert!(
        hex::decode(&key_hex).is_ok(),
        "Public key should be valid hex"
    );
}

/// Test creating an agent with a specific DID
#[wasm_bindgen_test]
async fn test_new_agent_with_did() {
    let config = Object::new();
    let test_did = "did:key:z6MkTestDID123";
    Reflect::set(
        &config,
        &JsValue::from_str("did"),
        &JsValue::from_str(test_did),
    )
    .unwrap();

    let agent = WasmTapAgent::new(config.into()).expect("Should create agent with DID");
    let did = agent.get_did();

    assert_eq!(did, test_did, "Agent should use provided DID");
}
