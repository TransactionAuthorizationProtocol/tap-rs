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
pub mod examples;

// Re-export public types for easier access
pub use error::{Error, Result};
pub use message::{
    create_tap_message, AddAgents, Attachment, AttachmentData, Authorize, ErrorBody, Participant,
    Presentation, Reject, Settle, TapMessageBody, Transfer, Validate,
};

// Conditional compilation for WASM targets
#[cfg(target_arch = "wasm32")]
pub mod wasm {
    //! WASM-specific functionality

    use wasm_bindgen::prelude::*;

    /// Initialize the WASM module.
    #[wasm_bindgen(start)]
    pub fn init() {
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
    }
}

// Test modules
#[cfg(test)]
mod tests {
    // Tests are now in the tests directory
}
