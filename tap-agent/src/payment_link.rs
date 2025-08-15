//! Payment link functionality for TAP agents
//!
//! This module provides utilities for creating payment links using Out-of-Band messages
//! containing signed Payment messages according to TAIP-14 and TAIP-2.

use crate::error::{Error, Result};
use crate::oob::OutOfBandInvitation;
use serde_json::Value;
use std::collections::HashMap;
use tap_msg::message::{Payment, TapMessageBody};

/// Default service URL for payment links
pub const DEFAULT_PAYMENT_SERVICE_URL: &str = "https://flow-connect.notabene.dev/payin";

/// Configuration for creating payment links
#[derive(Debug, Clone)]
pub struct PaymentLinkConfig {
    /// Base URL for the payment service
    pub service_url: String,
    /// Additional metadata to include in the OOB invitation
    pub metadata: HashMap<String, Value>,
    /// Custom goal description (defaults to "Process payment request")
    pub goal: Option<String>,
}

impl Default for PaymentLinkConfig {
    fn default() -> Self {
        Self {
            service_url: DEFAULT_PAYMENT_SERVICE_URL.to_string(),
            metadata: HashMap::new(),
            goal: None,
        }
    }
}

impl PaymentLinkConfig {
    /// Create a new configuration with the default service URL
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a custom service URL
    pub fn with_service_url(mut self, url: &str) -> Self {
        self.service_url = url.to_string();
        self
    }

    /// Add metadata to the OOB invitation
    pub fn with_metadata(mut self, key: &str, value: Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }

    /// Set a custom goal description
    pub fn with_goal(mut self, goal: &str) -> Self {
        self.goal = Some(goal.to_string());
        self
    }
}

/// Builder for creating payment links
pub struct PaymentLinkBuilder {
    agent_did: String,
    payment: Payment,
    config: PaymentLinkConfig,
}

impl PaymentLinkBuilder {
    /// Create a new payment link builder
    pub fn new(agent_did: &str, payment: Payment) -> Self {
        Self {
            agent_did: agent_did.to_string(),
            payment,
            config: PaymentLinkConfig::default(),
        }
    }

    /// Set the configuration
    pub fn with_config(mut self, config: PaymentLinkConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the service URL
    pub fn with_service_url(mut self, url: &str) -> Self {
        self.config.service_url = url.to_string();
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: Value) -> Self {
        self.config.metadata.insert(key.to_string(), value);
        self
    }

    /// Build the payment link (requires signing the payment message)
    pub async fn build_with_signer<F, Fut>(self, sign_fn: F) -> Result<PaymentLink>
    where
        F: FnOnce(String) -> Fut,
        Fut: std::future::Future<Output = Result<String>>,
    {
        // Create the DIDComm PlainMessage for the payment
        let plain_message = self.payment.to_didcomm(&self.agent_did)?;
        
        // Serialize the plain message for signing
        let message_json = serde_json::to_string(&plain_message)
            .map_err(|e| Error::Serialization(format!("Failed to serialize payment: {}", e)))?;

        // Sign the message using the provided signing function
        let signed_message = sign_fn(message_json).await?;

        // Create the OOB invitation
        let goal = self.config.goal.unwrap_or_else(|| "Process payment request".to_string());
        
        let mut oob_builder = OutOfBandInvitation::builder(&self.agent_did, "tap.payment", &goal)
            .add_signed_attachment(
                "payment-request",
                &signed_message,
                Some("Signed payment request message"),
            );

        // Add any additional metadata
        for (key, value) in &self.config.metadata {
            oob_builder = oob_builder.add_metadata(key, value.clone());
        }

        let oob_invitation = oob_builder.build();

        // Generate the URL
        let url = oob_invitation.to_url(&self.config.service_url)?;

        Ok(PaymentLink {
            url,
            oob_invitation,
            payment: self.payment,
            signed_message,
        })
    }
}

/// A payment link containing the URL and associated data
#[derive(Debug, Clone)]
pub struct PaymentLink {
    /// The payment link URL
    pub url: String,
    /// The Out-of-Band invitation
    pub oob_invitation: OutOfBandInvitation,
    /// The original payment message
    pub payment: Payment,
    /// The signed message string
    pub signed_message: String,
}

impl PaymentLink {
    /// Create a new payment link builder
    pub fn builder(agent_did: &str, payment: Payment) -> PaymentLinkBuilder {
        PaymentLinkBuilder::new(agent_did, payment)
    }

    /// Get the payment amount as a string
    pub fn amount(&self) -> &str {
        &self.payment.amount
    }

    /// Get the payment currency (if specified)
    pub fn currency(&self) -> Option<&str> {
        self.payment.currency_code.as_deref()
    }

    /// Get the payment asset (if specified)  
    pub fn asset(&self) -> Option<String> {
        self.payment.asset.as_ref().map(|a| a.to_string())
    }

    /// Get the merchant information
    pub fn merchant(&self) -> &tap_msg::message::Party {
        &self.payment.merchant
    }

    /// Check if the payment link has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expiry) = &self.payment.expiry {
            // Parse ISO 8601 timestamp and compare with current time
            if let Ok(expiry_time) = chrono::DateTime::parse_from_rfc3339(expiry) {
                return chrono::Utc::now() > expiry_time.with_timezone(&chrono::Utc);
            }
        }
        false
    }

    /// Get the payment link as a QR code-friendly format
    pub fn to_qr_data(&self) -> &str {
        &self.url
    }

    /// Parse a payment link from a URL
    pub fn from_url(url: &str) -> Result<PaymentLinkInfo> {
        let oob_invitation = OutOfBandInvitation::from_url(url)?;
        
        // Validate it's a payment invitation
        if !oob_invitation.is_payment_invitation() {
            return Err(Error::Validation(
                "OOB invitation is not a payment request".to_string(),
            ));
        }

        // Extract the payment attachment
        let attachment = oob_invitation
            .get_signed_attachment()
            .ok_or_else(|| Error::Validation("No signed payment attachment found".to_string()))?;

        Ok(PaymentLinkInfo {
            oob_invitation: oob_invitation.clone(),
            attachment_id: attachment.id.clone().unwrap_or_default(),
        })
    }

    /// Create a short link URL using just the invitation ID
    pub fn to_short_url(&self, base_url: &str) -> Result<String> {
        self.oob_invitation.to_id_url(base_url)
    }
}

/// Information extracted from a payment link URL
#[derive(Debug, Clone)]
pub struct PaymentLinkInfo {
    /// The Out-of-Band invitation
    pub oob_invitation: OutOfBandInvitation,
    /// ID of the payment attachment
    pub attachment_id: String,
}

impl PaymentLinkInfo {
    /// Get the signed payment message JSON
    pub fn get_signed_payment(&self) -> Option<&Value> {
        self.oob_invitation.extract_attachment_json(&self.attachment_id)
    }

    /// Get the merchant DID from the invitation
    pub fn merchant_did(&self) -> &str {
        &self.oob_invitation.from
    }

    /// Get the goal description
    pub fn goal(&self) -> &str {
        &self.oob_invitation.body.goal
    }

    /// Validate the payment link structure
    pub fn validate(&self) -> Result<()> {
        self.oob_invitation.validate()?;
        
        // Check that the signed payment attachment exists
        if self.get_signed_payment().is_none() {
            return Err(Error::Validation(
                "Signed payment attachment not found".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_payment_link_config_defaults() {
        let config = PaymentLinkConfig::default();
        assert_eq!(config.service_url, DEFAULT_PAYMENT_SERVICE_URL);
        assert!(config.metadata.is_empty());
        assert!(config.goal.is_none());
    }

    #[test]
    fn test_payment_link_config_builder() {
        let config = PaymentLinkConfig::new()
            .with_service_url("https://custom.com/pay")
            .with_metadata("order_id", json!("12345"))
            .with_goal("Complete your purchase");

        assert_eq!(config.service_url, "https://custom.com/pay");
        assert_eq!(config.metadata.get("order_id"), Some(&json!("12345")));
        assert_eq!(config.goal, Some("Complete your purchase".to_string()));
    }

    #[test]
    fn test_payment_link_parsing_error() {
        // Test error handling for invalid URLs
        let result = PaymentLink::from_url("https://example.com/invalid");
        assert!(result.is_err());
    }
}