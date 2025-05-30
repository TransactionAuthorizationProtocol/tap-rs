//! Update Policies message type for the Transaction Authorization Protocol.
//!
//! This module defines the UpdatePolicies message type, which is used
//! to update policies in an existing transaction.

use crate::error::{Error, Result};
use crate::message::policy::Policy;
use crate::TapMessage;
use serde::{Deserialize, Serialize};

/// UpdatePolicies message body (TAIP-7).
///
/// This message type allows agents to update their policies for a transaction.
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
#[tap(message_type = "https://tap.rsvp/schema/1.0#UpdatePolicies", custom_validation)]
pub struct UpdatePolicies {
    #[tap(thread_id)]
    pub transaction_id: String,

    pub policies: Vec<Policy>,
}

impl UpdatePolicies {
    /// Creates a new UpdatePolicies message.
    pub fn new(transaction_id: &str, policies: Vec<Policy>) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            policies,
        }
    }

    /// Custom validation for UpdatePolicies messages
    pub fn validate_updatepolicies(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "UpdatePolicies must have a transaction_id".to_string(),
            ));
        }

        if self.policies.is_empty() {
            return Err(Error::Validation(
                "UpdatePolicies must have at least one policy".to_string(),
            ));
        }

        for policy in &self.policies {
            policy.validate()?;
        }

        Ok(())
    }
}


