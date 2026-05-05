//! Composable Lock and Capture message types (TAIP-17).
//!
//! TAIP-17 was renamed from "Escrow" to "Lock" in the May 2026 spec advance to
//! Review. The body shape is unchanged: a Lock requests an agent to hold assets
//! on behalf of one party for the benefit of another, and Capture authorises
//! the release of those assets to the beneficiary. The role string `EscrowAgent`
//! is preserved by the spec and continues to identify the agent holding funds.
//!
//! `Escrow` is kept as a type alias for `Lock` to avoid breaking downstream
//! callers; on-the-wire messages bearing the legacy `#Escrow` type URI are
//! still accepted by the dispatcher in [`crate::message::tap_message_enum`].

use crate::error::{Error, Result};
use crate::message::agent::Agent;
use crate::message::party::Party;
use crate::message::tap_message_trait::TapMessageBody;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Lock message for holding assets on behalf of parties (TAIP-17).
///
/// A Lock allows one agent to request another agent to hold a specified
/// amount of currency or asset from a party in escrow on behalf of another
/// party.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Lock {
    /// Cryptocurrency asset to be held in escrow (CAIP-19 identifier).
    /// Either `asset` OR `currency` MUST be present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,

    /// ISO 4217 currency code (e.g. "USD", "EUR") for fiat-denominated locks.
    /// Either `asset` OR `currency` MUST be present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,

    /// Amount to be held in escrow (decimal string).
    pub amount: String,

    /// Party whose assets will be placed in escrow.
    pub originator: Party,

    /// Party who will receive the assets when released.
    pub beneficiary: Party,

    /// Timestamp after which the lock automatically expires and funds are
    /// released back to the originator.
    pub expiry: String,

    /// URL or URI referencing the terms and conditions of the lock.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agreement: Option<String>,

    /// Agents involved in the lock. Exactly one agent MUST have role "EscrowAgent".
    pub agents: Vec<Agent>,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, Value>,
}

/// Backward-compatible alias for the renamed `Lock` message.
pub type Escrow = Lock;

impl Lock {
    /// Create a new Lock for cryptocurrency assets.
    pub fn new_with_asset(
        asset: String,
        amount: String,
        originator: Party,
        beneficiary: Party,
        expiry: String,
        agents: Vec<Agent>,
    ) -> Self {
        Self {
            asset: Some(asset),
            currency: None,
            amount,
            originator,
            beneficiary,
            expiry,
            agreement: None,
            agents,
            metadata: HashMap::new(),
        }
    }

    /// Create a new Lock for fiat currency.
    pub fn new_with_currency(
        currency: String,
        amount: String,
        originator: Party,
        beneficiary: Party,
        expiry: String,
        agents: Vec<Agent>,
    ) -> Self {
        Self {
            asset: None,
            currency: Some(currency),
            amount,
            originator,
            beneficiary,
            expiry,
            agreement: None,
            agents,
            metadata: HashMap::new(),
        }
    }

    /// Set the agreement URL.
    pub fn with_agreement(mut self, agreement: String) -> Self {
        self.agreement = Some(agreement);
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: String, value: Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Find the escrow agent (the agent with `role == "EscrowAgent"`) in the
    /// agents list.
    pub fn escrow_agent(&self) -> Option<&Agent> {
        self.agents
            .iter()
            .find(|a| a.role == Some("EscrowAgent".to_string()))
    }

    /// Find agents that can authorise release (agents acting `for` the beneficiary).
    pub fn authorizing_agents(&self) -> Vec<&Agent> {
        self.agents
            .iter()
            .filter(|a| a.for_parties.0.contains(&self.beneficiary.id))
            .collect()
    }
}

impl TapMessageBody for Lock {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#Lock"
    }

    fn validate(&self) -> Result<()> {
        match (&self.asset, &self.currency) {
            (Some(_), Some(_)) => {
                return Err(Error::Validation(
                    "Lock cannot have both asset and currency specified".to_string(),
                ));
            }
            (None, None) => {
                return Err(Error::Validation(
                    "Lock must have either asset or currency specified".to_string(),
                ));
            }
            _ => {}
        }

        if self.amount.is_empty() {
            return Err(Error::Validation("Lock amount cannot be empty".to_string()));
        }

        if self.expiry.is_empty() {
            return Err(Error::Validation("Lock expiry cannot be empty".to_string()));
        }

        let escrow_agent_count = self
            .agents
            .iter()
            .filter(|a| a.role == Some("EscrowAgent".to_string()))
            .count();

        if escrow_agent_count == 0 {
            return Err(Error::Validation(
                "Lock must have exactly one agent with role 'EscrowAgent'".to_string(),
            ));
        }

        if escrow_agent_count > 1 {
            return Err(Error::Validation(
                "Lock cannot have more than one agent with role 'EscrowAgent'".to_string(),
            ));
        }

        if self.originator.id == self.beneficiary.id {
            return Err(Error::Validation(
                "Lock originator and beneficiary must be different parties".to_string(),
            ));
        }

        Ok(())
    }
}

/// Capture message for releasing locked funds (TAIP-17).
///
/// Capture authorises the release of locked funds to the beneficiary. It can
/// only be sent by agents acting for the beneficiary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Capture {
    /// Amount to capture (decimal string). If omitted, captures the full
    /// locked amount. MUST be ≤ original amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,

    /// Blockchain address for settlement. If omitted, uses the address from
    /// an earlier Authorize.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_address: Option<String>,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, Value>,
}

impl Capture {
    /// Create a new Capture for the full locked amount.
    pub fn new() -> Self {
        Self {
            amount: None,
            settlement_address: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new Capture for a partial amount.
    pub fn with_amount(amount: String) -> Self {
        Self {
            amount: Some(amount),
            settlement_address: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the settlement address.
    pub fn with_settlement_address(mut self, address: String) -> Self {
        self.settlement_address = Some(address);
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: String, value: Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl Default for Capture {
    fn default() -> Self {
        Self::new()
    }
}

impl TapMessageBody for Capture {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#Capture"
    }

    fn validate(&self) -> Result<()> {
        if let Some(ref amount) = self.amount {
            if amount.is_empty() {
                return Err(Error::Validation(
                    "Capture amount cannot be empty".to_string(),
                ));
            }
        }

        if let Some(ref address) = self.settlement_address {
            if address.is_empty() {
                return Err(Error::Validation(
                    "Capture settlement_address cannot be empty".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_with_asset() {
        let originator = Party::new("did:example:alice");
        let beneficiary = Party::new("did:example:bob");
        let agent1 = Agent::new(
            "did:example:alice-wallet",
            "OriginatorAgent",
            "did:example:alice",
        );
        let agent2 = Agent::new(
            "did:example:bob-wallet",
            "BeneficiaryAgent",
            "did:example:bob",
        );
        let escrow_agent = Agent::new(
            "did:example:escrow-service",
            "EscrowAgent",
            "did:example:escrow-service",
        );

        let lock = Lock::new_with_asset(
            "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(),
            "100.00".to_string(),
            originator,
            beneficiary,
            "2025-06-25T00:00:00Z".to_string(),
            vec![agent1, agent2, escrow_agent],
        );

        assert!(lock.validate().is_ok());
        assert!(lock.escrow_agent().is_some());
        assert_eq!(
            lock.escrow_agent().unwrap().role,
            Some("EscrowAgent".to_string())
        );
    }

    #[test]
    fn test_lock_with_currency() {
        let originator = Party::new("did:example:buyer");
        let beneficiary = Party::new("did:example:seller");
        let escrow_agent = Agent::new(
            "did:example:escrow-bank",
            "EscrowAgent",
            "did:example:escrow-bank",
        );

        let lock = Lock::new_with_currency(
            "USD".to_string(),
            "500.00".to_string(),
            originator,
            beneficiary,
            "2025-07-01T00:00:00Z".to_string(),
            vec![escrow_agent],
        )
        .with_agreement("https://marketplace.example/purchase/98765".to_string());

        assert!(lock.validate().is_ok());
        assert_eq!(lock.currency, Some("USD".to_string()));
        assert_eq!(
            lock.agreement,
            Some("https://marketplace.example/purchase/98765".to_string())
        );
    }

    #[test]
    fn test_lock_validation_errors() {
        let originator = Party::new("did:example:alice");
        let beneficiary = Party::new("did:example:bob");

        let lock_no_agent = Lock::new_with_asset(
            "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(),
            "100.00".to_string(),
            originator.clone(),
            beneficiary.clone(),
            "2025-06-25T00:00:00Z".to_string(),
            vec![],
        );
        assert!(lock_no_agent.validate().is_err());

        let mut lock_both = Lock::new_with_asset(
            "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(),
            "100.00".to_string(),
            originator.clone(),
            beneficiary.clone(),
            "2025-06-25T00:00:00Z".to_string(),
            vec![Agent::new(
                "did:example:escrow",
                "EscrowAgent",
                "did:example:escrow",
            )],
        );
        lock_both.currency = Some("USD".to_string());
        assert!(lock_both.validate().is_err());

        let lock_same_party = Lock::new_with_currency(
            "USD".to_string(),
            "100.00".to_string(),
            originator.clone(),
            originator.clone(),
            "2025-06-25T00:00:00Z".to_string(),
            vec![Agent::new(
                "did:example:escrow",
                "EscrowAgent",
                "did:example:escrow",
            )],
        );
        assert!(lock_same_party.validate().is_err());
    }

    #[test]
    fn test_capture() {
        let capture = Capture::new();
        assert!(capture.validate().is_ok());
        assert!(capture.amount.is_none());
        assert!(capture.settlement_address.is_none());

        let capture_with_amount = Capture::with_amount("95.00".to_string())
            .with_settlement_address(
                "eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f1234".to_string(),
            );
        assert!(capture_with_amount.validate().is_ok());
        assert_eq!(capture_with_amount.amount, Some("95.00".to_string()));
        assert_eq!(
            capture_with_amount.settlement_address,
            Some("eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f1234".to_string())
        );
    }

    #[test]
    fn test_capture_validation_errors() {
        let mut capture = Capture::new();
        capture.amount = Some("".to_string());
        assert!(capture.validate().is_err());

        let mut capture2 = Capture::new();
        capture2.settlement_address = Some("".to_string());
        assert!(capture2.validate().is_err());
    }
}
