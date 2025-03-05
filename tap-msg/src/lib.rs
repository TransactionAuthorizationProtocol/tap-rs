//! Implementation of the Transaction Authorization Protocol (TAP)
//!
//! This crate provides the core functionality for the Transaction Authorization
//! Protocol (TAP), including message definitions, serialization, validation,
//! and DIDComm integration.
//!
//! The Transaction Authorization Protocol (TAP) is a multi-party protocol for
//! authorizing, documenting, and recording financial transactions for
//! cryptocurrency asset transfers.

// Re-export the didcomm crate for convenience to users of tap-msg
pub use didcomm;

// Internal modules
pub mod error;
pub mod message;
pub mod utils;

// Re-export public types for easier access
pub use error::{Error, Result};
pub use message::{
    AddAgents, Participant, Attachment, AttachmentData, Authorize, ErrorBody,
    Presentation, Reject, RequestPresentation, Settle, 
    TapMessageType, Transfer, Validate, TapMessage, TapMessageBody, create_tap_message
};

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
