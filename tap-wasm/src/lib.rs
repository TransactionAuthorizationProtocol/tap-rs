//! WebAssembly bindings for the Transaction Authorization Protocol
//!
//! This crate provides WebAssembly bindings for the TAP agent, allowing it to be used in
//! browser and other JavaScript environments. It wraps the tap-agent crate's functionality
//! with JavaScript-friendly interfaces.

mod util;
mod wasm_agent;

use js_sys::{Array, Object, Reflect};
use std::collections::HashMap;
use std::fmt;
use tap_agent::did::KeyType as TapKeyType;
use wasm_bindgen::prelude::*;
use web_sys::console;

pub use wasm_agent::WasmTapAgent;

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

/// Key type enumeration for WASM
#[wasm_bindgen]
pub enum WasmKeyType {
    /// Ed25519 key type
    Ed25519,
    /// P-256 key type
    P256,
    /// Secp256k1 key type
    Secp256k1,
}

impl From<WasmKeyType> for TapKeyType {
    fn from(key_type: WasmKeyType) -> Self {
        match key_type {
            WasmKeyType::Ed25519 => TapKeyType::Ed25519,
            WasmKeyType::P256 => TapKeyType::P256,
            WasmKeyType::Secp256k1 => TapKeyType::Secp256k1,
        }
    }
}

/// Represents a node on the TAP network that manages multiple agents
#[wasm_bindgen]
#[derive(Clone)]
pub struct TapNode {
    agents: HashMap<String, WasmTapAgent>,
    debug: bool,
}

#[wasm_bindgen]
impl TapNode {
    /// Creates a new TapNode
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Self {
        console_error_panic_hook::set_once();

        let debug = if let Ok(debug_prop) = Reflect::get(&config, &JsValue::from_str("debug")) {
            debug_prop.is_truthy()
        } else {
            false
        };

        TapNode {
            agents: HashMap::new(),
            debug,
        }
    }

    /// Adds an agent to this node
    pub fn add_agent(&mut self, agent: WasmTapAgent) -> Result<(), JsValue> {
        let did = agent.get_did();
        self.agents.insert(did.clone(), agent);

        if self.debug {
            console::log_1(&JsValue::from_str(&format!("Added agent {} to node", did)));
        }

        Ok(())
    }

    /// Gets an agent by DID
    pub fn get_agent(&self, did: &str) -> Option<WasmTapAgent> {
        self.agents.get(did).cloned()
    }

    /// Lists all agents in this node
    pub fn list_agents(&self) -> JsValue {
        let result = Array::new();

        for (did, agent) in &self.agents {
            let agent_obj = Object::new();
            Reflect::set(
                &agent_obj,
                &JsValue::from_str("did"),
                &JsValue::from_str(did),
            )
            .unwrap();

            if let Some(nickname) = agent.nickname() {
                Reflect::set(
                    &agent_obj,
                    &JsValue::from_str("nickname"),
                    &JsValue::from_str(&nickname),
                )
                .unwrap();
            }

            result.push(&agent_obj);
        }

        result.into()
    }

    /// Removes an agent from this node
    pub fn remove_agent(&mut self, did: &str) -> bool {
        let removed = self.agents.remove(did).is_some();

        if removed && self.debug {
            console::log_1(&JsValue::from_str(&format!(
                "Removed agent {} from node",
                did
            )));
        }

        removed
    }
}

/// Generates a UUID v4
#[wasm_bindgen]
pub fn generate_uuid_v4() -> String {
    uuid::Uuid::new_v4().to_string()
}
