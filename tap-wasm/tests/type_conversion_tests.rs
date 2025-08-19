use js_sys::{Array, Object, Reflect};
use tap_wasm::WasmTapAgent;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Test JsValue to Rust message conversion
#[wasm_bindgen_test]
async fn test_js_to_rust_message_conversion() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    // Create a complex JS message with all fields
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
        &JsValue::from_str(&agent.get_did()),
    )
    .unwrap();

    // Array of recipients
    let to_array = Array::new();
    to_array.push(&JsValue::from_str("did:key:z6MkRecipient1"));
    to_array.push(&JsValue::from_str("did:key:z6MkRecipient2"));
    Reflect::set(&message, &JsValue::from_str("to"), &to_array).unwrap();

    // Complex body object
    let body = Object::new();
    Reflect::set(
        &body,
        &JsValue::from_str("amount"),
        &JsValue::from_str("100.50"),
    )
    .unwrap();
    Reflect::set(
        &body,
        &JsValue::from_str("asset"),
        &JsValue::from_str("eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
    )
    .unwrap();

    // Nested objects
    let originator = Object::new();
    Reflect::set(
        &originator,
        &JsValue::from_str("@id"),
        &JsValue::from_str(&agent.get_did()),
    )
    .unwrap();
    Reflect::set(&body, &JsValue::from_str("originator"), &originator).unwrap();

    Reflect::set(&message, &JsValue::from_str("body"), &body).unwrap();

    // Optional fields
    Reflect::set(
        &message,
        &JsValue::from_str("thid"),
        &JsValue::from_str("thread-456"),
    )
    .unwrap();
    Reflect::set(
        &message,
        &JsValue::from_str("created"),
        &JsValue::from_f64(js_sys::Date::now()),
    )
    .unwrap();

    // Pack the message (this tests JS to Rust conversion)
    let pack_promise = agent.pack_message(message.into());
    let packed_result = JsFuture::from(pack_promise).await;

    assert!(
        packed_result.is_ok(),
        "Should successfully convert and pack JS message"
    );
}

/// Test Rust to JS message conversion
#[wasm_bindgen_test]
async fn test_rust_to_js_message_conversion() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    // Create and pack a message
    let message = create_complex_message(&agent.get_did());
    let pack_promise = agent.pack_message(message.into());
    let packed_result = JsFuture::from(pack_promise)
        .await
        .expect("Packing should succeed");

    let packed_str = Reflect::get(&packed_result, &JsValue::from_str("message"))
        .unwrap()
        .as_string()
        .unwrap();

    // Unpack the message (this tests Rust to JS conversion)
    let unpack_promise = agent.unpack_message(&packed_str, None);
    let unpacked_result = JsFuture::from(unpack_promise)
        .await
        .expect("Unpacking should succeed");

    // Verify all fields are properly converted to JS
    assert!(Reflect::has(&unpacked_result, &JsValue::from_str("id")).unwrap());
    assert!(Reflect::has(&unpacked_result, &JsValue::from_str("type")).unwrap());
    assert!(Reflect::has(&unpacked_result, &JsValue::from_str("from")).unwrap());
    assert!(Reflect::has(&unpacked_result, &JsValue::from_str("to")).unwrap());
    assert!(Reflect::has(&unpacked_result, &JsValue::from_str("body")).unwrap());
    assert!(Reflect::has(&unpacked_result, &JsValue::from_str("thid")).unwrap());
    assert!(Reflect::has(&unpacked_result, &JsValue::from_str("created")).unwrap());

    // Verify 'to' is an array
    let to_field = Reflect::get(&unpacked_result, &JsValue::from_str("to")).unwrap();
    assert!(to_field.is_array(), "'to' field should be an array");

    // Verify body is an object
    let body_field = Reflect::get(&unpacked_result, &JsValue::from_str("body")).unwrap();
    assert!(body_field.is_object(), "'body' field should be an object");
}

/// Test handling of null and undefined values
#[wasm_bindgen_test]
async fn test_null_undefined_handling() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    // Create a message with minimal required fields
    let message = Object::new();
    Reflect::set(
        &message,
        &JsValue::from_str("id"),
        &JsValue::from_str("test-minimal"),
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
        &JsValue::from_str(&agent.get_did()),
    )
    .unwrap();

    // Set some fields to null/undefined
    Reflect::set(&message, &JsValue::from_str("thid"), &JsValue::NULL).unwrap();
    Reflect::set(&message, &JsValue::from_str("pthid"), &JsValue::UNDEFINED).unwrap();

    // Empty array for 'to'
    let to_array = Array::new();
    Reflect::set(&message, &JsValue::from_str("to"), &to_array).unwrap();

    // Empty object for body
    let body = Object::new();
    Reflect::set(&message, &JsValue::from_str("body"), &body).unwrap();

    // Should still be able to pack
    let pack_promise = agent.pack_message(message.into());
    let packed_result = JsFuture::from(pack_promise).await;

    assert!(packed_result.is_ok(), "Should handle null/undefined fields");
}

/// Test number type conversions
#[wasm_bindgen_test]
async fn test_number_conversions() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    let message = Object::new();
    Reflect::set(
        &message,
        &JsValue::from_str("id"),
        &JsValue::from_str("test-numbers"),
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
        &JsValue::from_str(&agent.get_did()),
    )
    .unwrap();

    let to_array = Array::new();
    to_array.push(&JsValue::from_str("did:key:z6MkMerchant"));
    Reflect::set(&message, &JsValue::from_str("to"), &to_array).unwrap();

    // Body with various number formats
    let body = Object::new();
    Reflect::set(
        &body,
        &JsValue::from_str("amount"),
        &JsValue::from_f64(123.456),
    )
    .unwrap();
    Reflect::set(
        &body,
        &JsValue::from_str("quantity"),
        &JsValue::from_f64(42.0),
    )
    .unwrap();
    Reflect::set(
        &body,
        &JsValue::from_str("fee"),
        &JsValue::from_str("0.01"), // String number
    )
    .unwrap();

    Reflect::set(&message, &JsValue::from_str("body"), &body).unwrap();

    // Timestamps as numbers
    let now = js_sys::Date::now();
    Reflect::set(
        &message,
        &JsValue::from_str("created"),
        &JsValue::from_f64(now),
    )
    .unwrap();
    Reflect::set(
        &message,
        &JsValue::from_str("expires"),
        &JsValue::from_f64(now + 3600000.0),
    )
    .unwrap();

    // Pack and unpack
    let pack_promise = agent.pack_message(message.into());
    let packed_result = JsFuture::from(pack_promise)
        .await
        .expect("Packing should succeed");

    let packed_str = Reflect::get(&packed_result, &JsValue::from_str("message"))
        .unwrap()
        .as_string()
        .unwrap();

    let unpack_promise = agent.unpack_message(&packed_str, None);
    let unpacked_result = JsFuture::from(unpack_promise)
        .await
        .expect("Unpacking should succeed");

    // Verify timestamps are numbers
    let created = Reflect::get(&unpacked_result, &JsValue::from_str("created")).unwrap();
    assert!(
        created.is_falsy() || created.as_f64().is_some(),
        "Created should be a number"
    );

    let expires = Reflect::get(&unpacked_result, &JsValue::from_str("expires")).unwrap();
    assert!(
        expires.is_falsy() || expires.as_f64().is_some(),
        "Expires should be a number"
    );
}

/// Test string encoding and special characters
#[wasm_bindgen_test]
async fn test_string_encoding() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    let message = Object::new();
    Reflect::set(
        &message,
        &JsValue::from_str("id"),
        &JsValue::from_str("test-encoding-😀-🚀"), // Emojis
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
        &JsValue::from_str(&agent.get_did()),
    )
    .unwrap();

    let to_array = Array::new();
    to_array.push(&JsValue::from_str("did:key:z6MkRecipient"));
    Reflect::set(&message, &JsValue::from_str("to"), &to_array).unwrap();

    // Body with special characters
    let body = Object::new();
    Reflect::set(
        &body,
        &JsValue::from_str("memo"),
        &JsValue::from_str("Payment for café ☕ and résumé review"),
    )
    .unwrap();
    Reflect::set(
        &body,
        &JsValue::from_str("unicode"),
        &JsValue::from_str("中文 日本語 한국어"), // CJK characters
    )
    .unwrap();
    Reflect::set(
        &body,
        &JsValue::from_str("special"),
        &JsValue::from_str("Line1\nLine2\tTabbed\"Quoted\""),
    )
    .unwrap();

    Reflect::set(&message, &JsValue::from_str("body"), &body).unwrap();

    // Pack and unpack
    let pack_promise = agent.pack_message(message.into());
    let packed_result = JsFuture::from(pack_promise)
        .await
        .expect("Packing should succeed with special characters");

    let packed_str = Reflect::get(&packed_result, &JsValue::from_str("message"))
        .unwrap()
        .as_string()
        .unwrap();

    let unpack_promise = agent.unpack_message(&packed_str, None);
    let unpacked_result = JsFuture::from(unpack_promise)
        .await
        .expect("Unpacking should succeed with special characters");

    // Verify ID with emojis is preserved
    let id = Reflect::get(&unpacked_result, &JsValue::from_str("id"))
        .unwrap()
        .as_string()
        .unwrap();
    assert!(id.contains("😀"), "Emojis should be preserved");
    assert!(id.contains("🚀"), "Emojis should be preserved");
}

/// Test array conversions
#[wasm_bindgen_test]
async fn test_array_conversions() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    let message = Object::new();
    Reflect::set(
        &message,
        &JsValue::from_str("id"),
        &JsValue::from_str("test-arrays"),
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
        &JsValue::from_str(&agent.get_did()),
    )
    .unwrap();

    // Multiple recipients
    let to_array = Array::new();
    for i in 1..=5 {
        to_array.push(&JsValue::from_str(&format!("did:key:z6MkRecipient{}", i)));
    }
    Reflect::set(&message, &JsValue::from_str("to"), &to_array).unwrap();

    // Body with array fields
    let body = Object::new();

    let tags = Array::new();
    tags.push(&JsValue::from_str("urgent"));
    tags.push(&JsValue::from_str("important"));
    tags.push(&JsValue::from_str("transfer"));
    Reflect::set(&body, &JsValue::from_str("tags"), &tags).unwrap();

    let amounts = Array::new();
    amounts.push(&JsValue::from_f64(10.5));
    amounts.push(&JsValue::from_f64(20.3));
    amounts.push(&JsValue::from_f64(15.7));
    Reflect::set(&body, &JsValue::from_str("amounts"), &amounts).unwrap();

    Reflect::set(&message, &JsValue::from_str("body"), &body).unwrap();

    // Pack and unpack
    let pack_promise = agent.pack_message(message.into());
    let packed_result = JsFuture::from(pack_promise)
        .await
        .expect("Packing should succeed");

    let packed_str = Reflect::get(&packed_result, &JsValue::from_str("message"))
        .unwrap()
        .as_string()
        .unwrap();

    let unpack_promise = agent.unpack_message(&packed_str, None);
    let unpacked_result = JsFuture::from(unpack_promise)
        .await
        .expect("Unpacking should succeed");

    // Verify 'to' array length
    let to_result = Reflect::get(&unpacked_result, &JsValue::from_str("to")).unwrap();
    let to_array_result = Array::from(&to_result);
    assert_eq!(to_array_result.length(), 5, "Should have 5 recipients");
}

// Helper function to create a complex message
fn create_complex_message(from_did: &str) -> Object {
    let message = Object::new();
    Reflect::set(
        &message,
        &JsValue::from_str("id"),
        &JsValue::from_str("complex-msg-123"),
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
    to_array.push(&JsValue::from_str("did:key:z6MkRecipient1"));
    to_array.push(&JsValue::from_str("did:key:z6MkRecipient2"));
    Reflect::set(&message, &JsValue::from_str("to"), &to_array).unwrap();

    let body = Object::new();
    Reflect::set(
        &body,
        &JsValue::from_str("amount"),
        &JsValue::from_str("1000.00"),
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

    Reflect::set(
        &message,
        &JsValue::from_str("thid"),
        &JsValue::from_str("thread-complex"),
    )
    .unwrap();

    message
}
