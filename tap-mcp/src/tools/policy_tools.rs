//! Policy management tools for TAP transactions

use super::schema;
use super::{error_text_response, success_text_response, ToolHandler};
use crate::error::{Error, Result};
use crate::mcp::protocol::{CallToolResult, Tool};
use crate::tap_integration::TapIntegration;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tap_msg::message::policy::Policy;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::UpdatePolicies;
use tracing::{debug, error};

/// Tool for updating policies on a transaction
pub struct UpdatePoliciesTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for updating policies
#[derive(Debug, Deserialize)]
struct UpdatePoliciesParams {
    agent_did: String, // The DID of the agent that will sign and send this message
    transaction_id: String,
    policies: Vec<PolicyInfo>,
}

#[derive(Debug, Deserialize)]
struct PolicyInfo {
    #[serde(rename = "@type")]
    policy_type: String,
    #[serde(flatten)]
    attributes: serde_json::Map<String, serde_json::Value>,
}

/// Response for updating policies
#[derive(Debug, Serialize)]
struct UpdatePoliciesResponse {
    transaction_id: String,
    message_id: String,
    status: String,
    policies_updated: usize,
    updated_at: String,
}

impl UpdatePoliciesTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for UpdatePoliciesTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: UpdatePoliciesParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Updating {} policies for transaction: {}",
            params.policies.len(),
            params.transaction_id
        );

        // Create policies from the PolicyInfo
        let policies: Vec<Policy> = params
            .policies
            .into_iter()
            .map(|policy_info| {
                // Convert PolicyInfo back to JSON then parse as Policy
                let mut policy_json = serde_json::Map::new();
                policy_json.insert(
                    "@type".to_string(),
                    serde_json::Value::String(policy_info.policy_type),
                );
                for (key, value) in policy_info.attributes {
                    policy_json.insert(key, value);
                }

                serde_json::from_value::<Policy>(serde_json::Value::Object(policy_json))
                    .map_err(|e| Error::invalid_parameter(format!("Invalid policy: {}", e)))
            })
            .collect::<Result<Vec<Policy>>>()?;

        // Create update policies message
        let update_policies = UpdatePolicies::new(&params.transaction_id, policies);

        // Validate the update policies message
        if let Err(e) = update_policies.validate() {
            return Ok(error_text_response(format!(
                "UpdatePolicies validation failed: {}",
                e
            )));
        }

        // Create DIDComm message using the specified agent DID
        let didcomm_message = match update_policies.to_didcomm(&params.agent_did) {
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
                "No recipient found for update policies message".to_string(),
            ));
        };

        debug!(
            "Sending update policies from {} to {} for transaction: {}",
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
                    "UpdatePolicies message sent successfully to {}, packed message length: {}",
                    recipient_did,
                    packed_message.len()
                );

                let response = UpdatePoliciesResponse {
                    transaction_id: params.transaction_id,
                    message_id: didcomm_message.id,
                    status: "sent".to_string(),
                    policies_updated: update_policies.policies.len(),
                    updated_at: chrono::Utc::now().to_rfc3339(),
                };

                let response_json = serde_json::to_string_pretty(&response).map_err(|e| {
                    Error::tool_execution(format!("Failed to serialize response: {}", e))
                })?;

                Ok(success_text_response(response_json))
            }
            Err(e) => {
                error!("Failed to send update policies message: {}", e);
                Ok(error_text_response(format!(
                    "Failed to send update policies message: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_update_policies".to_string(),
            description:
                "Updates policies for a TAP transaction using the UpdatePolicies message (TAIP-7)"
                    .to_string(),
            input_schema: schema::update_policies_schema(),
        }
    }
}
