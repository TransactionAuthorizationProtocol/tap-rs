use crate::error::{Error, Result};
use crate::output::{print_success, OutputFormat};
use crate::tap_integration::TapIntegration;
use clap::Subcommand;
use serde::Serialize;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::{BasicMessage, TrustPing};
use tracing::debug;

#[derive(Subcommand, Debug)]
pub enum CommunicationCommands {
    /// Send a trust ping to verify connectivity
    Ping {
        /// Recipient DID
        #[arg(long)]
        recipient: String,
    },
    /// Send a basic text message
    Message {
        /// Recipient DID
        #[arg(long)]
        recipient: String,
        /// Message content
        #[arg(long)]
        content: String,
    },
}

#[derive(Debug, Serialize)]
struct PingResponse {
    message_id: String,
    recipient: String,
    status: String,
    timestamp: String,
}

#[derive(Debug, Serialize)]
struct MessageResponse {
    message_id: String,
    recipient: String,
    status: String,
    timestamp: String,
}

pub async fn handle(
    cmd: &CommunicationCommands,
    format: OutputFormat,
    agent_did: &str,
    tap_integration: &TapIntegration,
) -> Result<()> {
    match cmd {
        CommunicationCommands::Ping { recipient } => {
            handle_ping(agent_did, recipient, format, tap_integration).await
        }
        CommunicationCommands::Message { recipient, content } => {
            handle_message(agent_did, recipient, content, format, tap_integration).await
        }
    }
}

async fn handle_ping(
    agent_did: &str,
    recipient: &str,
    format: OutputFormat,
    tap_integration: &TapIntegration,
) -> Result<()> {
    let ping = TrustPing::new();

    let mut didcomm_message = ping
        .to_didcomm(agent_did)
        .map_err(|e| Error::command_failed(format!("Failed to create DIDComm message: {}", e)))?;

    didcomm_message.to = vec![recipient.to_string()];

    debug!("Sending trust ping from {} to {}", agent_did, recipient);

    tap_integration
        .node()
        .send_message(agent_did.to_string(), didcomm_message.clone())
        .await
        .map_err(|e| Error::command_failed(format!("Failed to send ping: {}", e)))?;

    let response = PingResponse {
        message_id: didcomm_message.id,
        recipient: recipient.to_string(),
        status: "sent".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    print_success(format, &response);
    Ok(())
}

async fn handle_message(
    agent_did: &str,
    recipient: &str,
    content: &str,
    format: OutputFormat,
    tap_integration: &TapIntegration,
) -> Result<()> {
    let message = BasicMessage::new(content.to_string());

    let mut didcomm_message = message
        .to_didcomm(agent_did)
        .map_err(|e| Error::command_failed(format!("Failed to create DIDComm message: {}", e)))?;

    didcomm_message.to = vec![recipient.to_string()];

    debug!("Sending basic message from {} to {}", agent_did, recipient);

    tap_integration
        .node()
        .send_message(agent_did.to_string(), didcomm_message.clone())
        .await
        .map_err(|e| Error::command_failed(format!("Failed to send message: {}", e)))?;

    let response = MessageResponse {
        message_id: didcomm_message.id,
        recipient: recipient.to_string(),
        status: "sent".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    print_success(format, &response);
    Ok(())
}
