//! Message types and utilities for the TAP Agent.
//!
//! This module provides constants and types for working with TAP messages,
//! including security modes and message type identifiers.

/// Security mode for message packing and unpacking.
///
/// Defines the level of protection applied to messages:
/// - `Plain`: No encryption or signing (insecure, only for testing)
/// - `Signed`: Message is signed but not encrypted (integrity protected)
/// - `AuthCrypt`: Message is authenticated and encrypted (confidentiality + integrity)
/// - `Any`: Accept any security mode when unpacking (only used for receiving)
#[derive(Debug, Clone, Copy)]
pub enum SecurityMode {
    /// Plaintext - no encryption or signatures
    Plain,
    /// Signed - message is signed but not encrypted
    Signed,
    /// Authenticated and Encrypted - message is both signed and encrypted
    AuthCrypt,
    /// Any security mode - used for unpacking when any mode is acceptable
    Any,
}

/// Message type identifiers used by the TAP Protocol
/// These constant strings are used to identify different message types
/// in the TAP protocol communications.
/// Type identifier for Presentation messages
pub const PRESENTATION_MESSAGE_TYPE: &str = "https://tap.rsvp/schema/1.0#Presentation";
