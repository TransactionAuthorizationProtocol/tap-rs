use js_sys::{Object, Reflect};
use tap_wasm::WasmTapAgent;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Test creating a WasmTapAgent wrapper around existing TapAgent
#[wasm_bindgen_test]
async fn test_wasm_agent_creation() {
    // Create a config object
    let config = Object::new();
    Reflect::set(&config, &JsValue::from_str("debug"), &JsValue::TRUE).unwrap();

    // Create agent
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    // Verify DID was generated
    let did = agent.get_did();
    assert!(
        did.starts_with("did:key:"),
        "DID should start with did:key:"
    );
}

/// Test creating agent from existing private key
#[wasm_bindgen_test]
async fn test_wasm_agent_from_private_key() {
    use tap_wasm::generate_private_key;

    // Generate a new private key
    let private_key =
        generate_private_key("Ed25519".to_string()).expect("Failed to generate private key");

    // Create agent from private key - this is an async function, not a Promise
    let agent = WasmTapAgent::from_private_key(private_key.clone(), "Ed25519".to_string())
        .await
        .expect("Failed to create agent from private key");

    // Verify the agent was created
    let did = agent.get_did();
    assert!(
        did.starts_with("did:key:"),
        "DID should start with did:key:"
    );
}

/// Test exporting private key from agent
#[wasm_bindgen_test]
async fn test_private_key_export() {
    // Create an agent first
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    // Export private key
    let private_key = agent
        .export_private_key()
        .expect("Failed to export private key");
    assert!(!private_key.is_empty(), "Private key should not be empty");

    // Verify it's a valid hex string (Ed25519 key is 32 bytes = 64 hex chars)
    assert_eq!(
        private_key.len(),
        64,
        "Ed25519 private key should be 64 hex characters"
    );
}

/// Test importing private key to create agent
#[wasm_bindgen_test]
async fn test_private_key_import() {
    // Test data - a sample Ed25519 private key (32 bytes)
    let test_private_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    // Create agent from private key
    let agent = WasmTapAgent::from_private_key(test_private_key.to_string(), "Ed25519".to_string())
        .await
        .expect("Failed to create agent from private key");

    // Verify the agent was created with correct DID
    let did = agent.get_did();
    assert!(
        did.starts_with("did:key:"),
        "DID should start with did:key:"
    );

    // The DID should be deterministic for the same private key
    // Create another agent with the same key
    let agent2 =
        WasmTapAgent::from_private_key(test_private_key.to_string(), "Ed25519".to_string())
            .await
            .expect("Failed to create second agent from same private key");
    let did2 = agent2.get_did();

    assert_eq!(did, did2, "Same private key should produce same DID");
}

/// Test JsValue to Rust type conversions for messages
#[wasm_bindgen_test]
async fn test_jsvalue_message_conversion() {
    use js_sys::Array;

    // Create a JavaScript message object
    let message = Object::new();
    Reflect::set(
        &message,
        &JsValue::from_str("id"),
        &JsValue::from_str("test-123"),
    )
    .unwrap();
    Reflect::set(
        &message,
        &JsValue::from_str("type"),
        &JsValue::from_str("https://tap.rsvp/schema/1.0#Transfer"),
    )
    .unwrap();
    Reflect::set(
        &message,
        &JsValue::from_str("from"),
        &JsValue::from_str("did:key:z6MkTest"),
    )
    .unwrap();

    let to_array = Array::new();
    to_array.push(&JsValue::from_str("did:key:z6MkRecipient"));
    Reflect::set(&message, &JsValue::from_str("to"), &to_array).unwrap();

    // Create a body object
    let body = Object::new();
    Reflect::set(
        &body,
        &JsValue::from_str("amount"),
        &JsValue::from_str("100.0"),
    )
    .unwrap();
    Reflect::set(
        &body,
        &JsValue::from_str("asset"),
        &JsValue::from_str("eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
    )
    .unwrap();

    // Create originator
    let originator = Object::new();
    Reflect::set(
        &originator,
        &JsValue::from_str("@id"),
        &JsValue::from_str("did:key:z6MkOriginator"),
    )
    .unwrap();
    Reflect::set(&body, &JsValue::from_str("originator"), &originator).unwrap();

    // Create beneficiary
    let beneficiary = Object::new();
    Reflect::set(
        &beneficiary,
        &JsValue::from_str("@id"),
        &JsValue::from_str("did:key:z6MkBeneficiary"),
    )
    .unwrap();
    Reflect::set(&body, &JsValue::from_str("beneficiary"), &beneficiary).unwrap();

    Reflect::set(&message, &JsValue::from_str("body"), &body).unwrap();

    // The conversion happens inside pack_message, which we can test
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    // This will test the js_to_tap_message conversion internally
    let pack_promise = agent.pack_message(message.into());

    // Wait for the promise to resolve
    let result = wasm_bindgen_futures::JsFuture::from(pack_promise).await;

    // The test passes if no error was thrown during conversion
    assert!(
        result.is_ok(),
        "Message conversion and packing should succeed"
    );
}

/// Test that WasmTapAgent properly delegates to underlying TapAgent
#[wasm_bindgen_test]
async fn test_delegation_to_tap_agent() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    // Test that get_did delegates properly
    let did1 = agent.get_did();
    let did2 = agent.get_did();
    assert_eq!(did1, did2, "DID should be consistent");

    // Test that the DID format is correct
    assert!(
        did1.starts_with("did:key:z"),
        "DID should be in did:key format"
    );
}

/// Test exporting public key from agent
#[wasm_bindgen_test]
async fn test_public_key_export() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    // Export public key
    let public_key = agent
        .export_public_key()
        .expect("Failed to export public key");
    assert!(!public_key.is_empty(), "Public key should not be empty");

    // Ed25519 public key is 32 bytes = 64 hex chars
    assert_eq!(
        public_key.len(),
        64,
        "Ed25519 public key should be 64 hex characters"
    );
}

/// Test creating agent with different key types
#[wasm_bindgen_test]
async fn test_different_key_types() {
    // Test Ed25519 (default)
    let config = Object::new();
    Reflect::set(
        &config,
        &JsValue::from_str("keyType"),
        &JsValue::from_str("Ed25519"),
    )
    .unwrap();
    let agent_ed = WasmTapAgent::new(config.into());
    assert!(agent_ed.is_ok(), "Should create agent with Ed25519 key");

    // Test P256 (to be implemented)
    // let config_p256 = Object::new();
    // Reflect::set(&config_p256, &JsValue::from_str("keyType"), &JsValue::from_str("P256")).unwrap();
    // let agent_p256 = WasmTapAgent::new(config_p256.into());
    // assert!(agent_p256.is_ok(), "Should create agent with P256 key");

    // Test Secp256k1 (to be implemented)
    // let config_secp = Object::new();
    // Reflect::set(&config_secp, &JsValue::from_str("keyType"), &JsValue::from_str("Secp256k1")).unwrap();
    // let agent_secp = WasmTapAgent::new(config_secp.into());
    // assert!(agent_secp.is_ok(), "Should create agent with Secp256k1 key");
}

/// Test error handling for invalid configurations
#[wasm_bindgen_test]
async fn test_error_handling() {
    // Test with invalid DID format (when implemented with from_private_key)
    // This will be expanded when more error cases are implemented

    // For now, ensure basic creation doesn't panic
    let config = Object::new();
    let result = std::panic::catch_unwind(|| WasmTapAgent::new(config.into()));
    assert!(result.is_ok(), "Agent creation should not panic");
}
