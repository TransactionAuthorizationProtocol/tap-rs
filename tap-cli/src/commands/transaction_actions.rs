use crate::error::{Error, Result};
use crate::output::{print_success, OutputFormat};
use crate::tap_integration::TapIntegration;
use clap::Subcommand;
use serde::Serialize;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::{Authorize, Cancel, Reject, Revert, Settle};
use tracing::debug;

#[derive(Subcommand, Debug)]
pub enum ActionCommands {
    /// Authorize a transaction (TAIP-4)
    Authorize {
        /// Transaction ID to authorize
        #[arg(long)]
        transaction_id: String,
        /// Settlement address (CAIP-10)
        #[arg(long)]
        settlement_address: Option<String>,
        /// Expiry timestamp (ISO 8601)
        #[arg(long)]
        expiry: Option<String>,
    },
    /// Reject a transaction (TAIP-4)
    Reject {
        /// Transaction ID to reject
        #[arg(long)]
        transaction_id: String,
        /// Rejection reason
        #[arg(long)]
        reason: String,
    },
    /// Cancel a transaction (TAIP-5)
    Cancel {
        /// Transaction ID to cancel
        #[arg(long)]
        transaction_id: String,
        /// DID of the party requesting cancellation
        #[arg(long)]
        by: String,
        /// Cancellation reason
        #[arg(long)]
        reason: Option<String>,
    },
    /// Settle a transaction (TAIP-6)
    Settle {
        /// Transaction ID to settle
        #[arg(long)]
        transaction_id: String,
        /// Settlement identifier (CAIP-220 or tx hash)
        #[arg(long)]
        settlement_id: String,
        /// Settled amount (if different from original)
        #[arg(long)]
        amount: Option<String>,
    },
    /// Revert a settled transaction (TAIP-12)
    Revert {
        /// Transaction ID to revert
        #[arg(long)]
        transaction_id: String,
        /// Settlement address for revert (CAIP-10)
        #[arg(long)]
        settlement_address: String,
        /// Revert reason
        #[arg(long)]
        reason: String,
    },
}

#[derive(Debug, Serialize)]
struct ActionResponse {
    transaction_id: String,
    message_id: String,
    status: String,
    action: String,
    timestamp: String,
}

pub async fn handle(
    cmd: &ActionCommands,
    format: OutputFormat,
    agent_did: &str,
    tap_integration: &TapIntegration,
) -> Result<()> {
    match cmd {
        ActionCommands::Authorize {
            transaction_id,
            settlement_address,
            expiry,
        } => {
            handle_authorize(
                agent_did,
                transaction_id,
                settlement_address.clone(),
                expiry.clone(),
                format,
                tap_integration,
            )
            .await
        }
        ActionCommands::Reject {
            transaction_id,
            reason,
        } => handle_reject(agent_did, transaction_id, reason, format, tap_integration).await,
        ActionCommands::Cancel {
            transaction_id,
            by,
            reason,
        } => {
            handle_cancel(
                agent_did,
                transaction_id,
                by,
                reason.clone(),
                format,
                tap_integration,
            )
            .await
        }
        ActionCommands::Settle {
            transaction_id,
            settlement_id,
            amount,
        } => {
            handle_settle(
                agent_did,
                transaction_id,
                settlement_id,
                amount.clone(),
                format,
                tap_integration,
            )
            .await
        }
        ActionCommands::Revert {
            transaction_id,
            settlement_address,
            reason,
        } => {
            handle_revert(
                agent_did,
                transaction_id,
                settlement_address,
                reason,
                format,
                tap_integration,
            )
            .await
        }
    }
}

async fn handle_authorize(
    agent_did: &str,
    transaction_id: &str,
    settlement_address: Option<String>,
    expiry: Option<String>,
    format: OutputFormat,
    tap_integration: &TapIntegration,
) -> Result<()> {
    let authorize = Authorize {
        transaction_id: transaction_id.to_string(),
        settlement_address,
        expiry,
    };

    authorize
        .validate()
        .map_err(|e| Error::invalid_parameter(format!("Authorize validation failed: {}", e)))?;

    let didcomm_message = authorize
        .to_didcomm(agent_did)
        .map_err(|e| Error::command_failed(format!("Failed to create DIDComm message: {}", e)))?;

    debug!("Sending authorize for transaction {}", transaction_id);

    tap_integration
        .node()
        .send_message(agent_did.to_string(), didcomm_message.clone())
        .await
        .map_err(|e| Error::command_failed(format!("Failed to send authorize: {}", e)))?;

    let response = ActionResponse {
        transaction_id: transaction_id.to_string(),
        message_id: didcomm_message.id,
        status: "sent".to_string(),
        action: "authorize".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    print_success(format, &response);
    Ok(())
}

async fn handle_reject(
    agent_did: &str,
    transaction_id: &str,
    reason: &str,
    format: OutputFormat,
    tap_integration: &TapIntegration,
) -> Result<()> {
    let reject = Reject {
        transaction_id: transaction_id.to_string(),
        reason: Some(reason.to_string()),
    };

    reject
        .validate()
        .map_err(|e| Error::invalid_parameter(format!("Reject validation failed: {}", e)))?;

    let didcomm_message = reject
        .to_didcomm(agent_did)
        .map_err(|e| Error::command_failed(format!("Failed to create DIDComm message: {}", e)))?;

    debug!("Sending reject for transaction {}", transaction_id);

    tap_integration
        .node()
        .send_message(agent_did.to_string(), didcomm_message.clone())
        .await
        .map_err(|e| Error::command_failed(format!("Failed to send reject: {}", e)))?;

    let response = ActionResponse {
        transaction_id: transaction_id.to_string(),
        message_id: didcomm_message.id,
        status: "sent".to_string(),
        action: "reject".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    print_success(format, &response);
    Ok(())
}

async fn handle_cancel(
    agent_did: &str,
    transaction_id: &str,
    by: &str,
    reason: Option<String>,
    format: OutputFormat,
    tap_integration: &TapIntegration,
) -> Result<()> {
    let cancel = Cancel {
        transaction_id: transaction_id.to_string(),
        by: by.to_string(),
        reason,
    };

    cancel
        .validate()
        .map_err(|e| Error::invalid_parameter(format!("Cancel validation failed: {}", e)))?;

    let didcomm_message = cancel
        .to_didcomm(agent_did)
        .map_err(|e| Error::command_failed(format!("Failed to create DIDComm message: {}", e)))?;

    debug!("Sending cancel for transaction {}", transaction_id);

    tap_integration
        .node()
        .send_message(agent_did.to_string(), didcomm_message.clone())
        .await
        .map_err(|e| Error::command_failed(format!("Failed to send cancel: {}", e)))?;

    let response = ActionResponse {
        transaction_id: transaction_id.to_string(),
        message_id: didcomm_message.id,
        status: "sent".to_string(),
        action: "cancel".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    print_success(format, &response);
    Ok(())
}

async fn handle_settle(
    agent_did: &str,
    transaction_id: &str,
    settlement_id: &str,
    amount: Option<String>,
    format: OutputFormat,
    tap_integration: &TapIntegration,
) -> Result<()> {
    let settle = Settle {
        transaction_id: transaction_id.to_string(),
        settlement_id: Some(settlement_id.to_string()),
        amount,
    };

    settle
        .validate()
        .map_err(|e| Error::invalid_parameter(format!("Settle validation failed: {}", e)))?;

    let didcomm_message = settle
        .to_didcomm(agent_did)
        .map_err(|e| Error::command_failed(format!("Failed to create DIDComm message: {}", e)))?;

    debug!("Sending settle for transaction {}", transaction_id);

    tap_integration
        .node()
        .send_message(agent_did.to_string(), didcomm_message.clone())
        .await
        .map_err(|e| Error::command_failed(format!("Failed to send settle: {}", e)))?;

    let response = ActionResponse {
        transaction_id: transaction_id.to_string(),
        message_id: didcomm_message.id,
        status: "sent".to_string(),
        action: "settle".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    print_success(format, &response);
    Ok(())
}

async fn handle_revert(
    agent_did: &str,
    transaction_id: &str,
    settlement_address: &str,
    reason: &str,
    format: OutputFormat,
    tap_integration: &TapIntegration,
) -> Result<()> {
    let revert = Revert {
        transaction_id: transaction_id.to_string(),
        settlement_address: settlement_address.to_string(),
        reason: reason.to_string(),
    };

    revert
        .validate()
        .map_err(|e| Error::invalid_parameter(format!("Revert validation failed: {}", e)))?;

    let didcomm_message = revert
        .to_didcomm(agent_did)
        .map_err(|e| Error::command_failed(format!("Failed to create DIDComm message: {}", e)))?;

    debug!("Sending revert for transaction {}", transaction_id);

    tap_integration
        .node()
        .send_message(agent_did.to_string(), didcomm_message.clone())
        .await
        .map_err(|e| Error::command_failed(format!("Failed to send revert: {}", e)))?;

    let response = ActionResponse {
        transaction_id: transaction_id.to_string(),
        message_id: didcomm_message.id,
        status: "sent".to_string(),
        action: "revert".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    print_success(format, &response);
    Ok(())
}
