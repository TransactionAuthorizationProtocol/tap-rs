#![cfg(target_arch = "wasm32")]

use js_sys::Object;
use tap_wasm::{generate_uuid_v4, WasmTapAgent};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// MessageType enum test removed - no longer part of WASM API

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

// TapNode test removed - no longer part of WASM API

// Message creation test removed - TypeScript handles message creation
