//! WebAssembly bindings for the Transaction Authorization Protocol
//!
//! This crate provides WebAssembly bindings for the TAP agent, allowing it to be used in
//! browser and other JavaScript environments. It wraps the tap-agent crate's functionality
//! with JavaScript-friendly interfaces.

mod util;
mod wasm_agent;

use tap_agent::did::KeyType as TapKeyType;
use wasm_bindgen::prelude::*;

pub use wasm_agent::WasmTapAgent;

// Use wee_alloc as the global allocator to reduce WASM binary size
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Set up panic hook for better error messages in browser
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    Ok(())
}

// MessageType enum removed - TypeScript handles message types natively
// Key type enumeration removed - we use strings directly to avoid wasm_bindgen issues with conditional compilation

// TapNode removed - tap-node functionality not needed in browser WASM

/// Generates a UUID v4
#[wasm_bindgen]
pub fn generate_uuid_v4() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Generates a new private key for the specified key type
#[wasm_bindgen(js_name = generatePrivateKey)]
pub fn generate_private_key(key_type_str: String) -> Result<String, JsValue> {
    #[cfg(any(
        feature = "crypto-ed25519",
        feature = "crypto-p256",
        feature = "crypto-secp256k1"
    ))]
    {
        use tap_agent::did::{DIDGenerationOptions, DIDKeyGenerator};

        // Convert key type string to KeyType enum
        let key_type = match key_type_str.as_str() {
            #[cfg(feature = "crypto-ed25519")]
            "Ed25519" => TapKeyType::Ed25519,
            #[cfg(feature = "crypto-p256")]
            "P256" => TapKeyType::P256,
            #[cfg(feature = "crypto-secp256k1")]
            "Secp256k1" => TapKeyType::Secp256k1,
            _ => {
                return Err(JsValue::from_str(&format!(
                    "Invalid or disabled key type: {}",
                    key_type_str
                )))
            }
        };

        // Generate a new key
        let generator = DIDKeyGenerator::new();
        let options = DIDGenerationOptions { key_type };
        let generated_key = generator
            .generate_did(options)
            .map_err(|e| JsValue::from_str(&format!("Failed to generate key: {}", e)))?;

        // Return the private key as hex string
        Ok(hex::encode(&generated_key.private_key))
    }

    #[cfg(not(any(
        feature = "crypto-ed25519",
        feature = "crypto-p256",
        feature = "crypto-secp256k1"
    )))]
    {
        Err(JsValue::from_str("No cryptographic features enabled"))
    }
}

/// Alias for generate_uuid_v4 to match the PRD specification
#[wasm_bindgen(js_name = generateUUID)]
pub fn generate_uuid() -> String {
    generate_uuid_v4()
}
