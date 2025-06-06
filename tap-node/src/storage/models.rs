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
    pub message_json: serde_json::Value,
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
    pub message_json: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryStatus {
    Pending,
    Success,
    Failed,
}

impl fmt::Display for DeliveryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeliveryStatus::Pending => write!(f, "pending"),
            DeliveryStatus::Success => write!(f, "success"),
            DeliveryStatus::Failed => write!(f, "failed"),
        }
    }
}

impl TryFrom<&str> for DeliveryStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pending" => Ok(DeliveryStatus::Pending),
            "success" => Ok(DeliveryStatus::Success),
            "failed" => Ok(DeliveryStatus::Failed),
            _ => Err(format!("Invalid delivery status: {}", value)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryType {
    /// HTTP/HTTPS delivery to external endpoints
    Https,
    /// Internal delivery to agents within the same node
    Internal,
    /// Return path delivery for future implementation
    ReturnPath,
    /// Pickup delivery for future implementation
    Pickup,
}

impl fmt::Display for DeliveryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeliveryType::Https => write!(f, "https"),
            DeliveryType::Internal => write!(f, "internal"),
            DeliveryType::ReturnPath => write!(f, "return_path"),
            DeliveryType::Pickup => write!(f, "pickup"),
        }
    }
}

impl TryFrom<&str> for DeliveryType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "https" => Ok(DeliveryType::Https),
            "internal" => Ok(DeliveryType::Internal),
            "return_path" => Ok(DeliveryType::ReturnPath),
            "pickup" => Ok(DeliveryType::Pickup),
            _ => Err(format!("Invalid delivery type: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delivery {
    pub id: i64,
    pub message_id: String,
    pub message_text: String,
    pub recipient_did: String,
    pub delivery_url: Option<String>,
    pub delivery_type: DeliveryType,
    pub status: DeliveryStatus,
    pub retry_count: i32,
    pub last_http_status_code: Option<i32>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub delivered_at: Option<String>,
}
