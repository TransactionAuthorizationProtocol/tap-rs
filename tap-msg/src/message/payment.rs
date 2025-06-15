//! Payment types for TAP messages.
//!
//! This module defines the structure of payment messages and related types
//! used in the Transaction Authorization Protocol (TAP).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use tap_caip::AssetId;

use crate::error::{Error, Result};
use crate::message::agent::TapParticipant;
use crate::message::tap_message_trait::{TapMessage as TapMessageTrait, TapMessageBody};
use crate::message::{Agent, Party};
use crate::TapMessage;

/// Invoice reference that can be either a URL or an Invoice object
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InvoiceReference {
    /// URL to an invoice
    Url(String),
    /// Structured invoice object
    Object(Box<crate::message::Invoice>),
}

impl InvoiceReference {
    /// Check if this is a URL reference
    pub fn is_url(&self) -> bool {
        matches!(self, InvoiceReference::Url(_))
    }

    /// Check if this is an object reference
    pub fn is_object(&self) -> bool {
        matches!(self, InvoiceReference::Object(_))
    }

    /// Get the URL if this is a URL reference
    pub fn as_url(&self) -> Option<&str> {
        match self {
            InvoiceReference::Url(url) => Some(url),
            _ => None,
        }
    }

    /// Get the invoice object if this is an object reference
    pub fn as_object(&self) -> Option<&crate::message::Invoice> {
        match self {
            InvoiceReference::Object(invoice) => Some(invoice.as_ref()),
            _ => None,
        }
    }

    /// Validate the invoice reference
    pub fn validate(&self) -> Result<()> {
        match self {
            InvoiceReference::Url(url) => {
                // Basic URL validation - just check it's not empty
                if url.is_empty() {
                    return Err(Error::Validation("Invoice URL cannot be empty".to_string()));
                }
                // Could add more URL validation here if needed
                Ok(())
            }
            InvoiceReference::Object(invoice) => {
                // Validate the invoice object
                invoice.validate()
            }
        }
    }
}

/// Payment message body (TAIP-14).
///
/// A Payment is a DIDComm message initiated by the merchant's agent and sent
/// to the customer's agent to request a blockchain payment. It must include either
/// an asset or a currency to denominate the payment, along with the amount and
/// recipient information.
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
#[tap(
    message_type = "https://tap.rsvp/schema/1.0#Payment",
    initiator,
    authorizable,
    transactable
)]
pub struct Payment {
    /// Asset identifier (CAIP-19 format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<AssetId>,

    /// Payment amount.
    pub amount: String,

    /// Currency code for fiat amounts (e.g., USD).
    #[serde(rename = "currency", skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,

    /// Supported assets for this payment (when currency_code is specified)
    #[serde(rename = "supportedAssets", skip_serializing_if = "Option::is_none")]
    pub supported_assets: Option<Vec<AssetId>>,

    /// Customer (payer) details.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tap(participant)]
    pub customer: Option<Party>,

    /// Merchant (payee) details.
    #[tap(participant)]
    pub merchant: Party,

    /// Transaction identifier (only available after creation).
    #[serde(skip)]
    #[tap(transaction_id)]
    pub transaction_id: Option<String>,

    /// Memo for the payment (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    /// Expiration time in ISO 8601 format (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,

    /// Invoice details (optional) per TAIP-16 - can be either a URL or an Invoice object
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice: Option<InvoiceReference>,

    /// Other agents involved in the payment.
    #[serde(default)]
    #[tap(participant_list)]
    pub agents: Vec<Agent>,

    /// Connection ID for linking to Connect messages
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tap(connection_id)]
    pub connection_id: Option<String>,

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
    customer: Option<Party>,
    merchant: Option<Party>,
    transaction_id: Option<String>,
    memo: Option<String>,
    expiry: Option<String>,
    invoice: Option<InvoiceReference>,
    agents: Vec<Agent>,
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
    pub fn customer(mut self, customer: Party) -> Self {
        self.customer = Some(customer);
        self
    }

    /// Set the merchant for this payment
    pub fn merchant(mut self, merchant: Party) -> Self {
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

    /// Set the invoice for this payment with an Invoice object
    pub fn invoice(mut self, invoice: crate::message::Invoice) -> Self {
        self.invoice = Some(InvoiceReference::Object(Box::new(invoice)));
        self
    }

    /// Set the invoice URL for this payment
    pub fn invoice_url(mut self, url: String) -> Self {
        self.invoice = Some(InvoiceReference::Url(url));
        self
    }

    /// Add an agent to this payment
    pub fn add_agent(mut self, agent: Agent) -> Self {
        self.agents.push(agent);
        self
    }

    /// Set all agents for this payment
    pub fn agents(mut self, agents: Vec<Agent>) -> Self {
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
            transaction_id: self.transaction_id,
            memo: self.memo,
            expiry: self.expiry,
            invoice: self.invoice,
            agents: self.agents,
            connection_id: None,
            metadata: self.metadata,
        }
    }
}

impl Payment {
    /// Creates a new Payment with an asset
    pub fn with_asset(asset: AssetId, amount: String, merchant: Party, agents: Vec<Agent>) -> Self {
        Self {
            asset: Some(asset),
            amount,
            currency_code: None,
            supported_assets: None,
            customer: None,
            merchant,
            transaction_id: None,
            memo: None,
            expiry: None,
            invoice: None,
            agents,
            connection_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Creates a new Payment with a currency
    pub fn with_currency(
        currency_code: String,
        amount: String,
        merchant: Party,
        agents: Vec<Agent>,
    ) -> Self {
        Self {
            asset: None,
            amount,
            currency_code: Some(currency_code),
            supported_assets: None,
            customer: None,
            merchant,
            transaction_id: None,
            memo: None,
            expiry: None,
            invoice: None,
            agents,
            connection_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Creates a new Payment with a currency and supported assets
    pub fn with_currency_and_assets(
        currency_code: String,
        amount: String,
        supported_assets: Vec<AssetId>,
        merchant: Party,
        agents: Vec<Agent>,
    ) -> Self {
        Self {
            asset: None,
            amount,
            currency_code: Some(currency_code),
            supported_assets: Some(supported_assets),
            customer: None,
            merchant,
            transaction_id: None,
            memo: None,
            expiry: None,
            invoice: None,
            agents,
            connection_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Custom validation for Payment messages
    pub fn validate(&self) -> Result<()> {
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
        if self.merchant.id().is_empty() {
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
}
