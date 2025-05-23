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
use crate::message::tap_message_trait::{Authorizable, Connectable, TapMessageBody};
use crate::message::{Authorize, Participant, Policy, RemoveAgent, ReplaceAgent, UpdatePolicies};
use chrono::Utc;

/// Payment message body (TAIP-14).
///
/// A Payment is a DIDComm message initiated by the merchant's agent and sent
/// to the customer's agent to request a blockchain payment. It must include either
/// an asset or a currency to denominate the payment, along with the amount and
/// recipient information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    /// Asset identifier (CAIP-19 format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<AssetId>,

    /// Payment amount.
    pub amount: String,

    /// Currency code for fiat amounts (e.g., USD).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,

    /// Supported assets for this payment (when currency_code is specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_assets: Option<Vec<AssetId>>,

    /// Customer (payer) details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<Participant>,

    /// Merchant (payee) details.
    pub merchant: Participant,

    /// Transaction identifier.
    pub transaction_id: String,

    /// Memo for the payment (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    /// Expiration time in ISO 8601 format (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,

    /// Invoice details (optional) per TAIP-16
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice: Option<crate::message::Invoice>,

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
    supported_assets: Option<Vec<AssetId>>,
    customer: Option<Participant>,
    merchant: Option<Participant>,
    transaction_id: Option<String>,
    memo: Option<String>,
    expiry: Option<String>,
    invoice: Option<crate::message::Invoice>,
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

    /// Set the supported assets for this payment
    pub fn supported_assets(mut self, supported_assets: Vec<AssetId>) -> Self {
        self.supported_assets = Some(supported_assets);
        self
    }

    /// Add a supported asset for this payment
    pub fn add_supported_asset(mut self, asset: AssetId) -> Self {
        if let Some(assets) = &mut self.supported_assets {
            assets.push(asset);
        } else {
            self.supported_assets = Some(vec![asset]);
        }
        self
    }

    /// Set the customer for this payment
    pub fn customer(mut self, customer: Participant) -> Self {
        self.customer = Some(customer);
        self
    }

    /// Set the merchant for this payment
    pub fn merchant(mut self, merchant: Participant) -> Self {
        self.merchant = Some(merchant);
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
    pub fn expiry(mut self, expiry: String) -> Self {
        self.expiry = Some(expiry);
        self
    }

    /// Set the invoice for this payment
    pub fn invoice(mut self, invoice: crate::message::Invoice) -> Self {
        self.invoice = Some(invoice);
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
        // Ensure either asset or currency_code is provided
        if self.asset.is_none() && self.currency_code.is_none() {
            panic!("Either asset or currency_code is required");
        }

        Payment {
            asset: self.asset,
            amount: self.amount.expect("Amount is required"),
            currency_code: self.currency_code,
            supported_assets: self.supported_assets,
            customer: self.customer,
            merchant: self.merchant.expect("Merchant is required"),
            transaction_id: self
                .transaction_id
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            memo: self.memo,
            expiry: self.expiry,
            invoice: self.invoice,
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
        // Validate either asset or currency_code is provided
        if self.asset.is_none() && self.currency_code.is_none() {
            return Err(Error::Validation(
                "Either asset or currency_code must be provided".to_string(),
            ));
        }

        // Validate asset ID if provided
        if let Some(asset) = &self.asset {
            if asset.namespace().is_empty() || asset.reference().is_empty() {
                return Err(Error::Validation("Asset ID is invalid".to_string()));
            }
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

        // Validate merchant
        if self.merchant.id.is_empty() {
            return Err(Error::Validation("Merchant ID is required".to_string()));
        }

        // Validate supported_assets if provided
        if let Some(supported_assets) = &self.supported_assets {
            if supported_assets.is_empty() {
                return Err(Error::Validation(
                    "Supported assets list cannot be empty".to_string(),
                ));
            }

            // Validate each asset ID in the supported_assets list
            for (i, asset) in supported_assets.iter().enumerate() {
                if asset.namespace().is_empty() || asset.reference().is_empty() {
                    return Err(Error::Validation(format!(
                        "Supported asset at index {} is invalid",
                        i
                    )));
                }
            }
        }

        // If invoice is provided, validate it
        if let Some(invoice) = &self.invoice {
            // Call the validate method on the invoice
            if let Err(e) = invoice.validate() {
                return Err(Error::Validation(format!(
                    "Invoice validation failed: {}",
                    e
                )));
            }
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

        // Add merchant DID
        agent_dids.push(self.merchant.id.clone());

        // Add customer DID if present
        if let Some(customer) = &self.customer {
            agent_dids.push(customer.id.clone());
        }

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

        // Set expires_time based on the expiry field if provided
        let expires_time = self.expiry.as_ref().and_then(|expiry| {
            // Try to parse ISO 8601 date to epoch seconds
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(expiry) {
                Some(dt.timestamp() as u64)
            } else {
                None
            }
        });

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
            expires_time,
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
        let payment: Payment = serde_json::from_value(message.body.clone())
            .map_err(|e| Error::SerializationError(e.to_string()))?;

        Ok(payment)
    }
}

impl Authorizable for Payment {
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

impl Payment {
    /// Creates a new Payment with an asset
    pub fn with_asset(
        asset: AssetId,
        amount: String,
        merchant: Participant,
        agents: Vec<Participant>,
    ) -> Self {
        Self {
            asset: Some(asset),
            amount,
            currency_code: None,
            supported_assets: None,
            customer: None,
            merchant,
            transaction_id: uuid::Uuid::new_v4().to_string(),
            memo: None,
            expiry: None,
            invoice: None,
            agents,
            metadata: HashMap::new(),
        }
    }

    /// Creates a new Payment with a currency
    pub fn with_currency(
        currency_code: String,
        amount: String,
        merchant: Participant,
        agents: Vec<Participant>,
    ) -> Self {
        Self {
            asset: None,
            amount,
            currency_code: Some(currency_code),
            supported_assets: None,
            customer: None,
            merchant,
            transaction_id: uuid::Uuid::new_v4().to_string(),
            memo: None,
            expiry: None,
            invoice: None,
            agents,
            metadata: HashMap::new(),
        }
    }

    /// Creates a new Payment with a currency and supported assets
    pub fn with_currency_and_assets(
        currency_code: String,
        amount: String,
        supported_assets: Vec<AssetId>,
        merchant: Participant,
        agents: Vec<Participant>,
    ) -> Self {
        Self {
            asset: None,
            amount,
            currency_code: Some(currency_code),
            supported_assets: Some(supported_assets),
            customer: None,
            merchant,
            transaction_id: uuid::Uuid::new_v4().to_string(),
            memo: None,
            expiry: None,
            invoice: None,
            agents,
            metadata: HashMap::new(),
        }
    }
}

impl_tap_message!(Payment);
