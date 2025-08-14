//! Transaction management tools

use super::schema;
use super::{default_limit, error_text_response, success_text_response, ToolHandler};
use crate::error::{Error, Result};
use crate::mcp::protocol::{CallToolResult, Tool};
use crate::tap_integration::TapIntegration;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tap_caip::AssetId;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::{
    Agent, Authorize, Cancel, Capture, Connect, ConnectionConstraints, Escrow, Party, Payment,
    Reject, Revert, Settle, TransactionLimits, Transfer,
};
use tap_node::storage::models::SchemaType;
use tracing::{debug, error};
use uuid;

/// Tool for creating transfer transactions
pub struct CreateTransferTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for creating a transfer
#[derive(Debug, Deserialize)]
struct CreateTransferParams {
    agent_did: String, // The DID of the agent that will sign and send this message
    asset: String,
    amount: String,
    originator: PartyInfo,
    beneficiary: PartyInfo,
    #[serde(default)]
    agents: Vec<AgentInfo>,
    #[serde(default)]
    memo: Option<String>,
    #[serde(default)]
    metadata: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct PartyInfo {
    #[serde(rename = "@id")]
    id: String,
    #[serde(default)]
    metadata: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct AgentInfo {
    #[serde(rename = "@id")]
    id: String,
    role: String,
    #[serde(rename = "for")]
    for_party: String,
}

/// Response for creating a transfer
#[derive(Debug, Serialize)]
struct CreateTransferResponse {
    transaction_id: String,
    message_id: String,
    status: String,
    created_at: String,
}

impl CreateTransferTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for CreateTransferTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: CreateTransferParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Creating transfer: asset={}, amount={}, originator={}, beneficiary={}",
            params.asset, params.amount, params.originator.id, params.beneficiary.id
        );

        // Parse asset ID
        let asset_id = params
            .asset
            .parse::<AssetId>()
            .map_err(|e| Error::invalid_parameter(format!("Invalid asset ID: {}", e)))?;

        // Get storage for the agent to look up customer metadata
        let storage = match self
            .tap_integration()
            .storage_for_agent(&params.agent_did)
            .await
        {
            Ok(storage) => storage,
            Err(e) => {
                error!(
                    "Failed to get storage for agent {}: {}",
                    params.agent_did, e
                );
                return Ok(error_text_response(format!(
                    "Failed to get storage for agent {}: {}",
                    params.agent_did, e
                )));
            }
        };

        // Create parties with metadata from customer database if available
        let mut originator = Party::new(&params.originator.id);
        if let Ok(Some(customer)) = storage.get_customer(&params.originator.id).await {
            // Extract relevant metadata from customer profile
            if let Some(_profile) = customer.profile.as_object() {
                let mut metadata = HashMap::new();

                match customer.schema_type {
                    SchemaType::Person => {
                        // For natural persons, use name hash instead of PII
                        let full_name = match (&customer.given_name, &customer.family_name) {
                            (Some(given), Some(family)) => format!("{} {}", given, family),
                            (Some(given), None) => given.clone(),
                            (None, Some(family)) => family.clone(),
                            (None, None) => customer.display_name.clone().unwrap_or_default(),
                        };

                        if !full_name.is_empty() {
                            // Add name hash according to TAIP-12
                            originator = originator.with_name_hash(&full_name);
                        }

                        // Add address information if available (still needed for compliance)
                        if let Some(country) = customer.address_country {
                            metadata.insert(
                                "addressCountry".to_string(),
                                serde_json::Value::String(country),
                            );
                        }
                    }
                    SchemaType::Organization => {
                        // For organizations, include LEI code if available
                        if let Some(lei_code) = customer.lei_code {
                            originator = originator.with_lei(&lei_code);
                        }

                        // Add legal name for organizations
                        if let Some(legal_name) = customer.legal_name {
                            metadata.insert(
                                "legalName".to_string(),
                                serde_json::Value::String(legal_name),
                            );
                        }

                        // Add address information if available
                        if let Some(country) = customer.address_country {
                            metadata.insert(
                                "addressCountry".to_string(),
                                serde_json::Value::String(country),
                            );
                        }
                    }
                    SchemaType::Thing => {
                        // For other entity types, include minimal metadata
                        if let Some(display_name) = customer.display_name {
                            metadata.insert(
                                "name".to_string(),
                                serde_json::Value::String(display_name),
                            );
                        }
                    }
                }

                // Apply any additional metadata
                if !metadata.is_empty() {
                    originator = Party::with_metadata(&originator.id, metadata);
                }
            }
        }
        // Also merge any provided metadata
        if let Some(provided_metadata) = params.originator.metadata {
            if let Some(obj) = provided_metadata.as_object() {
                let mut metadata = originator.metadata.clone();
                for (k, v) in obj {
                    metadata.insert(k.clone(), v.clone());
                }
                originator = Party::with_metadata(&originator.id, metadata);
            }
        }

        let mut beneficiary = Party::new(&params.beneficiary.id);
        if let Ok(Some(customer)) = storage.get_customer(&params.beneficiary.id).await {
            // Extract relevant metadata from customer profile
            if let Some(_profile) = customer.profile.as_object() {
                let mut metadata = HashMap::new();

                match customer.schema_type {
                    SchemaType::Person => {
                        // For natural persons, use name hash instead of PII
                        let full_name = match (&customer.given_name, &customer.family_name) {
                            (Some(given), Some(family)) => format!("{} {}", given, family),
                            (Some(given), None) => given.clone(),
                            (None, Some(family)) => family.clone(),
                            (None, None) => customer.display_name.clone().unwrap_or_default(),
                        };

                        if !full_name.is_empty() {
                            // Add name hash according to TAIP-12
                            beneficiary = beneficiary.with_name_hash(&full_name);
                        }

                        // Add address information if available (still needed for compliance)
                        if let Some(country) = customer.address_country {
                            metadata.insert(
                                "addressCountry".to_string(),
                                serde_json::Value::String(country),
                            );
                        }
                    }
                    SchemaType::Organization => {
                        // For organizations, include LEI code if available
                        if let Some(lei_code) = customer.lei_code {
                            beneficiary = beneficiary.with_lei(&lei_code);
                        }

                        // Add legal name for organizations
                        if let Some(legal_name) = customer.legal_name {
                            metadata.insert(
                                "legalName".to_string(),
                                serde_json::Value::String(legal_name),
                            );
                        }

                        // Add address information if available
                        if let Some(country) = customer.address_country {
                            metadata.insert(
                                "addressCountry".to_string(),
                                serde_json::Value::String(country),
                            );
                        }
                    }
                    SchemaType::Thing => {
                        // For other entity types, include minimal metadata
                        if let Some(display_name) = customer.display_name {
                            metadata.insert(
                                "name".to_string(),
                                serde_json::Value::String(display_name),
                            );
                        }
                    }
                }

                // Apply any additional metadata
                if !metadata.is_empty() {
                    beneficiary = Party::with_metadata(&beneficiary.id, metadata);
                }
            }
        }
        // Also merge any provided metadata
        if let Some(provided_metadata) = params.beneficiary.metadata {
            if let Some(obj) = provided_metadata.as_object() {
                let mut metadata = beneficiary.metadata.clone();
                for (k, v) in obj {
                    metadata.insert(k.clone(), v.clone());
                }
                beneficiary = Party::with_metadata(&beneficiary.id, metadata);
            }
        }

        // Create agents
        let agents: Vec<Agent> = params
            .agents
            .iter()
            .map(|agent_info| Agent::new(&agent_info.id, &agent_info.role, &agent_info.for_party))
            .collect();

        // Create transfer message (transaction_id will be generated when creating DIDComm message)
        let transfer = Transfer {
            transaction_id: None,
            asset: asset_id,
            originator: Some(originator),
            beneficiary: Some(beneficiary),
            amount: params.amount,
            agents,
            memo: params.memo,
            settlement_id: None,
            connection_id: None,
            metadata: params
                .metadata
                .and_then(|v| {
                    v.as_object()
                        .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                })
                .unwrap_or_default(),
        };

        // Validate the transfer
        if let Err(e) = transfer.validate() {
            return Ok(error_text_response(format!(
                "Transfer validation failed: {}",
                e
            )));
        }

        // Create DIDComm message using the specified agent DID
        let didcomm_message = match transfer.to_didcomm(&params.agent_did) {
            Ok(msg) => msg,
            Err(e) => {
                return Ok(error_text_response(format!(
                    "Failed to create DIDComm message: {}",
                    e
                )));
            }
        };

        // Determine recipient - use beneficiary if available, otherwise first recipient in the message
        let recipient_did = if let Some(beneficiary) = &transfer.beneficiary {
            beneficiary.id.clone()
        } else if !didcomm_message.to.is_empty() {
            didcomm_message.to[0].clone()
        } else {
            return Ok(error_text_response(
                "No recipient found for transfer message".to_string(),
            ));
        };

        debug!(
            "Sending transfer from {} to {}",
            params.agent_did, recipient_did
        );

        // Send the message through the TAP node (this will handle storage, logging, and delivery tracking)
        match self
            .tap_integration()
            .node()
            .send_message(params.agent_did.clone(), didcomm_message.clone())
            .await
        {
            Ok(packed_message) => {
                debug!(
                    "Transfer message sent successfully to {}, packed message length: {}",
                    recipient_did,
                    packed_message.len()
                );

                let response = CreateTransferResponse {
                    transaction_id: didcomm_message
                        .thid
                        .clone()
                        .unwrap_or(didcomm_message.id.clone()),
                    message_id: didcomm_message.id,
                    status: "sent".to_string(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                };

                let response_json = serde_json::to_string_pretty(&response).map_err(|e| {
                    Error::tool_execution(format!("Failed to serialize response: {}", e))
                })?;

                Ok(success_text_response(response_json))
            }
            Err(e) => {
                error!("Failed to send transfer message: {}", e);
                Ok(error_text_response(format!(
                    "Failed to send transfer message: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_create_transfer".to_string(),
            description:
                "Initiates a new transfer between parties using the TAP Transfer message (TAIP-3)"
                    .to_string(),
            input_schema: schema::create_transfer_schema(),
        }
    }
}

/// Tool for authorizing transactions
pub struct AuthorizeTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for authorizing a transaction
#[derive(Debug, Deserialize)]
struct AuthorizeParams {
    agent_did: String, // The DID of the agent that will sign and send this message
    transaction_id: String,
    #[serde(default)]
    settlement_address: Option<String>,
    #[serde(default)]
    expiry: Option<String>,
}

/// Response for authorizing a transaction
#[derive(Debug, Serialize)]
struct AuthorizeResponse {
    transaction_id: String,
    message_id: String,
    status: String,
    authorized_at: String,
}

impl AuthorizeTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for AuthorizeTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: AuthorizeParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!("Authorizing transaction: {}", params.transaction_id);

        // Create authorize message
        let authorize = Authorize {
            transaction_id: params.transaction_id.clone(),
            settlement_address: params.settlement_address,
            expiry: params.expiry,
        };

        // Validate the authorize message
        if let Err(e) = authorize.validate() {
            return Ok(error_text_response(format!(
                "Authorize validation failed: {}",
                e
            )));
        }

        // Create DIDComm message using the specified agent DID
        let didcomm_message = match authorize.to_didcomm(&params.agent_did) {
            Ok(msg) => msg,
            Err(e) => {
                return Ok(error_text_response(format!(
                    "Failed to create DIDComm message: {}",
                    e
                )));
            }
        };

        // Determine recipient from the message
        let recipient_did = if !didcomm_message.to.is_empty() {
            didcomm_message.to[0].clone()
        } else {
            return Ok(error_text_response(
                "No recipient found for authorize message".to_string(),
            ));
        };

        debug!(
            "Sending authorize from {} to {} for transaction: {}",
            params.agent_did, recipient_did, params.transaction_id
        );

        // Send the message through the TAP node (this will handle storage, logging, and delivery tracking)
        match self
            .tap_integration()
            .node()
            .send_message(params.agent_did.clone(), didcomm_message.clone())
            .await
        {
            Ok(packed_message) => {
                debug!(
                    "Authorize message sent successfully to {}, packed message length: {}",
                    recipient_did,
                    packed_message.len()
                );

                let response = AuthorizeResponse {
                    transaction_id: params.transaction_id,
                    message_id: didcomm_message.id,
                    status: "sent".to_string(),
                    authorized_at: chrono::Utc::now().to_rfc3339(),
                };

                let response_json = serde_json::to_string_pretty(&response).map_err(|e| {
                    Error::tool_execution(format!("Failed to serialize response: {}", e))
                })?;

                Ok(success_text_response(response_json))
            }
            Err(e) => {
                error!("Failed to send authorize message: {}", e);
                Ok(error_text_response(format!(
                    "Failed to send authorize message: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_authorize".to_string(),
            description: "Authorizes a TAP transaction using the Authorize message (TAIP-4)"
                .to_string(),
            input_schema: schema::authorize_schema(),
        }
    }
}

/// Tool for rejecting transactions
pub struct RejectTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for rejecting a transaction
#[derive(Debug, Deserialize)]
struct RejectParams {
    agent_did: String, // The DID of the agent that will sign and send this message
    transaction_id: String,
    reason: String,
}

/// Response for rejecting a transaction
#[derive(Debug, Serialize)]
struct RejectResponse {
    transaction_id: String,
    message_id: String,
    status: String,
    reason: String,
    rejected_at: String,
}

impl RejectTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for RejectTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: RejectParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Rejecting transaction: {} with reason: {}",
            params.transaction_id, params.reason
        );

        // Create reject message
        let reject = Reject {
            transaction_id: params.transaction_id.clone(),
            reason: Some(params.reason.clone()),
        };

        // Validate the reject message
        if let Err(e) = reject.validate() {
            return Ok(error_text_response(format!(
                "Reject validation failed: {}",
                e
            )));
        }

        // Create DIDComm message using the specified agent DID
        let didcomm_message = match reject.to_didcomm(&params.agent_did) {
            Ok(msg) => msg,
            Err(e) => {
                return Ok(error_text_response(format!(
                    "Failed to create DIDComm message: {}",
                    e
                )));
            }
        };

        // Determine recipient from the message
        let recipient_did = if !didcomm_message.to.is_empty() {
            didcomm_message.to[0].clone()
        } else {
            return Ok(error_text_response(
                "No recipient found for reject message".to_string(),
            ));
        };

        debug!(
            "Sending reject from {} to {} for transaction: {}",
            params.agent_did, recipient_did, params.transaction_id
        );

        // Send the message through the TAP node (this will handle storage, logging, and delivery tracking)
        match self
            .tap_integration()
            .node()
            .send_message(params.agent_did.clone(), didcomm_message.clone())
            .await
        {
            Ok(packed_message) => {
                debug!(
                    "Reject message sent successfully to {}, packed message length: {}",
                    recipient_did,
                    packed_message.len()
                );

                let response = RejectResponse {
                    transaction_id: params.transaction_id,
                    message_id: didcomm_message.id,
                    status: "sent".to_string(),
                    reason: params.reason,
                    rejected_at: chrono::Utc::now().to_rfc3339(),
                };

                let response_json = serde_json::to_string_pretty(&response).map_err(|e| {
                    Error::tool_execution(format!("Failed to serialize response: {}", e))
                })?;

                Ok(success_text_response(response_json))
            }
            Err(e) => {
                error!("Failed to send reject message: {}", e);
                Ok(error_text_response(format!(
                    "Failed to send reject message: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_reject".to_string(),
            description: "Rejects a TAP transaction using the Reject message (TAIP-4)".to_string(),
            input_schema: schema::reject_schema(),
        }
    }
}

/// Tool for canceling transactions
pub struct CancelTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for canceling a transaction
#[derive(Debug, Deserialize)]
struct CancelParams {
    agent_did: String, // The DID of the agent that will sign and send this message
    transaction_id: String,
    by: String,
    #[serde(default)]
    reason: Option<String>,
}

/// Response for canceling a transaction
#[derive(Debug, Serialize)]
struct CancelResponse {
    transaction_id: String,
    message_id: String,
    status: String,
    canceled_by: String,
    reason: Option<String>,
    canceled_at: String,
}

impl CancelTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for CancelTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: CancelParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Canceling transaction: {} by: {}",
            params.transaction_id, params.by
        );

        // Create cancel message
        let cancel = Cancel {
            transaction_id: params.transaction_id.clone(),
            by: params.by.clone(),
            reason: params.reason.clone(),
        };

        // Validate the cancel message
        if let Err(e) = cancel.validate() {
            return Ok(error_text_response(format!(
                "Cancel validation failed: {}",
                e
            )));
        }

        // Create DIDComm message using the specified agent DID
        let didcomm_message = match cancel.to_didcomm(&params.agent_did) {
            Ok(msg) => msg,
            Err(e) => {
                return Ok(error_text_response(format!(
                    "Failed to create DIDComm message: {}",
                    e
                )));
            }
        };

        // Determine recipient from the message
        let recipient_did = if !didcomm_message.to.is_empty() {
            didcomm_message.to[0].clone()
        } else {
            return Ok(error_text_response(
                "No recipient found for cancel message".to_string(),
            ));
        };

        debug!(
            "Sending cancel from {} to {} for transaction: {}",
            params.agent_did, recipient_did, params.transaction_id
        );

        // Send the message through the TAP node (this will handle storage, logging, and delivery tracking)
        match self
            .tap_integration()
            .node()
            .send_message(params.agent_did.clone(), didcomm_message.clone())
            .await
        {
            Ok(packed_message) => {
                debug!(
                    "Cancel message sent successfully to {}, packed message length: {}",
                    recipient_did,
                    packed_message.len()
                );

                let response = CancelResponse {
                    transaction_id: params.transaction_id,
                    message_id: didcomm_message.id,
                    status: "sent".to_string(),
                    canceled_by: params.by,
                    reason: params.reason,
                    canceled_at: chrono::Utc::now().to_rfc3339(),
                };

                let response_json = serde_json::to_string_pretty(&response).map_err(|e| {
                    Error::tool_execution(format!("Failed to serialize response: {}", e))
                })?;

                Ok(success_text_response(response_json))
            }
            Err(e) => {
                error!("Failed to send cancel message: {}", e);
                Ok(error_text_response(format!(
                    "Failed to send cancel message: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_cancel".to_string(),
            description: "Cancels a TAP transaction using the Cancel message (TAIP-5)".to_string(),
            input_schema: schema::cancel_schema(),
        }
    }
}

/// Tool for settling transactions
pub struct SettleTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for settling a transaction
#[derive(Debug, Deserialize)]
struct SettleParams {
    agent_did: String, // The DID of the agent that will sign and send this message
    transaction_id: String,
    settlement_id: String,
    #[serde(default)]
    amount: Option<String>,
}

/// Response for settling a transaction
#[derive(Debug, Serialize)]
struct SettleResponse {
    transaction_id: String,
    settlement_id: String,
    message_id: String,
    status: String,
    amount: Option<String>,
    settled_at: String,
}

impl SettleTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for SettleTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: SettleParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Settling transaction: {} with settlement_id: {}",
            params.transaction_id, params.settlement_id
        );

        // Create settle message
        let settle = Settle {
            transaction_id: params.transaction_id.clone(),
            settlement_id: Some(params.settlement_id.clone()),
            amount: params.amount.clone(),
        };

        // Validate the settle message
        if let Err(e) = settle.validate() {
            return Ok(error_text_response(format!(
                "Settle validation failed: {}",
                e
            )));
        }

        // Create DIDComm message using the specified agent DID
        let didcomm_message = match settle.to_didcomm(&params.agent_did) {
            Ok(msg) => msg,
            Err(e) => {
                return Ok(error_text_response(format!(
                    "Failed to create DIDComm message: {}",
                    e
                )));
            }
        };

        // Determine recipient from the message
        let recipient_did = if !didcomm_message.to.is_empty() {
            didcomm_message.to[0].clone()
        } else {
            return Ok(error_text_response(
                "No recipient found for settle message".to_string(),
            ));
        };

        debug!(
            "Sending settle from {} to {} for transaction: {}",
            params.agent_did, recipient_did, params.transaction_id
        );

        // Send the message through the TAP node (this will handle storage, logging, and delivery tracking)
        match self
            .tap_integration()
            .node()
            .send_message(params.agent_did.clone(), didcomm_message.clone())
            .await
        {
            Ok(packed_message) => {
                debug!(
                    "Settle message sent successfully to {}, packed message length: {}",
                    recipient_did,
                    packed_message.len()
                );

                let response = SettleResponse {
                    transaction_id: params.transaction_id,
                    settlement_id: params.settlement_id,
                    message_id: didcomm_message.id,
                    status: "sent".to_string(),
                    amount: params.amount,
                    settled_at: chrono::Utc::now().to_rfc3339(),
                };

                let response_json = serde_json::to_string_pretty(&response).map_err(|e| {
                    Error::tool_execution(format!("Failed to serialize response: {}", e))
                })?;

                Ok(success_text_response(response_json))
            }
            Err(e) => {
                error!("Failed to send settle message: {}", e);
                Ok(error_text_response(format!(
                    "Failed to send settle message: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_settle".to_string(),
            description: "Settles a TAP transaction using the Settle message (TAIP-6)".to_string(),
            input_schema: schema::settle_schema(),
        }
    }
}

/// Tool for reverting transactions
pub struct RevertTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for reverting a transaction
#[derive(Debug, Deserialize)]
struct RevertParams {
    agent_did: String, // The DID of the agent that will sign and send this message
    transaction_id: String,
    settlement_address: String,
    reason: String,
}

/// Response for reverting a transaction
#[derive(Debug, Serialize)]
struct RevertResponse {
    transaction_id: String,
    message_id: String,
    status: String,
    reason: String,
    settlement_address: String,
    reverted_at: String,
}

impl RevertTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for RevertTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: RevertParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Reverting transaction: {} with reason: {}",
            params.transaction_id, params.reason
        );

        // Create revert message
        let revert = Revert {
            transaction_id: params.transaction_id.clone(),
            settlement_address: params.settlement_address.clone(),
            reason: params.reason.clone(),
        };

        // Validate the revert message
        if let Err(e) = revert.validate() {
            return Ok(error_text_response(format!(
                "Revert validation failed: {}",
                e
            )));
        }

        // Create DIDComm message using the specified agent DID
        let didcomm_message = match revert.to_didcomm(&params.agent_did) {
            Ok(msg) => msg,
            Err(e) => {
                return Ok(error_text_response(format!(
                    "Failed to create DIDComm message: {}",
                    e
                )));
            }
        };

        // Determine recipient from the message
        let recipient_did = if !didcomm_message.to.is_empty() {
            didcomm_message.to[0].clone()
        } else {
            return Ok(error_text_response(
                "No recipient found for revert message".to_string(),
            ));
        };

        debug!(
            "Sending revert from {} to {} for transaction: {}",
            params.agent_did, recipient_did, params.transaction_id
        );

        // Send the message through the TAP node
        match self
            .tap_integration()
            .node()
            .send_message(params.agent_did.clone(), didcomm_message.clone())
            .await
        {
            Ok(packed_message) => {
                debug!(
                    "Revert message sent successfully to {}, packed message length: {}",
                    recipient_did,
                    packed_message.len()
                );

                let response = RevertResponse {
                    transaction_id: params.transaction_id,
                    message_id: didcomm_message.id,
                    status: "sent".to_string(),
                    reason: params.reason,
                    settlement_address: params.settlement_address,
                    reverted_at: chrono::Utc::now().to_rfc3339(),
                };

                let response_json = serde_json::to_string_pretty(&response).map_err(|e| {
                    Error::tool_execution(format!("Failed to serialize response: {}", e))
                })?;

                Ok(success_text_response(response_json))
            }
            Err(e) => {
                error!("Failed to send revert message: {}", e);
                Ok(error_text_response(format!(
                    "Failed to send revert message: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_revert".to_string(),
            description: "Reverts a settled TAP transaction using the Revert message (TAIP-12)"
                .to_string(),
            input_schema: schema::revert_schema(),
        }
    }
}

/// Tool for listing transactions
pub struct ListTransactionsTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for listing transactions
#[derive(Debug, Deserialize, Serialize)]
struct ListTransactionsParams {
    agent_did: String, // The DID of the agent whose transactions to list
    #[serde(default)]
    filter: Option<TransactionFilter>,
    #[serde(default)]
    sort: Option<TransactionSort>,
    #[serde(default = "default_limit")]
    limit: u32,
    #[serde(default)]
    offset: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct TransactionFilter {
    message_type: Option<String>,
    thread_id: Option<String>,
    from_did: Option<String>,
    to_did: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TransactionSort {
    field: Option<String>,
    order: Option<String>,
}

/// Response for listing transactions
#[derive(Debug, Serialize)]
struct ListTransactionsResponse {
    transactions: Vec<TransactionInfo>,
    total: usize,
    applied_filters: ListTransactionsParams,
}

#[derive(Debug, Serialize)]
struct TransactionInfo {
    id: String,
    #[serde(rename = "type")]
    message_type: String,
    thread_id: Option<String>,
    from: Option<String>,
    to: Option<String>,
    direction: String,
    created_at: String,
    body: serde_json::Value,
}

impl ListTransactionsTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for ListTransactionsTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: ListTransactionsParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters: agent_did is required".to_string(),
                ))
            }
        };

        debug!(
            "Listing transactions for agent {} with limit: {}, offset: {}",
            params.agent_did, params.limit, params.offset
        );

        // Get messages from the agent's specific storage
        let storage = self
            .tap_integration()
            .storage_for_agent(&params.agent_did)
            .await?;
        let direction_filter = None; // No direction filter for now
        let messages = storage
            .list_messages(params.limit, params.offset, direction_filter)
            .await?;

        // Apply additional filters
        let filtered_messages: Vec<_> = messages
            .into_iter()
            .filter(|msg| {
                if let Some(ref filter) = params.filter {
                    if let Some(ref msg_type) = filter.message_type {
                        if !msg.message_type.contains(msg_type) {
                            return false;
                        }
                    }
                    if let Some(ref thread_id) = filter.thread_id {
                        if msg.thread_id.as_ref() != Some(thread_id) {
                            return false;
                        }
                    }
                    if let Some(ref from_did) = filter.from_did {
                        if msg.from_did.as_ref() != Some(from_did) {
                            return false;
                        }
                    }
                    if let Some(ref to_did) = filter.to_did {
                        if msg.to_did.as_ref() != Some(to_did) {
                            return false;
                        }
                    }
                    // TODO: Apply date filters
                }
                true
            })
            .collect();

        // Convert to transaction info
        let transactions: Vec<TransactionInfo> = filtered_messages
            .iter()
            .map(|msg| TransactionInfo {
                id: msg.message_id.clone(),
                message_type: msg.message_type.clone(),
                thread_id: msg.thread_id.clone(),
                from: msg.from_did.clone(),
                to: msg.to_did.clone(),
                direction: msg.direction.to_string(),
                created_at: msg.created_at.clone(),
                body: msg.message_json.clone(),
            })
            .collect();

        let response = ListTransactionsResponse {
            total: transactions.len(),
            transactions,
            applied_filters: params,
        };

        let response_json = serde_json::to_string_pretty(&response)
            .map_err(|e| Error::tool_execution(format!("Failed to serialize response: {}", e)))?;

        Ok(success_text_response(response_json))
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_list_transactions".to_string(),
            description: "Lists TAP transactions with filtering and pagination support".to_string(),
            input_schema: schema::list_transactions_schema(),
        }
    }
}
// New tools for Payment, Connect, Escrow, and Capture messages

/// Tool for creating Payment messages (TAIP-14)
pub struct CreatePaymentTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for creating a payment
#[derive(Debug, Deserialize)]
struct CreatePaymentParams {
    agent_did: String,
    #[serde(default)]
    asset: Option<String>,
    #[serde(default)]
    currency: Option<String>,
    amount: String,
    merchant: PartyInfo,
    #[serde(default)]
    agents: Vec<AgentInfo>,
    #[serde(default)]
    memo: Option<String>,
    #[serde(default)]
    invoice: Option<Value>,
    #[serde(default)]
    settlement_address: Option<String>,
    #[serde(default)]
    fallback_settlement_addresses: Option<Vec<String>>,
    #[serde(default)]
    metadata: Option<Value>,
}

/// Response for creating a payment
#[derive(Debug, Serialize)]
struct CreatePaymentResponse {
    transaction_id: String,
    message_id: String,
    status: String,
    created_at: String,
}

impl CreatePaymentTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for CreatePaymentTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: CreatePaymentParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Creating payment: amount={}, merchant={}",
            params.amount, params.merchant.id
        );

        // Create merchant party
        let mut merchant = Party::new(&params.merchant.id);
        if let Some(metadata) = params.merchant.metadata {
            if let Some(obj) = metadata.as_object() {
                for (key, value) in obj {
                    merchant = merchant.with_metadata_field(key.clone(), value.clone());
                }
            }
        }

        // Create agents
        let agents: Vec<Agent> = params
            .agents
            .iter()
            .map(|info| Agent::new(&info.id, &info.role, &info.for_party))
            .collect();

        // Create payment message based on whether it's asset or currency
        let mut payment = if let Some(asset) = params.asset {
            // Parse asset ID
            let asset_id = asset
                .parse::<AssetId>()
                .map_err(|e| Error::invalid_parameter(format!("Invalid asset ID: {}", e)))?;
            Payment::with_asset(asset_id, params.amount, merchant, agents)
        } else if let Some(currency) = params.currency {
            Payment::with_currency(currency, params.amount, merchant, agents)
        } else {
            return Ok(error_text_response(
                "Either asset or currency must be specified".to_string(),
            ));
        };

        // Add optional fields
        if let Some(memo) = params.memo {
            payment.memo = Some(memo);
        }
        // Note: Payment struct doesn't have settlement_address field
        // Settlement addresses are handled via fallback_settlement_addresses or through agents
        if let Some(_settlement_address) = params.settlement_address {
            // This would need to be handled through fallback_settlement_addresses field
            // or through an agent with SettlementAddress role
        }
        if let Some(metadata) = params.metadata {
            if let Some(obj) = metadata.as_object() {
                for (key, value) in obj {
                    payment.metadata.insert(key.clone(), value.clone());
                }
            }
        }

        // Validate the payment message
        if let Err(e) = payment.validate() {
            return Ok(error_text_response(format!(
                "Payment validation failed: {}",
                e
            )));
        }

        // Create DIDComm message
        let didcomm_message = match payment.to_didcomm(&params.agent_did) {
            Ok(msg) => msg,
            Err(e) => {
                return Ok(error_text_response(format!(
                    "Failed to create DIDComm message: {}",
                    e
                )));
            }
        };

        // Send the message through the TAP node
        match self
            .tap_integration()
            .node()
            .send_message(params.agent_did.clone(), didcomm_message.clone())
            .await
        {
            Ok(_) => {
                let response = CreatePaymentResponse {
                    transaction_id: didcomm_message.id.clone(),
                    message_id: didcomm_message.id,
                    status: "sent".to_string(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                };

                let response_json = serde_json::to_string_pretty(&response).map_err(|e| {
                    Error::tool_execution(format!("Failed to serialize response: {}", e))
                })?;

                Ok(success_text_response(response_json))
            }
            Err(e) => {
                error!("Failed to send payment: {}", e);
                Ok(error_text_response(format!(
                    "Failed to send payment: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_payment".to_string(),
            description: "Creates a TAP payment request (TAIP-14) with optional invoice"
                .to_string(),
            input_schema: schema::create_payment_schema(),
        }
    }
}

/// Tool for creating Connect messages (TAIP-15)
pub struct CreateConnectTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for creating a connect message
#[derive(Debug, Deserialize)]
struct CreateConnectParams {
    agent_did: String,
    recipient_did: String,
    for_party: String,
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    constraints: Option<ConnectionConstraintsInfo>,
    #[serde(default)]
    metadata: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ConnectionConstraintsInfo {
    #[serde(default)]
    transaction_limits: Option<TransactionLimitsInfo>,
    #[serde(default)]
    asset_types: Option<Vec<String>>,
    #[serde(default)]
    currency_types: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct TransactionLimitsInfo {
    #[serde(default)]
    max_amount: Option<String>,
    #[serde(default)]
    min_amount: Option<String>,
    #[serde(default)]
    daily_limit: Option<String>,
    #[serde(default)]
    monthly_limit: Option<String>,
}

/// Response for creating a connect message
#[derive(Debug, Serialize)]
struct CreateConnectResponse {
    connection_id: String,
    message_id: String,
    status: String,
    created_at: String,
}

impl CreateConnectTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for CreateConnectTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: CreateConnectParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Creating connect message from {} to {}",
            params.agent_did, params.recipient_did
        );

        // Create connect message
        // Connect requires transaction_id, agent_id, for_id, and optional role
        let transaction_id = format!("connect-{}", uuid::Uuid::new_v4());
        let mut connect = Connect::new(
            &transaction_id,
            &params.agent_did,
            &params.for_party,
            params.role.as_deref(),
        );

        // Add constraints if provided
        if let Some(constraints_info) = params.constraints {
            let mut constraints = ConnectionConstraints {
                purposes: None,
                category_purposes: None,
                limits: None,
            };

            if let Some(limits_info) = constraints_info.transaction_limits {
                let mut limits = TransactionLimits {
                    per_transaction: None,
                    daily: None,
                    currency: None,
                };
                // Map the fields to the actual TransactionLimits struct
                limits.per_transaction = limits_info.max_amount;
                limits.daily = limits_info.daily_limit;
                // Note: Currency and other fields would need to be handled separately
                constraints.limits = Some(limits);
            }

            // Note: ConnectionConstraints doesn't have asset_types and currency_types
            // These would need to be handled through purposes or category_purposes

            connect.constraints = Some(constraints);
        }

        // Add metadata if provided
        if let Some(metadata) = params.metadata {
            if let Some(obj) = metadata.as_object() {
                for (key, value) in obj {
                    // Note: Connect struct doesn't have direct metadata field
                    // Metadata would be handled through the principal or agent objects
                }
            }
        }

        // Validate the connect message
        if let Err(e) = connect.validate() {
            return Ok(error_text_response(format!(
                "Connect validation failed: {}",
                e
            )));
        }

        // Create DIDComm message
        let didcomm_message = match connect.to_didcomm(&params.agent_did) {
            Ok(mut msg) => {
                msg.to = vec![params.recipient_did.clone()];
                msg
            }
            Err(e) => {
                return Ok(error_text_response(format!(
                    "Failed to create DIDComm message: {}",
                    e
                )));
            }
        };

        // Send the message through the TAP node
        match self
            .tap_integration()
            .node()
            .send_message(params.agent_did.clone(), didcomm_message.clone())
            .await
        {
            Ok(_) => {
                let response = CreateConnectResponse {
                    connection_id: didcomm_message.id.clone(),
                    message_id: didcomm_message.id,
                    status: "sent".to_string(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                };

                let response_json = serde_json::to_string_pretty(&response).map_err(|e| {
                    Error::tool_execution(format!("Failed to serialize response: {}", e))
                })?;

                Ok(success_text_response(response_json))
            }
            Err(e) => {
                error!("Failed to send connect message: {}", e);
                Ok(error_text_response(format!(
                    "Failed to send connect message: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_connect".to_string(),
            description: "Creates a TAP connection request (TAIP-15) to establish a relationship between parties".to_string(),
            input_schema: schema::create_connect_schema(),
        }
    }
}

/// Tool for creating Escrow messages (TAIP-17)
pub struct CreateEscrowTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for creating an escrow
#[derive(Debug, Deserialize)]
struct CreateEscrowParams {
    agent_did: String,
    #[serde(default)]
    asset: Option<String>,
    #[serde(default)]
    currency: Option<String>,
    amount: String,
    originator: PartyInfo,
    beneficiary: PartyInfo,
    expiry: String,
    agents: Vec<AgentInfo>,
    #[serde(default)]
    agreement: Option<String>,
    #[serde(default)]
    metadata: Option<Value>,
}

/// Response for creating an escrow
#[derive(Debug, Serialize)]
struct CreateEscrowResponse {
    escrow_id: String,
    message_id: String,
    status: String,
    expiry: String,
    created_at: String,
}

impl CreateEscrowTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for CreateEscrowTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: CreateEscrowParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Creating escrow: amount={}, originator={}, beneficiary={}, expiry={}",
            params.amount, params.originator.id, params.beneficiary.id, params.expiry
        );

        // Create parties
        let mut originator = Party::new(&params.originator.id);
        if let Some(metadata) = params.originator.metadata {
            if let Some(obj) = metadata.as_object() {
                for (key, value) in obj {
                    originator = originator.with_metadata_field(key.clone(), value.clone());
                }
            }
        }

        let mut beneficiary = Party::new(&params.beneficiary.id);
        if let Some(metadata) = params.beneficiary.metadata {
            if let Some(obj) = metadata.as_object() {
                for (key, value) in obj {
                    beneficiary = beneficiary.with_metadata_field(key.clone(), value.clone());
                }
            }
        }

        // Create agents
        let agents: Vec<Agent> = params
            .agents
            .iter()
            .map(|info| Agent::new(&info.id, &info.role, &info.for_party))
            .collect();

        // Verify exactly one EscrowAgent exists
        let escrow_agent_count = agents
            .iter()
            .filter(|a| a.role == Some("EscrowAgent".to_string()))
            .count();
        if escrow_agent_count != 1 {
            return Ok(error_text_response(format!(
                "Escrow must have exactly one agent with role 'EscrowAgent', found {}",
                escrow_agent_count
            )));
        }

        // Create escrow message based on whether it's asset or currency
        let mut escrow = if let Some(asset) = params.asset {
            Escrow::new_with_asset(
                asset,
                params.amount,
                originator,
                beneficiary,
                params.expiry,
                agents,
            )
        } else if let Some(currency) = params.currency {
            Escrow::new_with_currency(
                currency,
                params.amount,
                originator,
                beneficiary,
                params.expiry,
                agents,
            )
        } else {
            return Ok(error_text_response(
                "Either asset or currency must be specified".to_string(),
            ));
        };

        // Add optional fields
        if let Some(agreement) = params.agreement {
            escrow = escrow.with_agreement(agreement);
        }
        if let Some(metadata) = params.metadata {
            if let Some(obj) = metadata.as_object() {
                for (key, value) in obj {
                    escrow = escrow.with_metadata(key.clone(), value.clone());
                }
            }
        }

        // Validate the escrow message
        if let Err(e) = escrow.validate() {
            return Ok(error_text_response(format!(
                "Escrow validation failed: {}",
                e
            )));
        }

        // Create DIDComm message
        let didcomm_message = match escrow.to_didcomm(&params.agent_did) {
            Ok(msg) => msg,
            Err(e) => {
                return Ok(error_text_response(format!(
                    "Failed to create DIDComm message: {}",
                    e
                )));
            }
        };

        // Send the message through the TAP node
        match self
            .tap_integration()
            .node()
            .send_message(params.agent_did.clone(), didcomm_message.clone())
            .await
        {
            Ok(_) => {
                let response = CreateEscrowResponse {
                    escrow_id: didcomm_message.id.clone(),
                    message_id: didcomm_message.id,
                    status: "created".to_string(),
                    expiry: escrow.expiry,
                    created_at: chrono::Utc::now().to_rfc3339(),
                };

                let response_json = serde_json::to_string_pretty(&response).map_err(|e| {
                    Error::tool_execution(format!("Failed to serialize response: {}", e))
                })?;

                Ok(success_text_response(response_json))
            }
            Err(e) => {
                error!("Failed to send escrow: {}", e);
                Ok(error_text_response(format!("Failed to send escrow: {}", e)))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_escrow".to_string(),
            description:
                "Creates a TAP escrow request (TAIP-17) for holding assets on behalf of parties"
                    .to_string(),
            input_schema: schema::create_escrow_schema(),
        }
    }
}

/// Tool for creating Capture messages (TAIP-17)
pub struct CaptureTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for capturing escrowed funds
#[derive(Debug, Deserialize)]
struct CaptureParams {
    agent_did: String,
    escrow_id: String,
    #[serde(default)]
    amount: Option<String>,
    #[serde(default)]
    settlement_address: Option<String>,
}

/// Response for capturing escrowed funds
#[derive(Debug, Serialize)]
struct CaptureResponse {
    escrow_id: String,
    message_id: String,
    status: String,
    amount_captured: Option<String>,
    captured_at: String,
}

impl CaptureTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for CaptureTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: CaptureParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!("Capturing escrow: {}", params.escrow_id);

        // Create capture message
        let mut capture = if let Some(amount) = params.amount.clone() {
            Capture::with_amount(amount)
        } else {
            Capture::new()
        };

        if let Some(address) = params.settlement_address {
            capture = capture.with_settlement_address(address);
        }

        // Validate the capture message
        if let Err(e) = capture.validate() {
            return Ok(error_text_response(format!(
                "Capture validation failed: {}",
                e
            )));
        }

        // Create DIDComm message with thread ID linking to the escrow
        let didcomm_message = match capture.to_didcomm(&params.agent_did) {
            Ok(mut msg) => {
                msg.thid = Some(params.escrow_id.clone());
                msg
            }
            Err(e) => {
                return Ok(error_text_response(format!(
                    "Failed to create DIDComm message: {}",
                    e
                )));
            }
        };

        // Send the message through the TAP node
        match self
            .tap_integration()
            .node()
            .send_message(params.agent_did.clone(), didcomm_message.clone())
            .await
        {
            Ok(_) => {
                let response = CaptureResponse {
                    escrow_id: params.escrow_id,
                    message_id: didcomm_message.id,
                    status: "sent".to_string(),
                    amount_captured: params.amount,
                    captured_at: chrono::Utc::now().to_rfc3339(),
                };

                let response_json = serde_json::to_string_pretty(&response).map_err(|e| {
                    Error::tool_execution(format!("Failed to serialize response: {}", e))
                })?;

                Ok(success_text_response(response_json))
            }
            Err(e) => {
                error!("Failed to send capture: {}", e);
                Ok(error_text_response(format!(
                    "Failed to send capture: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_capture".to_string(),
            description: "Captures escrowed funds (TAIP-17) to release them to the beneficiary"
                .to_string(),
            input_schema: schema::create_capture_schema(),
        }
    }
}
