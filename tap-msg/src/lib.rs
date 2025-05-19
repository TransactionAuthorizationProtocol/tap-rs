//! Implementation of the Transaction Authorization Protocol (TAP)
//!
//! This crate provides the core functionality for the Transaction Authorization
//! Protocol (TAP), including message definitions, serialization, validation,
//! and DIDComm integration.
//!
//! The Transaction Authorization Protocol (TAP) is a multi-party protocol for
//! authorizing, documenting, and recording financial transactions for
//! cryptocurrency asset transfers.

// Internal modules
pub mod derive;
pub mod didcomm;
pub mod error;
pub mod examples;
pub mod message;
pub mod utils;

// Re-export public types for easier access
pub use didcomm::{Attachment, OutOfBand, PlainMessage};
pub use error::{Error, Result};
pub use message::{
    create_tap_message, AddAgents, Authorize, DocumentReference, ErrorBody, Invoice, LineItem,
    OrderReference, Participant, Payment, Presentation, Reject, Settle, TapMessageBody,
    TaxCategory, TaxSubtotal, TaxTotal, Transfer,
};

// Conditional compilation for WASM targets
#[cfg(target_arch = "wasm32")]
pub mod wasm {
    //! WASM-specific functionality

    use wasm_bindgen::prelude::*;

    /// Initialize the WASM module.
    #[wasm_bindgen(js_name = init_tap_msg)]
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
