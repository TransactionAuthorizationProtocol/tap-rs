//! Connection types for TAP messages.
//!
//! This module defines the structure of connection messages and related types
//! used in the Transaction Authorization Protocol (TAP).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::didcomm::PlainMessage;
use crate::error::{Error, Result};
use crate::impl_tap_message;
use crate::message::tap_message_trait::{Connectable, TapMessageBody};
use chrono::Utc;

/// Transaction limits for connection constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLimits {
    /// Maximum amount for a transaction.
    pub max_amount: Option<String>,

    /// Maximum total amount for all transactions.
    pub max_total_amount: Option<String>,

    /// Maximum number of transactions allowed.
    pub max_transactions: Option<u64>,
}

/// Connection constraints for the Connect message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConstraints {
    /// Limit on transaction amount.
    pub transaction_limits: Option<TransactionLimits>,
}

/// Connect message body (TAIP-2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connect {
    /// Transaction ID.
    pub transaction_id: String,

    /// Agent DID.
    pub agent_id: String,

    /// The entity this connection is for.
    #[serde(rename = "for")]
    pub for_: String,

    /// The role of the agent (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Connection constraints (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<ConnectionConstraints>,
}

impl Connect {
    /// Create a new Connect message.
    pub fn new(transaction_id: &str, agent_id: &str, for_id: &str, role: Option<&str>) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            agent_id: agent_id.to_string(),
            for_: for_id.to_string(),
            role: role.map(|s| s.to_string()),
            constraints: None,
        }
    }

    /// Add constraints to the Connect message.
    pub fn with_constraints(mut self, constraints: ConnectionConstraints) -> Self {
        self.constraints = Some(constraints);
        self
    }
}

impl Connectable for Connect {
    fn with_connection(&mut self, _connect_id: &str) -> &mut Self {
        // Connect messages don't have a connection ID
        self
    }

    fn has_connection(&self) -> bool {
        false
    }

    fn connection_id(&self) -> Option<&str> {
        None
    }
}

impl TapMessageBody for Connect {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#connect"
    }

    fn validate(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation("transaction_id is required".to_string()));
        }
        if self.agent_id.is_empty() {
            return Err(Error::Validation("agent_id is required".to_string()));
        }
        if self.for_.is_empty() {
            return Err(Error::Validation("for is required".to_string()));
        }
        Ok(())
    }

    fn to_didcomm(&self, from_did: &str) -> Result<PlainMessage> {
        // 1. Serialize self to JSON value
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // 2. Add/ensure '@type' field
        if let Some(body_obj) = body_json.as_object_mut() {
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
            // Note: serde handles #[serde(rename = "for")] automatically during serialization
        }

        // 3. Generate ID and timestamp
        let id = uuid::Uuid::new_v4().to_string(); // Use new_v4 as per workspace UUID settings
        let created_time = Utc::now().timestamp_millis() as u64;

        // 4. Explicitly set the recipient using agent_id
        let to = vec![self.agent_id.clone()];

        // 5. Create the Message struct
        let message = PlainMessage {
            id,
            typ: "application/didcomm-plain+json".to_string(), // Standard type
            type_: Self::message_type().to_string(),
            from: from_did.to_string(),
            to, // Use the explicitly set 'to' field
            thid: Some(self.transaction_id.clone()),
            pthid: None, // Parent Thread ID usually set later
            created_time: Some(created_time),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
            body: body_json,
            attachments: None,
        };

        Ok(message)
    }

    fn from_didcomm(message: &PlainMessage) -> Result<Self> {
        let body = message
            .body
            .as_object()
            .ok_or_else(|| Error::Validation("Message body is not a JSON object".to_string()))?;

        let transfer_id = body
            .get("transaction_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Validation("Missing or invalid transaction_id".to_string()))?;

        let agent_id = body
            .get("agent_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Validation("Missing or invalid agent_id".to_string()))?;

        let for_id = body
            .get("for")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Validation("Missing or invalid for".to_string()))?;

        let role = body
            .get("role")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        let constraints = if let Some(constraints_value) = body.get("constraints") {
            if constraints_value.is_null() {
                None
            } else {
                // Parse constraints
                let constraints_json = serde_json::to_value(constraints_value).map_err(|e| {
                    Error::SerializationError(format!("Invalid constraints: {}", e))
                })?;

                Some(serde_json::from_value(constraints_json).map_err(|e| {
                    Error::SerializationError(format!("Invalid constraints format: {}", e))
                })?)
            }
        } else {
            None
        };

        Ok(Connect {
            transaction_id: transfer_id.to_string(),
            agent_id: agent_id.to_string(),
            for_: for_id.to_string(),
            role,
            constraints,
        })
    }
}

impl_tap_message!(Connect);

impl TapMessageBody for AuthorizationRequired {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#authorizationrequired"
    }

    fn validate(&self) -> Result<()> {
        if self.url.is_empty() {
            return Err(Error::Validation("Authorization URL is required".to_string()));
        }

        // Validate expiry date if present
        if let Some(expires) = self.metadata.get("expires") {
            if let Some(expires_str) = expires.as_str() {
                // Simple format check
                if !expires_str.contains('T') || !expires_str.contains(':') {
                    return Err(Error::Validation("Invalid expiry date format. Expected ISO8601/RFC3339 format".to_string()));
                }
            }
        }

        Ok(())
    }

    fn to_didcomm(&self, from: &str) -> Result<PlainMessage> {
        // Serialize to JSON
        let mut body_json = serde_json::to_value(self)
            .map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        let now = Utc::now().timestamp() as u64;

        // Create a new Message with required fields
        let message = PlainMessage {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body: body_json,
            from: from.to_string(),
            to: Vec::new(), // Recipients will be set separately
            thid: None,
            pthid: None,
            created_time: Some(now),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
            attachments: None,
        };

        Ok(message)
    }

    fn from_didcomm(message: &PlainMessage) -> Result<Self> {
        // Validate message type
        if message.type_ != Self::message_type() {
            return Err(Error::InvalidMessageType(format!(
                "Expected {} but got {}",
                Self::message_type(),
                message.type_
            )));
        }

        // Extract fields from message body
        let auth_req: AuthorizationRequired = serde_json::from_value(message.body.clone())
            .map_err(|e| Error::SerializationError(e.to_string()))?;

        Ok(auth_req)
    }
}

/// Out of Band invitation for TAP connections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutOfBand {
    /// The goal code for this invitation.
    pub goal_code: String,

    /// The goal for this invitation.
    pub goal: String,

    /// The public DID or endpoint URL for the inviter.
    pub service: String,

    /// Accept media types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept: Option<Vec<String>>,

    /// Handshake protocols supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handshake_protocols: Option<Vec<String>>,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl OutOfBand {
    /// Create a new OutOfBand message.
    pub fn new(goal_code: String, goal: String, service: String) -> Self {
        Self {
            goal_code,
            goal,
            service,
            accept: None,
            handshake_protocols: None,
            metadata: HashMap::new(),
        }
    }
}

impl TapMessageBody for OutOfBand {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#outofband"
    }

    fn validate(&self) -> Result<()> {
        if self.goal_code.is_empty() {
            return Err(Error::Validation("Goal code is required".to_string()));
        }

        if self.service.is_empty() {
            return Err(Error::Validation("Service is required".to_string()));
        }

        Ok(())
    }

    fn to_didcomm(&self, from: &str) -> Result<PlainMessage> {
        // Serialize to JSON
        let mut body_json = serde_json::to_value(self)
            .map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        let now = Utc::now().timestamp() as u64;

        // Create a new Message with required fields
        let message = PlainMessage {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body: body_json,
            from: from.to_string(),
            to: Vec::new(), // Recipients will be set separately
            thid: None,
            pthid: None,
            created_time: Some(now),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
            attachments: None,
        };

        Ok(message)
    }

    fn from_didcomm(message: &PlainMessage) -> Result<Self> {
        // Validate message type
        if message.type_ != Self::message_type() {
            return Err(Error::InvalidMessageType(format!(
                "Expected {} but got {}",
                Self::message_type(),
                message.type_
            )));
        }

        // Extract fields from message body
        let oob: OutOfBand = serde_json::from_value(message.body.clone())
            .map_err(|e| Error::SerializationError(e.to_string()))?;

        Ok(oob)
    }
}

/// Authorization Required message body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationRequired {
    /// Authorization URL.
    pub url: String,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AuthorizationRequired {
    /// Create a new AuthorizationRequired message.
    pub fn new(url: String, expires: String) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("expires".to_string(), serde_json::Value::String(expires));
        
        Self {
            url,
            metadata,
        }
    }

    /// Add metadata to the message.
    pub fn add_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
}
