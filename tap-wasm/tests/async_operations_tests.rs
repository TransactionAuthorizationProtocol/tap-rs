use js_sys::{Array, Object, Promise, Reflect};
use tap_wasm::WasmTapAgent;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Test that pack_message returns a Promise
#[wasm_bindgen_test]
async fn test_pack_message_returns_promise() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    let message = create_test_message(&agent.get_did());
    let result = agent.pack_message(message.into());

    // Verify it's a Promise
    assert!(
        result.is_instance_of::<Promise>(),
        "pack_message should return a Promise"
    );

    // Await the promise
    let packed = JsFuture::from(result).await;
    assert!(packed.is_ok(), "Promise should resolve successfully");
}

/// Test that unpack_message returns a Promise
#[wasm_bindgen_test]
async fn test_unpack_message_returns_promise() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    // First pack a message
    let message = create_test_message(&agent.get_did());
    let pack_promise = agent.pack_message(message.into());
    let packed_result = JsFuture::from(pack_promise)
        .await
        .expect("Packing should succeed");

    let packed_str = Reflect::get(&packed_result, &JsValue::from_str("message"))
        .unwrap()
        .as_string()
        .unwrap();

    // Now test unpacking
    let result = agent.unpack_message(&packed_str, None);

    // Verify it's a Promise
    assert!(
        result.is_instance_of::<Promise>(),
        "unpack_message should return a Promise"
    );

    // Await the promise
    let unpacked = JsFuture::from(result).await;
    assert!(unpacked.is_ok(), "Promise should resolve successfully");
}

/// Test concurrent pack operations
#[wasm_bindgen_test]
async fn test_concurrent_pack_operations() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    // Create multiple messages
    let message1 = create_test_message(&agent.get_did());
    let message2 = create_test_message(&agent.get_did());
    let message3 = create_test_message(&agent.get_did());

    // Start all pack operations concurrently
    let promise1 = agent.pack_message(message1.into());
    let promise2 = agent.pack_message(message2.into());
    let promise3 = agent.pack_message(message3.into());

    // All should be Promises
    assert!(promise1.is_instance_of::<Promise>());
    assert!(promise2.is_instance_of::<Promise>());
    assert!(promise3.is_instance_of::<Promise>());

    // Await all promises
    let result1 = JsFuture::from(promise1).await;
    let result2 = JsFuture::from(promise2).await;
    let result3 = JsFuture::from(promise3).await;

    // All should succeed
    assert!(result1.is_ok(), "First pack should succeed");
    assert!(result2.is_ok(), "Second pack should succeed");
    assert!(result3.is_ok(), "Third pack should succeed");
}

/// Test concurrent unpack operations
#[wasm_bindgen_test]
async fn test_concurrent_unpack_operations() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    // Create and pack multiple messages
    let messages = vec![
        create_test_message(&agent.get_did()),
        create_test_message(&agent.get_did()),
        create_test_message(&agent.get_did()),
    ];

    let mut packed_messages = Vec::new();
    for msg in messages {
        let pack_promise = agent.pack_message(msg.into());
        let packed_result = JsFuture::from(pack_promise)
            .await
            .expect("Packing should succeed");

        let packed_str = Reflect::get(&packed_result, &JsValue::from_str("message"))
            .unwrap()
            .as_string()
            .unwrap();
        packed_messages.push(packed_str);
    }

    // Start all unpack operations concurrently
    let promise1 = agent.unpack_message(&packed_messages[0], None);
    let promise2 = agent.unpack_message(&packed_messages[1], None);
    let promise3 = agent.unpack_message(&packed_messages[2], None);

    // All should be Promises
    assert!(promise1.is_instance_of::<Promise>());
    assert!(promise2.is_instance_of::<Promise>());
    assert!(promise3.is_instance_of::<Promise>());

    // Await all promises
    let result1 = JsFuture::from(promise1).await;
    let result2 = JsFuture::from(promise2).await;
    let result3 = JsFuture::from(promise3).await;

    // All should succeed
    assert!(result1.is_ok(), "First unpack should succeed");
    assert!(result2.is_ok(), "Second unpack should succeed");
    assert!(result3.is_ok(), "Third unpack should succeed");
}

/// Test generate_key returns a Promise
#[wasm_bindgen_test]
async fn test_generate_key_returns_promise() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    let result = agent.generate_key("Ed25519");

    // Verify it's a Promise
    assert!(
        result.is_instance_of::<Promise>(),
        "generate_key should return a Promise"
    );

    // Await the promise
    let key_result = JsFuture::from(result).await;
    assert!(key_result.is_ok(), "Key generation should succeed");

    // Verify the result structure
    let key_obj = key_result.unwrap();
    assert!(Reflect::has(&key_obj, &JsValue::from_str("keyType")).unwrap());
    assert!(Reflect::has(&key_obj, &JsValue::from_str("agentDid")).unwrap());
}

/// Test process_message returns a Promise
#[wasm_bindgen_test]
async fn test_process_message_returns_promise() {
    let config = Object::new();
    let mut agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    // Register a handler
    let handler = js_sys::Function::new_no_args("return true;");
    agent.register_message_handler("TestMessage", handler);

    // Create a test message
    let message = Object::new();
    Reflect::set(
        &message,
        &JsValue::from_str("type"),
        &JsValue::from_str("TestMessage"),
    )
    .unwrap();

    let metadata = Object::new();

    let result = agent.process_message(message.into(), metadata.into());

    // Verify it's a Promise
    assert!(
        result.is_instance_of::<Promise>(),
        "process_message should return a Promise"
    );

    // Await the promise
    let process_result = JsFuture::from(result).await;
    assert!(process_result.is_ok(), "Message processing should succeed");
}

/// Test error propagation in async operations
#[wasm_bindgen_test]
async fn test_async_error_propagation() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    // Try to pack an invalid message
    let invalid_message = Object::new();
    // Missing required fields

    let pack_promise = agent.pack_message(invalid_message.into());
    let result = JsFuture::from(pack_promise).await;

    assert!(result.is_err(), "Packing invalid message should fail");

    // Try to unpack invalid data
    let unpack_promise = agent.unpack_message("invalid-jws-data", None);
    let result = JsFuture::from(unpack_promise).await;

    assert!(result.is_err(), "Unpacking invalid data should fail");
}

/// Test promise rejection handling
#[wasm_bindgen_test]
async fn test_promise_rejection_handling() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    // Generate key with invalid type should reject
    let result = agent.generate_key("InvalidKeyType");

    assert!(
        result.is_instance_of::<Promise>(),
        "Should still return a Promise"
    );

    let key_result = JsFuture::from(result).await;
    assert!(
        key_result.is_err(),
        "Promise should reject with invalid key type"
    );
}

// Helper function to create a test message
fn create_test_message(from_did: &str) -> Object {
    let message = Object::new();
    Reflect::set(
        &message,
        &JsValue::from_str("id"),
        &JsValue::from_str("test-msg"),
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
        &JsValue::from_str(from_did),
    )
    .unwrap();

    let to_array = Array::new();
    to_array.push(&JsValue::from_str("did:key:z6MkRecipient"));
    Reflect::set(&message, &JsValue::from_str("to"), &to_array).unwrap();

    let body = Object::new();
    Reflect::set(
        &body,
        &JsValue::from_str("amount"),
        &JsValue::from_str("100"),
    )
    .unwrap();
    Reflect::set(&message, &JsValue::from_str("body"), &body).unwrap();

    message
}
