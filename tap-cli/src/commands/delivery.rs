use crate::error::{Error, Result};
use crate::output::{print_success, OutputFormat};
use crate::tap_integration::TapIntegration;
use clap::Subcommand;
use serde::Serialize;

#[derive(Subcommand, Debug)]
pub enum DeliveryCommands {
    /// List message deliveries
    List {
        /// Filter by recipient DID
        #[arg(long, group = "filter")]
        recipient: Option<String>,
        /// Filter by message ID
        #[arg(long, group = "filter")]
        message: Option<String>,
        /// Filter by thread ID
        #[arg(long, group = "filter")]
        thread: Option<String>,
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
}

#[derive(Debug, Serialize)]
struct DeliveryInfo {
    id: i64,
    message_id: String,
    recipient_did: String,
    status: String,
    retry_count: i32,
    delivery_type: String,
    created_at: String,
    updated_at: String,
    delivered_at: Option<String>,
    error_message: Option<String>,
}

#[derive(Debug, Serialize)]
struct DeliveryListResponse {
    deliveries: Vec<DeliveryInfo>,
    total: usize,
}

fn to_delivery_info(d: &tap_node::storage::models::Delivery) -> DeliveryInfo {
    DeliveryInfo {
        id: d.id,
        message_id: d.message_id.clone(),
        recipient_did: d.recipient_did.clone(),
        status: format!("{:?}", d.status),
        retry_count: d.retry_count,
        delivery_type: format!("{:?}", d.delivery_type),
        created_at: d.created_at.clone(),
        updated_at: d.updated_at.clone(),
        delivered_at: d.delivered_at.clone(),
        error_message: d.error_message.clone(),
    }
}

pub async fn handle(
    cmd: &DeliveryCommands,
    format: OutputFormat,
    default_agent_did: &str,
    tap_integration: &TapIntegration,
) -> Result<()> {
    match cmd {
        DeliveryCommands::List {
            recipient,
            message,
            thread,
            agent_did,
            limit,
            offset,
        } => {
            let effective_did = agent_did.as_deref().unwrap_or(default_agent_did);
            let storage = tap_integration.storage_for_agent(effective_did).await?;

            let deliveries = if let Some(recipient) = recipient {
                storage
                    .get_deliveries_by_recipient(recipient, *limit, *offset)
                    .await?
            } else if let Some(message_id) = message {
                storage.get_deliveries_for_message(message_id).await?
            } else if let Some(thread_id) = thread {
                storage
                    .get_deliveries_for_thread(thread_id, *limit, *offset)
                    .await?
            } else {
                return Err(Error::invalid_parameter(
                    "One of --recipient, --message, or --thread is required",
                ));
            };

            let delivery_infos: Vec<DeliveryInfo> =
                deliveries.iter().map(to_delivery_info).collect();

            let response = DeliveryListResponse {
                total: delivery_infos.len(),
                deliveries: delivery_infos,
            };
            print_success(format, &response);
            Ok(())
        }
    }
}
