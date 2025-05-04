//! Message types and processing for TAP messages.
//!
//! This module defines the message structures and types used in the
//! Transaction Authorization Protocol (TAP).

pub mod invoice;
pub mod policy;
pub mod tap_message_trait;
pub mod types;
pub mod validation;

// Re-export specific types to avoid ambiguity
pub use types::{
    AddAgents, Attachment, AttachmentData, AuthorizationRequired, Authorize, Connect,
    ConnectionConstraints, DIDCommPresentation, ErrorBody, OutOfBand, Participant, PaymentRequest,
    Presentation, Reject, RemoveAgent, ReplaceAgent, Settle, TransactionLimits, Transfer,
    UpdatePolicies, Validate,
};

// Re-export invoice types
pub use invoice::{
    DocumentReference, Invoice, LineItem, OrderReference, TaxCategory, TaxSubtotal, TaxTotal,
};

// Re-export policy types
pub use policy::{Policy, RequireAuthorization, RequirePresentation, RequireProofOfControl};

// Re-export the TapMessage trait and related functionality
pub use tap_message_trait::{create_tap_message, TapMessage, TapMessageBody};
