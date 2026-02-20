use crate::error::{Error, Result};
use crate::output::{print_success, OutputFormat};
use crate::tap_integration::TapIntegration;
use clap::Subcommand;
use serde::Serialize;

#[derive(Subcommand, Debug)]
pub enum ReceivedCommands {
    /// List received messages
    List {
        /// Agent DID for storage lookup
        #[arg(long)]
        agent_did: Option<String>,
        /// Maximum results
        #[arg(long, default_value = "50")]
        limit: u32,
        /// Offset for pagination
        #[arg(long, default_value = "0")]
        offset: u32,
    },
    /// List pending (unprocessed) received messages
    Pending {
        /// Agent DID for storage lookup
        #[arg(long)]
        agent_did: Option<String>,
        /// Maximum results
        #[arg(long, default_value = "50")]
        limit: u32,
    },
    /// View a raw received message by ID
    View {
        /// Received message ID (numeric)
        id: i64,
        /// Agent DID for storage lookup
        #[arg(long)]
        agent_did: Option<String>,
    },
}

#[derive(Debug, Serialize)]
struct ReceivedInfo {
    id: i64,
    message_id: Option<String>,
    source_type: String,
    status: String,
    received_at: String,
    processed_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct ReceivedListResponse {
    messages: Vec<ReceivedInfo>,
    total: usize,
}

#[derive(Debug, Serialize)]
struct ReceivedViewResponse {
    id: i64,
    message_id: Option<String>,
    raw_message: serde_json::Value,
    source_type: String,
    status: String,
    received_at: String,
    processed_at: Option<String>,
}

fn to_received_info(r: &tap_node::storage::models::Received) -> ReceivedInfo {
    ReceivedInfo {
        id: r.id,
        message_id: r.message_id.clone(),
        source_type: format!("{:?}", r.source_type),
        status: format!("{:?}", r.status),
        received_at: r.received_at.clone(),
        processed_at: r.processed_at.clone(),
    }
}

pub async fn handle(
    cmd: &ReceivedCommands,
    format: OutputFormat,
    default_agent_did: &str,
    tap_integration: &TapIntegration,
) -> Result<()> {
    match cmd {
        ReceivedCommands::List {
            agent_did,
            limit,
            offset,
        } => {
            let effective_did = agent_did.as_deref().unwrap_or(default_agent_did);
            let storage = tap_integration.storage_for_agent(effective_did).await?;
            let received = storage.list_received(*limit, *offset, None, None).await?;

            let messages: Vec<ReceivedInfo> = received.iter().map(to_received_info).collect();

            let response = ReceivedListResponse {
                total: messages.len(),
                messages,
            };
            print_success(format, &response);
            Ok(())
        }
        ReceivedCommands::Pending { agent_did, limit } => {
            let effective_did = agent_did.as_deref().unwrap_or(default_agent_did);
            let storage = tap_integration.storage_for_agent(effective_did).await?;
            let received = storage.get_pending_received(*limit).await?;

            let messages: Vec<ReceivedInfo> = received.iter().map(to_received_info).collect();

            let response = ReceivedListResponse {
                total: messages.len(),
                messages,
            };
            print_success(format, &response);
            Ok(())
        }
        ReceivedCommands::View { id, agent_did } => {
            let effective_did = agent_did.as_deref().unwrap_or(default_agent_did);
            let storage = tap_integration.storage_for_agent(effective_did).await?;
            let received = storage.get_received_by_id(*id).await?;

            match received {
                Some(r) => {
                    let raw_message = serde_json::from_str(&r.raw_message)
                        .unwrap_or(serde_json::Value::String(r.raw_message.clone()));

                    let response = ReceivedViewResponse {
                        id: r.id,
                        message_id: r.message_id.clone(),
                        raw_message,
                        source_type: format!("{:?}", r.source_type),
                        status: format!("{:?}", r.status),
                        received_at: r.received_at.clone(),
                        processed_at: r.processed_at.clone(),
                    };
                    print_success(format, &response);
                    Ok(())
                }
                None => Err(Error::command_failed(format!(
                    "Received message '{}' not found",
                    id
                ))),
            }
        }
    }
}
