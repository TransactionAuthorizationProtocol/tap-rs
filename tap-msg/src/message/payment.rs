//! Payment types for TAP messages.
//!
//! This module defines the structure of payment messages and related types
//! used in the Transaction Authorization Protocol (TAP).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use tap_caip::AssetId;

use crate::didcomm::PlainMessage;
use crate::error::{Error, Result};
use crate::impl_tap_message;
use crate::message::participant::Participant;
use crate::message::tap_message_trait::{Connectable, TapMessageBody};
use chrono::Utc;

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

impl_tap_message!(Payment);

/// Payment request message to request a payment from another entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    /// Asset identifier (CAIP-19 format)
    pub asset: AssetId,

    /// Payment amount
    pub amount: String,

    /// Currency code for fiat amounts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,

    /// Additional metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,

    /// Transaction identifier
    pub transaction_id: String,

    /// Memo for the payment request (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    /// Expiration time in ISO 8601 format (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<String>,
}

impl PaymentRequest {
    /// Create a new PaymentRequest
    pub fn new(transaction_id: &str, asset: AssetId, amount: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            asset,
            amount: amount.to_string(),
            currency_code: None,
            metadata: HashMap::new(),
            memo: None,
            expires: None,
        }
    }

    /// Add additional metadata to the request
    pub fn add_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }

    /// Set the memo
    pub fn with_memo(mut self, memo: &str) -> Self {
        self.memo = Some(memo.to_string());
        self
    }

    /// Set the currency code
    pub fn with_currency_code(mut self, code: &str) -> Self {
        self.currency_code = Some(code.to_string());
        self
    }

    /// Set the expiration time
    pub fn with_expiration(mut self, expires: &str) -> Self {
        self.expires = Some(expires.to_string());
        self
    }
}

impl Connectable for PaymentRequest {
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

        // Validate transaction ID
        if self.transaction_id.is_empty() {
            return Err(Error::Validation("Transaction ID is required".to_string()));
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
            to: Vec::new(), // Recipients will be set separately
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
        let payment_request: PaymentRequest = serde_json::from_value(message.body.clone())
            .map_err(|e| Error::SerializationError(e.to_string()))?;

        Ok(payment_request)
    }
}

impl_tap_message!(PaymentRequest);
