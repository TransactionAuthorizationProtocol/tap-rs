use js_sys::{Array, Object, Reflect};
use tap_wasm::WasmTapAgent;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Simple test to debug pack_message
#[wasm_bindgen_test]
async fn test_simple_pack() {
    // Create an agent with debug enabled
    let config = Object::new();
    Reflect::set(&config, &JsValue::from_str("debug"), &JsValue::TRUE).unwrap();
    let agent = WasmTapAgent::new(config.into()).expect("Failed to create agent");

    // Create the simplest possible message
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

    let to_array = Array::new();
    to_array.push(&JsValue::from_str("did:key:z6MkTest"));
    Reflect::set(&message, &JsValue::from_str("to"), &to_array).unwrap();

    // Minimal body
    let body = Object::new();
    Reflect::set(
        &body,
        &JsValue::from_str("test"),
        &JsValue::from_str("value"),
    )
    .unwrap();
    Reflect::set(&message, &JsValue::from_str("body"), &body).unwrap();

    // Try to pack
    web_sys::console::log_1(&JsValue::from_str("Starting pack_message..."));
    let pack_promise = agent.pack_message(message.into());
    let packed_result = JsFuture::from(pack_promise).await;

    // Check result
    match &packed_result {
        Ok(val) => {
            web_sys::console::log_1(&JsValue::from_str("Pack succeeded!"));

            // Check if we have a message field
            if Reflect::has(val, &JsValue::from_str("message")).unwrap() {
                let msg = Reflect::get(val, &JsValue::from_str("message")).unwrap();
                web_sys::console::log_1(&JsValue::from_str(&format!(
                    "Message field exists, type: {:?}, is_string: {}",
                    msg.js_typeof().as_string(),
                    msg.is_string()
                )));

                if let Some(msg_str) = msg.as_string() {
                    web_sys::console::log_1(&JsValue::from_str(&format!(
                        "Packed message: length={}, first_100_chars={}",
                        msg_str.len(),
                        &msg_str.chars().take(100).collect::<String>()
                    )));
                }
            }
        }
        Err(e) => {
            web_sys::console::error_1(&JsValue::from_str(&format!("Pack failed: {:?}", e)));
        }
    }

    assert!(packed_result.is_ok(), "Packing should succeed");
}
