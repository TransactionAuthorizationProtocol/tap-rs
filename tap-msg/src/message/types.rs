//! TAP message types and structures.
//!
//! This module defines the structure of all TAP message types according to the specification.

extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use tap_caip::AssetId;

use crate::didcomm::PlainMessage;
use crate::error::{Error, Result};
use crate::impl_tap_message;
use crate::message::policy::Policy;
use crate::message::tap_message_trait::{Connectable, TapMessageBody};
use crate::message::RequireProofOfControl;
use chrono::Utc;

/// Participant in a transfer (TAIP-3, TAIP-11).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(non_snake_case)]
pub struct Participant {
    /// DID of the participant.
    #[serde(default)]
    pub id: String,

    /// Role of the participant (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub role: Option<String>,

    /// Policies of the participant according to TAIP-7 (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub policies: Option<Vec<Policy>>,

    /// Legal Entity Identifier (LEI) code of the participant (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leiCode: Option<String>,
}

impl Participant {
    /// Create a new participant with the given DID.
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            role: None,
            policies: None,
            leiCode: None,
        }
    }

    /// Create a new participant with the given DID and role.
    pub fn with_role(id: &str, role: &str) -> Self {
        Self {
            id: id.to_string(),
            role: Some(role.to_string()),
            policies: None,
            leiCode: None,
        }
    }
}

/// Attachment data for a TAP message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttachmentData {
    /// Base64-encoded data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base64: Option<String>,

    /// JSON data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<serde_json::Value>,
}

/// Attachment for a TAP message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Attachment {
    /// ID of the attachment.
    pub id: String,

    /// Media type of the attachment.
    #[serde(rename = "media_type")]
    pub media_type: String,

    /// Attachment data (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<AttachmentData>,
}

/// Attachment format enumeration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentFormat {
    /// Base64-encoded data.
    Base64,

    /// JSON data.
    Json,
}

/// Agent information for TAP messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// DID of the agent.
    pub id: String,

    /// Role of the agent.
    pub role: String,
}

/// Message for out-of-band invitations (TAIP-2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutOfBand {
    /// Goal code for the invitation.
    #[serde(rename = "goal_code")]
    pub goal_code: String,

    /// Invitation message ID.
    pub id: String,

    /// Label for the invitation.
    pub label: String,

    /// Accept option for the invitation.
    pub accept: Option<String>,

    /// The DIDComm services to connect to.
    pub services: Vec<serde_json::Value>,
}

/// Connection constraints for the Connect message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConstraints {
    /// Limit on transaction amount.
    pub transaction_limits: Option<TransactionLimits>,
}

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
    fn with_connection(&mut self, connect_id: &str) -> &mut Self {
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
                let constraints_json = serde_json::to_value(constraints_value)
                    .map_err(|e| Error::SerializationError(format!("Invalid constraints: {}", e)))?;

                Some(
                    serde_json::from_value(constraints_json).map_err(|e| {
                        Error::SerializationError(format!("Invalid constraints format: {}", e))
                    })?,
                )
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
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the message.
    pub fn add_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
}

/// Payment message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    /// Asset identifier (CAIP-19 format).
    pub asset: AssetId,

    /// Payment amount.
    pub amount: String,

    /// Currency code for fiat amounts (e.g., USD).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,

    /// Originator details.
    pub originator: Participant,

    /// Beneficiary details.
    pub beneficiary: Participant,

    /// Transaction identifier.
    pub transaction_id: String,

    /// Memo for the payment (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    /// Expiration time in ISO 8601 format (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<String>,

    /// Other agents involved in the payment.
    #[serde(default)]
    pub agents: Vec<Participant>,

    /// Additional metadata (optional).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Builder for Payment objects.
#[derive(Default)]
pub struct PaymentBuilder {
    asset: Option<AssetId>,
    amount: Option<String>,
    currency_code: Option<String>,
    originator: Option<Participant>,
    beneficiary: Option<Participant>,
    transaction_id: Option<String>,
    memo: Option<String>,
    expires: Option<String>,
    agents: Vec<Participant>,
    metadata: HashMap<String, serde_json::Value>,
}

impl PaymentBuilder {
    /// Set the asset for this payment
    pub fn asset(mut self, asset: AssetId) -> Self {
        self.asset = Some(asset);
        self
    }

    /// Set the amount for this payment
    pub fn amount(mut self, amount: String) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the currency code for this payment
    pub fn currency_code(mut self, currency_code: String) -> Self {
        self.currency_code = Some(currency_code);
        self
    }

    /// Set the originator for this payment
    pub fn originator(mut self, originator: Participant) -> Self {
        self.originator = Some(originator);
        self
    }

    /// Set the beneficiary for this payment
    pub fn beneficiary(mut self, beneficiary: Participant) -> Self {
        self.beneficiary = Some(beneficiary);
        self
    }

    /// Set the transaction ID for this payment
    pub fn transaction_id(mut self, transaction_id: String) -> Self {
        self.transaction_id = Some(transaction_id);
        self
    }

    /// Set the memo for this payment
    pub fn memo(mut self, memo: String) -> Self {
        self.memo = Some(memo);
        self
    }

    /// Set the expiration time for this payment
    pub fn expires(mut self, expires: String) -> Self {
        self.expires = Some(expires);
        self
    }

    /// Add an agent to this payment
    pub fn add_agent(mut self, agent: Participant) -> Self {
        self.agents.push(agent);
        self
    }

    /// Set all agents for this payment
    pub fn agents(mut self, agents: Vec<Participant>) -> Self {
        self.agents = agents;
        self
    }

    /// Add a metadata field
    pub fn add_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set all metadata for this payment
    pub fn metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Build the Payment object
    ///
    /// # Panics
    ///
    /// Panics if required fields are not set
    pub fn build(self) -> Payment {
        Payment {
            asset: self.asset.expect("Asset is required"),
            amount: self.amount.expect("Amount is required"),
            currency_code: self.currency_code,
            originator: self.originator.expect("Originator is required"),
            beneficiary: self.beneficiary.expect("Beneficiary is required"),
            transaction_id: self
                .transaction_id
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            memo: self.memo,
            expires: self.expires,
            agents: self.agents,
            metadata: self.metadata,
        }
    }
}

/// DIDComm Presentation message body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIDCommPresentation {
    /// The format of the presentation.
    pub formats: Vec<AttachmentFormat>,

    /// Attachments containing the presentation data.
    pub attachments: Vec<Attachment>,

    /// Thread ID for this presentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thid: Option<String>,
}

impl DIDCommPresentation {
    /// Create a new DIDCommPresentation message.
    pub fn new(
        formats: Vec<AttachmentFormat>,
        attachments: Vec<Attachment>,
        thid: Option<String>,
    ) -> Self {
        Self {
            formats,
            attachments,
            thid,
        }
    }
    
    /// Validate the presentation
    pub fn validate(&self) -> Result<()> {
        // Basic validation - ensure we have attachments
        if self.attachments.is_empty() {
            return Err(Error::Validation(
                "Presentation must have at least one attachment".to_string(),
            ));
        }
        
        Ok(())
    }
}

/// Payment Request message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    /// Asset identifier (CAIP-19 format).
    pub asset: AssetId,

    /// Payment amount.
    pub amount: String,

    /// Currency code for fiat amounts (e.g., USD).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,

    /// Beneficiary details.
    pub beneficiary: Participant,

    /// Transaction identifier.
    pub transaction_id: String,

    /// Memo for the payment request (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    /// Expiration time in ISO 8601 format (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<String>,

    /// Other agents involved in the payment request.
    #[serde(default)]
    pub agents: Vec<Participant>,

    /// Additional metadata (optional).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl PaymentRequest {
    /// Create a new payment request
    pub fn new(
        asset: AssetId,
        amount: String,
        beneficiary: Participant,
        transaction_id: Option<String>,
    ) -> Self {
        Self {
            asset,
            amount,
            currency_code: None,
            beneficiary,
            transaction_id: transaction_id
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            memo: None,
            expires: None,
            agents: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set the currency code for this payment request
    pub fn with_currency_code(mut self, currency_code: String) -> Self {
        self.currency_code = Some(currency_code);
        self
    }

    /// Set the memo for this payment request
    pub fn with_memo(mut self, memo: String) -> Self {
        self.memo = Some(memo);
        self
    }

    /// Set the expiration time for this payment request
    pub fn with_expires(mut self, expires: String) -> Self {
        self.expires = Some(expires);
        self
    }

    /// Add an agent to this payment request
    pub fn add_agent(mut self, agent: Participant) -> Self {
        self.agents.push(agent);
        self
    }

    /// Add a metadata field
    pub fn add_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl Connectable for Payment {
    fn with_connection(&mut self, connect_id: &str) -> &mut Self {
        // Store the connect_id in metadata
        self.metadata.insert(
            "connect_id".to_string(),
            serde_json::Value::String(connect_id.to_string()),
        );
        self
    }

    fn has_connection(&self) -> bool {
        self.metadata.contains_key("connect_id")
    }

    fn connection_id(&self) -> Option<&str> {
        self.metadata.get("connect_id").and_then(|v| v.as_str())
    }
}

impl TapMessageBody for Payment {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#payment"
    }

    fn validate(&self) -> Result<()> {
        // Validate asset ID
        if self.asset.namespace().is_empty() || self.asset.reference().is_empty() {
            return Err(Error::Validation("Asset ID is invalid".to_string()));
        }

        // Validate amount
        if self.amount.is_empty() {
            return Err(Error::Validation("Amount is required".to_string()));
        }

        // Validate amount is a positive number
        match self.amount.parse::<f64>() {
            Ok(amount) if amount <= 0.0 => {
                return Err(Error::Validation("Amount must be positive".to_string()));
            }
            Err(_) => {
                return Err(Error::Validation(
                    "Amount must be a valid number".to_string(),
                ));
            }
            _ => {}
        }

        // Validate originator
        if self.originator.id.is_empty() {
            return Err(Error::Validation("Originator ID is required".to_string()));
        }

        // Validate beneficiary
        if self.beneficiary.id.is_empty() {
            return Err(Error::Validation("Beneficiary ID is required".to_string()));
        }

        Ok(())
    }

    fn to_didcomm(&self, from_did: &str) -> Result<PlainMessage> {
        // Serialize the Payment to a JSON value
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        // Extract agent DIDs directly from the message
        let mut agent_dids = Vec::new();

        // Add originator DID
        agent_dids.push(self.originator.id.clone());

        // Add beneficiary DID
        agent_dids.push(self.beneficiary.id.clone());

        // Add DIDs from agents array
        for agent in &self.agents {
            agent_dids.push(agent.id.clone());
        }

        // Remove duplicates
        agent_dids.sort();
        agent_dids.dedup();

        // Remove the sender from the recipients list to avoid sending to self
        agent_dids.retain(|did| did != from_did);

        let now = Utc::now().timestamp() as u64;

        // Get the connection ID if this message is connected to a previous message
        let pthid = self
            .connection_id()
            .map(|connect_id| connect_id.to_string());

        // Create a new Message with required fields
        let message = PlainMessage {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body: body_json,
            from: from_did.to_string(),
            to: agent_dids,
            thid: Some(self.transaction_id.clone()),
            pthid,
            created_time: Some(now),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
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
        let payment: Payment = serde_json::from_value(message.body.clone())
            .map_err(|e| Error::SerializationError(e.to_string()))?;

        Ok(payment)
    }
}

impl TapMessageBody for PaymentRequest {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#payment-request"
    }

    fn validate(&self) -> Result<()> {
        // Validate asset ID
        if self.asset.namespace().is_empty() || self.asset.reference().is_empty() {
            return Err(Error::Validation("Asset ID is invalid".to_string()));
        }

        // Validate amount
        if self.amount.is_empty() {
            return Err(Error::Validation("Amount is required".to_string()));
        }

        // Validate amount is a positive number
        match self.amount.parse::<f64>() {
            Ok(amount) if amount <= 0.0 => {
                return Err(Error::Validation("Amount must be positive".to_string()));
            }
            Err(_) => {
                return Err(Error::Validation(
                    "Amount must be a valid number".to_string(),
                ));
            }
            _ => {}
        }

        // Validate beneficiary
        if self.beneficiary.id.is_empty() {
            return Err(Error::Validation("Beneficiary ID is required".to_string()));
        }

        Ok(())
    }

    fn to_didcomm(&self, from_did: &str) -> Result<PlainMessage> {
        // Serialize the PaymentRequest to a JSON value
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        // Extract agent DIDs directly from the message
        let mut agent_dids = Vec::new();

        // Add beneficiary DID
        agent_dids.push(self.beneficiary.id.clone());

        // Add DIDs from agents array
        for agent in &self.agents {
            agent_dids.push(agent.id.clone());
        }

        // Remove duplicates
        agent_dids.sort();
        agent_dids.dedup();

        // Remove the sender from the recipients list to avoid sending to self
        agent_dids.retain(|did| did != from_did);

        let now = Utc::now().timestamp() as u64;

        // Create a new Message with required fields
        let message = PlainMessage {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body: body_json,
            from: from_did.to_string(),
            to: agent_dids,
            thid: Some(self.transaction_id.clone()),
            pthid: None,
            created_time: Some(now),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
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
        let payment_request: PaymentRequest = serde_json::from_value(message.body.clone())
            .map_err(|e| Error::SerializationError(e.to_string()))?;

        Ok(payment_request)
    }
}

impl_tap_message!(Payment);
impl_tap_message!(PaymentRequest);