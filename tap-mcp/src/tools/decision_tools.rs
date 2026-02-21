//! Tools for managing external decisions

use super::{default_limit, error_text_response, success_text_response, ToolHandler};
use crate::error::Result;
use crate::mcp::protocol::{CallToolResult, Tool};
use crate::tap_integration::TapIntegration;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tap_node::storage::DecisionStatus;
use tracing::{debug, error};

// -----------------------------------------------------------------------
// tap_list_pending_decisions
// -----------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListPendingDecisionsInput {
    pub agent_did: String,
    pub status: Option<String>,
    pub since_id: Option<i64>,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

#[derive(Debug, Serialize)]
pub struct DecisionOutput {
    pub id: i64,
    pub transaction_id: String,
    pub agent_did: String,
    pub decision_type: String,
    pub context: Value,
    pub status: String,
    pub resolution: Option<String>,
    pub resolution_detail: Option<Value>,
    pub created_at: String,
    pub delivered_at: Option<String>,
    pub resolved_at: Option<String>,
}

pub struct ListPendingDecisionsTool {
    tap_integration: Arc<TapIntegration>,
}

impl ListPendingDecisionsTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }
}

#[async_trait]
impl ToolHandler for ListPendingDecisionsTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let input: ListPendingDecisionsInput = match arguments {
            Some(args) => serde_json::from_value(args)?,
            None => {
                return Ok(error_text_response(
                    "Missing required arguments".to_string(),
                ));
            }
        };

        debug!(
            "Listing decisions for agent: {} status: {:?}",
            input.agent_did, input.status
        );

        let status = input
            .status
            .as_deref()
            .and_then(|s| DecisionStatus::try_from(s).ok());

        let storage = match self
            .tap_integration
            .storage_for_agent(&input.agent_did)
            .await
        {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to get agent storage: {}", e);
                return Ok(error_text_response(format!(
                    "Failed to get storage for agent {}: {}",
                    input.agent_did, e
                )));
            }
        };

        let entries = match storage
            .list_decisions(Some(&input.agent_did), status, input.since_id, input.limit)
            .await
        {
            Ok(entries) => entries,
            Err(e) => {
                error!("Failed to list decisions: {}", e);
                return Ok(error_text_response(format!(
                    "Failed to list decisions: {}",
                    e
                )));
            }
        };

        let decisions: Vec<DecisionOutput> = entries
            .into_iter()
            .map(|e| DecisionOutput {
                id: e.id,
                transaction_id: e.transaction_id,
                agent_did: e.agent_did,
                decision_type: e.decision_type.to_string(),
                context: e.context_json,
                status: e.status.to_string(),
                resolution: e.resolution,
                resolution_detail: e.resolution_detail,
                created_at: e.created_at,
                delivered_at: e.delivered_at,
                resolved_at: e.resolved_at,
            })
            .collect();

        let total = decisions.len();
        let response = json!({
            "decisions": decisions,
            "total": total,
        });

        Ok(success_text_response(
            serde_json::to_string_pretty(&response).unwrap(),
        ))
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_list_pending_decisions".to_string(),
            description: "List pending decisions from the decision log. Returns decisions that need external resolution (authorization, policy satisfaction, settlement).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "agent_did": {
                        "type": "string",
                        "description": "The DID of the agent whose decisions to list"
                    },
                    "status": {
                        "type": "string",
                        "description": "Filter by status: pending, delivered, resolved, expired",
                        "enum": ["pending", "delivered", "resolved", "expired"]
                    },
                    "since_id": {
                        "type": "number",
                        "description": "Only return decisions with ID greater than this value (for pagination/replay)"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of decisions to return",
                        "default": 50
                    }
                },
                "required": ["agent_did"],
                "additionalProperties": false
            }),
        }
    }
}

// -----------------------------------------------------------------------
// tap_resolve_decision
// -----------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ResolveDecisionInput {
    pub agent_did: String,
    pub decision_id: i64,
    pub action: String,
    pub detail: Option<Value>,
}

pub struct ResolveDecisionTool {
    tap_integration: Arc<TapIntegration>,
}

impl ResolveDecisionTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }
}

#[async_trait]
impl ToolHandler for ResolveDecisionTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let input: ResolveDecisionInput = match arguments {
            Some(args) => serde_json::from_value(args)?,
            None => {
                return Ok(error_text_response(
                    "Missing required arguments".to_string(),
                ));
            }
        };

        debug!(
            "Resolving decision {} with action: {}",
            input.decision_id, input.action
        );

        let storage = match self
            .tap_integration
            .storage_for_agent(&input.agent_did)
            .await
        {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to get agent storage: {}", e);
                return Ok(error_text_response(format!(
                    "Failed to get storage for agent {}: {}",
                    input.agent_did, e
                )));
            }
        };

        // Verify the decision exists and is actionable
        let entry = match storage.get_decision_by_id(input.decision_id).await {
            Ok(Some(e)) => e,
            Ok(None) => {
                return Ok(error_text_response(format!(
                    "Decision {} not found",
                    input.decision_id
                )));
            }
            Err(e) => {
                error!("Failed to get decision: {}", e);
                return Ok(error_text_response(format!(
                    "Failed to get decision: {}",
                    e
                )));
            }
        };

        // Only pending or delivered decisions can be resolved
        if entry.status != DecisionStatus::Pending && entry.status != DecisionStatus::Delivered {
            return Ok(error_text_response(format!(
                "Decision {} is already {} and cannot be resolved",
                input.decision_id, entry.status
            )));
        }

        // Mark the decision as resolved
        if let Err(e) = storage
            .update_decision_status(
                input.decision_id,
                DecisionStatus::Resolved,
                Some(&input.action),
                input.detail.as_ref(),
            )
            .await
        {
            error!("Failed to update decision status: {}", e);
            return Ok(error_text_response(format!(
                "Failed to resolve decision: {}",
                e
            )));
        }

        let now = chrono::Utc::now().to_rfc3339();
        let response = json!({
            "decision_id": input.decision_id,
            "transaction_id": entry.transaction_id,
            "status": "resolved",
            "action": input.action,
            "resolved_at": now,
        });

        Ok(success_text_response(
            serde_json::to_string_pretty(&response).unwrap(),
        ))
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_resolve_decision".to_string(),
            description: "Resolve a pending decision by specifying the action to take. This marks the decision as resolved in the decision log.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "agent_did": {
                        "type": "string",
                        "description": "The DID of the agent that owns this decision"
                    },
                    "decision_id": {
                        "type": "number",
                        "description": "The ID of the decision to resolve"
                    },
                    "action": {
                        "type": "string",
                        "description": "The action to take: authorize, reject, settle, cancel, present, defer, update_policies"
                    },
                    "detail": {
                        "type": "object",
                        "description": "Optional detail about the resolution (e.g., settlement_address, reason)"
                    }
                },
                "required": ["agent_did", "decision_id", "action"],
                "additionalProperties": false
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tap_integration::TapIntegration;
    use tap_node::storage::DecisionType;
    use tempfile::tempdir;

    async fn setup_test() -> (Arc<TapIntegration>, String) {
        let dir = tempdir().unwrap();
        let tap_root = dir.path().to_str().unwrap();

        // Create an ephemeral agent for testing
        let (agent, did) = tap_agent::TapAgent::from_ephemeral_key().await.unwrap();
        let agent_arc = Arc::new(agent);

        let integration = TapIntegration::new(Some(&did), Some(tap_root), Some(agent_arc))
            .await
            .unwrap();

        // Leak the tempdir so it doesn't get cleaned up during test
        std::mem::forget(dir);

        (Arc::new(integration), did)
    }

    #[tokio::test]
    async fn test_list_pending_decisions_empty() {
        let (integration, did) = setup_test().await;
        let tool = ListPendingDecisionsTool::new(integration);

        let result = tool
            .handle(Some(json!({
                "agent_did": did,
            })))
            .await
            .unwrap();

        assert_eq!(result.is_error, Some(false));
        let text = match &result.content[0] {
            crate::mcp::protocol::ToolContent::Text { text } => text,
            _ => panic!("Expected text content"),
        };
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["total"], 0);
        assert_eq!(parsed["decisions"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_list_pending_decisions_with_entries() {
        let (integration, did) = setup_test().await;

        // Insert test decisions directly
        let storage = integration.storage_for_agent(&did).await.unwrap();
        let context = json!({"transaction": {"type": "transfer", "amount": "100"}});
        storage
            .insert_decision(
                "txn-200",
                &did,
                DecisionType::AuthorizationRequired,
                &context,
            )
            .await
            .unwrap();
        storage
            .insert_decision("txn-201", &did, DecisionType::SettlementRequired, &context)
            .await
            .unwrap();

        let tool = ListPendingDecisionsTool::new(integration);
        let result = tool
            .handle(Some(json!({
                "agent_did": did,
                "status": "pending",
            })))
            .await
            .unwrap();

        assert_eq!(result.is_error, Some(false));
        let text = match &result.content[0] {
            crate::mcp::protocol::ToolContent::Text { text } => text,
            _ => panic!("Expected text content"),
        };
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["total"], 2);
    }

    #[tokio::test]
    async fn test_resolve_decision_success() {
        let (integration, did) = setup_test().await;

        let storage = integration.storage_for_agent(&did).await.unwrap();
        let context = json!({"transaction": {"type": "transfer"}});
        let decision_id = storage
            .insert_decision(
                "txn-300",
                &did,
                DecisionType::AuthorizationRequired,
                &context,
            )
            .await
            .unwrap();

        let tool = ResolveDecisionTool::new(integration.clone());
        let result = tool
            .handle(Some(json!({
                "agent_did": did,
                "decision_id": decision_id,
                "action": "authorize",
                "detail": {"settlement_address": "eip155:1:0xABC"},
            })))
            .await
            .unwrap();

        assert_eq!(result.is_error, Some(false));
        let text = match &result.content[0] {
            crate::mcp::protocol::ToolContent::Text { text } => text,
            _ => panic!("Expected text content"),
        };
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["status"], "resolved");
        assert_eq!(parsed["action"], "authorize");
        assert_eq!(parsed["decision_id"], decision_id);

        // Verify in storage
        let entry = storage
            .get_decision_by_id(decision_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(entry.status, DecisionStatus::Resolved);
        assert_eq!(entry.resolution.as_deref(), Some("authorize"));
    }

    #[tokio::test]
    async fn test_resolve_decision_not_found() {
        let (integration, did) = setup_test().await;

        let tool = ResolveDecisionTool::new(integration);
        let result = tool
            .handle(Some(json!({
                "agent_did": did,
                "decision_id": 99999,
                "action": "authorize",
            })))
            .await
            .unwrap();

        assert_eq!(result.is_error, Some(true));
    }

    #[tokio::test]
    async fn test_resolve_decision_already_resolved() {
        let (integration, did) = setup_test().await;

        let storage = integration.storage_for_agent(&did).await.unwrap();
        let context = json!({"transaction": {"type": "transfer"}});
        let decision_id = storage
            .insert_decision(
                "txn-301",
                &did,
                DecisionType::AuthorizationRequired,
                &context,
            )
            .await
            .unwrap();

        // Resolve it first
        storage
            .update_decision_status(decision_id, DecisionStatus::Resolved, Some("reject"), None)
            .await
            .unwrap();

        let tool = ResolveDecisionTool::new(integration);
        let result = tool
            .handle(Some(json!({
                "agent_did": did,
                "decision_id": decision_id,
                "action": "authorize",
            })))
            .await
            .unwrap();

        // Should fail because already resolved
        assert_eq!(result.is_error, Some(true));
    }
}
