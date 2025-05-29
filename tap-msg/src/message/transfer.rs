//! Transfer message implementation for the Transaction Authorization Protocol.
//!
//! This module defines the Transfer message type and its builder, which is
//! the foundational message type for initiating a transfer in the TAP protocol.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tap_caip::AssetId;

use crate::error::{Error, Result};
use crate::message::tap_message_trait::{Authorizable, Connectable, TapMessageBody};
use crate::message::{Authorize, Participant, Policy, RemoveAgent, ReplaceAgent, UpdatePolicies};
use crate::TapMessage;

/// Transfer message body (TAIP-3).
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
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

    /// Transaction identifier (not stored in the struct but accessible via the TapMessage trait).
    #[serde(skip)]
    #[tap(transaction_id)]
    pub transaction_id: String,

    /// Additional metadata for the transfer.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Transfer {
    /// Create a new Transfer
    ///
    /// # Example
    /// ```
    /// use tap_msg::message::Transfer;
    /// use tap_caip::{AssetId, ChainId};
    /// use tap_msg::message::Participant;
    /// use std::collections::HashMap;
    ///
    /// // Create chain ID and asset ID
    /// let chain_id = ChainId::new("eip155", "1").unwrap();
    /// let asset = AssetId::new(chain_id, "erc20", "0x6b175474e89094c44da98b954eedeac495271d0f").unwrap();
    ///
    /// // Create participant
    /// let originator = Participant {
    ///     id: "did:example:alice".to_string(),
    ///     role: Some("originator".to_string()),
    ///     policies: None,
    ///     leiCode: None, name: None,
    /// };
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

    /// Validate the Transfer
    pub fn validate(&self) -> Result<()> {
        // CAIP-19 asset ID is validated by the AssetId type
        // Validate asset
        if self.asset.namespace().is_empty() || self.asset.reference().is_empty() {
            return Err(Error::Validation("Asset ID is invalid".to_string()));
        }

        // Validate originator
        if self.originator.id.is_empty() {
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

        // Validate agents (if any are defined)
        for agent in &self.agents {
            if agent.id.is_empty() {
                return Err(Error::Validation("Agent ID cannot be empty".to_string()));
            }
        }

        Ok(())
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
            metadata: self.metadata,
        };

        // Validate the created transfer
        transfer.validate()?;

        Ok(transfer)
    }
}

impl Connectable for Transfer {
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

impl Authorizable for Transfer {
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

impl TapMessageBody for Transfer {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#transfer"
    }

    fn validate(&self) -> Result<()> {
        // Validate asset
        if self.asset.namespace().is_empty() || self.asset.reference().is_empty() {
            return Err(Error::Validation("Asset ID is invalid".to_string()));
        }

        // Validate originator
        if self.originator.id.is_empty() {
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

    fn to_didcomm(&self, from: &str) -> Result<crate::didcomm::PlainMessage> {
        // Serialize the Transfer to a JSON value
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

        // Add beneficiary DID if present
        if let Some(beneficiary) = &self.beneficiary {
            agent_dids.push(beneficiary.id.clone());
        }

        // Add DIDs from agents array
        for agent in &self.agents {
            agent_dids.push(agent.id.clone());
        }

        // Remove duplicates
        agent_dids.sort();
        agent_dids.dedup();

        // Remove the sender from the recipients list to avoid sending to self
        agent_dids.retain(|did| did != from);

        let now = chrono::Utc::now().timestamp() as u64;

        // Get the connection ID if this message is connected to a previous message
        let pthid = self
            .connection_id()
            .map(|connect_id| connect_id.to_string());

        // Create a new Message with required fields
        let message = crate::didcomm::PlainMessage {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body: body_json,
            from: from.to_string(),
            to: agent_dids,
            thid: None,
            pthid,
            created_time: Some(now),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
            attachments: None,
        };

        Ok(message)
    }
}
