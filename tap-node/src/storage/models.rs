use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use tap_msg::utils::NameHashable;

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

impl FromStr for DeliveryStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
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

impl FromStr for DeliveryType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    /// HTTP/HTTPS delivery from external endpoints
    Https,
    /// Internal delivery from agents within the same node
    Internal,
    /// WebSocket connection
    WebSocket,
    /// Return path delivery
    ReturnPath,
    /// Pickup delivery
    Pickup,
}

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceType::Https => write!(f, "https"),
            SourceType::Internal => write!(f, "internal"),
            SourceType::WebSocket => write!(f, "websocket"),
            SourceType::ReturnPath => write!(f, "return_path"),
            SourceType::Pickup => write!(f, "pickup"),
        }
    }
}

impl TryFrom<&str> for SourceType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "https" => Ok(SourceType::Https),
            "internal" => Ok(SourceType::Internal),
            "websocket" => Ok(SourceType::WebSocket),
            "return_path" => Ok(SourceType::ReturnPath),
            "pickup" => Ok(SourceType::Pickup),
            _ => Err(format!("Invalid source type: {}", value)),
        }
    }
}

impl FromStr for SourceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceivedStatus {
    Pending,
    Processed,
    Failed,
}

impl fmt::Display for ReceivedStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReceivedStatus::Pending => write!(f, "pending"),
            ReceivedStatus::Processed => write!(f, "processed"),
            ReceivedStatus::Failed => write!(f, "failed"),
        }
    }
}

impl TryFrom<&str> for ReceivedStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pending" => Ok(ReceivedStatus::Pending),
            "processed" => Ok(ReceivedStatus::Processed),
            "failed" => Ok(ReceivedStatus::Failed),
            _ => Err(format!("Invalid received status: {}", value)),
        }
    }
}

impl FromStr for ReceivedStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Received {
    pub id: i64,
    pub message_id: Option<String>,
    pub raw_message: String,
    pub source_type: SourceType,
    pub source_identifier: Option<String>,
    pub status: ReceivedStatus,
    pub error_message: Option<String>,
    pub received_at: String,
    pub processed_at: Option<String>,
    pub processed_message_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaType {
    Person,
    Organization,
    Thing,
}

impl fmt::Display for SchemaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchemaType::Person => write!(f, "Person"),
            SchemaType::Organization => write!(f, "Organization"),
            SchemaType::Thing => write!(f, "Thing"),
        }
    }
}

impl TryFrom<&str> for SchemaType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Person" => Ok(SchemaType::Person),
            "Organization" => Ok(SchemaType::Organization),
            "Thing" => Ok(SchemaType::Thing),
            _ => Err(format!("Invalid schema type: {}", value)),
        }
    }
}

impl FromStr for SchemaType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: String,
    pub agent_did: String,
    pub schema_type: SchemaType,

    // Core fields for natural persons
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub display_name: Option<String>,

    // Core fields for organizations
    pub legal_name: Option<String>,
    pub lei_code: Option<String>,
    pub mcc_code: Option<String>,

    // Address fields
    pub address_country: Option<String>,
    pub address_locality: Option<String>,
    pub postal_code: Option<String>,
    pub street_address: Option<String>,

    // Full schema.org JSON-LD profile
    pub profile: serde_json::Value,

    // Cached IVMS101 data
    pub ivms101_data: Option<serde_json::Value>,

    // Metadata
    pub verified_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentifierType {
    Did,
    Email,
    Phone,
    Url,
    Account,
    Other,
}

impl fmt::Display for IdentifierType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdentifierType::Did => write!(f, "did"),
            IdentifierType::Email => write!(f, "email"),
            IdentifierType::Phone => write!(f, "phone"),
            IdentifierType::Url => write!(f, "url"),
            IdentifierType::Account => write!(f, "account"),
            IdentifierType::Other => write!(f, "other"),
        }
    }
}

impl TryFrom<&str> for IdentifierType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "did" => Ok(IdentifierType::Did),
            "email" => Ok(IdentifierType::Email),
            "phone" => Ok(IdentifierType::Phone),
            "url" => Ok(IdentifierType::Url),
            "account" => Ok(IdentifierType::Account),
            "other" => Ok(IdentifierType::Other),
            _ => Err(format!("Invalid identifier type: {}", value)),
        }
    }
}

impl FromStr for IdentifierType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerIdentifier {
    pub id: String, // The IRI itself
    pub customer_id: String,
    pub identifier_type: IdentifierType,
    pub verified: bool,
    pub verification_method: Option<String>,
    pub verified_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerRelationship {
    pub id: String,
    pub customer_id: String,
    pub relationship_type: String,
    pub related_identifier: String,
    pub proof: Option<serde_json::Value>,
    pub confirmed_at: Option<String>,
    pub created_at: String,
}

// Implement NameHashable for Customer
impl NameHashable for Customer {}

impl Customer {
    /// Generate a name hash based on the customer's name
    pub fn generate_name_hash(&self) -> Option<String> {
        match self.schema_type {
            SchemaType::Person => {
                // For persons, combine given name and family name
                if let (Some(given), Some(family)) = (&self.given_name, &self.family_name) {
                    Some(Self::hash_name(&format!("{} {}", given, family)))
                } else {
                    self.display_name
                        .as_ref()
                        .map(|display| Self::hash_name(display))
                }
            }
            SchemaType::Organization => {
                // For organizations, use legal name
                self.legal_name.as_ref().map(|name| Self::hash_name(name))
            }
            _ => None,
        }
    }

    /// Add name hash to the profile metadata
    pub fn add_name_hash_to_profile(&mut self) {
        if let Some(hash) = self.generate_name_hash() {
            if let serde_json::Value::Object(ref mut map) = self.profile {
                map.insert("nameHash".to_string(), serde_json::Value::String(hash));
            }
        }
    }

    /// Get name hash from profile if present
    pub fn get_name_hash(&self) -> Option<String> {
        if let serde_json::Value::Object(map) = &self.profile {
            map.get("nameHash")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else {
            None
        }
    }
}
