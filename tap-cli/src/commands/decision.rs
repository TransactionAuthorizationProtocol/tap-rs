use crate::error::{Error, Result};
use crate::output::{print_success, OutputFormat};
use crate::tap_integration::TapIntegration;
use clap::Subcommand;
use serde::Serialize;
use serde_json::Value;
use tap_node::storage::{DecisionStatus, DecisionType};
use tracing::debug;

#[derive(Subcommand, Debug)]
pub enum DecisionCommands {
    /// List decisions from the decision log
    #[command(long_about = "\
List decisions from the decision log.

Decisions are created when the TAP node reaches a decision point in the \
transaction lifecycle (e.g., a transfer needs authorization, or a transaction \
is ready for settlement). In poll mode (--decision-mode poll on tap-http), \
decisions accumulate in the database for external systems to act on.

Decision types:
  authorization_required      A new transaction needs approval
  policy_satisfaction_required Policies must be fulfilled before proceeding
  settlement_required         All agents authorized, ready to settle

Decision statuses:
  pending    Written to DB, not yet acted upon
  delivered  Sent to external process, awaiting action
  resolved   Action taken (authorize, reject, settle, etc.)
  expired    Transaction reached terminal state before resolution

Examples:
  # List all pending decisions
  tap-cli decision list --status pending

  # List all decisions (any status)
  tap-cli decision list

  # Paginate through decisions
  tap-cli decision list --since-id 100 --limit 20

  # List decisions for a specific agent
  tap-cli decision list --agent-did did:key:z6Mk...")]
    List {
        /// Agent DID for storage lookup (defaults to --agent-did global flag)
        #[arg(long)]
        agent_did: Option<String>,
        /// Filter by status: pending, delivered, resolved, expired
        #[arg(long)]
        status: Option<String>,
        /// Only return decisions with ID greater than this value (for pagination)
        #[arg(long)]
        since_id: Option<i64>,
        /// Maximum results
        #[arg(long, default_value = "50")]
        limit: u32,
    },
    /// Resolve a pending decision by specifying the action to take
    #[command(long_about = "\
Resolve a pending decision by specifying the action to take.

This marks the decision as resolved in the decision log. Only decisions \
with status 'pending' or 'delivered' can be resolved.

Valid actions per decision type:
  authorization_required:       authorize, reject, update_policies, defer
  policy_satisfaction_required:  present, reject, cancel, defer
  settlement_required:          settle, cancel, defer

The 'defer' action marks the decision as delivered rather than resolved, \
indicating you've seen it but will act later.

Note: This command only updates the decision log. To actually send the \
corresponding TAP message (e.g., Authorize), use the 'action' commands:
  tap-cli action authorize --transaction-id <ID>
  tap-cli action reject --transaction-id <ID> --reason <TEXT>
  tap-cli action settle --transaction-id <ID> --settlement-id <CAIP-220>

The action commands automatically resolve matching decisions when they succeed.

Examples:
  # Resolve a decision by authorizing the transaction
  tap-cli decision resolve --decision-id 42 --action authorize

  # Resolve with additional detail
  tap-cli decision resolve --decision-id 42 --action authorize \\
    --detail '{\"settlement_address\":\"eip155:1:0xABC\"}'

  # Reject a decision
  tap-cli decision resolve --decision-id 42 --action reject

  # Defer a decision (mark as seen, act later)
  tap-cli decision resolve --decision-id 42 --action defer")]
    Resolve {
        /// Decision ID to resolve (numeric, from 'decision list' output)
        #[arg(long)]
        decision_id: i64,
        /// Action to take: authorize, reject, settle, cancel, present, defer, update_policies
        #[arg(long)]
        action: String,
        /// Agent DID for storage lookup (defaults to --agent-did global flag)
        #[arg(long)]
        agent_did: Option<String>,
        /// Optional JSON detail about the resolution (e.g., settlement_address, reason)
        #[arg(long)]
        detail: Option<String>,
    },
}

#[derive(Debug, Serialize)]
struct DecisionInfo {
    id: i64,
    transaction_id: String,
    agent_did: String,
    decision_type: String,
    context: Value,
    status: String,
    resolution: Option<String>,
    resolution_detail: Option<Value>,
    created_at: String,
    delivered_at: Option<String>,
    resolved_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct DecisionListResponse {
    decisions: Vec<DecisionInfo>,
    total: usize,
}

#[derive(Debug, Serialize)]
struct DecisionResolveResponse {
    decision_id: i64,
    transaction_id: String,
    status: String,
    action: String,
    resolved_at: String,
}

pub async fn handle(
    cmd: &DecisionCommands,
    format: OutputFormat,
    default_agent_did: &str,
    tap_integration: &TapIntegration,
) -> Result<()> {
    match cmd {
        DecisionCommands::List {
            agent_did,
            status,
            since_id,
            limit,
        } => {
            let effective_did = agent_did.as_deref().unwrap_or(default_agent_did);
            let storage = tap_integration.storage_for_agent(effective_did).await?;

            let status_filter = status
                .as_deref()
                .map(DecisionStatus::try_from)
                .transpose()
                .map_err(|e| Error::invalid_parameter(format!("Invalid status: {}", e)))?;

            let entries = storage
                .list_decisions(Some(effective_did), status_filter, *since_id, *limit)
                .await?;

            let decisions: Vec<DecisionInfo> = entries
                .into_iter()
                .map(|e| DecisionInfo {
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

            let response = DecisionListResponse {
                total: decisions.len(),
                decisions,
            };
            print_success(format, &response);
            Ok(())
        }
        DecisionCommands::Resolve {
            decision_id,
            action,
            agent_did,
            detail,
        } => {
            let effective_did = agent_did.as_deref().unwrap_or(default_agent_did);
            let storage = tap_integration.storage_for_agent(effective_did).await?;

            let detail_value: Option<Value> = match detail {
                Some(d) => Some(serde_json::from_str(d).map_err(|e| {
                    Error::invalid_parameter(format!("Invalid JSON in --detail: {}", e))
                })?),
                None => None,
            };

            // Verify the decision exists and is actionable
            let entry = storage
                .get_decision_by_id(*decision_id)
                .await?
                .ok_or_else(|| {
                    Error::command_failed(format!("Decision {} not found", decision_id))
                })?;

            if entry.status != DecisionStatus::Pending && entry.status != DecisionStatus::Delivered
            {
                return Err(Error::command_failed(format!(
                    "Decision {} is already {} and cannot be resolved",
                    decision_id, entry.status
                )));
            }

            debug!("Resolving decision {} with action: {}", decision_id, action);

            storage
                .update_decision_status(
                    *decision_id,
                    DecisionStatus::Resolved,
                    Some(action),
                    detail_value.as_ref(),
                )
                .await?;

            let response = DecisionResolveResponse {
                decision_id: *decision_id,
                transaction_id: entry.transaction_id,
                status: "resolved".to_string(),
                action: action.clone(),
                resolved_at: chrono::Utc::now().to_rfc3339(),
            };
            print_success(format, &response);
            Ok(())
        }
    }
}

/// Resolve decisions in the decision_log after a successful action.
///
/// When an action command (authorize, reject, settle, cancel, revert) succeeds,
/// this function resolves matching pending/delivered decisions in the shared
/// database, matching the behavior of tap-mcp's auto-resolve.
pub async fn auto_resolve_decisions(
    tap_integration: &TapIntegration,
    agent_did: &str,
    transaction_id: &str,
    action: &str,
    decision_type: Option<DecisionType>,
) {
    if let Ok(storage) = tap_integration.storage_for_agent(agent_did).await {
        match storage
            .resolve_decisions_for_transaction(transaction_id, action, decision_type)
            .await
        {
            Ok(count) => {
                if count > 0 {
                    debug!(
                        "Auto-resolved {} decisions for transaction {} with action: {}",
                        count, transaction_id, action
                    );
                }
            }
            Err(e) => {
                debug!(
                    "Could not auto-resolve decisions for transaction {}: {}",
                    transaction_id, e
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tap_integration::TapIntegration;
    use serde_json::json;
    use tempfile::tempdir;

    async fn setup_test() -> (TapIntegration, String) {
        let dir = tempdir().unwrap();
        let tap_root = dir.path().to_str().unwrap();

        let (agent, did) = tap_agent::TapAgent::from_ephemeral_key().await.unwrap();
        let agent_arc = std::sync::Arc::new(agent);

        let integration = TapIntegration::new(Some(&did), Some(tap_root), Some(agent_arc))
            .await
            .unwrap();

        std::mem::forget(dir);
        (integration, did)
    }

    #[tokio::test]
    async fn test_decision_list_empty() {
        let (integration, did) = setup_test().await;

        let cmd = DecisionCommands::List {
            agent_did: Some(did.clone()),
            status: None,
            since_id: None,
            limit: 50,
        };

        let result = handle(&cmd, OutputFormat::Json, &did, &integration).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_decision_list_with_entries() {
        let (integration, did) = setup_test().await;

        let storage = integration.storage_for_agent(&did).await.unwrap();
        let context = json!({"transaction": {"type": "transfer", "amount": "100"}});
        storage
            .insert_decision(
                "txn-cli-1",
                &did,
                DecisionType::AuthorizationRequired,
                &context,
            )
            .await
            .unwrap();
        storage
            .insert_decision(
                "txn-cli-2",
                &did,
                DecisionType::SettlementRequired,
                &context,
            )
            .await
            .unwrap();

        let cmd = DecisionCommands::List {
            agent_did: Some(did.clone()),
            status: Some("pending".to_string()),
            since_id: None,
            limit: 50,
        };

        let result = handle(&cmd, OutputFormat::Json, &did, &integration).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_decision_list_with_status_filter() {
        let (integration, did) = setup_test().await;

        let storage = integration.storage_for_agent(&did).await.unwrap();
        let context = json!({"transaction": {"type": "transfer"}});
        let id = storage
            .insert_decision(
                "txn-cli-3",
                &did,
                DecisionType::AuthorizationRequired,
                &context,
            )
            .await
            .unwrap();

        // Resolve one
        storage
            .update_decision_status(id, DecisionStatus::Resolved, Some("authorize"), None)
            .await
            .unwrap();

        // Insert another that stays pending
        storage
            .insert_decision(
                "txn-cli-4",
                &did,
                DecisionType::SettlementRequired,
                &context,
            )
            .await
            .unwrap();

        // List only resolved
        let cmd = DecisionCommands::List {
            agent_did: Some(did.clone()),
            status: Some("resolved".to_string()),
            since_id: None,
            limit: 50,
        };

        let result = handle(&cmd, OutputFormat::Json, &did, &integration).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_decision_list_invalid_status() {
        let (integration, did) = setup_test().await;

        let cmd = DecisionCommands::List {
            agent_did: Some(did.clone()),
            status: Some("invalid_status".to_string()),
            since_id: None,
            limit: 50,
        };

        let result = handle(&cmd, OutputFormat::Json, &did, &integration).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_decision_resolve_success() {
        let (integration, did) = setup_test().await;

        let storage = integration.storage_for_agent(&did).await.unwrap();
        let context = json!({"transaction": {"type": "transfer"}});
        let decision_id = storage
            .insert_decision(
                "txn-cli-10",
                &did,
                DecisionType::AuthorizationRequired,
                &context,
            )
            .await
            .unwrap();

        let cmd = DecisionCommands::Resolve {
            decision_id,
            action: "authorize".to_string(),
            agent_did: Some(did.clone()),
            detail: Some(r#"{"settlement_address":"eip155:1:0xABC"}"#.to_string()),
        };

        let result = handle(&cmd, OutputFormat::Json, &did, &integration).await;
        assert!(result.is_ok());

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
    async fn test_decision_resolve_not_found() {
        let (integration, did) = setup_test().await;

        let cmd = DecisionCommands::Resolve {
            decision_id: 99999,
            action: "authorize".to_string(),
            agent_did: Some(did.clone()),
            detail: None,
        };

        let result = handle(&cmd, OutputFormat::Json, &did, &integration).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_decision_resolve_already_resolved() {
        let (integration, did) = setup_test().await;

        let storage = integration.storage_for_agent(&did).await.unwrap();
        let context = json!({"transaction": {"type": "transfer"}});
        let decision_id = storage
            .insert_decision(
                "txn-cli-11",
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

        let cmd = DecisionCommands::Resolve {
            decision_id,
            action: "authorize".to_string(),
            agent_did: Some(did.clone()),
            detail: None,
        };

        let result = handle(&cmd, OutputFormat::Json, &did, &integration).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_decision_resolve_invalid_detail_json() {
        let (integration, did) = setup_test().await;

        let storage = integration.storage_for_agent(&did).await.unwrap();
        let context = json!({"transaction": {"type": "transfer"}});
        let decision_id = storage
            .insert_decision(
                "txn-cli-12",
                &did,
                DecisionType::AuthorizationRequired,
                &context,
            )
            .await
            .unwrap();

        let cmd = DecisionCommands::Resolve {
            decision_id,
            action: "authorize".to_string(),
            agent_did: Some(did.clone()),
            detail: Some("not valid json".to_string()),
        };

        let result = handle(&cmd, OutputFormat::Json, &did, &integration).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_auto_resolve_decisions() {
        let (integration, did) = setup_test().await;

        let storage = integration.storage_for_agent(&did).await.unwrap();
        let context = json!({"transaction": {"type": "transfer"}});
        let decision_id = storage
            .insert_decision(
                "txn-cli-20",
                &did,
                DecisionType::AuthorizationRequired,
                &context,
            )
            .await
            .unwrap();

        // Auto-resolve
        super::auto_resolve_decisions(
            &integration,
            &did,
            "txn-cli-20",
            "authorize",
            Some(DecisionType::AuthorizationRequired),
        )
        .await;

        let entry = storage
            .get_decision_by_id(decision_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(entry.status, DecisionStatus::Resolved);
        assert_eq!(entry.resolution.as_deref(), Some("authorize"));
    }
}
