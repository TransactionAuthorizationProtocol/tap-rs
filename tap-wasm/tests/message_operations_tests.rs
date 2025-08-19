use js_sys::{Array, Object, Reflect};
use tap_wasm::WasmTapAgent;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Test that pack_message properly delegates to TapAgent
#[wasm_bindgen_test]
async fn test_pack_message_delegation() {
    // Create an agent
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    // Create a simple Transfer message
    let message = create_transfer_message(&agent.get_did());

    // Pack the message
    let pack_promise = agent.pack_message(message.into());
    let packed_result = JsFuture::from(pack_promise).await;

    assert!(packed_result.is_ok(), "Message packing should succeed");

    // Verify the result has the expected structure
    let packed = packed_result.unwrap();
    assert!(Reflect::has(&packed, &JsValue::from_str("message")).unwrap());
    assert!(Reflect::has(&packed, &JsValue::from_str("metadata")).unwrap());

    // Get the packed message string
    let message_js = Reflect::get(&packed, &JsValue::from_str("message")).unwrap();
    let message_str = message_js.as_string().expect("Message should be a string");

    // Verify it's not empty
    assert!(
        !message_str.is_empty(),
        "Packed message should not be empty"
    );

    // The message should be in General JWS JSON format (DIDComm v2)
    // Parse it as JSON to verify structure
    let parsed: serde_json::Value =
        serde_json::from_str(&message_str).expect("Packed message should be valid JSON");

    // For signed messages, it should have payload and signatures fields
    assert!(
        parsed.get("payload").is_some(),
        "JWS should have payload field"
    );
    assert!(
        parsed.get("signatures").is_some(),
        "JWS should have signatures field"
    );
}

/// Test that unpack_message properly delegates to TapAgent
#[wasm_bindgen_test]
async fn test_unpack_message_delegation() {
    // Create a single agent that will both pack and unpack
    // This tests the basic pack/unpack delegation
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    // Create a message
    let message = create_transfer_message(&agent.get_did());

    // Pack the message
    let pack_promise = agent.pack_message(message.into());
    let packed_result = JsFuture::from(pack_promise)
        .await
        .expect("Packing should succeed");

    // Extract the packed message string
    let packed_message = Reflect::get(&packed_result, &JsValue::from_str("message"))
        .unwrap()
        .as_string()
        .expect("Should have message string");

    // Unpack the message with the same agent
    // For signed messages, any agent can unpack/verify if they have access to the sender's public key
    // Since we're using the same agent, it has access to its own public key
    let unpack_promise = agent.unpack_message(&packed_message, None);
    let unpacked_result = JsFuture::from(unpack_promise).await;

    // Log the error if unpacking failed
    if unpacked_result.is_err() {
        web_sys::console::error_1(&JsValue::from_str(&format!(
            "Failed to unpack message: {:?}",
            unpacked_result.as_ref().err()
        )));
    }

    assert!(unpacked_result.is_ok(), "Message unpacking should succeed");

    // Verify the unpacked message structure
    let unpacked = unpacked_result.unwrap();
    assert!(Reflect::has(&unpacked, &JsValue::from_str("id")).unwrap());
    assert!(Reflect::has(&unpacked, &JsValue::from_str("type")).unwrap());
    assert!(Reflect::has(&unpacked, &JsValue::from_str("from")).unwrap());
    assert!(Reflect::has(&unpacked, &JsValue::from_str("to")).unwrap());
    assert!(Reflect::has(&unpacked, &JsValue::from_str("body")).unwrap());
}

/// Test packing and unpacking Transfer messages
#[wasm_bindgen_test]
async fn test_transfer_message() {
    test_message_type("Transfer", create_transfer_message).await;
}

/// Test packing and unpacking Payment messages
#[wasm_bindgen_test]
async fn test_payment_message() {
    test_message_type("Payment", create_payment_message).await;
}

/// Test packing and unpacking Authorize messages
#[wasm_bindgen_test]
async fn test_authorize_message() {
    test_message_type("Authorize", create_authorize_message).await;
}

/// Test packing and unpacking Reject messages
#[wasm_bindgen_test]
async fn test_reject_message() {
    test_message_type("Reject", create_reject_message).await;
}

/// Test packing and unpacking Cancel messages
#[wasm_bindgen_test]
async fn test_cancel_message() {
    test_message_type("Cancel", create_cancel_message).await;
}

/// Test error handling for invalid message
#[wasm_bindgen_test]
async fn test_invalid_message_error() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    // Create an invalid message (missing required fields)
    let invalid_message = Object::new();
    Reflect::set(
        &invalid_message,
        &JsValue::from_str("type"),
        &JsValue::from_str("Invalid"),
    )
    .unwrap();

    // Try to pack the invalid message
    let pack_promise = agent.pack_message(invalid_message.into());
    let result = JsFuture::from(pack_promise).await;

    // Should fail
    assert!(result.is_err(), "Packing invalid message should fail");
}

/// Test error handling for corrupted packed message
#[wasm_bindgen_test]
async fn test_corrupted_message_error() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    // Try to unpack corrupted message
    let corrupted = "not.a.valid.jws.message";
    let unpack_promise = agent.unpack_message(corrupted, None);
    let result = JsFuture::from(unpack_promise).await;

    // Should fail
    assert!(result.is_err(), "Unpacking corrupted message should fail");
}

/// Test unpacking with expected type validation
#[wasm_bindgen_test]
async fn test_unpack_with_type_validation() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    // Create and pack a Transfer message
    let message = create_transfer_message(&agent.get_did());
    let pack_promise = agent.pack_message(message.into());
    let packed_result = JsFuture::from(pack_promise)
        .await
        .expect("Packing should succeed");

    let packed_message = Reflect::get(&packed_result, &JsValue::from_str("message"))
        .unwrap()
        .as_string()
        .unwrap();

    // Unpack expecting correct type - should succeed
    let unpack_promise = agent.unpack_message(
        &packed_message,
        Some("https://tap.rsvp/schema/1.0#Transfer".to_string()),
    );
    let result = JsFuture::from(unpack_promise).await;
    assert!(result.is_ok(), "Should unpack with correct expected type");

    // Unpack expecting wrong type - should fail
    let unpack_promise2 = agent.unpack_message(
        &packed_message,
        Some("https://tap.rsvp/schema/1.0#Payment".to_string()),
    );
    let result2 = JsFuture::from(unpack_promise2).await;
    assert!(result2.is_err(), "Should fail with wrong expected type");
}

// Helper function to test a specific message type
async fn test_message_type<F>(message_type: &str, create_fn: F)
where
    F: Fn(&str) -> Object,
{
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    // Create message
    let message = create_fn(&agent.get_did());

    // Pack the message
    let pack_promise = agent.pack_message(message.clone().into());
    let packed_result = JsFuture::from(pack_promise).await;
    assert!(
        packed_result.is_ok(),
        "Should pack {} message",
        message_type
    );

    // Get packed message
    let packed = packed_result.unwrap();
    let packed_str = Reflect::get(&packed, &JsValue::from_str("message"))
        .unwrap()
        .as_string()
        .unwrap();

    // Unpack the message
    let unpack_promise = agent.unpack_message(&packed_str, None);
    let unpacked_result = JsFuture::from(unpack_promise).await;
    assert!(
        unpacked_result.is_ok(),
        "Should unpack {} message",
        message_type
    );

    // Verify the type is preserved
    let unpacked = unpacked_result.unwrap();
    let unpacked_type = Reflect::get(&unpacked, &JsValue::from_str("type"))
        .unwrap()
        .as_string()
        .unwrap();

    let expected_type = format!("https://tap.rsvp/schema/1.0#{}", message_type);
    assert_eq!(
        unpacked_type, expected_type,
        "Message type should be preserved"
    );
}

// Helper functions to create different message types
fn create_transfer_message(from_did: &str) -> Object {
    let message = Object::new();
    Reflect::set(
        &message,
        &JsValue::from_str("id"),
        &JsValue::from_str("test-transfer-123"),
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

    // Create Transfer body
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

    let originator = Object::new();
    Reflect::set(
        &originator,
        &JsValue::from_str("@id"),
        &JsValue::from_str(from_did),
    )
    .unwrap();
    Reflect::set(&body, &JsValue::from_str("originator"), &originator).unwrap();

    let beneficiary = Object::new();
    Reflect::set(
        &beneficiary,
        &JsValue::from_str("@id"),
        &JsValue::from_str("did:key:z6MkBeneficiary"),
    )
    .unwrap();
    Reflect::set(&body, &JsValue::from_str("beneficiary"), &beneficiary).unwrap();

    Reflect::set(&message, &JsValue::from_str("body"), &body).unwrap();
    message
}

fn create_payment_message(from_did: &str) -> Object {
    let message = Object::new();
    Reflect::set(
        &message,
        &JsValue::from_str("id"),
        &JsValue::from_str("test-payment-123"),
    )
    .unwrap();
    Reflect::set(
        &message,
        &JsValue::from_str("type"),
        &JsValue::from_str("https://tap.rsvp/schema/1.0#Payment"),
    )
    .unwrap();
    Reflect::set(
        &message,
        &JsValue::from_str("from"),
        &JsValue::from_str(from_did),
    )
    .unwrap();

    let to_array = Array::new();
    to_array.push(&JsValue::from_str("did:key:z6MkMerchant"));
    Reflect::set(&message, &JsValue::from_str("to"), &to_array).unwrap();

    // Create Payment body
    let body = Object::new();
    Reflect::set(
        &body,
        &JsValue::from_str("amount"),
        &JsValue::from_str("50.0"),
    )
    .unwrap();
    Reflect::set(
        &body,
        &JsValue::from_str("currency"),
        &JsValue::from_str("USD"),
    )
    .unwrap();

    let merchant = Object::new();
    Reflect::set(
        &merchant,
        &JsValue::from_str("@id"),
        &JsValue::from_str("did:key:z6MkMerchant"),
    )
    .unwrap();
    Reflect::set(&body, &JsValue::from_str("merchant"), &merchant).unwrap();

    Reflect::set(&message, &JsValue::from_str("body"), &body).unwrap();
    message
}

fn create_authorize_message(from_did: &str) -> Object {
    let message = Object::new();
    Reflect::set(
        &message,
        &JsValue::from_str("id"),
        &JsValue::from_str("test-authorize-123"),
    )
    .unwrap();
    Reflect::set(
        &message,
        &JsValue::from_str("type"),
        &JsValue::from_str("https://tap.rsvp/schema/1.0#Authorize"),
    )
    .unwrap();
    Reflect::set(
        &message,
        &JsValue::from_str("from"),
        &JsValue::from_str(from_did),
    )
    .unwrap();

    let to_array = Array::new();
    to_array.push(&JsValue::from_str("did:key:z6MkOriginator"));
    Reflect::set(&message, &JsValue::from_str("to"), &to_array).unwrap();

    // Create Authorize body
    let body = Object::new();
    Reflect::set(
        &body,
        &JsValue::from_str("transaction_id"),
        &JsValue::from_str("tx-123"),
    )
    .unwrap();

    Reflect::set(&message, &JsValue::from_str("body"), &body).unwrap();
    message
}

fn create_reject_message(from_did: &str) -> Object {
    let message = Object::new();
    Reflect::set(
        &message,
        &JsValue::from_str("id"),
        &JsValue::from_str("test-reject-123"),
    )
    .unwrap();
    Reflect::set(
        &message,
        &JsValue::from_str("type"),
        &JsValue::from_str("https://tap.rsvp/schema/1.0#Reject"),
    )
    .unwrap();
    Reflect::set(
        &message,
        &JsValue::from_str("from"),
        &JsValue::from_str(from_did),
    )
    .unwrap();

    let to_array = Array::new();
    to_array.push(&JsValue::from_str("did:key:z6MkOriginator"));
    Reflect::set(&message, &JsValue::from_str("to"), &to_array).unwrap();

    // Create Reject body
    let body = Object::new();
    Reflect::set(
        &body,
        &JsValue::from_str("transaction_id"),
        &JsValue::from_str("tx-123"),
    )
    .unwrap();
    Reflect::set(
        &body,
        &JsValue::from_str("reason"),
        &JsValue::from_str("Insufficient funds"),
    )
    .unwrap();

    Reflect::set(&message, &JsValue::from_str("body"), &body).unwrap();
    message
}

fn create_cancel_message(from_did: &str) -> Object {
    let message = Object::new();
    Reflect::set(
        &message,
        &JsValue::from_str("id"),
        &JsValue::from_str("test-cancel-123"),
    )
    .unwrap();
    Reflect::set(
        &message,
        &JsValue::from_str("type"),
        &JsValue::from_str("https://tap.rsvp/schema/1.0#Cancel"),
    )
    .unwrap();
    Reflect::set(
        &message,
        &JsValue::from_str("from"),
        &JsValue::from_str(from_did),
    )
    .unwrap();

    let to_array = Array::new();
    to_array.push(&JsValue::from_str("did:key:z6MkCounterparty"));
    Reflect::set(&message, &JsValue::from_str("to"), &to_array).unwrap();

    // Create Cancel body
    let body = Object::new();
    Reflect::set(
        &body,
        &JsValue::from_str("transaction_id"),
        &JsValue::from_str("tx-123"),
    )
    .unwrap();
    Reflect::set(
        &body,
        &JsValue::from_str("by"),
        &JsValue::from_str(from_did),
    )
    .unwrap();

    Reflect::set(&message, &JsValue::from_str("body"), &body).unwrap();
    message
}
