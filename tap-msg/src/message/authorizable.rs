//! Authorizable trait implementation for various TAP message types.
//!
//! This module defines the Authorizable trait, which allows message types
//! to be authorized, and implementations for relevant TAP message types.

use crate::message::policy::Policy;
use crate::message::{Authorize, Participant, RemoveAgent, ReplaceAgent, Transfer, UpdatePolicies};

/// Authorizable trait for types that can be authorized or can generate authorization-related messages.
pub trait Authorizable {
    /// Create an Authorize message for this object.
    fn authorize(&self, note: Option<String>) -> Authorize;

    /// Create an UpdatePolicies message for this object.
    fn update_policies(&self, transaction_id: String, policies: Vec<Policy>) -> UpdatePolicies;

    /// Create a ReplaceAgent message for this object.
    fn replace_agent(
        &self,
        transaction_id: String,
        original_agent: String,
        replacement: Participant,
    ) -> ReplaceAgent;

    /// Create a RemoveAgent message for this object.
    fn remove_agent(&self, transaction_id: String, agent: String) -> RemoveAgent;
}

impl Authorizable for Transfer {
    fn authorize(&self, note: Option<String>) -> Authorize {
        Authorize {
            transaction_id: self.transaction_id.clone(),
            note,
        }
    }

    fn update_policies(&self, transaction_id: String, policies: Vec<Policy>) -> UpdatePolicies {
        UpdatePolicies {
            transaction_id,
            policies,
        }
    }

    fn replace_agent(
        &self,
        transaction_id: String,
        original_agent: String,
        replacement: Participant,
    ) -> ReplaceAgent {
        ReplaceAgent {
            transaction_id,
            original: original_agent,
            replacement,
        }
    }

    fn remove_agent(&self, transaction_id: String, agent: String) -> RemoveAgent {
        RemoveAgent {
            transaction_id,
            agent,
        }
    }
}
