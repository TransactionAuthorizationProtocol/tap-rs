//! Update Policies message type for the Transaction Authorization Protocol.
//!
//! This module defines the UpdatePolicies message type, which is used
//! to update policies in an existing transaction.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::Utc;

use crate::didcomm::PlainMessage;
use crate::error::{Error, Result};
use crate::impl_tap_message;
use crate::message::policy::Policy;
use crate::message::tap_message_trait::TapMessageBody;

/// UpdatePolicies message body (TAIP-7).
///
/// This message type allows agents to update their policies for a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePolicies {
    #[serde(rename = "transactionId")]
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

    /// Validates the UpdatePolicies message.
    pub fn validate(&self) -> Result<()> {
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

impl TapMessageBody for UpdatePolicies {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#update-policies"
    }

    fn validate(&self) -> Result<()> {
        self.validate()
    }

    fn to_didcomm(&self, from_did: Option<&str>) -> Result<PlainMessage> {
        // Serialize the UpdatePolicies to a JSON value
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        let now = Utc::now().timestamp() as u64;
        
        // The from field is required in our PlainMessage, so ensure we have a valid value
        let from = from_did.map_or_else(String::new, |s| s.to_string());

        // Create a new Message with required fields
        let message = PlainMessage {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body: body_json,
            from,
            to: Vec::new(),
            thid: Some(self.transaction_id.clone()),
            pthid: None,
            created_time: Some(now),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
        };

        Ok(message)
    }
}

impl_tap_message!(UpdatePolicies);