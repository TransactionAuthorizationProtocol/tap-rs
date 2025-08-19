#![cfg(target_arch = "wasm32")]

use js_sys::Object;
use tap_wasm::{generate_uuid_v4, MessageType, TapNode, WasmTapAgent};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_message_type_enum() {
    // Test that MessageType enum has expected values
    assert_eq!(MessageType::Transfer as u32, 0);
    assert_eq!(MessageType::Payment as u32, 1);
    assert_eq!(MessageType::Authorize as u32, 3);
    assert_eq!(MessageType::Reject as u32, 4);
}

#[wasm_bindgen_test]
fn test_uuid_generation() {
    let uuid1 = generate_uuid_v4();
    let uuid2 = generate_uuid_v4();

    // UUIDs should be different
    assert_ne!(uuid1, uuid2);

    // UUIDs should have proper format (36 characters with dashes)
    assert_eq!(uuid1.len(), 36);
    assert_eq!(uuid2.len(), 36);
}

#[wasm_bindgen_test]
fn test_wasm_tap_agent_creation() {
    let config = Object::new();
    js_sys::Reflect::set(
        &config,
        &JsValue::from_str("debug"),
        &JsValue::from_bool(true),
    )
    .unwrap();
    js_sys::Reflect::set(
        &config,
        &JsValue::from_str("nickname"),
        &JsValue::from_str("test-agent"),
    )
    .unwrap();

    let agent_result = WasmTapAgent::new(config.into());
    assert!(agent_result.is_ok());

    let agent = agent_result.unwrap();
    let did = agent.get_did();

    // DID should not be empty
    assert!(!did.is_empty());

    // Should have nickname
    let nickname = agent.nickname();
    assert_eq!(nickname, Some("test-agent".to_string()));
}

#[wasm_bindgen_test]
fn test_tap_node_creation() {
    let config = Object::new();
    js_sys::Reflect::set(
        &config,
        &JsValue::from_str("debug"),
        &JsValue::from_bool(false),
    )
    .unwrap();

    let node = TapNode::new(config.into());

    // Initially should have no agents
    let agents_list = node.list_agents();
    let agents_array = js_sys::Array::from(&agents_list);
    assert_eq!(agents_array.length(), 0);
}

#[wasm_bindgen_test]
fn test_agent_message_creation() {
    let config = Object::new();
    let agent = WasmTapAgent::new(config.into()).unwrap();

    let message = agent.create_message("https://tap.rsvp/schema/1.0#Transfer");

    // Message should have required properties
    let message_type = js_sys::Reflect::get(&message, &JsValue::from_str("type")).unwrap();
    assert_eq!(
        message_type.as_string().unwrap(),
        "https://tap.rsvp/schema/1.0#Transfer"
    );

    let from_did = js_sys::Reflect::get(&message, &JsValue::from_str("from")).unwrap();
    assert_eq!(from_did.as_string().unwrap(), agent.get_did());
}
