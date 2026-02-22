use crate::error::{Error, Result};
use crate::output::{print_success, OutputFormat};
use crate::tap_integration::TapIntegration;
use clap::Subcommand;
use serde::Serialize;
use tap_msg::message::policy::Policy;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::{AddAgents, Agent, RemoveAgent, ReplaceAgent, UpdatePolicies};
use tracing::debug;

#[derive(Subcommand, Debug)]
pub enum AgentManagementCommands {
    /// Add agents to a transaction (TAIP-5)
    #[command(long_about = "\
Add agents to an existing transaction (TAIP-5).

Sends an AddAgents message to include new agents (VASPs, compliance officers, etc.) \
in the transaction.

Examples:
  tap-cli agent-mgmt add-agents --transaction-id <ID> \\
    --agents '[{\"@id\":\"did:key:z6Mk...\",\"role\":\"ComplianceOfficer\",\"for\":\"did:key:z6Mk...\"}]'")]
    AddAgents {
        /// Transaction ID to add agents to
        #[arg(long)]
        transaction_id: String,
        /// Agents as JSON array of objects with @id, role, and for fields
        #[arg(long)]
        agents: String,
    },
    /// Remove an agent from a transaction (TAIP-5)
    #[command(long_about = "\
Remove an agent from an existing transaction (TAIP-5).

Sends a RemoveAgent message to remove an agent from the transaction.

Examples:
  tap-cli agent-mgmt remove-agent --transaction-id <ID> --agent-to-remove did:key:z6Mk...")]
    RemoveAgent {
        /// Transaction ID to remove agent from
        #[arg(long)]
        transaction_id: String,
        /// DID of the agent to remove
        #[arg(long)]
        agent_to_remove: String,
    },
    /// Replace an agent in a transaction (TAIP-5)
    #[command(long_about = "\
Replace an agent in an existing transaction (TAIP-5).

Sends a ReplaceAgent message to swap one agent for another.

Examples:
  tap-cli agent-mgmt replace-agent --transaction-id <ID> \\
    --original did:key:z6MkOld... \\
    --new-agent '{\"@id\":\"did:key:z6MkNew...\",\"role\":\"SourceAgent\",\"for\":\"did:key:z6Mk...\"}'")]
    ReplaceAgent {
        /// Transaction ID to replace agent in
        #[arg(long)]
        transaction_id: String,
        /// DID of the agent to replace
        #[arg(long)]
        original: String,
        /// New agent as JSON object with @id, role, and for fields
        #[arg(long)]
        new_agent: String,
    },
    /// Update policies for a transaction (TAIP-7)
    #[command(long_about = "\
Update policies for an existing transaction (TAIP-7).

Sends an UpdatePolicies message to set or modify the transaction's policies. \
Policies control what is required before certain actions can be taken.

Policy types: RequireAuthorization, RequirePresentation, RequireProofOfControl

Examples:
  tap-cli agent-mgmt update-policies --transaction-id <ID> \\
    --policies '[{\"@type\":\"RequireAuthorization\"}]'
  tap-cli agent-mgmt update-policies --transaction-id <ID> \\
    --policies '[{\"@type\":\"RequirePresentation\",\"presentation_definition\":{...}}]'")]
    UpdatePolicies {
        /// Transaction ID to update policies for
        #[arg(long)]
        transaction_id: String,
        /// Policies as JSON array of objects with @type and optional attributes
        #[arg(long)]
        policies: String,
    },
}

#[derive(Debug, Serialize)]
struct AgentManagementResponse {
    transaction_id: String,
    message_id: String,
    status: String,
    action: String,
    timestamp: String,
}

#[derive(Debug, serde::Deserialize)]
struct AgentInput {
    #[serde(rename = "@id")]
    id: String,
    role: String,
    #[serde(rename = "for")]
    for_party: String,
}

pub async fn handle(
    cmd: &AgentManagementCommands,
    format: OutputFormat,
    agent_did: &str,
    tap_integration: &TapIntegration,
) -> Result<()> {
    match cmd {
        AgentManagementCommands::AddAgents {
            transaction_id,
            agents,
        } => handle_add_agents(agent_did, transaction_id, agents, format, tap_integration).await,
        AgentManagementCommands::RemoveAgent {
            transaction_id,
            agent_to_remove,
        } => {
            handle_remove_agent(
                agent_did,
                transaction_id,
                agent_to_remove,
                format,
                tap_integration,
            )
            .await
        }
        AgentManagementCommands::ReplaceAgent {
            transaction_id,
            original,
            new_agent,
        } => {
            handle_replace_agent(
                agent_did,
                transaction_id,
                original,
                new_agent,
                format,
                tap_integration,
            )
            .await
        }
        AgentManagementCommands::UpdatePolicies {
            transaction_id,
            policies,
        } => {
            handle_update_policies(agent_did, transaction_id, policies, format, tap_integration)
                .await
        }
    }
}

async fn handle_add_agents(
    agent_did: &str,
    transaction_id: &str,
    agents_json: &str,
    format: OutputFormat,
    tap_integration: &TapIntegration,
) -> Result<()> {
    let inputs: Vec<AgentInput> = serde_json::from_str(agents_json)
        .map_err(|e| Error::invalid_parameter(format!("Invalid agents JSON: {}", e)))?;

    let agents: Vec<Agent> = inputs
        .iter()
        .map(|a| Agent::new(&a.id, &a.role, &a.for_party))
        .collect();

    let add_agents = AddAgents::new(transaction_id, agents);

    add_agents
        .validate()
        .map_err(|e| Error::invalid_parameter(format!("AddAgents validation failed: {}", e)))?;

    let didcomm_message = add_agents
        .to_didcomm(agent_did)
        .map_err(|e| Error::command_failed(format!("Failed to create DIDComm message: {}", e)))?;

    debug!("Sending add-agents for transaction {}", transaction_id);

    tap_integration
        .node()
        .send_message(agent_did.to_string(), didcomm_message.clone())
        .await
        .map_err(|e| Error::command_failed(format!("Failed to send add-agents: {}", e)))?;

    let response = AgentManagementResponse {
        transaction_id: transaction_id.to_string(),
        message_id: didcomm_message.id,
        status: "sent".to_string(),
        action: "add_agents".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    print_success(format, &response);
    Ok(())
}

async fn handle_remove_agent(
    agent_did: &str,
    transaction_id: &str,
    agent_to_remove: &str,
    format: OutputFormat,
    tap_integration: &TapIntegration,
) -> Result<()> {
    let remove_agent = RemoveAgent::new(transaction_id, agent_to_remove);

    remove_agent
        .validate()
        .map_err(|e| Error::invalid_parameter(format!("RemoveAgent validation failed: {}", e)))?;

    let didcomm_message = remove_agent
        .to_didcomm(agent_did)
        .map_err(|e| Error::command_failed(format!("Failed to create DIDComm message: {}", e)))?;

    debug!("Sending remove-agent for transaction {}", transaction_id);

    tap_integration
        .node()
        .send_message(agent_did.to_string(), didcomm_message.clone())
        .await
        .map_err(|e| Error::command_failed(format!("Failed to send remove-agent: {}", e)))?;

    let response = AgentManagementResponse {
        transaction_id: transaction_id.to_string(),
        message_id: didcomm_message.id,
        status: "sent".to_string(),
        action: "remove_agent".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    print_success(format, &response);
    Ok(())
}

async fn handle_replace_agent(
    agent_did: &str,
    transaction_id: &str,
    original_agent: &str,
    new_agent_json: &str,
    format: OutputFormat,
    tap_integration: &TapIntegration,
) -> Result<()> {
    let input: AgentInput = serde_json::from_str(new_agent_json)
        .map_err(|e| Error::invalid_parameter(format!("Invalid new agent JSON: {}", e)))?;

    let replacement = Agent::new(&input.id, &input.role, &input.for_party);
    let replace_agent = ReplaceAgent::new(transaction_id, original_agent, replacement);

    replace_agent
        .validate()
        .map_err(|e| Error::invalid_parameter(format!("ReplaceAgent validation failed: {}", e)))?;

    let didcomm_message = replace_agent
        .to_didcomm(agent_did)
        .map_err(|e| Error::command_failed(format!("Failed to create DIDComm message: {}", e)))?;

    debug!("Sending replace-agent for transaction {}", transaction_id);

    tap_integration
        .node()
        .send_message(agent_did.to_string(), didcomm_message.clone())
        .await
        .map_err(|e| Error::command_failed(format!("Failed to send replace-agent: {}", e)))?;

    let response = AgentManagementResponse {
        transaction_id: transaction_id.to_string(),
        message_id: didcomm_message.id,
        status: "sent".to_string(),
        action: "replace_agent".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    print_success(format, &response);
    Ok(())
}

async fn handle_update_policies(
    agent_did: &str,
    transaction_id: &str,
    policies_json: &str,
    format: OutputFormat,
    tap_integration: &TapIntegration,
) -> Result<()> {
    let policies: Vec<Policy> = serde_json::from_str(policies_json)
        .map_err(|e| Error::invalid_parameter(format!("Invalid policies JSON: {}", e)))?;

    let update_policies = UpdatePolicies::new(transaction_id, policies);

    update_policies.validate().map_err(|e| {
        Error::invalid_parameter(format!("UpdatePolicies validation failed: {}", e))
    })?;

    let didcomm_message = update_policies
        .to_didcomm(agent_did)
        .map_err(|e| Error::command_failed(format!("Failed to create DIDComm message: {}", e)))?;

    debug!("Sending update-policies for transaction {}", transaction_id);

    tap_integration
        .node()
        .send_message(agent_did.to_string(), didcomm_message.clone())
        .await
        .map_err(|e| Error::command_failed(format!("Failed to send update-policies: {}", e)))?;

    let response = AgentManagementResponse {
        transaction_id: transaction_id.to_string(),
        message_id: didcomm_message.id,
        status: "sent".to_string(),
        action: "update_policies".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    print_success(format, &response);
    Ok(())
}
