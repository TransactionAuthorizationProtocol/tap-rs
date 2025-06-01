//! Transaction management tools

use super::schema;
use super::{error_text_response, success_text_response, ToolHandler};
use crate::error::{Error, Result};
use crate::mcp::protocol::{CallToolResult, Tool};
use crate::tap_integration::TapIntegration;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tap_caip::AssetId;
use tap_msg::message::{Agent, Party, Transfer};
use tap_msg::message::tap_message_trait::TapMessageBody;
use tracing::{debug, error};
use uuid::Uuid;

/// Tool for creating transfer transactions
pub struct CreateTransferTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for creating a transfer
#[derive(Debug, Deserialize)]
struct CreateTransferParams {
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
        Self {
            tap_integration,
        }
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
            None => return Ok(error_text_response("Missing required parameters".to_string())),
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
            originator,
            beneficiary: Some(beneficiary),
            amount: params.amount,
            agents,
            memo: params.memo,
            settlement_id: None,
            connection_id: None,
            metadata: params
                .metadata
                .and_then(|v| v.as_object().map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect()))
                .unwrap_or_default(),
        };

        // Validate the transfer
        if let Err(e) = transfer.validate() {
            return Ok(error_text_response(format!("Transfer validation failed: {}", e)));
        }

        // Create DIDComm message
        let creator_did = &transfer.originator.id;
        let didcomm_message = match transfer.to_didcomm(creator_did) {
            Ok(msg) => msg,
            Err(e) => {
                return Ok(error_text_response(format!(
                    "Failed to create DIDComm message: {}",
                    e
                )));
            }
        };

        // Store the transaction in tap-node's database
        match self.tap_integration().storage().insert_transaction(&didcomm_message).await {
            Ok(_) => {
                // Log the message in the audit trail
                if let Err(e) = self
                    .tap_integration()
                    .storage()
                    .log_message(
                        &didcomm_message,
                        tap_node::storage::MessageDirection::Outgoing,
                        None,
                    )
                    .await
                {
                    error!("Failed to log message: {}", e);
                }

                let response = CreateTransferResponse {
                    transaction_id,
                    message_id: didcomm_message.id,
                    status: "created".to_string(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                };

                let response_json = serde_json::to_string_pretty(&response)
                    .map_err(|e| Error::tool_execution(format!("Failed to serialize response: {}", e)))?;

                Ok(success_text_response(response_json))
            }
            Err(e) => {
                error!("Failed to store transaction: {}", e);
                Ok(error_text_response(format!(
                    "Failed to store transaction: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap.create_transfer".to_string(),
            description: "Initiates a new transfer between parties using the TAP Transfer message (TAIP-3)".to_string(),
            input_schema: schema::create_transfer_schema(),
        }
    }
}