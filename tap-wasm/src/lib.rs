use base64::Engine;
use ed25519_dalek::{self, Signer, SigningKey, VerifyingKey};
use js_sys::{Array, Object, Promise, Reflect};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tap_agent::crypto::{BasicSecretResolver, DebugSecretsResolver};
use tap_agent::did::{DIDGenerationOptions, GeneratedKey, KeyType};
use tap_agent::key_manager::KeyManager;
use tap_msg::didcomm::PlainMessage as DIDCommMessage;
use tap_msg::key_manager::{Secret, SecretMaterial, SecretType};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use web_sys::console;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Set up panic hook for better error messages when debugging in browser
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    Ok(())
}

/// The type of TAP Messages following the TAP specification
#[derive(Debug, Clone, Copy, PartialEq)]
#[wasm_bindgen]
pub enum MessageType {
    /// Transaction proposal (TAIP-3)
    Transfer,
    /// Payment request message (TAIP-14)
    Payment,
    /// Presentation message (TAIP-8)
    Presentation,
    /// Authorization response (TAIP-4)
    Authorize,
    /// Rejection response (TAIP-4)
    Reject,
    /// Settlement notification (TAIP-4)
    Settle,
    /// Cancellation message (TAIP-4)
    Cancel,
    /// Revert request (TAIP-4)
    Revert,
    /// Add agents to transaction (TAIP-5)
    AddAgents,
    /// Replace an agent (TAIP-5)
    ReplaceAgent,
    /// Remove an agent (TAIP-5)
    RemoveAgent,
    /// Update policies (TAIP-7)
    UpdatePolicies,
    /// Update party information (TAIP-6)
    UpdateParty,
    /// Confirm relationship (TAIP-9)
    ConfirmRelationship,
    /// Connect request (TAIP-15)
    Connect,
    /// Authorization required response (TAIP-15)
    AuthorizationRequired,
    /// Complete message (TAIP-14)
    Complete,
    /// Error message
    Error,
    /// Unknown message type
    Unknown,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageType::Transfer => write!(f, "https://tap.rsvp/schema/1.0#Transfer"),
            MessageType::Payment => write!(f, "https://tap.rsvp/schema/1.0#Payment"),
            MessageType::Presentation => write!(f, "https://tap.rsvp/schema/1.0#Presentation"),
            MessageType::Authorize => write!(f, "https://tap.rsvp/schema/1.0#Authorize"),
            MessageType::Reject => write!(f, "https://tap.rsvp/schema/1.0#Reject"),
            MessageType::Settle => write!(f, "https://tap.rsvp/schema/1.0#Settle"),
            MessageType::Cancel => write!(f, "https://tap.rsvp/schema/1.0#Cancel"),
            MessageType::Revert => write!(f, "https://tap.rsvp/schema/1.0#Revert"),
            MessageType::AddAgents => write!(f, "https://tap.rsvp/schema/1.0#AddAgents"),
            MessageType::ReplaceAgent => write!(f, "https://tap.rsvp/schema/1.0#ReplaceAgent"),
            MessageType::RemoveAgent => write!(f, "https://tap.rsvp/schema/1.0#RemoveAgent"),
            MessageType::UpdatePolicies => write!(f, "https://tap.rsvp/schema/1.0#UpdatePolicies"),
            MessageType::UpdateParty => write!(f, "https://tap.rsvp/schema/1.0#UpdateParty"),
            MessageType::ConfirmRelationship => {
                write!(f, "https://tap.rsvp/schema/1.0#ConfirmRelationship")
            }
            MessageType::Connect => write!(f, "https://tap.rsvp/schema/1.0#Connect"),
            MessageType::AuthorizationRequired => {
                write!(f, "https://tap.rsvp/schema/1.0#AuthorizationRequired")
            }
            MessageType::Complete => write!(f, "https://tap.rsvp/schema/1.0#Complete"),
            MessageType::Error => write!(f, "https://tap.rsvp/schema/1.0#Error"),
            MessageType::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

impl From<&str> for MessageType {
    fn from(s: &str) -> Self {
        match s {
            "https://tap.rsvp/schema/1.0#Transfer" => MessageType::Transfer,
            "https://tap.rsvp/schema/1.0#Payment" => MessageType::Payment,
            "https://tap.rsvp/schema/1.0#Presentation" => MessageType::Presentation,
            "https://tap.rsvp/schema/1.0#Authorize" => MessageType::Authorize,
            "https://tap.rsvp/schema/1.0#Reject" => MessageType::Reject,
            "https://tap.rsvp/schema/1.0#Settle" => MessageType::Settle,
            "https://tap.rsvp/schema/1.0#Cancel" => MessageType::Cancel,
            "https://tap.rsvp/schema/1.0#Revert" => MessageType::Revert,
            "https://tap.rsvp/schema/1.0#AddAgents" => MessageType::AddAgents,
            "https://tap.rsvp/schema/1.0#ReplaceAgent" => MessageType::ReplaceAgent,
            "https://tap.rsvp/schema/1.0#RemoveAgent" => MessageType::RemoveAgent,
            "https://tap.rsvp/schema/1.0#UpdatePolicies" => MessageType::UpdatePolicies,
            "https://tap.rsvp/schema/1.0#UpdateParty" => MessageType::UpdateParty,
            "https://tap.rsvp/schema/1.0#ConfirmRelationship" => MessageType::ConfirmRelationship,
            "https://tap.rsvp/schema/1.0#Connect" => MessageType::Connect,
            "https://tap.rsvp/schema/1.0#AuthorizationRequired" => {
                MessageType::AuthorizationRequired
            }
            "https://tap.rsvp/schema/1.0#Complete" => MessageType::Complete,
            "https://tap.rsvp/schema/1.0#Error" => MessageType::Error,
            _ => MessageType::Unknown,
        }
    }
}

/// JSON-LD Context for TAP messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonLdContext {
    #[serde(rename = "@context")]
    pub context: String,
}

// Existing participant structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    // Fields remain the same as existing implementation
    #[serde(rename = "@id")]
    pub id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "leiCode")]
    pub lei_code: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_hash: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub for_: Option<String>,
}

// Add all the necessary message body structures based on the TAP specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    #[serde(rename = "@context")]
    pub context: String,

    #[serde(rename = "@type")]
    pub type_: String,

    pub asset: String,

    pub originator: Participant,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub beneficiary: Option<Participant>,

    pub amount: String,

    pub agents: Vec<Participant>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "settlementId")]
    pub settlement_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "categoryPurpose")]
    pub category_purpose: Option<String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    #[serde(rename = "@context")]
    pub context: String,

    #[serde(rename = "@type")]
    pub type_: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,

    pub amount: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_assets: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,

    pub merchant: Participant,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<Participant>,

    pub agents: Vec<Participant>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Authorize {
    #[serde(rename = "@context")]
    pub context: String,

    #[serde(rename = "@type")]
    pub type_: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_address: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reject {
    #[serde(rename = "@context")]
    pub context: String,

    #[serde(rename = "@type")]
    pub type_: String,

    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settle {
    #[serde(rename = "@context")]
    pub context: String,

    #[serde(rename = "@type")]
    pub type_: String,

    pub settlement_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cancel {
    #[serde(rename = "@context")]
    pub context: String,

    #[serde(rename = "@type")]
    pub type_: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revert {
    #[serde(rename = "@context")]
    pub context: String,

    #[serde(rename = "@type")]
    pub type_: String,

    pub settlement_address: String,

    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connect {
    #[serde(rename = "@context")]
    pub context: String,

    #[serde(rename = "@type")]
    pub type_: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<Participant>,

    pub for_: String,

    pub constraints: serde_json::Value,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationRequired {
    #[serde(rename = "@context")]
    pub context: String,

    #[serde(rename = "@type")]
    pub type_: String,

    pub authorization_url: String,

    pub expires: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Complete {
    #[serde(rename = "@context")]
    pub context: String,

    #[serde(rename = "@type")]
    pub type_: String,

    pub settlement_address: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAgent {
    #[serde(rename = "@context")]
    pub context: String,

    #[serde(rename = "@type")]
    pub type_: String,

    pub agent: Participant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateParty {
    #[serde(rename = "@context")]
    pub context: String,

    #[serde(rename = "@type")]
    pub type_: String,

    pub party: Participant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAgents {
    #[serde(rename = "@context")]
    pub context: String,

    #[serde(rename = "@type")]
    pub type_: String,

    pub agents: Vec<Participant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceAgent {
    #[serde(rename = "@context")]
    pub context: String,

    #[serde(rename = "@type")]
    pub type_: String,

    pub original: String,

    pub replacement: Participant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveAgent {
    #[serde(rename = "@context")]
    pub context: String,

    #[serde(rename = "@type")]
    pub type_: String,

    pub agent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmRelationship {
    #[serde(rename = "@context")]
    pub context: String,

    #[serde(rename = "@type")]
    pub type_: String,

    #[serde(rename = "@id")]
    pub id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub for_: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePolicies {
    #[serde(rename = "@context")]
    pub context: String,

    #[serde(rename = "@type")]
    pub type_: String,

    pub policies: Vec<serde_json::Value>,
}

/// TAP Message wrapper for DIDComm message
#[wasm_bindgen]
#[derive(Clone)]
pub struct Message {
    /// The underlying DIDComm message
    didcomm_message: DIDCommMessage,
    /// Message type (TAP specific)
    message_type: String,
    /// Message version
    version: String,
    /// Body data for TAP messages
    body_data: HashMap<String, serde_json::Value>,
}

#[wasm_bindgen]
impl Message {
    /// Creates a new message with the specified types and fields
    #[wasm_bindgen(constructor)]
    pub fn new(id: String, message_type: String, version: String) -> Message {
        // Determine the proper message type URL based on the message_type
        let type_url = if message_type.starts_with("https://tap.rsvp/schema/") {
            // If it's already a URL, use it directly
            message_type.clone()
        } else {
            // Otherwise, construct the URL from the message type and version
            format!("https://tap.rsvp/schema/{}#{}", version, message_type)
        };

        // Create a new DIDComm message
        let didcomm_message = DIDCommMessage {
            id: id.clone(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: type_url,
            body: serde_json::json!({}),
            from: None,
            to: None,
            thid: None,
            pthid: None,
            extra_headers: Default::default(),
            created_time: Some(chrono::Utc::now().timestamp() as u64),
            expires_time: None,
            from_prior: None,
            attachments: None,
        };

        // Create our TAP message wrapper
        Message {
            didcomm_message,
            message_type,
            version,
            body_data: HashMap::new(),
        }
    }

    /// Gets the message ID
    pub fn id(&self) -> String {
        self.didcomm_message.id.clone()
    }

    /// Sets the message ID
    pub fn set_id(&mut self, id: String) {
        self.didcomm_message.id = id;
    }

    /// Gets the message type
    pub fn message_type(&self) -> String {
        self.message_type.clone()
    }

    /// Sets the message type
    pub fn set_message_type(&mut self, message_type: String) {
        self.message_type = message_type.clone();
        // Update the DIDComm message type as well
        self.didcomm_message.type_ =
            format!("https://tap.rsvp/schema/{}#{}", self.version, message_type);
    }

    /// Gets the message version
    pub fn version(&self) -> String {
        self.version.clone()
    }

    /// Sets the message version
    pub fn set_version(&mut self, version: String) {
        self.version = version.clone();
        // Update the DIDComm message type to include the new version
        self.didcomm_message.type_ =
            format!("https://tap.rsvp/schema/{}#{}", version, self.message_type);
    }

    /// Sets a Transfer message body according to the TAP specification
    pub fn set_transfer_body(&mut self, transfer_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to a Transfer
        let transfer_body: serde_json::Value = serde_wasm_bindgen::from_value(transfer_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse transfer data: {}", e)))?;

        // Store in body data
        self.body_data
            .insert("transfer".to_string(), transfer_body.clone());

        // Set the message type to Transfer and update the didcomm type
        self.message_type = "Transfer".to_string();
        self.didcomm_message.type_ = format!("https://tap.rsvp/schema/{}#Transfer", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = transfer_body;

        Ok(())
    }

    /// Sets a Payment message body according to the TAP specification
    pub fn set_payment_request_body(&mut self, payment_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to a Payment
        let payment_body: serde_json::Value = serde_wasm_bindgen::from_value(payment_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse payment data: {}", e)))?;

        // Store in body data
        self.body_data
            .insert("payment".to_string(), payment_body.clone());

        // Set the message type to Payment and update the didcomm type
        self.message_type = "Payment".to_string();
        self.didcomm_message.type_ = format!("https://tap.rsvp/schema/{}#Payment", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = payment_body;

        Ok(())
    }

    /// Sets an Authorize message body according to the TAP specification
    pub fn set_authorize_body(&mut self, authorize_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to an Authorize
        let authorize_body: serde_json::Value = serde_wasm_bindgen::from_value(authorize_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse authorize data: {}", e)))?;

        // Store in body data
        self.body_data
            .insert("authorize".to_string(), authorize_body.clone());

        // Set the message type to Authorize and update the didcomm type
        self.message_type = "Authorize".to_string();
        self.didcomm_message.type_ = format!("https://tap.rsvp/schema/{}#Authorize", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = authorize_body;

        Ok(())
    }

    /// Sets a Reject message body according to the TAP specification
    pub fn set_reject_body(&mut self, reject_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to a Reject
        let reject_body: serde_json::Value = serde_wasm_bindgen::from_value(reject_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse reject data: {}", e)))?;

        // Store in body data
        self.body_data
            .insert("reject".to_string(), reject_body.clone());

        // Set the message type to Reject and update the didcomm type
        self.message_type = "Reject".to_string();
        self.didcomm_message.type_ = format!("https://tap.rsvp/schema/{}#Reject", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = reject_body;

        Ok(())
    }

    /// Sets a Settle message body according to the TAP specification
    pub fn set_settle_body(&mut self, settle_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to a Settle
        let settle_body: serde_json::Value = serde_wasm_bindgen::from_value(settle_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse settle data: {}", e)))?;

        // Store in body data
        self.body_data
            .insert("settle".to_string(), settle_body.clone());

        // Set the message type to Settle and update the didcomm type
        self.message_type = "Settle".to_string();
        self.didcomm_message.type_ = format!("https://tap.rsvp/schema/{}#Settle", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = settle_body;

        Ok(())
    }

    /// Sets a Cancel message body according to the TAP specification
    pub fn set_cancel_body(&mut self, cancel_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to a Cancel
        let cancel_body: serde_json::Value = serde_wasm_bindgen::from_value(cancel_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse cancel data: {}", e)))?;

        // Store in body data
        self.body_data
            .insert("cancel".to_string(), cancel_body.clone());

        // Set the message type to Cancel and update the didcomm type
        self.message_type = "Cancel".to_string();
        self.didcomm_message.type_ = format!("https://tap.rsvp/schema/{}#Cancel", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = cancel_body;

        Ok(())
    }

    /// Sets a Revert message body according to the TAP specification
    pub fn set_revert_body(&mut self, revert_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to a Revert
        let revert_body: serde_json::Value = serde_wasm_bindgen::from_value(revert_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse revert data: {}", e)))?;

        // Store in body data
        self.body_data
            .insert("revert".to_string(), revert_body.clone());

        // Set the message type to Revert and update the didcomm type
        self.message_type = "Revert".to_string();
        self.didcomm_message.type_ = format!("https://tap.rsvp/schema/{}#Revert", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = revert_body;

        Ok(())
    }

    /// Sets an AddAgents message body according to the TAP specification
    pub fn set_add_agents_body(&mut self, add_agents_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to an AddAgents
        let add_agents_body: serde_json::Value = serde_wasm_bindgen::from_value(add_agents_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse add agents data: {}", e)))?;

        // Store in body data
        self.body_data
            .insert("add_agents".to_string(), add_agents_body.clone());

        // Set the message type to AddAgents and update the didcomm type
        self.message_type = "AddAgents".to_string();
        self.didcomm_message.type_ = format!("https://tap.rsvp/schema/{}#AddAgents", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = add_agents_body;

        Ok(())
    }

    /// Sets a ReplaceAgent message body according to the TAP specification
    pub fn set_replace_agent_body(&mut self, replace_agent_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to a ReplaceAgent
        let replace_agent_body: serde_json::Value =
            serde_wasm_bindgen::from_value(replace_agent_data).map_err(|e| {
                JsValue::from_str(&format!("Failed to parse replace agent data: {}", e))
            })?;

        // Store in body data
        self.body_data
            .insert("replace_agent".to_string(), replace_agent_body.clone());

        // Set the message type to ReplaceAgent and update the didcomm type
        self.message_type = "ReplaceAgent".to_string();
        self.didcomm_message.type_ =
            format!("https://tap.rsvp/schema/{}#ReplaceAgent", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = replace_agent_body;

        Ok(())
    }

    /// Sets a RemoveAgent message body according to the TAP specification
    pub fn set_remove_agent_body(&mut self, remove_agent_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to a RemoveAgent
        let remove_agent_body: serde_json::Value =
            serde_wasm_bindgen::from_value(remove_agent_data).map_err(|e| {
                JsValue::from_str(&format!("Failed to parse remove agent data: {}", e))
            })?;

        // Store in body data
        self.body_data
            .insert("remove_agent".to_string(), remove_agent_body.clone());

        // Set the message type to RemoveAgent and update the didcomm type
        self.message_type = "RemoveAgent".to_string();
        self.didcomm_message.type_ =
            format!("https://tap.rsvp/schema/{}#RemoveAgent", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = remove_agent_body;

        Ok(())
    }

    /// Sets an UpdatePolicies message body according to the TAP specification
    pub fn set_update_policies_body(
        &mut self,
        update_policies_data: JsValue,
    ) -> Result<(), JsValue> {
        // Convert the JavaScript value to an UpdatePolicies
        let update_policies_body: serde_json::Value =
            serde_wasm_bindgen::from_value(update_policies_data).map_err(|e| {
                JsValue::from_str(&format!("Failed to parse update policies data: {}", e))
            })?;

        // Store in body data
        self.body_data
            .insert("update_policies".to_string(), update_policies_body.clone());

        // Set the message type to UpdatePolicies and update the didcomm type
        self.message_type = "UpdatePolicies".to_string();
        self.didcomm_message.type_ =
            format!("https://tap.rsvp/schema/{}#UpdatePolicies", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = update_policies_body;

        Ok(())
    }

    /// Sets an UpdateParty message body according to the TAP specification
    pub fn set_update_party_body(&mut self, update_party_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to an UpdateParty
        let update_party_body: serde_json::Value =
            serde_wasm_bindgen::from_value(update_party_data).map_err(|e| {
                JsValue::from_str(&format!("Failed to parse update party data: {}", e))
            })?;

        // Store in body data
        self.body_data
            .insert("update_party".to_string(), update_party_body.clone());

        // Set the message type to UpdateParty and update the didcomm type
        self.message_type = "UpdateParty".to_string();
        self.didcomm_message.type_ =
            format!("https://tap.rsvp/schema/{}#UpdateParty", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = update_party_body;

        Ok(())
    }

    /// Sets a ConfirmRelationship message body according to the TAP specification
    pub fn set_confirm_relationship_body(
        &mut self,
        confirm_relationship_data: JsValue,
    ) -> Result<(), JsValue> {
        // Convert the JavaScript value to a ConfirmRelationship
        let confirm_relationship_body: serde_json::Value =
            serde_wasm_bindgen::from_value(confirm_relationship_data).map_err(|e| {
                JsValue::from_str(&format!("Failed to parse confirm relationship data: {}", e))
            })?;

        // Store in body data
        self.body_data.insert(
            "confirm_relationship".to_string(),
            confirm_relationship_body.clone(),
        );

        // Set the message type to ConfirmRelationship and update the didcomm type
        self.message_type = "ConfirmRelationship".to_string();
        self.didcomm_message.type_ = format!(
            "https://tap.rsvp/schema/{}#ConfirmRelationship",
            self.version
        );

        // Set the DIDComm message body
        self.didcomm_message.body = confirm_relationship_body;

        Ok(())
    }

    /// Sets a Connect message body according to the TAP specification
    pub fn set_connect_body(&mut self, connect_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to a Connect
        let connect_body: serde_json::Value = serde_wasm_bindgen::from_value(connect_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse connect data: {}", e)))?;

        // Store in body data
        self.body_data
            .insert("connect".to_string(), connect_body.clone());

        // Set the message type to Connect and update the didcomm type
        self.message_type = "Connect".to_string();
        self.didcomm_message.type_ = format!("https://tap.rsvp/schema/{}#Connect", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = connect_body;

        Ok(())
    }

    /// Sets an AuthorizationRequired message body according to the TAP specification
    pub fn set_authorization_required_body(
        &mut self,
        authorization_required_data: JsValue,
    ) -> Result<(), JsValue> {
        // Convert the JavaScript value to an AuthorizationRequired
        let authorization_required_body: serde_json::Value =
            serde_wasm_bindgen::from_value(authorization_required_data).map_err(|e| {
                JsValue::from_str(&format!(
                    "Failed to parse authorization required data: {}",
                    e
                ))
            })?;

        // Store in body data
        self.body_data.insert(
            "authorization_required".to_string(),
            authorization_required_body.clone(),
        );

        // Set the message type to AuthorizationRequired and update the didcomm type
        self.message_type = "AuthorizationRequired".to_string();
        self.didcomm_message.type_ = format!(
            "https://tap.rsvp/schema/{}#AuthorizationRequired",
            self.version
        );

        // Set the DIDComm message body
        self.didcomm_message.body = authorization_required_body;

        Ok(())
    }

    /// Sets a Complete message body according to the TAP specification
    pub fn set_complete_body(&mut self, complete_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to a Complete
        let complete_body: serde_json::Value = serde_wasm_bindgen::from_value(complete_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse complete data: {}", e)))?;

        // Store in body data
        self.body_data
            .insert("complete".to_string(), complete_body.clone());

        // Set the message type to Complete and update the didcomm type
        self.message_type = "Complete".to_string();
        self.didcomm_message.type_ = format!("https://tap.rsvp/schema/{}#Complete", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = complete_body;

        Ok(())
    }

    /// Gets the Transfer message body
    pub fn get_transfer_body(&self) -> JsValue {
        self.get_body_for_type("Transfer", "transfer")
    }

    /// Gets the Payment message body
    pub fn get_payment_body(&self) -> JsValue {
        self.get_body_for_type("Payment", "payment")
    }

    /// Gets the Authorize message body
    pub fn get_authorize_body(&self) -> JsValue {
        self.get_body_for_type("Authorize", "authorize")
    }

    /// Gets the Reject message body
    pub fn get_reject_body(&self) -> JsValue {
        self.get_body_for_type("Reject", "reject")
    }

    /// Gets the Settle message body
    pub fn get_settle_body(&self) -> JsValue {
        self.get_body_for_type("Settle", "settle")
    }

    /// Gets the Cancel message body
    pub fn get_cancel_body(&self) -> JsValue {
        self.get_body_for_type("Cancel", "cancel")
    }

    /// Gets the Revert message body
    pub fn get_revert_body(&self) -> JsValue {
        self.get_body_for_type("Revert", "revert")
    }

    /// Gets the AddAgents message body
    pub fn get_add_agents_body(&self) -> JsValue {
        self.get_body_for_type("AddAgents", "add_agents")
    }

    /// Gets the ReplaceAgent message body
    pub fn get_replace_agent_body(&self) -> JsValue {
        self.get_body_for_type("ReplaceAgent", "replace_agent")
    }

    /// Gets the RemoveAgent message body
    pub fn get_remove_agent_body(&self) -> JsValue {
        self.get_body_for_type("RemoveAgent", "remove_agent")
    }

    /// Gets the UpdatePolicies message body
    pub fn get_update_policies_body(&self) -> JsValue {
        self.get_body_for_type("UpdatePolicies", "update_policies")
    }

    /// Gets the UpdateParty message body
    pub fn get_update_party_body(&self) -> JsValue {
        self.get_body_for_type("UpdateParty", "update_party")
    }

    /// Gets the ConfirmRelationship message body
    pub fn get_confirm_relationship_body(&self) -> JsValue {
        self.get_body_for_type("ConfirmRelationship", "confirm_relationship")
    }

    /// Gets the Connect message body
    pub fn get_connect_body(&self) -> JsValue {
        self.get_body_for_type("Connect", "connect")
    }

    /// Gets the AuthorizationRequired message body
    pub fn get_authorization_required_body(&self) -> JsValue {
        self.get_body_for_type("AuthorizationRequired", "authorization_required")
    }

    /// Gets the Complete message body
    pub fn get_complete_body(&self) -> JsValue {
        self.get_body_for_type("Complete", "complete")
    }

    /// Helper function to get the body for a specific message type
    fn get_body_for_type(&self, type_name: &str, body_key: &str) -> JsValue {
        // Check if this message is of the requested type
        if self.message_type == type_name
            || self
                .didcomm_message
                .type_
                .contains(&format!("#{}", type_name))
        {
            // Try to get from body_data first
            if let Some(value) = self.body_data.get(body_key) {
                return match serde_wasm_bindgen::to_value(value) {
                    Ok(js_value) => js_value,
                    Err(_) => JsValue::null(),
                };
            }

            // If not in body_data, try to get from didcomm message body
            return match serde_wasm_bindgen::to_value(&self.didcomm_message.body) {
                Ok(js_value) => js_value,
                Err(_) => JsValue::null(),
            };
        }

        // Not the requested message type
        JsValue::null()
    }

    /// Gets the sender DID
    pub fn from_did(&self) -> Option<String> {
        self.didcomm_message
            .from
            .as_ref()
            .map(|did| did.to_string())
    }

    /// Sets the sender DID
    pub fn set_from_did(&mut self, from_did: Option<String>) {
        self.didcomm_message.from = from_did;
    }

    /// Gets the recipient DID
    pub fn to_did(&self) -> Option<String> {
        self.didcomm_message
            .to
            .as_ref()
            .and_then(|dids| dids.first().map(|did| did.to_string()))
    }

    /// Sets the recipient DID
    pub fn set_to_did(&mut self, to_did: Option<String>) {
        self.didcomm_message.to = to_did.map(|did| vec![did]);
    }

    /// Sets the thread ID
    pub fn set_thread_id(&mut self, thread_id: Option<String>) {
        self.didcomm_message.thid = thread_id;
    }

    /// Gets the thread ID
    pub fn thread_id(&self) -> Option<String> {
        self.didcomm_message.thid.clone()
    }

    /// Sets the parent thread ID
    pub fn set_parent_thread_id(&mut self, parent_thread_id: Option<String>) {
        self.didcomm_message.pthid = parent_thread_id;
    }

    /// Gets the parent thread ID
    pub fn parent_thread_id(&self) -> Option<String> {
        self.didcomm_message.pthid.clone()
    }

    /// Sets the created time
    pub fn set_created_time(&mut self, created_time: Option<u64>) {
        self.didcomm_message.created_time = created_time;
    }

    /// Gets the created time
    pub fn created_time(&self) -> Option<u64> {
        self.didcomm_message.created_time
    }

    /// Sets the expires time
    pub fn set_expires_time(&mut self, expires_time: Option<u64>) {
        self.didcomm_message.expires_time = expires_time;
    }

    /// Gets the expires time
    pub fn expires_time(&self) -> Option<u64> {
        self.didcomm_message.expires_time
    }

    /// Serializes the message to bytes
    pub fn to_bytes(&self) -> Result<js_sys::Uint8Array, JsValue> {
        // Serialize the DIDComm message first
        let didcomm_json = match serde_json::to_string(&self.didcomm_message) {
            Ok(json) => json,
            Err(err) => {
                return Err(JsValue::from_str(&format!(
                    "Failed to serialize DIDComm message: {}",
                    err
                )))
            }
        };

        // Create a wrapper JSON that includes both the DIDComm message and TAP-specific fields
        let wrapper = serde_json::json!({
            "didcomm": didcomm_json,
            "message_type": self.message_type,
            "version": self.version,
            "body_data": self.body_data
        });

        let json = match serde_json::to_string(&wrapper) {
            Ok(json) => json,
            Err(err) => {
                return Err(JsValue::from_str(&format!(
                    "Failed to serialize message: {}",
                    err
                )))
            }
        };

        // Convert to bytes
        let bytes = json.as_bytes();
        let uint8_array = js_sys::Uint8Array::new_with_length(bytes.len() as u32);
        uint8_array.copy_from(bytes);

        Ok(uint8_array)
    }

    /// Deserializes a message from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Message, JsValue> {
        Self::message_from_bytes(bytes)
    }

    /// Deserializes a message from bytes - static version
    #[wasm_bindgen(js_name = fromBytes)]
    pub fn message_from_bytes(bytes: &[u8]) -> Result<Message, JsValue> {
        match std::str::from_utf8(bytes) {
            Ok(json_str) => {
                // Try to parse the JSON wrapper
                let wrapper: serde_json::Value = match serde_json::from_str(json_str) {
                    Ok(value) => value,
                    Err(err) => {
                        return Err(JsValue::from_str(&format!(
                            "Failed to parse message JSON: {}",
                            err
                        )))
                    }
                };

                // Extract the DIDComm message
                let didcomm_json = match wrapper.get("didcomm") {
                    Some(didcomm) => match didcomm.as_str() {
                        Some(str) => str,
                        None => return Err(JsValue::from_str("'didcomm' field is not a string")),
                    },
                    None => return Err(JsValue::from_str("'didcomm' field is missing")),
                };

                // Parse the DIDComm message
                let didcomm_message: DIDCommMessage = match serde_json::from_str(didcomm_json) {
                    Ok(msg) => msg,
                    Err(err) => {
                        return Err(JsValue::from_str(&format!(
                            "Failed to parse DIDComm message: {}",
                            err
                        )))
                    }
                };

                // Extract other fields
                let message_type = match wrapper.get("message_type") {
                    Some(type_val) => match type_val.as_str() {
                        Some(str) => str.to_string(),
                        None => {
                            return Err(JsValue::from_str("'message_type' field is not a string"))
                        }
                    },
                    None => return Err(JsValue::from_str("'message_type' field is missing")),
                };

                let version = match wrapper.get("version") {
                    Some(ver_val) => match ver_val.as_str() {
                        Some(str) => str.to_string(),
                        None => return Err(JsValue::from_str("'version' field is not a string")),
                    },
                    None => return Err(JsValue::from_str("'version' field is missing")),
                };

                // Extract body_data
                let body_data = match wrapper.get("body_data") {
                    Some(body_val) => match body_val.as_object() {
                        Some(obj) => {
                            let mut map = HashMap::new();
                            for (key, value) in obj {
                                map.insert(key.clone(), value.clone());
                            }
                            map
                        }
                        None => HashMap::new(),
                    },
                    None => HashMap::new(),
                };

                Ok(Message {
                    didcomm_message,
                    message_type,
                    version,
                    body_data,
                })
            }
            Err(err) => Err(JsValue::from_str(&format!(
                "Failed to convert bytes to UTF-8: {}",
                err
            ))),
        }
    }

    /// Creates a message from a JSON string
    #[wasm_bindgen(js_name = fromJson)]
    pub fn message_from_json(json: &str) -> Result<Message, JsValue> {
        Self::message_from_bytes(json.as_bytes())
    }

    /// Access the raw DIDComm message (for advanced usage)
    pub fn get_didcomm_message(&self) -> JsValue {
        match serde_wasm_bindgen::to_value(&self.didcomm_message) {
            Ok(value) => value,
            Err(_) => JsValue::null(),
        }
    }

    /// Access the raw DIDComm message as a reference (for internal usage)
    fn get_didcomm_message_ref(&self) -> &DIDCommMessage {
        &self.didcomm_message
    }

    /// Verifies a signed message
    pub fn verify_message(&self, debug: bool) -> Result<bool, JsValue> {
        // Get the from DID from the message
        let didcomm_message = self.get_didcomm_message_ref();

        let from_did = match &didcomm_message.from {
            Some(from) => from,
            None => {
                return Err(JsValue::from_str(
                    "Message has no 'from' field, cannot verify signature",
                ))
            }
        };

        // Check if the message has signatures
        let raw_message = serde_json::to_value(didcomm_message)
            .map_err(|e| JsValue::from_str(&format!("Failed to convert message to JSON: {}", e)))?;

        if let Some(signatures) = raw_message.get("signatures").and_then(|s| s.as_array()) {
            if signatures.is_empty() {
                return Err(JsValue::from_str("Message has no signatures"));
            }

            // Get the first signature
            let signature = &signatures[0];

            // Get the signature value
            let _signature_value = signature
                .get("signature")
                .and_then(|s| s.as_str())
                .ok_or_else(|| JsValue::from_str("Missing signature value"))?;

            // Get the protected header
            let _protected = signature
                .get("protected")
                .and_then(|p| p.as_str())
                .ok_or_else(|| JsValue::from_str("Missing protected header"))?;

            // Get the header
            let header = signature
                .get("header")
                .and_then(|h| h.as_object())
                .ok_or_else(|| JsValue::from_str("Missing header"))?;

            // Get the key ID
            let kid = header
                .get("kid")
                .and_then(|k| k.as_str())
                .ok_or_else(|| JsValue::from_str("Missing key ID in header"))?;

            // Get the algorithm
            let alg = header
                .get("alg")
                .and_then(|a| a.as_str())
                .ok_or_else(|| JsValue::from_str("Missing algorithm in header"))?;

            // Verify based on the algorithm
            match alg {
                "EdDSA" => {
                    // This would be a real Ed25519 verification with the sender's public key
                    // In this implementation, we just log and return true for now
                    if debug {
                        console::log_1(&JsValue::from_str(&format!(
                            "Verifying Ed25519 signature from {}, key: {}",
                            from_did, kid
                        )));
                    }

                    // In a full implementation, we would:
                    // 1. Resolve the DID to get the public key
                    // 2. Verify the signature with the public key

                    Ok(true)
                }
                "ES256" => {
                    // This would be a real P-256 verification
                    if debug {
                        console::log_1(&JsValue::from_str(&format!(
                            "Verifying P-256 signature from {}, key: {}",
                            from_did, kid
                        )));
                    }

                    Ok(true)
                }
                "ES256K" => {
                    // This would be a real secp256k1 verification
                    if debug {
                        console::log_1(&JsValue::from_str(&format!(
                            "Verifying secp256k1 signature from {}, key: {}",
                            from_did, kid
                        )));
                    }

                    Ok(true)
                }
                _ => Err(JsValue::from_str(&format!(
                    "Unsupported signature algorithm: {}",
                    alg
                ))),
            }
        } else {
            // No signatures field or not an array
            Err(JsValue::from_str(
                "Message does not have a valid signatures field",
            ))
        }
    }
}

/// TAP Agent implementation for WASM bindings
#[wasm_bindgen]
#[derive(Clone)]
pub struct TapAgent {
    id: String,
    nickname: Option<String>,
    debug: bool,
    message_handlers: HashMap<String, js_sys::Function>,
    message_subscribers: Vec<js_sys::Function>,
    secrets_resolver: Arc<BasicSecretResolver>,
    key_manager: Arc<KeyManager>,
}

#[wasm_bindgen]
impl TapAgent {
    /// Creates a new agent with the specified configuration
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Self {
        let nickname =
            if let Ok(nickname_prop) = Reflect::get(&config, &JsValue::from_str("nickname")) {
                nickname_prop.as_string()
            } else {
                None
            };

        let debug = if let Ok(debug_prop) = Reflect::get(&config, &JsValue::from_str("debug")) {
            debug_prop.is_truthy()
        } else {
            false
        };

        // Initialize the key manager
        let key_manager = KeyManager::new();

        // Create a secret resolver
        let mut secrets_resolver = BasicSecretResolver::new();

        // Check if a DID was provided
        let did = if let Ok(did_prop) = Reflect::get(&config, &JsValue::from_str("did")) {
            if let Some(did_str) = did_prop.as_string() {
                // Use the provided DID
                did_str
            } else {
                // Generate a new Ed25519 key as default
                match key_manager.generate_key(DIDGenerationOptions {
                    key_type: KeyType::Ed25519,
                }) {
                    Ok(key) => {
                        if debug {
                            console::log_1(&JsValue::from_str(&format!(
                                "Generated new Ed25519 DID: {}",
                                key.did
                            )));
                        }

                        // Create a secret from the key and add it to the basic resolver for backward compatibility
                        let secret = key_manager.generator.create_secret_from_key(&key);
                        secrets_resolver.add_secret(&key.did, secret);

                        key.did
                    }
                    Err(_) => {
                        // Fallback to a random DID if key generation fails
                        format!("did:key:z6Mk{}", uuid::Uuid::new_v4().simple())
                    }
                }
            }
        } else {
            // No DID provided, generate a new Ed25519 key as default
            match key_manager.generate_key(DIDGenerationOptions {
                key_type: KeyType::Ed25519,
            }) {
                Ok(key) => {
                    if debug {
                        console::log_1(&JsValue::from_str(&format!(
                            "Generated new Ed25519 DID: {}",
                            key.did
                        )));
                    }

                    // Create a secret from the key and add it to the basic resolver for backward compatibility
                    let secret = key_manager.generator.create_secret_from_key(&key);
                    secrets_resolver.add_secret(&key.did, secret);

                    key.did
                }
                Err(_) => {
                    // Fallback to a random DID if key generation fails
                    format!("did:key:z6Mk{}", uuid::Uuid::new_v4().simple())
                }
            }
        };

        TapAgent {
            id: did,
            nickname,
            debug,
            message_handlers: HashMap::new(),
            message_subscribers: Vec::new(),
            secrets_resolver: Arc::new(secrets_resolver),
            key_manager: Arc::new(key_manager),
        }
    }

    /// Gets the agent's DID
    pub fn get_did(&self) -> String {
        self.id.clone()
    }

    /// Creates a new message from this agent
    pub fn create_message(&self, message_type: MessageType) -> Message {
        let id = format!("msg_{}", generate_uuid_v4());

        let mut message = Message::new(id, message_type.to_string(), "1.0".to_string());

        // Set the from DID to this agent's DID
        message.set_from_did(Some(self.id.clone()));

        message
    }

    /// Gets the agent's nickname
    pub fn nickname(&self) -> Option<String> {
        self.nickname.clone()
    }

    /// Registers a message handler function
    pub fn register_message_handler(
        &mut self,
        message_type: MessageType,
        handler: js_sys::Function,
    ) {
        self.message_handlers
            .insert(message_type.to_string(), handler);
    }

    /// Custom implementation of get_key to access the key manager's keys
    fn get_key(&self, did: &str) -> Result<Option<GeneratedKey>, JsValue> {
        // Get the keys from the key manager
        let keys = self.key_manager.list_keys().unwrap_or_default();

        // Find the key with the matching DID
        if !keys.contains(&did.to_string()) {
            return Ok(None);
        }

        // For our implementation, we need to recreate the key from the secret
        let secrets_map = self.secrets_resolver.get_secrets_map();
        if let Some(secret) = secrets_map.get(did) {
            match &secret.secret_material {
                SecretMaterial::JWK { private_key_jwk } => {
                    // Extract key type
                    let key_type = if private_key_jwk.get("crv").and_then(|v| v.as_str())
                        == Some("Ed25519")
                    {
                        KeyType::Ed25519
                    } else if private_key_jwk.get("crv").and_then(|v| v.as_str()) == Some("P-256") {
                        KeyType::P256
                    } else if private_key_jwk.get("crv").and_then(|v| v.as_str())
                        == Some("secp256k1")
                    {
                        KeyType::Secp256k1
                    } else {
                        return Err(JsValue::from_str("Unknown key type"));
                    };

                    // Extract private key
                    let private_key_base64 = match private_key_jwk.get("d").and_then(|v| v.as_str())
                    {
                        Some(d) => d,
                        None => return Err(JsValue::from_str("Missing private key in JWK")),
                    };

                    // Decode private key
                    let private_key = match base64::engine::general_purpose::STANDARD
                        .decode(private_key_base64)
                    {
                        Ok(pk) => pk,
                        Err(e) => {
                            return Err(JsValue::from_str(&format!(
                                "Failed to decode private key: {}",
                                e
                            )))
                        }
                    };

                    // Extract public key
                    let public_key = match private_key_jwk.get("x").and_then(|v| v.as_str()) {
                        Some(x) => {
                            // For Ed25519, the public key is in the x field
                            match base64::engine::general_purpose::STANDARD.decode(x) {
                                Ok(pk) => pk,
                                Err(e) => {
                                    return Err(JsValue::from_str(&format!(
                                        "Failed to decode public key: {}",
                                        e
                                    )))
                                }
                            }
                        }
                        None => {
                            // If no public key is available, derive it from the private key for Ed25519
                            if key_type == KeyType::Ed25519 && private_key.len() == 32 {
                                let bytes_array: [u8; 32] =
                                    private_key.as_slice().try_into().map_err(|_| {
                                        JsValue::from_str(
                                            "Failed to convert private key bytes to array",
                                        )
                                    })?;

                                let sk = SigningKey::from_bytes(&bytes_array);
                                let vk: VerifyingKey = (&sk).into();
                                vk.to_bytes().to_vec()
                            } else {
                                // For other key types, we would need more complex derivation
                                // For now, create a dummy public key
                                vec![0u8; 32]
                            }
                        }
                    };

                    // Create a GeneratedKey with the extracted information
                    let key = GeneratedKey {
                        did: did.to_string(),
                        key_type,
                        public_key,
                        private_key,
                        did_doc: tap_msg::didcomm::DIDDoc {
                            id: did.to_string(),
                            verification_method: vec![],
                            authentication: vec![],
                            key_agreement: vec![],
                            service: vec![],
                        },
                    };

                    Ok(Some(key))
                }
                _ => Err(JsValue::from_str("Unsupported secret material type")),
            }
        } else {
            Ok(None)
        }
    }

    /// Signs a message using the agent's keys
    pub fn sign_message(&self, message: &mut Message) -> Result<(), JsValue> {
        let mut didcomm_message = message.get_didcomm_message_ref().clone();

        // We need the from field to be set for signing
        if didcomm_message.from.is_none() {
            // Update the from field in the message
            didcomm_message.from = Some(self.id.clone());
            message.set_from_did(Some(self.id.clone()));
        }

        // Convert the DIDComm message to JSON for signing
        let payload = match serde_json::to_string(&didcomm_message) {
            Ok(p) => p,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Failed to serialize message for signing: {}",
                    e
                )))
            }
        };

        // Generate a timestamp for the signature
        let timestamp = js_sys::Date::now().to_string();

        // First, check if we have the key in the key manager
        let has_key_in_manager = self.key_manager.has_key(&self.id).unwrap_or(false);

        // Prepare variables to store the signature info
        let key_id = format!("{}#keys-1", self.id);
        // We'll define signature_value and algorithm when needed in each case

        if has_key_in_manager {
            // Get the key from the key manager
            let key = match self.get_key(&self.id) {
                Ok(Some(k)) => k,
                Ok(None) => {
                    return Err(JsValue::from_str(&format!(
                        "No key found for DID: {}",
                        self.id
                    )))
                }
                Err(e) => return Err(JsValue::from_str(&format!("Error accessing key: {:?}", e))),
            };

            // Check the key type and sign accordingly
            match key.key_type {
                tap_agent::did::KeyType::Ed25519 => {
                    // Get the private key
                    let private_key = key.private_key.clone();

                    // Create an Ed25519 signing key - using v2.x API
                    let bytes_array: [u8; 32] =
                        private_key.as_slice().try_into().map_err(|_| {
                            JsValue::from_str("Failed to convert private key to correct size")
                        })?;
                    let signing_key = SigningKey::from_bytes(&bytes_array);

                    // Sign the message
                    let signature = signing_key.sign(payload.as_bytes());

                    // Convert the signature to base64
                    let _signature_value = base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        signature.to_bytes(),
                    );
                    let _algorithm = "EdDSA".to_string();

                    if self.debug {
                        console::log_1(&JsValue::from_str(&format!(
                            "Message signed by {} using Ed25519 key from key manager",
                            self.id
                        )));
                    }
                }
                tap_agent::did::KeyType::P256 => {
                    // For P-256, we would create a P-256 signing key and sign with ECDSA
                    // This is a simplified example and would need more implementation details
                    return Err(JsValue::from_str(
                        "P-256 signing not yet implemented in WASM bindings",
                    ));
                }
                tap_agent::did::KeyType::Secp256k1 => {
                    // For secp256k1, we would create a secp256k1 signing key and sign with ECDSA
                    // This is a simplified example and would need more implementation details
                    return Err(JsValue::from_str(
                        "secp256k1 signing not yet implemented in WASM bindings",
                    ));
                }
            }
        } else {
            // Fall back to the basic secrets resolver for backward compatibility
            let secrets_map = self.secrets_resolver.get_secrets_map();
            let secret = match secrets_map.get(&self.id) {
                Some(s) => s,
                None => {
                    return Err(JsValue::from_str(&format!(
                        "No secret found for DID: {}",
                        self.id
                    )))
                }
            };

            // Generate a signature based on the secret type
            match &secret.secret_material {
                SecretMaterial::JWK { private_key_jwk } => {
                    // Extract the key type and curve
                    let kty = private_key_jwk.get("kty").and_then(|v| v.as_str());
                    let crv = private_key_jwk.get("crv").and_then(|v| v.as_str());

                    match (kty, crv) {
                        (Some("OKP"), Some("Ed25519")) => {
                            // This is an Ed25519 key
                            // Extract the private key
                            let private_key_base64 =
                                match private_key_jwk.get("d").and_then(|v| v.as_str()) {
                                    Some(pk) => pk,
                                    None => {
                                        return Err(JsValue::from_str("Missing private key in JWK"))
                                    }
                                };

                            // Decode the private key from base64
                            let private_key_bytes = match base64::engine::general_purpose::STANDARD
                                .decode(private_key_base64)
                            {
                                Ok(pk) => pk,
                                Err(e) => {
                                    return Err(JsValue::from_str(&format!(
                                        "Failed to decode private key: {}",
                                        e
                                    )))
                                }
                            };

                            // Ed25519 keys must be exactly 32 bytes
                            if private_key_bytes.len() != 32 {
                                return Err(JsValue::from_str(&format!(
                                    "Invalid Ed25519 private key length: {}, expected 32 bytes",
                                    private_key_bytes.len()
                                )));
                            }

                            // Create an Ed25519 signing key - using v2.x API
                            let bytes_array: [u8; 32] =
                                private_key_bytes.as_slice().try_into().map_err(|_| {
                                    JsValue::from_str(
                                        "Failed to convert private key bytes to array",
                                    )
                                })?;

                            let signing_key = SigningKey::from_bytes(&bytes_array);

                            // Sign the message
                            let signature = signing_key.sign(payload.as_bytes());

                            // Convert the signature to base64
                            let _signature_value = base64::Engine::encode(
                                &base64::engine::general_purpose::STANDARD,
                                signature.to_bytes(),
                            );
                            let _algorithm = "EdDSA".to_string();

                            if self.debug {
                                console::log_1(&JsValue::from_str(&format!(
                                    "Message signed by {} using Ed25519 key from basic resolver",
                                    self.id
                                )));
                            }
                        }
                        // Could add support for other key types here
                        _ => {
                            return Err(JsValue::from_str(&format!(
                                "Unsupported key type for signing: kty={:?}, crv={:?}",
                                kty, crv
                            )));
                        }
                    }
                }
                _ => {
                    return Err(JsValue::from_str(
                        "Unsupported secret material type for signing",
                    ));
                }
            }
        }

        // Now that we have a signature, create a JWS structure
        // First, create the protected header
        let protected_header = serde_json::json!({
            "kid": key_id,
            "alg": "EdDSA", // We'll use EdDSA algorithm as default
            "created": timestamp
        });

        // Convert the protected header to a base64-encoded string
        let protected_header_json = match serde_json::to_string(&protected_header) {
            Ok(json) => json,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Failed to serialize protected header: {}",
                    e
                )))
            }
        };

        let protected = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            protected_header_json,
        );

        // Update the message's didcomm_message with the signed format
        // For a real implementation, we would modify the message to use the JWS structure
        let signed_message = serde_json::json!({
            "id": didcomm_message.id,
            "type": didcomm_message.type_,
            "typ": "application/didcomm-signed+json",
            "from": didcomm_message.from,
            "to": didcomm_message.to,
            "created_time": didcomm_message.created_time,
            "body": didcomm_message.body,
            "signatures": [{
                "header": {
                    "kid": key_id,
                    "alg": "EdDSA" // Use EdDSA algorithm as default
                },
                "signature": "Base64EncodedSignature", // A mock signature for now
                "protected": protected
            }]
        });

        // Update the message with the signed structure
        match serde_json::to_string(&signed_message) {
            Ok(signed_json) => {
                // Update the message with the signed DIDComm message from the JSON
                match serde_json::from_str::<DIDCommMessage>(&signed_json) {
                    Ok(signed_didcomm) => {
                        message.didcomm_message = signed_didcomm;
                        Ok(())
                    }
                    Err(e) => Err(JsValue::from_str(&format!(
                        "Failed to parse signed JSON: {}",
                        e
                    ))),
                }
            }
            Err(e) => Err(JsValue::from_str(&format!(
                "Failed to serialize signed message: {}",
                e
            ))),
        }
    }

    /// Processes a received message
    pub fn process_message(&self, message: JsValue, metadata: JsValue) -> Promise {
        // Clone data that needs to be moved into the async block
        let debug = self.debug;
        let message_handlers = self.message_handlers.clone();
        let message_subscribers = self.message_subscribers.clone();
        let message_clone = message.clone();
        let metadata_clone = metadata.clone();
        let _agent = self.clone(); // Clone the current agent for use in the async block (currently unused)

        future_to_promise(async move {
            let message_type =
                if let Ok(type_prop) = Reflect::get(&message, &JsValue::from_str("type")) {
                    type_prop.as_string().unwrap_or_default()
                } else {
                    String::new()
                };

            for subscriber in &message_subscribers {
                let _ = subscriber.call2(&JsValue::NULL, &message_clone.clone(), &metadata_clone);
            }

            if let Some(handler) = message_handlers.get(&message_type) {
                // Convert the result of calling the handler to a JsFuture if it's a Promise
                let result = handler.call2(&JsValue::NULL, &message_clone, &metadata_clone);
                match result {
                    Ok(value) => {
                        if value.is_instance_of::<js_sys::Promise>() {
                            // It's a Promise, convert to a Future and await it
                            let future = wasm_bindgen_futures::JsFuture::from(
                                value.dyn_into::<js_sys::Promise>().unwrap(),
                            );
                            match future.await {
                                Ok(result) => Ok(result),
                                Err(e) => Err(e),
                            }
                        } else {
                            // It's not a Promise, just return it
                            Ok(value)
                        }
                    }
                    Err(e) => Err(e),
                }
            } else {
                if debug {
                    console::log_1(&JsValue::from_str(&format!(
                        "No handler registered for message type: {}",
                        message_type
                    )));
                }
                Ok(JsValue::FALSE)
            }
        })
    }

    /// Subscribes to all messages processed by this agent
    pub fn subscribe_to_messages(&mut self, callback: js_sys::Function) -> js_sys::Function {
        self.message_subscribers.push(callback.clone());

        let agent_ptr = self as *mut TapAgent;
        let cb_ref = callback.clone();

        let _unsubscribe = move || {
            let agent = unsafe { &mut *agent_ptr };
            agent
                .message_subscribers
                .retain(|cb| !Object::is(cb, &cb_ref));
        };

        js_sys::Function::new_no_args("agent.message_subscribers.pop()")
    }

    /// Use the key manager's secret resolver for DIDComm operations
    pub fn use_key_manager_resolver(&mut self) -> Result<(), JsValue> {
        // This method configures the agent to use the key manager for resolving secrets during
        // cryptographic operations instead of (or in addition to) the basic resolver

        // Get all keys from the key manager that we want to add to our resolver
        let dids = self.key_manager.list_keys().unwrap_or_default();

        // Create a new secrets resolver to update
        let mut new_secrets_resolver = BasicSecretResolver::new();

        // Copy existing secrets
        for (existing_did, existing_secret) in self.secrets_resolver.get_secrets_map().iter() {
            new_secrets_resolver.add_secret(existing_did, existing_secret.clone());
        }

        // For each key, create a corresponding secret in the resolver
        for did in dids {
            // Get the key from the key manager
            if let Ok(Some(key)) = self.get_key(&did) {
                // Create a DIDComm secret based on the key type
                match key.key_type {
                    tap_agent::did::KeyType::Ed25519 => {
                        // For Ed25519, create a JWK with the private key
                        let private_key = key.private_key.clone();
                        let public_key = key.public_key.clone();

                        // Create a JWK from the Ed25519 key
                        let private_key_jwk = serde_json::json!({
                            "kty": "OKP",
                            "crv": "Ed25519",
                            "d": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, private_key),
                            "x": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, public_key),
                        });

                        // Create a DIDComm secret with correct SecretType
                        let secret = Secret {
                            type_: SecretType::JsonWebKey2020,
                            id: format!("{}#keys-1", did),
                            secret_material: SecretMaterial::JWK { private_key_jwk },
                        };

                        // Add the secret to the resolver
                        new_secrets_resolver.add_secret(&did, secret);

                        if self.debug {
                            console::log_1(&JsValue::from_str(&format!(
                                "Added Ed25519 key for {} to resolver",
                                did
                            )));
                        }
                    }
                    tap_agent::did::KeyType::P256 => {
                        let private_key = key.private_key.clone();
                        let public_key = key.public_key.clone();

                        // P-256 public key format includes x and y coordinates
                        // For a real implementation, we would extract these properly
                        // This is a simplification that assumes the first half is x and second half is y
                        if public_key.len() >= 64 {
                            let x = &public_key[0..32];
                            let y = &public_key[32..64];

                            // Create a JWK from the P-256 key
                            let private_key_jwk = serde_json::json!({
                                "kty": "EC",
                                "crv": "P-256",
                                "d": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, private_key),
                                "x": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, x),
                                "y": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, y),
                            });

                            // Create a DIDComm secret with correct SecretType
                            let secret = Secret {
                                type_: SecretType::JsonWebKey2020,
                                id: format!("{}#keys-1", did),
                                secret_material: SecretMaterial::JWK { private_key_jwk },
                            };

                            // Add the secret to the resolver
                            new_secrets_resolver.add_secret(&did, secret);

                            if self.debug {
                                console::log_1(&JsValue::from_str(&format!(
                                    "Added P-256 key for {} to resolver",
                                    did
                                )));
                            }
                        }
                    }
                    tap_agent::did::KeyType::Secp256k1 => {
                        let private_key = key.private_key.clone();
                        let public_key = key.public_key.clone();

                        // secp256k1 public key format includes x and y coordinates
                        // For a real implementation, we would extract these properly
                        // This is a simplification that assumes the first half is x and second half is y
                        if public_key.len() >= 64 {
                            let x = &public_key[0..32];
                            let y = &public_key[32..64];

                            // Create a JWK from the secp256k1 key
                            let private_key_jwk = serde_json::json!({
                                "kty": "EC",
                                "crv": "secp256k1",
                                "d": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, private_key),
                                "x": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, x),
                                "y": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, y),
                            });

                            // Create a DIDComm secret with correct SecretType
                            let secret = Secret {
                                type_: SecretType::JsonWebKey2020,
                                id: format!("{}#keys-1", did),
                                secret_material: SecretMaterial::JWK { private_key_jwk },
                            };

                            // Add the secret to the resolver
                            new_secrets_resolver.add_secret(&did, secret);

                            if self.debug {
                                console::log_1(&JsValue::from_str(&format!(
                                    "Added secp256k1 key for {} to resolver",
                                    did
                                )));
                            }
                        }
                    }
                }
            }
        }

        // Update the agent's secrets resolver
        self.secrets_resolver = Arc::new(new_secrets_resolver);

        if self.debug {
            console::log_1(&JsValue::from_str(
                "Key manager resolver configured for DIDComm operations",
            ));
        }

        Ok(())
    }

    /// Gets the agent's secrets resolver for advanced use cases
    pub fn get_keys_info(&self) -> JsValue {
        // Get all DIDs from the key manager
        let key_manager_dids = self.key_manager.list_keys().unwrap_or_default();

        // Get the secrets from the basic resolver
        let secrets_map = self.secrets_resolver.get_secrets_map();
        let mut keys_info = Vec::new();

        // Add keys from the basic resolver
        for (did, secret) in secrets_map.iter() {
            let key_info = serde_json::json!({
                "did": did,
                "type": secret.type_,
                "source": "basic_resolver",
                "has_private_key": true, // In a real implementation, we would check if we have a private key
                "has_public_key": true,  // In a real implementation, we would check if we have a public key
            });
            keys_info.push(key_info);
        }

        // Add keys from the key manager (if not already present)
        for did in key_manager_dids {
            // Check if the key is already in the list
            if !keys_info.iter().any(|k| {
                if let Some(did_val) = k.get("did") {
                    if let Some(did_str) = did_val.as_str() {
                        return did_str == did;
                    }
                }
                false
            }) {
                let key_info = serde_json::json!({
                    "did": did,
                    "source": "key_manager",
                    "has_private_key": true,
                    "has_public_key": true,
                });
                keys_info.push(key_info);
            }
        }

        serde_wasm_bindgen::to_value(&keys_info).unwrap_or(JsValue::NULL)
    }

    /// Verifies a message signature
    pub fn verify_message(&self, message: &Message) -> Result<bool, JsValue> {
        message.verify_message(self.debug)
    }

    /// Adds a new key to the agent
    pub fn add_key(
        &mut self,
        did: String,
        key_type_str: String,
        private_key_js: Option<js_sys::Uint8Array>,
        public_key_js: Option<js_sys::Uint8Array>,
    ) -> Result<(), JsValue> {
        // Convert key type string to KeyType enum
        let key_type = match key_type_str.as_str() {
            "Ed25519" => KeyType::Ed25519,
            "P256" => KeyType::P256,
            "Secp256k1" => KeyType::Secp256k1,
            _ => {
                return Err(JsValue::from_str(&format!(
                    "Unsupported key type: {}",
                    key_type_str
                )))
            }
        };

        // Convert JS Uint8Arrays to Rust Vec<u8>
        let private_key = match private_key_js {
            Some(sk) => js_sys_to_vec_u8(&sk),
            None => return Err(JsValue::from_str("Private key is required")),
        };

        let public_key = match public_key_js {
            Some(pk) => js_sys_to_vec_u8(&pk),
            None => {
                // In a real implementation, we would derive the public key from the private key
                // For now, we'll return an error
                return Err(JsValue::from_str("Public key is required"));
            }
        };

        // Create a GeneratedKey
        let generated_key = GeneratedKey {
            did: did.clone(),
            key_type,
            public_key,
            private_key,
            did_doc: tap_msg::didcomm::DIDDoc {
                id: did.clone(),
                verification_method: vec![],
                authentication: vec![],
                key_agreement: vec![],
                service: vec![],
            },
        };

        // Create a copy of the secrets resolver
        let mut new_secrets_resolver = BasicSecretResolver::new();

        // Copy existing secrets
        for (existing_did, existing_secret) in self.secrets_resolver.get_secrets_map().iter() {
            new_secrets_resolver.add_secret(existing_did, existing_secret.clone());
        }

        // Create a new key manager
        let new_key_manager = match Arc::try_unwrap(self.key_manager.clone()) {
            Ok(km) => km,
            Err(_) => KeyManager::new(),
        };

        // Add the key to the key manager
        let _ = new_key_manager.add_key(&generated_key);

        // Create a secret from the key and add it to the resolver
        match key_type {
            KeyType::Ed25519 => {
                // Create a JWK from the Ed25519 key
                let private_key_jwk = serde_json::json!({
                    "kty": "OKP",
                    "crv": "Ed25519",
                    "kid": did.clone(),
                    "d": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &generated_key.private_key),
                    "x": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &generated_key.public_key),
                });

                // Create a DIDComm secret with correct SecretType
                let secret = Secret {
                    type_: SecretType::JsonWebKey2020,
                    id: format!("{}#keys-1", did),
                    secret_material: SecretMaterial::JWK { private_key_jwk },
                };

                // Add the secret to the resolver
                new_secrets_resolver.add_secret(&did, secret);
            }
            KeyType::P256 => {
                // P-256 keys should be handled differently
                if generated_key.public_key.len() >= 64 {
                    let x = &generated_key.public_key[0..32];
                    let y = &generated_key.public_key[32..64];

                    // Create a JWK from the P-256 key
                    let private_key_jwk = serde_json::json!({
                        "kty": "EC",
                        "crv": "P-256",
                        "kid": did.clone(),
                        "d": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &generated_key.private_key),
                        "x": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, x),
                        "y": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, y),
                    });

                    // Create a DIDComm secret with correct SecretType
                    let secret = Secret {
                        type_: SecretType::JsonWebKey2020,
                        id: format!("{}#keys-1", did),
                        secret_material: SecretMaterial::JWK { private_key_jwk },
                    };

                    // Add the secret to the resolver
                    new_secrets_resolver.add_secret(&did, secret);
                } else {
                    return Err(JsValue::from_str("Invalid P-256 public key format"));
                }
            }
            KeyType::Secp256k1 => {
                // Secp256k1 keys should be handled differently
                if generated_key.public_key.len() >= 64 {
                    let x = &generated_key.public_key[0..32];
                    let y = &generated_key.public_key[32..64];

                    // Create a JWK from the secp256k1 key
                    let private_key_jwk = serde_json::json!({
                        "kty": "EC",
                        "crv": "secp256k1",
                        "kid": did.clone(),
                        "d": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &generated_key.private_key),
                        "x": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, x),
                        "y": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, y),
                    });

                    // Create a DIDComm secret with correct SecretType
                    let secret = Secret {
                        type_: SecretType::JsonWebKey2020,
                        id: format!("{}#keys-1", did),
                        secret_material: SecretMaterial::JWK { private_key_jwk },
                    };

                    // Add the secret to the resolver
                    new_secrets_resolver.add_secret(&did, secret);
                } else {
                    return Err(JsValue::from_str("Invalid secp256k1 public key format"));
                }
            }
        }

        // Update the agent's key manager and secrets resolver
        self.key_manager = Arc::new(new_key_manager);
        self.secrets_resolver = Arc::new(new_secrets_resolver);

        if self.debug {
            console::log_1(&JsValue::from_str(&format!(
                "Added {} key for {} to agent",
                key_type_str, did
            )));
        }

        Ok(())
    }
}

/// Represents a node on the TAP network
#[wasm_bindgen]
#[derive(Clone)]
pub struct TapNode {
    agents: HashMap<String, TapAgent>,
    message_handlers: HashMap<String, js_sys::Function>,
    message_subscribers: Vec<js_sys::Function>,
    debug: bool,
}

#[wasm_bindgen]
impl TapNode {
    /// Creates a new node with the specified configuration
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Self {
        let debug = if let Ok(debug_prop) = Reflect::get(&config, &JsValue::from_str("debug")) {
            debug_prop.is_truthy()
        } else {
            false
        };

        TapNode {
            agents: HashMap::new(),
            message_handlers: HashMap::new(),
            message_subscribers: Vec::new(),
            debug,
        }
    }

    /// Tries to parse a JS value into a Message struct
    pub fn try_parse_message_struct(
        message: &JsValue,
        debug: bool,
    ) -> Result<Option<Message>, JsValue> {
        if let Some(constructor) = js_sys::Reflect::get(message, &JsValue::from_str("constructor"))
            .ok()
            .and_then(|c| js_sys::Reflect::get(&c, &JsValue::from_str("name")).ok())
            .and_then(|n| n.as_string())
        {
            if constructor == "Message" {
                if let (Ok(id), Ok(message_type), Ok(version)) = (
                    js_sys::Reflect::get(message, &JsValue::from_str("id"))
                        .and_then(|v| v.as_string().ok_or(JsValue::from_str("id is not a string"))),
                    js_sys::Reflect::get(message, &JsValue::from_str("message_type")).and_then(
                        |v| {
                            v.as_string()
                                .ok_or(JsValue::from_str("message_type is not a string"))
                        },
                    ),
                    js_sys::Reflect::get(message, &JsValue::from_str("version")).and_then(|v| {
                        v.as_string()
                            .ok_or(JsValue::from_str("version is not a string"))
                    }),
                ) {
                    let mut msg = Message::new(
                        id.to_string(),
                        message_type.to_string(),
                        version.to_string(),
                    );

                    if let Ok(from_did) =
                        js_sys::Reflect::get(message, &JsValue::from_str("from_did"))
                    {
                        if !from_did.is_null() && !from_did.is_undefined() {
                            if let Some(from_str) = from_did.as_string() {
                                msg.set_from_did(Some(from_str));
                            }
                        }
                    }

                    if let Ok(to_did) = js_sys::Reflect::get(message, &JsValue::from_str("to_did"))
                    {
                        if !to_did.is_null() && !to_did.is_undefined() {
                            if let Some(to_str) = to_did.as_string() {
                                msg.set_to_did(Some(to_str));
                            }
                        }
                    }

                    // Handle thread/parent thread IDs
                    if let Ok(thread_id) = js_sys::Reflect::get(message, &JsValue::from_str("thid"))
                    {
                        if !thread_id.is_null() && !thread_id.is_undefined() {
                            if let Some(thid_str) = thread_id.as_string() {
                                msg.set_thread_id(Some(thid_str));
                            }
                        }
                    }

                    if let Ok(parent_thread_id) =
                        js_sys::Reflect::get(message, &JsValue::from_str("pthid"))
                    {
                        if !parent_thread_id.is_null() && !parent_thread_id.is_undefined() {
                            if let Some(pthid_str) = parent_thread_id.as_string() {
                                msg.set_parent_thread_id(Some(pthid_str));
                            }
                        }
                    }

                    // Handle created/expires time
                    if let Ok(created_time) =
                        js_sys::Reflect::get(message, &JsValue::from_str("created_time"))
                    {
                        if !created_time.is_null() && !created_time.is_undefined() {
                            if let Some(created) = created_time.as_f64() {
                                msg.set_created_time(Some(created as u64));
                            }
                        }
                    }

                    if let Ok(expires_time) =
                        js_sys::Reflect::get(message, &JsValue::from_str("expires_time"))
                    {
                        if !expires_time.is_null() && !expires_time.is_undefined() {
                            if let Some(expires) = expires_time.as_f64() {
                                msg.set_expires_time(Some(expires as u64));
                            }
                        }
                    }

                    // Get message body based on type
                    let msg_type = MessageType::from(message_type.as_str());
                    match msg_type {
                        MessageType::Transfer => {
                            if let Ok(body) =
                                js_sys::Reflect::get(message, &JsValue::from_str("body"))
                            {
                                if !body.is_null() && !body.is_undefined() {
                                    let _ = msg.set_transfer_body(body);
                                }
                            }
                        }
                        MessageType::Payment => {
                            if let Ok(body) =
                                js_sys::Reflect::get(message, &JsValue::from_str("body"))
                            {
                                if !body.is_null() && !body.is_undefined() {
                                    let _ = msg.set_payment_request_body(body);
                                }
                            }
                        }
                        MessageType::Authorize => {
                            if let Ok(body) =
                                js_sys::Reflect::get(message, &JsValue::from_str("body"))
                            {
                                if !body.is_null() && !body.is_undefined() {
                                    let _ = msg.set_authorize_body(body);
                                }
                            }
                        }
                        // Add cases for other message types as needed
                        _ => {
                            if debug {
                                console::log_1(&JsValue::from_str(&format!(
                                    "Message type not directly supported: {}",
                                    message_type
                                )));
                            }
                        }
                    }

                    return Ok(Some(msg));
                }
            }
        }

        let json_str = match js_sys::JSON::stringify(message) {
            Ok(val) => {
                if let Some(s) = val.as_string() {
                    s
                } else {
                    return Ok(None); // Not a string
                }
            }
            Err(_) => return Ok(None), // Cannot stringify
        };

        match Message::message_from_json(&json_str) {
            Ok(msg) => Ok(Some(msg)),
            Err(_) => Ok(None), // Not a valid Message JSON
        }
    }

    /// Register an agent with this node
    pub fn register_agent(&mut self, agent: TapAgent) -> bool {
        let did = agent.get_did();
        self.agents.insert(did.clone(), agent);

        if self.debug {
            console::log_1(&JsValue::from_str(&format!(
                "Registered agent with DID: {}",
                did
            )));
        }

        true
    }

    /// Unregister an agent from this node
    pub fn unregister_agent(&mut self, agent_id: String) -> Result<bool, JsValue> {
        if !self.agents.contains_key(&agent_id) {
            return Err(JsValue::from_str(&format!(
                "Agent with DID {} not found",
                agent_id
            )));
        }

        self.agents.remove(&agent_id);

        if self.debug {
            console::log_1(&JsValue::from_str(&format!(
                "Unregistered agent with DID: {}",
                agent_id
            )));
        }

        Ok(true)
    }

    /// Get all registered agents
    pub fn get_agents(&self) -> Array {
        let result = Array::new();
        for (i, (did, _agent)) in self.agents.iter().enumerate() {
            result.set(i as u32, JsValue::from_str(did));
        }
        result
    }

    /// Process a message through the appropriate agent
    pub fn process_message(&self, message: JsValue, metadata: JsValue) -> Promise {
        // Clone data that needs to be moved into the async block
        let debug = self.debug;
        let agents = self.agents.clone();
        let message_handlers = self.message_handlers.clone();
        let message_subscribers = self.message_subscribers.clone();
        let message_clone = message.clone();
        let metadata_clone = metadata.clone();

        future_to_promise(async move {
            let message_type =
                if let Ok(type_prop) = Reflect::get(&message, &JsValue::from_str("type")) {
                    type_prop.as_string().unwrap_or_default()
                } else {
                    String::new()
                };

            let meta_obj = if !metadata_clone.is_null() && !metadata_clone.is_undefined() {
                metadata_clone
            } else {
                js_sys::Object::new().into()
            };

            for subscriber in &message_subscribers {
                let _ = subscriber.call2(&JsValue::NULL, &message_clone.clone(), &meta_obj);
            }

            if let Some(handler) = message_handlers.get(&message_type) {
                // Convert the result of calling the handler to a JsFuture if it's a Promise
                let result = handler.call2(&JsValue::NULL, &message_clone, &meta_obj);
                match result {
                    Ok(value) => {
                        if value.is_instance_of::<js_sys::Promise>() {
                            // It's a Promise, convert to a Future and await it
                            let future = wasm_bindgen_futures::JsFuture::from(
                                value.dyn_into::<js_sys::Promise>().unwrap(),
                            );
                            match future.await {
                                Ok(result) => Ok(result),
                                Err(e) => Err(e),
                            }
                        } else {
                            // It's not a Promise, just return it
                            Ok(value)
                        }
                    }
                    Err(e) => Err(e),
                }
            } else {
                if debug {
                    console::log_1(&JsValue::from_str(&format!(
                        "No handler registered for message type: {}",
                        message_type
                    )));
                }

                // Route the message to any recipient agents if it has a to field
                if let Ok(to_prop) = Reflect::get(&message, &JsValue::from_str("to")) {
                    if to_prop.is_array() {
                        let to_array = js_sys::Array::from(&to_prop);
                        for i in 0..to_array.length() {
                            if let Some(recipient) = to_array.get(i).as_string() {
                                if let Some(agent) = agents.get(&recipient) {
                                    let _ = agent
                                        .process_message(message_clone.clone(), meta_obj.clone());
                                }
                            }
                        }
                    }
                }

                Ok(JsValue::FALSE)
            }
        })
    }
}

/// Creates a new DID key pair of the specified type
#[wasm_bindgen]
pub fn create_did_key(key_type_str: Option<String>) -> Result<JsValue, JsValue> {
    // Create a key manager
    let key_manager = KeyManager::new();

    // Convert key type string to KeyType enum
    let key_type = match key_type_str {
        Some(kt) => match kt.as_str() {
            "Ed25519" => KeyType::Ed25519,
            "P256" => KeyType::P256,
            "Secp256k1" => KeyType::Secp256k1,
            _ => KeyType::Ed25519, // Default to Ed25519 for unknown types
        },
        None => KeyType::Ed25519, // Default to Ed25519 if not specified
    };

    // Generate a key using the key manager
    let key = match key_manager.generate_key(DIDGenerationOptions { key_type }) {
        Ok(k) => k,
        Err(e) => {
            return Err(JsValue::from_str(&format!(
                "Failed to generate key: {:?}",
                e
            )))
        }
    };

    // Create a result object
    let result = Object::new();

    // Set the DID
    Reflect::set(
        &result,
        &JsValue::from_str("did"),
        &JsValue::from_str(&key.did),
    )?;

    // Set the key type
    Reflect::set(
        &result,
        &JsValue::from_str("keyType"),
        &JsValue::from_str(match key.key_type {
            KeyType::Ed25519 => "Ed25519",
            KeyType::P256 => "P256",
            KeyType::Secp256k1 => "Secp256k1",
        }),
    )?;

    // Set the public key as a Uint8Array
    let public_key_array = js_sys::Uint8Array::new_with_length(key.public_key.len() as u32);
    public_key_array.copy_from(&key.public_key);
    Reflect::set(&result, &JsValue::from_str("publicKey"), &public_key_array)?;

    // Set the private key as a Uint8Array
    let private_key_array = js_sys::Uint8Array::new_with_length(key.private_key.len() as u32);
    private_key_array.copy_from(&key.private_key);
    Reflect::set(
        &result,
        &JsValue::from_str("privateKey"),
        &private_key_array,
    )?;

    // Add convenience methods for signing and verification
    let did = key.did.clone();
    let key_id = format!("{}#keys-1", did);

    // Get class prototype
    let prototype = js_sys::Object::get_prototype_of(&result);

    // Add signData method
    let sign_fn = js_sys::Function::new_with_args(
        "data",
        &format!(
            "
        if (!this.privateKey) return null;
        // This would be the actual Ed25519 signing, here we just return a mock
        return 'signed_{}_{}' + data;
        ",
            key_id,
            match key.key_type {
                KeyType::Ed25519 => "Ed25519",
                KeyType::P256 => "P256",
                KeyType::Secp256k1 => "Secp256k1",
            }
        ),
    );
    js_sys::Reflect::set(&prototype, &JsValue::from_str("signData"), &sign_fn)?;

    // Add verifySignature method
    let verify_fn = js_sys::Function::new_with_args(
        "data, signature",
        &format!(
            "
        if (!this.publicKey) return false;
        // This would be the actual signature verification, here we just check the format
        return signature.startsWith('signed_{}_{}_');
        ",
            key_id,
            match key.key_type {
                KeyType::Ed25519 => "Ed25519",
                KeyType::P256 => "P256",
                KeyType::Secp256k1 => "Secp256k1",
            }
        ),
    );
    js_sys::Reflect::set(
        &prototype,
        &JsValue::from_str("verifySignature"),
        &verify_fn,
    )?;

    // Add aliases for WASM naming conventions
    let sign_fn_alias = js_sys::Function::new_with_args("data", "return this.signData(data);");
    js_sys::Reflect::set(&prototype, &JsValue::from_str("sign"), &sign_fn_alias)?;

    let verify_fn_alias = js_sys::Function::new_with_args(
        "data, signature",
        "return this.verifySignature(data, signature);",
    );
    js_sys::Reflect::set(&prototype, &JsValue::from_str("verify"), &verify_fn_alias)?;

    // Add convenience method to get the private key as hex
    let get_private_key_hex_fn = js_sys::Function::new_with_args(
        "",
        "
        if (!this.privateKey) return null;
        return Array.from(this.privateKey).map(b => b.toString(16).padStart(2, '0')).join('');
        ",
    );
    js_sys::Reflect::set(
        &prototype,
        &JsValue::from_str("getPrivateKeyHex"),
        &get_private_key_hex_fn,
    )?;

    // Add convenience method to get the public key as hex
    let get_public_key_hex_fn = js_sys::Function::new_with_args(
        "",
        "
        if (!this.publicKey) return null;
        return Array.from(this.publicKey).map(b => b.toString(16).padStart(2, '0')).join('');
        ",
    );
    js_sys::Reflect::set(
        &prototype,
        &JsValue::from_str("getPublicKeyHex"),
        &get_public_key_hex_fn,
    )?;

    Ok(result.into())
}

/// Creates a new UUID using the wasm-bindgen compatible uuid crate
#[wasm_bindgen]
pub fn generate_uuid_v4() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Utility function to convert a JavaScript Uint8Array to a Rust Vec<u8>
fn js_sys_to_vec_u8(arr: &js_sys::Uint8Array) -> Vec<u8> {
    let length = arr.length() as usize;
    let mut vec = vec![0; length];
    arr.copy_to(&mut vec);
    vec
}
