use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Transfer,
    Payment,
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionType::Transfer => write!(f, "transfer"),
            TransactionType::Payment => write!(f, "payment"),
        }
    }
}

impl TryFrom<&str> for TransactionType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "transfer" => Ok(TransactionType::Transfer),
            "payment" => Ok(TransactionType::Payment),
            _ => Err(format!("Invalid transaction type: {}", value)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Cancelled,
    Reverted,
}

impl fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionStatus::Pending => write!(f, "pending"),
            TransactionStatus::Confirmed => write!(f, "confirmed"),
            TransactionStatus::Failed => write!(f, "failed"),
            TransactionStatus::Cancelled => write!(f, "cancelled"),
            TransactionStatus::Reverted => write!(f, "reverted"),
        }
    }
}

impl TryFrom<&str> for TransactionStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pending" => Ok(TransactionStatus::Pending),
            "confirmed" => Ok(TransactionStatus::Confirmed),
            "failed" => Ok(TransactionStatus::Failed),
            "cancelled" => Ok(TransactionStatus::Cancelled),
            "reverted" => Ok(TransactionStatus::Reverted),
            _ => Err(format!("Invalid transaction status: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i64,
    pub transaction_type: TransactionType,
    pub reference_id: String,
    pub from_did: Option<String>,
    pub to_did: Option<String>,
    pub thread_id: Option<String>,
    pub message_type: String,
    pub status: TransactionStatus,
    pub message_json: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageDirection {
    Incoming,
    Outgoing,
}

impl fmt::Display for MessageDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageDirection::Incoming => write!(f, "incoming"),
            MessageDirection::Outgoing => write!(f, "outgoing"),
        }
    }
}

impl TryFrom<&str> for MessageDirection {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "incoming" => Ok(MessageDirection::Incoming),
            "outgoing" => Ok(MessageDirection::Outgoing),
            _ => Err(format!("Invalid message direction: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub message_id: String,
    pub message_type: String,
    pub from_did: Option<String>,
    pub to_did: Option<String>,
    pub thread_id: Option<String>,
    pub parent_thread_id: Option<String>,
    pub direction: MessageDirection,
    pub message_json: String,
    pub created_at: String,
}
