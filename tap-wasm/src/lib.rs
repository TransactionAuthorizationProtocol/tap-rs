//! # TAP WASM Bindings
//!
//! This crate provides WebAssembly (WASM) bindings for the Transaction Authorization Protocol (TAP)
//! core libraries, including `tap-msg` and `tap-agent`. It allows TAP functionality
//! to be used in JavaScript environments such as web browsers and Node.js.
//!
//! ## Features
//!
//! - Exposes TAP message creation, parsing, and validation to JavaScript.
//! - Provides access to TAP agent functionalities for DIDComm message packing/unpacking.
//! - Enables key management and cryptographic operations in a WASM environment.
//!
//! See the `README.md` for usage examples.

use base64::Engine;
use didcomm::secrets::{Secret, SecretMaterial, SecretType};
use didcomm::Message as DIDCommMessage;
use js_sys::{Array, Object, Promise, Reflect};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tap_agent::crypto::{BasicSecretResolver, DebugSecretsResolver};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use web_sys::console;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Initialize the WASM module (doesn't use wasm_bindgen start anymore to avoid conflicts)
#[wasm_bindgen]
pub fn init_tap_wasm() -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    Ok(())
}

/// The type of TAP Messages following the TAP specification
#[derive(Debug, Clone, Copy, PartialEq)]
#[wasm_bindgen]
pub enum MessageType {
    /// Transaction proposal (Transfer in TAIP-3)
    Transfer,
    /// Payment Request message (TAIP-14)
    PaymentRequest,
    /// Presentation message (TAIP-8)
    Presentation,
    /// Authorization response (TAIP-4)
    Authorize,
    /// Rejection response (TAIP-4)
    Reject,
    /// Settlement notification (TAIP-4)
    Settle,
    /// Add agents to a transaction (TAIP-5)
    AddAgents,
    /// Replace an agent in a transaction (TAIP-5)
    ReplaceAgent,
    /// Remove an agent from a transaction (TAIP-5)
    RemoveAgent,
    /// Update policies for a transaction (TAIP-7)
    UpdatePolicies,
    /// Update party information (TAIP-6)
    UpdateParty,
    /// Confirm relationship between agent and entity (TAIP-9)
    ConfirmRelationship,
    /// Error message
    Error,
    /// Unknown message type
    Unknown,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageType::Transfer => write!(f, "https://tap.rsvp/schema/1.0#Transfer"),
            MessageType::PaymentRequest => write!(f, "https://tap.rsvp/schema/1.0#PaymentRequest"),
            MessageType::Presentation => write!(f, "https://tap.rsvp/schema/1.0#Presentation"),
            MessageType::Authorize => write!(f, "https://tap.rsvp/schema/1.0#Authorize"),
            MessageType::Reject => write!(f, "https://tap.rsvp/schema/1.0#Reject"),
            MessageType::Settle => write!(f, "https://tap.rsvp/schema/1.0#Settle"),
            MessageType::AddAgents => write!(f, "https://tap.rsvp/schema/1.0#AddAgents"),
            MessageType::ReplaceAgent => write!(f, "https://tap.rsvp/schema/1.0#ReplaceAgent"),
            MessageType::RemoveAgent => write!(f, "https://tap.rsvp/schema/1.0#RemoveAgent"),
            MessageType::UpdatePolicies => write!(f, "https://tap.rsvp/schema/1.0#UpdatePolicies"),
            MessageType::UpdateParty => write!(f, "https://tap.rsvp/schema/1.0#UpdateParty"),
            MessageType::ConfirmRelationship => {
                write!(f, "https://tap.rsvp/schema/1.0#confirmrelationship")
            }
            MessageType::Error => write!(f, "https://tap.rsvp/schema/1.0#Error"),
            MessageType::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// Participant structure for participants in a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    /// DID of the participant.
    #[serde(rename = "@id")]
    pub id: String,

    /// Role of the participant in the transaction (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Legal Entity Identifier (LEI) code of the participant (optional).
    #[serde(skip_serializing_if = "Option::is_none", rename = "leiCode")]
    pub lei_code: Option<String>,

    /// Human-readable name of the participant (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// SHA-256 hash of normalized name (uppercase, no whitespace) (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_hash: Option<String>,

    /// Reference to the party this agent is acting for (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub for_: Option<String>,
}

/// Transfer message body (TAIP-3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    /// Network asset identifier in CAIP-19 format.
    pub asset: String,

    /// Originator information.
    pub originator: Participant,

    /// Beneficiary information (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beneficiary: Option<Participant>,

    /// Amount as a decimal string.
    pub amount: String,

    /// Agents involved in the transaction.
    pub agents: Vec<Participant>,

    /// Optional settled transaction ID.
    #[serde(skip_serializing_if = "Option::is_none", rename = "settlementId")]
    pub settlement_id: Option<String>,

    /// Optional memo or note for the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    /// Optional ISO 20022 purpose code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,

    /// Optional ISO 20022 category purpose code.
    #[serde(skip_serializing_if = "Option::is_none", rename = "categoryPurpose")]
    pub category_purpose: Option<String>,

    /// Additional metadata for the transaction.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Payment Request message body (TAIP-14).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    /// Optional network asset identifier in CAIP-19 format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,

    /// Optional currency code (typically ISO 4217).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,

    /// Amount as a decimal string.
    pub amount: String,

    /// Optional list of supported assets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_assets: Option<Vec<String>>,

    /// Optional invoice data or reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice: Option<String>,

    /// Optional expiry time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,

    /// Merchant information.
    pub merchant: Participant,

    /// Optional customer information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<Participant>,

    /// Agents involved in the payment request.
    pub agents: Vec<Participant>,

    /// Additional metadata for the payment request.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Authorization message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Authorize {
    /// The ID of the transfer being authorized.
    pub transaction_id: String,

    /// Optional note about the authorization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Timestamp when the authorization was created.
    pub timestamp: String,

    /// Optional settlement address in CAIP-10 format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_address: Option<String>,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Reject message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reject {
    /// The ID of the transfer being rejected.
    pub transaction_id: String,

    /// Reason code for the rejection.
    pub code: String,

    /// Human-readable description of the rejection reason.
    pub description: String,

    /// Optional note about the rejection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Timestamp when the rejection was created.
    pub timestamp: String,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Settle message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settle {
    /// The ID of the transfer being settled.
    pub transaction_id: String,

    /// Settlement ID on the external ledger.
    pub settlement_id: String,

    /// Optional amount settled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
}

/// Cancel message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cancel {
    /// The ID of the transfer being cancelled.
    pub transaction_id: String,

    /// Optional reason for cancellation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Optional note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Timestamp when the cancellation was created.
    pub timestamp: String,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Revert message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revert {
    /// The ID of the transfer being reverted.
    pub transaction_id: String,

    /// Settlement address in CAIP-10 format to return the funds to.
    pub settlement_address: String,

    /// Reason for the reversal request.
    pub reason: String,

    /// Optional note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Timestamp when the revert request was created.
    pub timestamp: String,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
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
        self.didcomm_message.type_ = format!(
            "https://tap.org/protocols/{}/{}",
            message_type, self.version
        );
    }

    /// Gets the message version
    pub fn version(&self) -> String {
        self.version.clone()
    }

    /// Sets the message version
    pub fn set_version(&mut self, version: String) {
        self.version = version.clone();
        // Update the DIDComm message type to include the new version
        self.didcomm_message.type_ = format!(
            "https://tap.org/protocols/{}/{}",
            self.message_type, version
        );
    }

    // Deprecated method set_authorization_request removed

    /// Sets a Transfer message body according to TAIP-3
    pub fn set_transfer_body(&mut self, transfer_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to a Transfer
        let transfer_body: Transfer = serde_wasm_bindgen::from_value(transfer_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse transfer data: {}", e)))?;

        // Convert to JSON value and store in body data
        let json_value = serde_json::to_value(&transfer_body)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize transfer data: {}", e)))?;

        self.body_data.insert("transfer".to_string(), json_value);

        // Set the message type to Transfer and update the didcomm type
        self.message_type = "Transfer".to_string();
        self.didcomm_message.type_ = format!("https://tap.rsvp/schema/{}#Transfer", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = serde_json::json!(transfer_body);

        Ok(())
    }

    /// Sets a Payment Request message body according to TAIP-14
    pub fn set_payment_request_body(
        &mut self,
        payment_request_data: JsValue,
    ) -> Result<(), JsValue> {
        // Convert the JavaScript value to a PaymentRequest
        let payment_request_body: PaymentRequest =
            serde_wasm_bindgen::from_value(payment_request_data).map_err(|e| {
                JsValue::from_str(&format!("Failed to parse payment request data: {}", e))
            })?;

        // Convert to JSON value and store in body data
        let json_value = serde_json::to_value(&payment_request_body).map_err(|e| {
            JsValue::from_str(&format!("Failed to serialize payment request data: {}", e))
        })?;

        self.body_data
            .insert("payment_request".to_string(), json_value);

        // Set the message type to PaymentRequest and update the didcomm type
        self.message_type = "PaymentRequest".to_string();
        self.didcomm_message.type_ =
            format!("https://tap.rsvp/schema/{}#PaymentRequest", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = serde_json::json!(payment_request_body);

        Ok(())
    }

    /// Sets an Authorize message body according to TAIP-4
    pub fn set_authorize_body(&mut self, authorize_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to an Authorize
        let authorize_body: Authorize = serde_wasm_bindgen::from_value(authorize_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse authorize data: {}", e)))?;

        // Convert to JSON value and store in body data
        let json_value = serde_json::to_value(&authorize_body).map_err(|e| {
            JsValue::from_str(&format!("Failed to serialize authorize data: {}", e))
        })?;

        self.body_data.insert("authorize".to_string(), json_value);

        // Set the message type to Authorize and update the didcomm type
        self.message_type = "Authorize".to_string();
        self.didcomm_message.type_ = format!("https://tap.rsvp/schema/{}#Authorize", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = serde_json::json!(authorize_body);

        Ok(())
    }

    // Deprecated method set_authorization_response removed

    /// Sets a Reject message body according to TAIP-4
    pub fn set_reject_body(&mut self, reject_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to a Reject
        let reject_body: Reject = serde_wasm_bindgen::from_value(reject_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse reject data: {}", e)))?;

        // Convert to JSON value and store in body data
        let json_value = serde_json::to_value(&reject_body)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize reject data: {}", e)))?;

        self.body_data.insert("reject".to_string(), json_value);

        // Set the message type to Reject and update the didcomm type
        self.message_type = "Reject".to_string();
        self.didcomm_message.type_ = format!("https://tap.rsvp/schema/{}#Reject", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = serde_json::json!(reject_body);

        Ok(())
    }

    /// Sets a Settle message body according to TAIP-4
    pub fn set_settle_body(&mut self, settle_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to a Settle
        let settle_body: Settle = serde_wasm_bindgen::from_value(settle_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse settle data: {}", e)))?;

        // Convert to JSON value and store in body data
        let json_value = serde_json::to_value(&settle_body)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize settle data: {}", e)))?;

        self.body_data.insert("settle".to_string(), json_value);

        // Set the message type to Settle and update the didcomm type
        self.message_type = "Settle".to_string();
        self.didcomm_message.type_ = format!("https://tap.rsvp/schema/{}#Settle", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = serde_json::json!(settle_body);

        Ok(())
    }

    /// Sets a Cancel message body according to TAIP-4
    pub fn set_cancel_body(&mut self, cancel_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to a Cancel
        let cancel_body: Cancel = serde_wasm_bindgen::from_value(cancel_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse cancel data: {}", e)))?;

        // Convert to JSON value and store in body data
        let json_value = serde_json::to_value(&cancel_body)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize cancel data: {}", e)))?;

        self.body_data.insert("cancel".to_string(), json_value);

        // Set the message type to Cancel and update the didcomm type
        self.message_type = "Cancel".to_string();
        self.didcomm_message.type_ = format!("https://tap.rsvp/schema/{}#Cancel", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = serde_json::json!(cancel_body);

        Ok(())
    }

    /// Sets a Revert message body according to TAIP-4
    pub fn set_revert_body(&mut self, revert_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to a Revert
        let revert_body: Revert = serde_wasm_bindgen::from_value(revert_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse revert data: {}", e)))?;

        // Convert to JSON value and store in body data
        let json_value = serde_json::to_value(&revert_body)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize revert data: {}", e)))?;

        self.body_data.insert("revert".to_string(), json_value);

        // Set the message type to Revert and update the didcomm type
        self.message_type = "Revert".to_string();
        self.didcomm_message.type_ = format!("https://tap.rsvp/schema/{}#Revert", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = serde_json::json!(revert_body);

        Ok(())
    }

    /// Sets a presentation body for the message (TAIP-8)
    pub fn set_presentation_body(&mut self, presentation_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to a Presentation
        // Use a specific type to avoid never type fallback warnings
        let presentation_body: serde_json::Value =
            serde_wasm_bindgen::from_value(presentation_data).map_err(|e| {
                JsValue::from_str(&format!("Failed to parse presentation data: {}", e))
            })?;

        // Store the value directly in body_data
        self.body_data
            .insert("presentation".to_string(), presentation_body.clone());

        // Set the message type to Presentation and update the didcomm type
        self.message_type = "Presentation".to_string();
        self.didcomm_message.type_ =
            format!("https://tap.rsvp/schema/{}#Presentation", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = presentation_body;

        Ok(())
    }

    /// Sets an add agents body for the message (TAIP-5)
    pub fn set_add_agents_body(&mut self, add_agents_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to an AddAgents
        // Use a specific type to avoid never type fallback warnings
        let add_agents_body: serde_json::Value = serde_wasm_bindgen::from_value(add_agents_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse add agents data: {}", e)))?;

        // Store the value directly in body_data
        self.body_data
            .insert("add_agents".to_string(), add_agents_body.clone());

        // Set the message type to AddAgents and update the didcomm type
        self.message_type = "AddAgents".to_string();
        self.didcomm_message.type_ = format!("https://tap.rsvp/schema/{}#AddAgents", self.version);

        // Set the DIDComm message body
        self.didcomm_message.body = add_agents_body;

        Ok(())
    }

    /// Sets a replace agent body for the message (TAIP-5)
    pub fn set_replace_agent_body(&mut self, replace_agent_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to a ReplaceAgent
        // Use a specific type to avoid never type fallback warnings
        let replace_agent_body: serde_json::Value =
            serde_wasm_bindgen::from_value(replace_agent_data).map_err(|e| {
                JsValue::from_str(&format!("Failed to parse replace agent data: {}", e))
            })?;

        // Store the value directly in body_data
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

    /// Sets a remove agent body for the message (TAIP-5)
    pub fn set_remove_agent_body(&mut self, remove_agent_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to a RemoveAgent
        // Use a specific type to avoid never type fallback warnings
        let remove_agent_body: serde_json::Value =
            serde_wasm_bindgen::from_value(remove_agent_data).map_err(|e| {
                JsValue::from_str(&format!("Failed to parse remove agent data: {}", e))
            })?;

        // Store the value directly in body_data
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

    /// Sets an update policies body for the message (TAIP-7)
    pub fn set_update_policies_body(
        &mut self,
        update_policies_data: JsValue,
    ) -> Result<(), JsValue> {
        // Convert the JavaScript value to an UpdatePolicies
        // Use a specific type to avoid never type fallback warnings
        let update_policies_body: serde_json::Value =
            serde_wasm_bindgen::from_value(update_policies_data).map_err(|e| {
                JsValue::from_str(&format!("Failed to parse update policies data: {}", e))
            })?;

        // Store the value directly in body_data
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

    /// Gets the transfer body data
    pub fn get_transfer_body(&self) -> JsValue {
        // Check if this is a Transfer message
        if self.message_type == "Transfer" || self.didcomm_message.type_.contains("#Transfer") {
            // Try to get from body_data first
            if let Some(value) = self.body_data.get("transfer") {
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

        // Not a Transfer message
        JsValue::null()
    }

    /// Gets the payment request body data
    pub fn get_payment_request_body(&self) -> JsValue {
        // Check if this is a PaymentRequest message
        if self.message_type == "PaymentRequest"
            || self.didcomm_message.type_.contains("#PaymentRequest")
        {
            // Try to get from body_data first
            if let Some(value) = self.body_data.get("payment_request") {
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

        // Not a PaymentRequest message
        JsValue::null()
    }

    /// Gets the authorize body data
    pub fn get_authorize_body(&self) -> JsValue {
        // Check if this is an Authorize message
        if self.message_type == "Authorize" || self.didcomm_message.type_.contains("#Authorize") {
            // Try to get from body_data first
            if let Some(value) = self.body_data.get("authorize") {
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

        // Not an Authorize message
        JsValue::null()
    }

    /// Gets the reject body data
    pub fn get_reject_body(&self) -> JsValue {
        // Check if this is a Reject message
        if self.message_type == "Reject" || self.didcomm_message.type_.contains("#Reject") {
            // Try to get from body_data first
            if let Some(value) = self.body_data.get("reject") {
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

        // Not a Reject message
        JsValue::null()
    }

    /// Gets the settle body data
    pub fn get_settle_body(&self) -> JsValue {
        // Check if this is a Settle message
        if self.message_type == "Settle" || self.didcomm_message.type_.contains("#Settle") {
            // Try to get from body_data first
            if let Some(value) = self.body_data.get("settle") {
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

        // Not a Settle message
        JsValue::null()
    }

    /// Gets the cancel body data
    pub fn get_cancel_body(&self) -> JsValue {
        // Check if this is a Cancel message
        if self.message_type == "Cancel" || self.didcomm_message.type_.contains("#Cancel") {
            // Try to get from body_data first
            if let Some(value) = self.body_data.get("cancel") {
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

        // Not a Cancel message
        JsValue::null()
    }

    /// Gets the revert body data
    pub fn get_revert_body(&self) -> JsValue {
        // Check if this is a Revert message
        if self.message_type == "Revert" || self.didcomm_message.type_.contains("#Revert") {
            // Try to get from body_data first
            if let Some(value) = self.body_data.get("revert") {
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

        // Not a Revert message
        JsValue::null()
    }

    /// Gets the presentation body data (TAIP-8)
    pub fn get_presentation_body(&self) -> JsValue {
        // Check if this is a Presentation message
        if self.message_type == "Presentation"
            || self.didcomm_message.type_.contains("#Presentation")
        {
            // Try to get from body_data first
            if let Some(value) = self.body_data.get("presentation") {
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

        // Not a Presentation message
        JsValue::null()
    }

    /// Gets the add agents body data (TAIP-5)
    pub fn get_add_agents_body(&self) -> JsValue {
        // Check if this is an AddAgents message
        if self.message_type == "AddAgents" || self.didcomm_message.type_.contains("#AddAgents") {
            // Try to get from body_data first
            if let Some(value) = self.body_data.get("add_agents") {
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

        // Not an AddAgents message
        JsValue::null()
    }

    /// Gets the replace agent body data (TAIP-5)
    pub fn get_replace_agent_body(&self) -> JsValue {
        // Check if this is a ReplaceAgent message
        if self.message_type == "ReplaceAgent"
            || self.didcomm_message.type_.contains("#ReplaceAgent")
        {
            // Try to get from body_data first
            if let Some(value) = self.body_data.get("replace_agent") {
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

        // Not a ReplaceAgent message
        JsValue::null()
    }

    /// Gets the remove agent body data (TAIP-5)
    pub fn get_remove_agent_body(&self) -> JsValue {
        // Check if this is a RemoveAgent message
        if self.message_type == "RemoveAgent" || self.didcomm_message.type_.contains("#RemoveAgent")
        {
            // Try to get from body_data first
            if let Some(value) = self.body_data.get("remove_agent") {
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

        // Not a RemoveAgent message
        JsValue::null()
    }

    /// Gets the update policies body data (TAIP-7)
    pub fn get_update_policies_body(&self) -> JsValue {
        // Check if this is an UpdatePolicies message
        if self.message_type == "UpdatePolicies"
            || self.didcomm_message.type_.contains("#UpdatePolicies")
        {
            // Try to get from body_data first
            if let Some(value) = self.body_data.get("update_policies") {
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

        // Not an UpdatePolicies message
        JsValue::null()
    }

    /// Sets an update party body for the message (TAIP-6)
    pub fn set_update_party_body(&mut self, update_party_data: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to an UpdateParty
        // Use a specific type to avoid never type fallback warnings
        let update_party_body: serde_json::Value =
            serde_wasm_bindgen::from_value(update_party_data).map_err(|e| {
                JsValue::from_str(&format!("Failed to parse update party data: {}", e))
            })?;

        // Store the value directly in body_data
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

    /// Sets a confirm relationship body for the message (TAIP-9)
    pub fn set_confirm_relationship_body(
        &mut self,
        confirm_relationship_data: JsValue,
    ) -> Result<(), JsValue> {
        // Convert the JavaScript value to a JSON value
        let confirm_relationship_body: serde_json::Value =
            serde_wasm_bindgen::from_value(confirm_relationship_data).map_err(|e| {
                JsValue::from_str(&format!("Failed to parse confirm relationship data: {}", e))
            })?;

        // Store the value directly in body_data
        self.body_data.insert(
            "confirm_relationship".to_string(),
            confirm_relationship_body.clone(),
        );

        // Set the message type to ConfirmRelationship and update the didcomm type
        self.message_type = "ConfirmRelationship".to_string();
        self.didcomm_message.type_ = format!(
            "https://tap.rsvp/schema/{}#confirmrelationship",
            self.version
        );

        // Set the DIDComm message body
        self.didcomm_message.body = confirm_relationship_body;

        Ok(())
    }

    /// Gets the update party body data (TAIP-6)
    pub fn get_update_party_body(&self) -> JsValue {
        // Check if this is an UpdateParty message
        if self.message_type == "UpdateParty" || self.didcomm_message.type_.contains("#UpdateParty")
        {
            // Try to get from body_data first
            if let Some(value) = self.body_data.get("update_party") {
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

        // Not an UpdateParty message
        JsValue::null()
    }

    /// Gets the confirm relationship body data (TAIP-9)
    pub fn get_confirm_relationship_body(&self) -> JsValue {
        // Check if this is a ConfirmRelationship message
        if self.message_type == "ConfirmRelationship"
            || self.didcomm_message.type_.contains("#confirmrelationship")
        {
            // Try to get from body_data first
            if let Some(value) = self.body_data.get("confirm_relationship") {
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

        // Not a ConfirmRelationship message
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

    /// Creates a new message from this agent
    pub fn create_message(&self, message_type: MessageType) -> Message {
        let id = format!("msg_{}", generate_uuid_v4());

        Message::new(id, message_type.to_string(), "1.0".to_string())
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

        // In a real implementation, we would use the DIDComm library to validate the signature
        // against the public key of the sender's DID

        // For now, we'll simulate a signature check
        if debug {
            let expected_pattern = format!("signed_by_{}_with_didcomm", from_did);
            console::log_1(&JsValue::from_str(&format!(
                "Message verification result: true (simulated), would check for pattern: {}",
                expected_pattern
            )));
        }

        // Always return true for now, in a real implementation we'd check the signature
        Ok(true)
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
}

#[wasm_bindgen]
impl TapAgent {
    /// Creates a new agent with the specified configuration
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Self {
        let did = if let Ok(did_prop) = Reflect::get(&config, &JsValue::from_str("did")) {
            if let Some(did_str) = did_prop.as_string() {
                did_str
            } else {
                format!("did:key:z6Mk{}", uuid::Uuid::new_v4().to_simple())
            }
        } else {
            format!("did:key:z6Mk{}", uuid::Uuid::new_v4().to_simple())
        };

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

        // Create a secret resolver
        let mut secrets_resolver = BasicSecretResolver::new();

        // For now, we're just creating a mock secret with proper types from didcomm
        let secret = Secret {
            id: did.clone(),
            type_: SecretType::JsonWebKey2020,
            secret_material: SecretMaterial::JWK {
                private_key_jwk: serde_json::json!({
                    "kty": "OKP",
                    "kid": did.clone(),
                    "crv": "Ed25519",
                    "x": "11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo",
                    "d": "nWGxne/9WmC6hEr+BQh+uDpW6n7dZsN4c4C9rFfIz3Yh"
                }),
            },
        };

        // Add the secret to the resolver
        secrets_resolver.add_secret(&did, secret);

        TapAgent {
            id: did,
            nickname,
            debug,
            message_handlers: HashMap::new(),
            message_subscribers: Vec::new(),
            secrets_resolver: Arc::new(secrets_resolver),
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

        message.set_from_did(Some(self.id.clone()));

        message
    }

    /// Sets the from field for a message
    pub fn set_from(&self, message: &mut Message) {
        message.set_from_did(Some(self.id.clone()));
    }

    /// Sets the to field for a message
    pub fn set_to(&self, message: &mut Message, to_did: String) {
        message.set_to_did(Some(to_did));
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

    /// Signs a message using the agent's keys
    pub fn sign_message(&self, message: &mut Message) -> Result<(), JsValue> {
        let didcomm_message = message.get_didcomm_message_ref().clone();

        // We need the from field to be set for signing
        if didcomm_message.from.is_none() {
            // This modifies a copy, not the actual message
            // We need to update the from field in the message itself in a separate step
            message.set_from_did(Some(self.id.clone()));
        }

        // For a complete implementation, we would use the didcomm library's signing capabilities
        // through the secrets_resolver. Here's a placeholder that simulates the signing process.

        // Check if we have a secret for this DID
        let secrets_map = self.secrets_resolver.get_secrets_map();
        if !secrets_map.contains_key(&self.id) {
            return Err(JsValue::from_str(&format!(
                "No secret found for DID: {}",
                self.id
            )));
        }

        if self.debug {
            console::log_1(&JsValue::from_str(&format!(
                "Message signed by {}",
                self.id
            )));
        }

        Ok(())
    }

    /// Processes a received message
    pub fn process_message(&self, message: JsValue, metadata: JsValue) -> Promise {
        // Clone data that needs to be moved into the async block
        let debug = self.debug;
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

    /// Gets the agent's secrets resolver for advanced use cases
    pub fn get_keys_info(&self) -> JsValue {
        let secrets_map = self.secrets_resolver.get_secrets_map();
        let mut keys_info = Vec::new();

        for (did, secret) in secrets_map.iter() {
            let key_info = serde_json::json!({
                "did": did,
                "type": secret.type_,
                "has_private_key": true, // In a real implementation, we would check if we have a private key
                "has_public_key": true,  // In a real implementation, we would check if we have a public key
            });
            keys_info.push(key_info);
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
        _key_type: String,
        private_key: Option<js_sys::Uint8Array>,
        public_key: Option<js_sys::Uint8Array>,
    ) -> Result<(), JsValue> {
        // Create a copy of the secrets resolver
        let mut secrets_resolver = BasicSecretResolver::new();

        // Copy existing secrets
        for (existing_did, existing_secret) in self.secrets_resolver.get_secrets_map().iter() {
            secrets_resolver.add_secret(existing_did, existing_secret.clone());
        }

        // Add the new secret
        let secret = Secret {
            id: did.clone(),
            type_: SecretType::JsonWebKey2020,
            secret_material: SecretMaterial::JWK {
                private_key_jwk: serde_json::json!({
                    "kty": "OKP",
                    "kid": did.clone(),
                    "crv": "Ed25519",
                    "x": match &public_key {
                        Some(pk) => {
                            let vec = js_sys_to_vec_u8(pk);
                            base64::engine::general_purpose::STANDARD.encode(vec)
                        },
                        None => "11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo".to_string()
                    },
                    "d": match &private_key {
                        Some(sk) => {
                            let vec = js_sys_to_vec_u8(sk);
                            base64::engine::general_purpose::STANDARD.encode(vec)
                        },
                        None => "nWGxne/9WmC6hEr+BQh+uDpW6n7dZsN4c4C9rFfIz3Yh".to_string()
                    }
                }),
            },
        };

        secrets_resolver.add_secret(&did, secret);

        // Update the agent's secrets resolver
        self.secrets_resolver = Arc::new(secrets_resolver);

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

                    // Handle message body data based on message types
                    if let Ok(transfer) =
                        js_sys::Reflect::get(message, &JsValue::from_str("transfer"))
                    {
                        if !transfer.is_null() && !transfer.is_undefined() {
                            if let Err(e) = msg.set_transfer_body(transfer) {
                                if debug {
                                    console::log_1(&JsValue::from_str(&format!(
                                        "Error setting transfer body: {}",
                                        e.as_string().unwrap_or_default()
                                    )));
                                }
                            }
                        }
                    }

                    if let Ok(payment_request) =
                        js_sys::Reflect::get(message, &JsValue::from_str("payment_request"))
                    {
                        if !payment_request.is_null() && !payment_request.is_undefined() {
                            if let Err(e) = msg.set_payment_request_body(payment_request) {
                                if debug {
                                    console::log_1(&JsValue::from_str(&format!(
                                        "Error setting payment request body: {}",
                                        e.as_string().unwrap_or_default()
                                    )));
                                }
                            }
                        }
                    }

                    if let Ok(authorize) =
                        js_sys::Reflect::get(message, &JsValue::from_str("authorize"))
                    {
                        if !authorize.is_null() && !authorize.is_undefined() {
                            if let Err(e) = msg.set_authorize_body(authorize) {
                                if debug {
                                    console::log_1(&JsValue::from_str(&format!(
                                        "Error setting authorize body: {}",
                                        e.as_string().unwrap_or_default()
                                    )));
                                }
                            }
                        }
                    }

                    if let Ok(reject) = js_sys::Reflect::get(message, &JsValue::from_str("reject"))
                    {
                        if !reject.is_null() && !reject.is_undefined() {
                            if let Err(e) = msg.set_reject_body(reject) {
                                if debug {
                                    console::log_1(&JsValue::from_str(&format!(
                                        "Error setting reject body: {}",
                                        e.as_string().unwrap_or_default()
                                    )));
                                }
                            }
                        }
                    }

                    if let Ok(settle) = js_sys::Reflect::get(message, &JsValue::from_str("settle"))
                    {
                        if !settle.is_null() && !settle.is_undefined() {
                            if let Err(e) = msg.set_settle_body(settle) {
                                if debug {
                                    console::log_1(&JsValue::from_str(&format!(
                                        "Error setting settle body: {}",
                                        e.as_string().unwrap_or_default()
                                    )));
                                }
                            }
                        }
                    }

                    // Legacy support for auth_req and auth_resp removed

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

    /// Adds a new agent to the node
    pub fn add_agent(&mut self, agent: TapAgent) {
        let did = agent.get_did();
        self.agents.insert(did, agent);
    }

    /// Gets an agent by DID
    pub fn get_agent(&self, did: &str) -> Option<TapAgent> {
        self.agents.get(did).cloned()
    }

    /// Gets all agents in the node
    pub fn get_agents(&self) -> Array {
        let result = Array::new();
        for (i, (did, _agent)) in self.agents.iter().enumerate() {
            // Just return the DIDs as strings for now
            result.set(i as u32, JsValue::from_str(did));
        }
        result
    }

    /// Processes a message through the appropriate agent
    pub fn process_message(&self, message: JsValue, metadata: JsValue) -> Promise {
        // Clone data that needs to be moved into the async block
        let debug = self.debug;
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
                Ok(JsValue::FALSE)
            }
        })
    }
}

/// Creates a new DID key pair
#[wasm_bindgen]
pub fn create_did_key() -> Result<JsValue, JsValue> {
    let uuid_str = uuid::Uuid::new_v4().to_simple().to_string(); // This actually needs the to_string() as it's assigning to a String variable

    let mock_did = format!("did:key:z6Mk{}", uuid_str);

    let result = Object::new();
    Reflect::set(
        &result,
        &JsValue::from_str("did"),
        &JsValue::from_str(&mock_did),
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
