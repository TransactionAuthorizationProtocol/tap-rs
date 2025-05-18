//! Message types and processing for TAP messages.
//!
//! This module defines the message structures and types used in the
//! Transaction Authorization Protocol (TAP).

// Import all message modules
pub mod agent_management;
pub mod authorize;
pub mod cancel;
pub mod error;
pub mod invoice;
pub mod policy;
pub mod presentation;
pub mod reject;
pub mod relationship;
pub mod revert;
pub mod settle;
pub mod tap_message_trait;
pub mod transfer;
pub mod types;  // Keep for backwards compatibility
pub mod update_party;
pub mod update_policies;
pub mod validation;

// Re-export agent management types
pub use agent_management::{AddAgents, RemoveAgent, ReplaceAgent};

// Re-export authorization types
pub use authorize::Authorize;

// Re-export cancel type
pub use cancel::Cancel;

// Re-export error type
pub use error::ErrorBody;

// Re-export invoice types
pub use invoice::{
    DocumentReference, Invoice, LineItem, OrderReference, TaxCategory, TaxSubtotal, TaxTotal,
};

// Re-export policy types
pub use policy::{Policy, RequireAuthorization, RequirePresentation, RequireProofOfControl};

// Re-export presentation types
pub use presentation::{Presentation, RequestPresentation};

// Re-export reject type
pub use reject::Reject;

// Re-export relationship type
pub use relationship::ConfirmRelationship;

// Re-export revert type
pub use revert::Revert;

// Re-export settle type
pub use settle::Settle;

// Re-export transfer types
pub use transfer::Transfer;

// Re-export update party type
pub use update_party::UpdateParty;

// Re-export update policies type
pub use update_policies::UpdatePolicies;

// Re-export common types from types.rs until fully migrated
pub use types::{AuthorizationRequired, Attachment, AttachmentData, Connect, ConnectionConstraints, 
    DIDCommPresentation, OutOfBand, Participant, Payment, PaymentBuilder, PaymentRequest, TransactionLimits};

// Re-export the TapMessage trait and related functionality
pub use tap_message_trait::{Connectable, TapMessage, TapMessageBody, create_tap_message};