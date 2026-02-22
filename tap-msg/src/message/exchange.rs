//! Exchange and Quote message implementations for the Transaction Authorization Protocol.
//!
//! This module defines the Exchange and Quote message types (TAIP-18) for requesting
//! and executing asset exchanges and price quotations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::message::agent::TapParticipant;
use crate::message::tap_message_trait::{TapMessage as TapMessageTrait, TapMessageBody};
use crate::message::{Agent, Party};
use crate::TapMessage;

/// Exchange message body (TAIP-18).
///
/// Initiates an exchange request for cross-asset quotes. Supports multiple source
/// and target assets, enabling complex exchange scenarios like cross-currency swaps,
/// on/off-ramp pricing, and cross-chain bridging.
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
#[tap(
    message_type = "https://tap.rsvp/schema/1.0#Exchange",
    initiator,
    authorizable,
    transactable
)]
pub struct Exchange {
    /// Available source assets (CAIP-19, DTI, or ISO 4217 currency codes).
    #[serde(rename = "fromAssets")]
    pub from_assets: Vec<String>,

    /// Desired target assets (CAIP-19, DTI, or ISO 4217 currency codes).
    #[serde(rename = "toAssets")]
    pub to_assets: Vec<String>,

    /// Amount of source asset to exchange (conditional: either this or to_amount required).
    #[serde(rename = "fromAmount", skip_serializing_if = "Option::is_none")]
    pub from_amount: Option<String>,

    /// Amount of target asset desired (conditional: either this or from_amount required).
    #[serde(rename = "toAmount", skip_serializing_if = "Option::is_none")]
    pub to_amount: Option<String>,

    /// The party requesting the exchange.
    #[tap(participant)]
    pub requester: Party,

    /// The preferred liquidity provider (optional, omit to broadcast).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tap(participant)]
    pub provider: Option<Party>,

    /// Agents involved in the exchange request.
    #[serde(default)]
    #[tap(participant_list)]
    pub agents: Vec<Agent>,

    /// Compliance or presentation requirements (TAIP-7).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policies: Option<Vec<serde_json::Value>>,

    /// Transaction identifier (only available after creation).
    #[serde(skip)]
    #[tap(transaction_id)]
    pub transaction_id: Option<String>,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Exchange {
    /// Create a new Exchange with from_amount specified.
    pub fn new_from(
        from_assets: Vec<String>,
        to_assets: Vec<String>,
        from_amount: String,
        requester: Party,
        agents: Vec<Agent>,
    ) -> Self {
        Self {
            from_assets,
            to_assets,
            from_amount: Some(from_amount),
            to_amount: None,
            requester,
            provider: None,
            agents,
            policies: None,
            transaction_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new Exchange with to_amount specified.
    pub fn new_to(
        from_assets: Vec<String>,
        to_assets: Vec<String>,
        to_amount: String,
        requester: Party,
        agents: Vec<Agent>,
    ) -> Self {
        Self {
            from_assets,
            to_assets,
            from_amount: None,
            to_amount: Some(to_amount),
            requester,
            provider: None,
            agents,
            policies: None,
            transaction_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the provider for this exchange.
    pub fn with_provider(mut self, provider: Party) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Set policies for this exchange.
    pub fn with_policies(mut self, policies: Vec<serde_json::Value>) -> Self {
        self.policies = Some(policies);
        self
    }

    /// Custom validation for Exchange messages.
    pub fn validate(&self) -> Result<()> {
        if self.from_assets.is_empty() {
            return Err(Error::Validation(
                "fromAssets must not be empty".to_string(),
            ));
        }
        if self.to_assets.is_empty() {
            return Err(Error::Validation("toAssets must not be empty".to_string()));
        }
        if self.from_amount.is_none() && self.to_amount.is_none() {
            return Err(Error::Validation(
                "Either fromAmount or toAmount must be provided".to_string(),
            ));
        }
        if self.requester.id().is_empty() {
            return Err(Error::Validation(
                "Requester ID cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

/// Quote message body (TAIP-18).
///
/// Sent by a liquidity provider in response to an Exchange request.
/// Specifies a specific asset pair with amounts and an expiration time.
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
#[tap(message_type = "https://tap.rsvp/schema/1.0#Quote")]
pub struct Quote {
    /// Source asset (CAIP-19, DTI, or ISO 4217 currency code).
    #[serde(rename = "fromAsset")]
    pub from_asset: String,

    /// Target asset (CAIP-19, DTI, or ISO 4217 currency code).
    #[serde(rename = "toAsset")]
    pub to_asset: String,

    /// Amount of source asset to be exchanged.
    #[serde(rename = "fromAmount")]
    pub from_amount: String,

    /// Amount of target asset to be received.
    #[serde(rename = "toAmount")]
    pub to_amount: String,

    /// The liquidity provider party.
    #[tap(participant)]
    pub provider: Party,

    /// All agents involved (original Exchange agents + provider agents).
    #[serde(default)]
    #[tap(participant_list)]
    pub agents: Vec<Agent>,

    /// ISO 8601 timestamp when the quote expires.
    pub expires: String,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Quote {
    /// Create a new Quote.
    pub fn new(
        from_asset: String,
        to_asset: String,
        from_amount: String,
        to_amount: String,
        provider: Party,
        agents: Vec<Agent>,
        expires: String,
    ) -> Self {
        Self {
            from_asset,
            to_asset,
            from_amount,
            to_amount,
            provider,
            agents,
            expires,
            metadata: HashMap::new(),
        }
    }

    /// Custom validation for Quote messages.
    pub fn validate(&self) -> Result<()> {
        if self.from_asset.is_empty() {
            return Err(Error::Validation("fromAsset must not be empty".to_string()));
        }
        if self.to_asset.is_empty() {
            return Err(Error::Validation("toAsset must not be empty".to_string()));
        }
        if self.from_amount.is_empty() {
            return Err(Error::Validation(
                "fromAmount must not be empty".to_string(),
            ));
        }
        if self.to_amount.is_empty() {
            return Err(Error::Validation("toAmount must not be empty".to_string()));
        }
        if self.provider.id().is_empty() {
            return Err(Error::Validation("Provider ID cannot be empty".to_string()));
        }
        if self.expires.is_empty() {
            return Err(Error::Validation("expires must not be empty".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_exchange_creation() {
        let exchange = Exchange::new_from(
            vec!["eip155:1/erc20:0xA0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string()],
            vec!["eip155:1/erc20:0xB00b00b00b00b00b00b00b00b00b00b00b00b00b".to_string()],
            "1000.00".to_string(),
            Party::new("did:web:business.example"),
            vec![Agent::new_without_role(
                "did:web:wallet.example",
                "did:web:business.example",
            )],
        )
        .with_provider(Party::new("did:web:liquidity.provider"));

        assert_eq!(exchange.from_assets.len(), 1);
        assert_eq!(exchange.to_assets.len(), 1);
        assert_eq!(exchange.from_amount, Some("1000.00".to_string()));
        assert!(exchange.to_amount.is_none());
        assert!(exchange.provider.is_some());
        assert!(exchange.validate().is_ok());
    }

    #[test]
    fn test_exchange_serialization() {
        let exchange = Exchange::new_from(
            vec!["USD".to_string()],
            vec!["eip155:1/erc20:0xA0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string()],
            "1000.00".to_string(),
            Party::new("did:web:user.entity"),
            vec![Agent::new_without_role(
                "did:web:user.wallet",
                "did:web:user.entity",
            )],
        );

        let json = serde_json::to_value(&exchange).unwrap();
        assert_eq!(json["fromAssets"][0], "USD");
        assert_eq!(json["fromAmount"], "1000.00");
        assert!(json.get("toAmount").is_none());

        let deserialized: Exchange = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.from_assets, exchange.from_assets);
    }

    #[test]
    fn test_exchange_validation_no_amount() {
        let exchange = Exchange {
            from_assets: vec!["USD".to_string()],
            to_assets: vec!["EUR".to_string()],
            from_amount: None,
            to_amount: None,
            requester: Party::new("did:example:user"),
            provider: None,
            agents: vec![],
            policies: None,
            transaction_id: None,
            metadata: HashMap::new(),
        };

        let result = exchange.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Either fromAmount or toAmount"));
    }

    #[test]
    fn test_quote_creation() {
        let quote = Quote::new(
            "eip155:1/erc20:0xA0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(),
            "eip155:1/erc20:0xB00b00b00b00b00b00b00b00b00b00b00b00b00b".to_string(),
            "1000.00".to_string(),
            "908.50".to_string(),
            Party::new("did:web:liquidity.provider"),
            vec![
                Agent::new_without_role("did:web:wallet.example", "did:web:business.example"),
                Agent::new_without_role("did:web:lp.example", "did:web:liquidity.provider"),
            ],
            "2025-07-21T00:00:00Z".to_string(),
        );

        assert_eq!(quote.from_amount, "1000.00");
        assert_eq!(quote.to_amount, "908.50");
        assert_eq!(quote.agents.len(), 2);
        assert!(quote.validate().is_ok());
    }

    #[test]
    fn test_quote_serialization() {
        let quote = Quote::new(
            "USD".to_string(),
            "eip155:1/erc20:0xA0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(),
            "1000.00".to_string(),
            "996.00".to_string(),
            Party::new("did:web:onramp.company"),
            vec![],
            "2025-07-21T00:00:00Z".to_string(),
        );

        let json = serde_json::to_value(&quote).unwrap();
        assert_eq!(json["fromAsset"], "USD");
        assert_eq!(json["toAmount"], "996.00");
        assert_eq!(json["expires"], "2025-07-21T00:00:00Z");

        let deserialized: Quote = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.from_amount, quote.from_amount);
    }
}
