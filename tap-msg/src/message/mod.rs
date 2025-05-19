//! Message types and processing for TAP messages.
//!
//! This module defines the message structures and types used in the
//! Transaction Authorization Protocol (TAP).

// Import all message modules
pub mod agent_management;
pub mod authorize;
pub mod cancel;
pub mod connection;
pub mod did_presentation;
pub mod error;
pub mod invoice;
pub mod participant;
pub mod payment;
pub mod policy;
pub mod presentation;
pub mod reject;
pub mod relationship;
pub mod revert;
pub mod settle;
pub mod tap_message_trait;
pub mod transfer;
pub mod update_party;
pub mod update_policies;
pub mod validation;

// Re-export agent management types
pub use agent_management::{AddAgents, RemoveAgent, ReplaceAgent};

// Re-export attachment types
pub use crate::didcomm::{Attachment, AttachmentData, SimpleAttachmentData};

// Re-export authorization types
pub use authorize::Authorize;

// Re-export cancel type
pub use cancel::Cancel;

// Re-export connection types
pub use connection::{
    AuthorizationRequired, Connect, ConnectionConstraints, OutOfBand, TransactionLimits,
};

// Re-export DIDComm presentation types
pub use did_presentation::DIDCommPresentation;

// Re-export error type
pub use error::ErrorBody;

// Re-export invoice types
pub use invoice::{
    DocumentReference, Invoice, LineItem, OrderReference, TaxCategory, TaxSubtotal, TaxTotal,
};

// Re-export participant types
pub use participant::Participant;

// Re-export payment types
pub use payment::{Payment, PaymentBuilder};

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

// Re-export the TapMessage trait and related functionality
pub use tap_message_trait::{create_tap_message, Connectable, TapMessage, TapMessageBody};
