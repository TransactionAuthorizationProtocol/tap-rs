//! Message types and processing for TAP messages.
//!
//! This module defines the message structures and types used in the
//! Transaction Authorization Protocol (TAP).

pub mod types;
pub mod validation;
pub mod tap_message_trait;

// Re-export specific types to avoid ambiguity
pub use types::{
    AddAgentsBody, Agent, Attachment, AttachmentData, AuthorizeBody, ErrorBody,
    PresentationBody, RejectBody, RequestPresentationBody, SettleBody, 
    TapMessageType, TransferBody, Validate
};

// Re-export the TapMessage trait and related functionality
pub use tap_message_trait::{
    TapMessage, TapMessageBody, create_tap_message
};
