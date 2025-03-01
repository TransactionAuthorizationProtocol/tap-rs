//! TAP Core - Core library for the Transaction Authorization Protocol
//!
//! This library provides the core functionality for working with
//! Transaction Authorization Protocol (TAP) messages, including
//! serialization, validation, and DIDComm integration.

// Re-export modules
pub mod didcomm;
pub mod error;
pub mod message;

// Re-export main types
pub use error::{Error, Result};
pub use message::TapMessage;

// Conditional compilation for WASM targets
#[cfg(target_arch = "wasm32")]
pub mod wasm {
    //! WASM-specific functionality
    //!
    //! This module provides functionality for using TAP Core in WebAssembly environments.

    use wasm_bindgen::prelude::*;

    /// Initialize the WASM module
    #[wasm_bindgen(start)]
    pub fn init() {
        // Set up panic hook for better debugging
        console_error_panic_hook::set_once();
    }
}
