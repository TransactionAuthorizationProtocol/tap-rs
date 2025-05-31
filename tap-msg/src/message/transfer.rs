//! Transfer message implementation for the Transaction Authorization Protocol.
//!
//! This module defines the Transfer message type and its builder, which is
//! the foundational message type for initiating a transfer in the TAP protocol.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tap_caip::AssetId;

use crate::error::{Error, Result};
use crate::message::tap_message_trait::{TapMessage as TapMessageTrait, TapMessageBody};
use crate::message::Participant;
use crate::TapMessage;

/// Transfer message body (TAIP-3).
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
#[tap(
    message_type = "https://tap.rsvp/schema/1.0#Transfer",
    initiator,
    authorizable,
    transactable
)]
pub struct Transfer {
    /// Network asset identifier (CAIP-19 format).
    pub asset: AssetId,

    /// Originator information.
    #[serde(rename = "originator")]
    #[tap(participant)]
    pub originator: Participant,

    /// Beneficiary information (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tap(participant)]
    pub beneficiary: Option<Participant>,

    /// Transfer amount.
    pub amount: String,

    /// Agents involved in the transfer.
    #[serde(default)]
    #[tap(participant_list)]
    pub agents: Vec<Participant>,

    /// Memo for the transfer (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    /// Settlement identifier (optional).
    #[serde(rename = "settlementId", skip_serializing_if = "Option::is_none")]
    pub settlement_id: Option<String>,

    /// Transaction identifier.
    #[tap(transaction_id)]
    pub transaction_id: String,

    /// Connection ID for linking to Connect messages
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tap(connection_id)]
    pub connection_id: Option<String>,

    /// Additional metadata for the transfer.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Transfer {
    /// Create a new Transfer
    ///
    /// # Example
    /// ```
    /// use tap_msg::message::{Transfer, Participant, Party};
    /// use tap_caip::{AssetId, ChainId};
    /// use std::collections::HashMap;
    ///
    /// // Create chain ID and asset ID
    /// let chain_id = ChainId::new("eip155", "1").unwrap();
    /// let asset = AssetId::new(chain_id, "erc20", "0x6b175474e89094c44da98b954eedeac495271d0f").unwrap();
    ///
    /// // Create originator participant
    /// let originator = Participant::from_party(Party::new("did:example:alice"));
    ///
    /// // Create a transfer with required fields
    /// let transfer = Transfer::builder()
    ///     .asset(asset)
    ///     .originator(originator)
    ///     .amount("100".to_string())
    ///     .build();
    /// ```
    pub fn builder() -> TransferBuilder {
        TransferBuilder::default()
    }

    /// Generates a unique message ID for authorization, rejection, or settlement
    pub fn message_id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

/// Builder for creating Transfer objects in a more idiomatic way
#[derive(Default)]
pub struct TransferBuilder {
    asset: Option<AssetId>,
    originator: Option<Participant>,
    amount: Option<String>,
    beneficiary: Option<Participant>,
    settlement_id: Option<String>,
    memo: Option<String>,
    transaction_id: Option<String>,
    agents: Vec<Participant>,
    metadata: HashMap<String, serde_json::Value>,
}

impl TransferBuilder {
    /// Set the asset for this transfer
    pub fn asset(mut self, asset: AssetId) -> Self {
        self.asset = Some(asset);
        self
    }

    /// Set the originator for this transfer
    pub fn originator(mut self, originator: Participant) -> Self {
        self.originator = Some(originator);
        self
    }

    /// Set the amount for this transfer
    pub fn amount(mut self, amount: String) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the beneficiary for this transfer
    pub fn beneficiary(mut self, beneficiary: Participant) -> Self {
        self.beneficiary = Some(beneficiary);
        self
    }

    /// Set the settlement ID for this transfer
    pub fn settlement_id(mut self, settlement_id: String) -> Self {
        self.settlement_id = Some(settlement_id);
        self
    }

    /// Set the memo for this transfer
    pub fn memo(mut self, memo: String) -> Self {
        self.memo = Some(memo);
        self
    }

    /// Set the transaction ID for this transfer
    pub fn transaction_id(mut self, transaction_id: String) -> Self {
        self.transaction_id = Some(transaction_id);
        self
    }

    /// Add an agent to this transfer
    pub fn add_agent(mut self, agent: Participant) -> Self {
        self.agents.push(agent);
        self
    }

    /// Set all agents for this transfer
    pub fn agents(mut self, agents: Vec<Participant>) -> Self {
        self.agents = agents;
        self
    }

    /// Add a metadata field
    pub fn add_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set all metadata for this transfer
    pub fn metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Build the Transfer object
    ///
    /// # Panics
    ///
    /// Panics if required fields (asset, originator, amount) are not set
    pub fn build(self) -> Transfer {
        Transfer {
            asset: self.asset.expect("Asset is required"),
            originator: self.originator.expect("Originator is required"),
            amount: self.amount.expect("Amount is required"),
            beneficiary: self.beneficiary,
            settlement_id: self.settlement_id,
            memo: self.memo,
            transaction_id: self
                .transaction_id
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            agents: self.agents,
            connection_id: None,
            metadata: self.metadata,
        }
    }

    /// Try to build the Transfer object, returning an error if required fields are missing
    pub fn try_build(self) -> Result<Transfer> {
        let asset = self
            .asset
            .ok_or_else(|| Error::Validation("Asset is required".to_string()))?;
        let originator = self
            .originator
            .ok_or_else(|| Error::Validation("Originator is required".to_string()))?;
        let amount = self
            .amount
            .ok_or_else(|| Error::Validation("Amount is required".to_string()))?;

        let transfer = Transfer {
            transaction_id: self
                .transaction_id
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            asset,
            originator,
            amount,
            beneficiary: self.beneficiary,
            settlement_id: self.settlement_id,
            memo: self.memo,
            agents: self.agents,
            connection_id: None,
            metadata: self.metadata,
        };

        // Validate the created transfer
        transfer.validate()?;

        Ok(transfer)
    }
}

impl Transfer {
    /// Custom validation for Transfer messages
    pub fn validate(&self) -> Result<()> {
        // Validate asset
        if self.asset.namespace().is_empty() || self.asset.reference().is_empty() {
            return Err(Error::Validation("Asset ID is invalid".to_string()));
        }

        // Validate originator
        if self.originator.id().is_empty() {
            return Err(Error::Validation("Originator ID is required".to_string()));
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

        Ok(())
    }
}
