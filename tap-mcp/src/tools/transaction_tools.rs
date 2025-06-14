//! Transaction management tools

use super::schema;
use super::{default_limit, error_text_response, success_text_response, ToolHandler};
use crate::error::{Error, Result};
use crate::mcp::protocol::{CallToolResult, Tool};
use crate::tap_integration::TapIntegration;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tap_caip::AssetId;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::{
    Agent, Authorize, Cancel, Complete, Party, Reject, Revert, Settle, Transfer,
};
use tracing::{debug, error};
use uuid::Uuid;

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

        // Create parties
        let originator = Party::new(&params.originator.id);
        let beneficiary = Party::new(&params.beneficiary.id);

        // Create agents
        let agents: Vec<Agent> = params
            .agents
            .iter()
            .map(|agent_info| Agent::new(&agent_info.id, &agent_info.role, &agent_info.for_party))
            .collect();

        // Generate transaction ID
        let transaction_id = Uuid::new_v4().to_string();

        // Create transfer message
        let transfer = Transfer {
            transaction_id: transaction_id.clone(),
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
            "Sending transfer from {} to {} with transaction ID: {}",
            params.agent_did, recipient_did, transaction_id
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
                    transaction_id,
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

/// Tool for completing transactions
pub struct CompleteTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for completing a transaction
#[derive(Debug, Deserialize)]
struct CompleteParams {
    agent_did: String, // The DID of the agent that will sign and send this message
    transaction_id: String,
    settlement_address: String,
    #[serde(default)]
    amount: Option<String>,
}

/// Response for completing a transaction
#[derive(Debug, Serialize)]
struct CompleteResponse {
    transaction_id: String,
    message_id: String,
    status: String,
    settlement_address: String,
    amount: Option<String>,
    completed_at: String,
}

impl CompleteTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for CompleteTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: CompleteParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Completing transaction: {} with settlement_address: {}",
            params.transaction_id, params.settlement_address
        );

        // Create complete message
        let complete = Complete {
            transaction_id: params.transaction_id.clone(),
            settlement_address: params.settlement_address.clone(),
            amount: params.amount.clone(),
        };

        // Validate the complete message
        if let Err(e) = complete.validate() {
            return Ok(error_text_response(format!(
                "Complete validation failed: {}",
                e
            )));
        }

        // Create DIDComm message using the specified agent DID
        let didcomm_message = match complete.to_didcomm(&params.agent_did) {
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
                "No recipient found for complete message".to_string(),
            ));
        };

        debug!(
            "Sending complete from {} to {} for transaction: {}",
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
                    "Complete message sent successfully to {}, packed message length: {}",
                    recipient_did,
                    packed_message.len()
                );

                let response = CompleteResponse {
                    transaction_id: params.transaction_id,
                    message_id: didcomm_message.id,
                    status: "sent".to_string(),
                    settlement_address: params.settlement_address,
                    amount: params.amount,
                    completed_at: chrono::Utc::now().to_rfc3339(),
                };

                let response_json = serde_json::to_string_pretty(&response).map_err(|e| {
                    Error::tool_execution(format!("Failed to serialize response: {}", e))
                })?;

                Ok(success_text_response(response_json))
            }
            Err(e) => {
                error!("Failed to send complete message: {}", e);
                Ok(error_text_response(format!(
                    "Failed to send complete message: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_complete".to_string(),
            description: "Completes a TAP payment transaction using the Complete message (TAIP-13)"
                .to_string(),
            input_schema: schema::complete_schema(),
        }
    }
}
