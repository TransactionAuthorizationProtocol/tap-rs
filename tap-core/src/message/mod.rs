//! Message types and processing for TAP messages.
//!
//! This module defines the message structures and types used in the
//! Transaction Authorization Protocol (TAP).

pub mod types;
pub mod validation;

// Re-export all types from the types module
pub use types::*;
