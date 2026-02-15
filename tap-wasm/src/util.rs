use js_sys::{Array, Reflect};
use std::collections::HashMap;
use tap_msg::didcomm::PlainMessage;
use wasm_bindgen::prelude::*;

/// Converts a JavaScript message object to a TapMessageBody
pub fn js_to_tap_message(js_message: &JsValue) -> Result<PlainMessage, String> {
    // Extract message ID
    let id = match get_string_prop(js_message, "id") {
        Some(val) => val,
        None => return Err("Message is missing 'id' property".to_string()),
    };

    // Extract message type
    let type_ = match get_string_prop(js_message, "type") {
        Some(val) => val,
        None => return Err("Message is missing 'type' property".to_string()),
    };

    // Extract from DID
    let from = get_string_prop(js_message, "from").unwrap_or_default();

    // Extract to DIDs
    let to = if let Ok(to_array) = Reflect::get(js_message, &JsValue::from_str("to")) {
        if to_array.is_array() {
            let array = Array::from(&to_array);
            let mut to_dids = Vec::new();
            for i in 0..array.length() {
                if let Some(did) = array.get(i).as_string() {
                    to_dids.push(did);
                }
            }
            to_dids
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Extract body
    let body = if let Ok(body_js) = Reflect::get(js_message, &JsValue::from_str("body")) {
        if body_js.is_null() || body_js.is_undefined() {
            serde_json::json!({})
        } else {
            serde_wasm_bindgen::from_value(body_js)
                .map_err(|e| format!("Failed to deserialize body: {}", e))?
        }
    } else {
        serde_json::json!({})
    };

    // Extract thread ID
    let thid = get_string_prop(js_message, "thid");

    // Extract parent thread ID
    let pthid = get_string_prop(js_message, "pthid");

    // Extract created time
    let created_time =
        if let Ok(created_js) = Reflect::get(js_message, &JsValue::from_str("created")) {
            if created_js.is_null() || created_js.is_undefined() {
                Some(js_sys::Date::now() as u64 / 1000)
            } else if let Some(created) = created_js.as_f64() {
                Some(created as u64 / 1000)
            } else {
                Some(js_sys::Date::now() as u64 / 1000)
            }
        } else {
            Some(js_sys::Date::now() as u64 / 1000)
        };

    // Extract expires time
    let expires_time =
        if let Ok(expires_js) = Reflect::get(js_message, &JsValue::from_str("expires")) {
            if expires_js.is_null() || expires_js.is_undefined() {
                None
            } else {
                expires_js.as_f64().map(|expires| expires as u64 / 1000)
            }
        } else {
            None
        };

    // Create the PlainMessage
    Ok(PlainMessage {
        id,
        typ: "application/didcomm-plain+json".to_string(),
        type_,
        body,
        from,
        to,
        thid,
        pthid,
        extra_headers: HashMap::new(),
        created_time,
        expires_time,
        from_prior: None,
        attachments: None,
    })
}

/// Helper function to extract a string property from a JsValue
fn get_string_prop(js_obj: &JsValue, prop_name: &str) -> Option<String> {
    if let Ok(prop) = Reflect::get(js_obj, &JsValue::from_str(prop_name)) {
        prop.as_string()
    } else {
        None
    }
}

/// Converts Vec<u8> to a JavaScript Uint8Array
///
/// This function is kept for future binary message handling support
#[allow(dead_code)]
pub fn vec_u8_to_js_array(data: &[u8]) -> js_sys::Uint8Array {
    let array = js_sys::Uint8Array::new_with_length(data.len() as u32);
    array.copy_from(data);
    array
}

/// Converts a JavaScript Uint8Array to Vec<u8>
///
/// This function is kept for future binary message handling support
#[allow(dead_code)]
pub fn js_array_to_vec_u8(array: &js_sys::Uint8Array) -> Vec<u8> {
    let mut result = vec![0; array.length() as usize];
    array.copy_to(&mut result);
    result
}
