use js_sys::{Array, Function, Object, Promise, Reflect};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen;
use std::collections::HashMap;
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

/// Message type enum in JavaScript
#[wasm_bindgen]
pub enum MessageType {
    AuthorizationRequest = 0,
    AuthorizationResponse = 1,
    Ping = 2,
}

impl ToString for MessageType {
    fn to_string(&self) -> String {
        match self {
            MessageType::AuthorizationRequest => "TAP_AUTHORIZATION_REQUEST".to_string(),
            MessageType::AuthorizationResponse => "TAP_AUTHORIZATION_RESPONSE".to_string(),
            MessageType::Ping => "TAP_PING".to_string(),
        }
    }
}

/// Network configuration
#[derive(Serialize, Deserialize, Clone)]
pub struct NetworkConfig {
    pub reference_id: String,
    pub network_id: String,
    pub sender_account_number: Option<String>,
    pub sender_address: Option<String>,
    pub sender_chain_address: Option<String>,
}

/// Authorization request
#[derive(Serialize, Deserialize, Clone)]
pub struct AuthorizationRequest {
    pub private_payload: serde_json::Value,
}

/// Authorization response
#[derive(Serialize, Deserialize, Clone)]
pub struct AuthorizationResponse {
    pub private_payload: serde_json::Value,
    pub signed_date: String,
    pub valid_until: Option<String>,
}

/// TAP Message
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct Message {
    id: String,
    message_type: String,
    version: String,
    ledger_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_request: Option<AuthorizationRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_response: Option<AuthorizationResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    from_did: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    to_did: Option<String>,
}

#[wasm_bindgen]
impl Message {
    /// Creates a new message with the specified types and fields
    #[wasm_bindgen(constructor)]
    pub fn new(id: String, message_type: String, version: String, ledger_id: String) -> Message {
        Message {
            id,
            message_type,
            version,
            ledger_id,
            auth_request: None,
            auth_response: None,
            from_did: None,
            to_did: None,
        }
    }

    /// Gets the message ID
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Sets the message ID
    #[wasm_bindgen(setter)]
    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }

    /// Gets the message type
    #[wasm_bindgen(getter)]
    pub fn message_type(&self) -> String {
        self.message_type.clone()
    }

    /// Sets the message type
    #[wasm_bindgen(setter)]
    pub fn set_message_type(&mut self, message_type: String) {
        self.message_type = message_type;
    }

    /// Gets the message version
    #[wasm_bindgen(getter)]
    pub fn version(&self) -> String {
        self.version.clone()
    }

    /// Sets the message version
    #[wasm_bindgen(setter)]
    pub fn set_version(&mut self, version: String) {
        self.version = version;
    }

    /// Gets the ledger ID
    #[wasm_bindgen(getter)]
    pub fn ledger_id(&self) -> String {
        self.ledger_id.clone()
    }

    /// Sets the ledger ID
    #[wasm_bindgen(setter)]
    pub fn set_ledger_id(&mut self, ledger_id: String) {
        self.ledger_id = ledger_id;
    }

    /// Sets the authorization request data
    pub fn set_authorization_request(&mut self, private_payload: JsValue) {
        let json_value: serde_json::Value =
            serde_wasm_bindgen::from_value(private_payload).unwrap_or(serde_json::Value::Null);
        self.auth_request = Some(AuthorizationRequest {
            private_payload: json_value,
        });
    }

    /// Sets the authorization response data
    pub fn set_authorization_response(
        &mut self,
        private_payload: JsValue,
        signed_date: String,
        valid_until: Option<String>,
    ) {
        let json_value: serde_json::Value =
            serde_wasm_bindgen::from_value(private_payload).unwrap_or(serde_json::Value::Null);
        self.auth_response = Some(AuthorizationResponse {
            private_payload: json_value,
            signed_date,
            valid_until,
        });
    }

    /// Gets the authorization request as a JavaScript object
    #[wasm_bindgen(getter)]
    pub fn authorization_request(&self) -> JsValue {
        match &self.auth_request {
            Some(request) => {
                let obj = js_sys::Object::new();
                let payload_js =
                    serde_wasm_bindgen::to_value(&request.private_payload).unwrap_or(JsValue::NULL);
                Reflect::set(&obj, &JsValue::from_str("privatePayload"), &payload_js).unwrap();
                obj.into()
            }
            None => JsValue::NULL,
        }
    }

    /// Gets the authorization response as a JavaScript object
    #[wasm_bindgen(getter)]
    pub fn authorization_response(&self) -> JsValue {
        match &self.auth_response {
            Some(response) => {
                let obj = js_sys::Object::new();
                let payload_js = serde_wasm_bindgen::to_value(&response.private_payload)
                    .unwrap_or(JsValue::NULL);
                Reflect::set(&obj, &JsValue::from_str("privatePayload"), &payload_js).unwrap();
                Reflect::set(
                    &obj,
                    &JsValue::from_str("signedDate"),
                    &JsValue::from_str(&response.signed_date),
                )
                .unwrap();
                if let Some(valid_until) = &response.valid_until {
                    Reflect::set(
                        &obj,
                        &JsValue::from_str("validUntil"),
                        &JsValue::from_str(valid_until),
                    )
                    .unwrap();
                }
                obj.into()
            }
            None => JsValue::NULL,
        }
    }

    /// Gets the sender DID
    #[wasm_bindgen(getter)]
    pub fn from_did(&self) -> Option<String> {
        self.from_did.clone()
    }

    /// Sets the sender DID
    #[wasm_bindgen(setter)]
    pub fn set_from_did(&mut self, from_did: Option<String>) {
        self.from_did = from_did;
    }

    /// Gets the recipient DID
    #[wasm_bindgen(getter)]
    pub fn to_did(&self) -> Option<String> {
        self.to_did.clone()
    }

    /// Sets the recipient DID
    #[wasm_bindgen(setter)]
    pub fn set_to_did(&mut self, to_did: Option<String>) {
        self.to_did = to_did;
    }

    /// Creates a new message from this agent
    pub fn create_message(&self, message_type: MessageType, ledger_id: String) -> Message {
        let message_type_str = message_type.to_string();

        let mut message = Message::new(
            uuid::Uuid::new_v4().to_string(),
            message_type_str,
            "1.0".to_string(),
            ledger_id,
        );

        // Set the sender DID
        if let Some(from_did) = &self.from_did {
            message.set_from_did(Some(from_did.clone()));
        }

        message
    }

    /// Serializes the message to bytes
    pub fn to_bytes(&self) -> Result<js_sys::Uint8Array, JsValue> {
        match serde_json::to_string(self) {
            Ok(json) => {
                let bytes = json.as_bytes();
                let array = js_sys::Uint8Array::new_with_length(bytes.len() as u32);
                array.copy_from(bytes);
                Ok(array)
            }
            Err(err) => Err(JsValue::from_str(&format!(
                "Failed to serialize message: {}",
                err
            ))),
        }
    }

    /// Deserializes a message from bytes
    #[wasm_bindgen(js_name = from_bytes)]
    pub fn from_bytes(bytes: &[u8]) -> Result<Message, JsValue> {
        match std::str::from_utf8(bytes) {
            Ok(json) => match serde_json::from_str(json) {
                Ok(message) => Ok(message),
                Err(err) => Err(JsValue::from_str(&format!(
                    "Failed to deserialize message: {}",
                    err
                ))),
            },
            Err(err) => Err(JsValue::from_str(&format!(
                "Failed to convert bytes to UTF-8: {}",
                err
            ))),
        }
    }

    /// Deserializes a message from bytes - static version
    #[wasm_bindgen]
    pub fn message_from_bytes(bytes: &[u8]) -> Result<Message, JsValue> {
        match std::str::from_utf8(bytes) {
            Ok(json) => match serde_json::from_str(json) {
                Ok(message) => Ok(message),
                Err(err) => Err(JsValue::from_str(&format!(
                    "Failed to deserialize message: {}",
                    err
                ))),
            },
            Err(err) => Err(JsValue::from_str(&format!(
                "Failed to convert bytes to UTF-8: {}",
                err
            ))),
        }
    }

    /// Creates a message from a JSON string
    #[wasm_bindgen]
    pub fn message_from_json(json: &str) -> Result<Message, JsValue> {
        match serde_json::from_str(json) {
            Ok(message) => Ok(message),
            Err(err) => Err(JsValue::from_str(&format!("Failed to parse JSON: {}", err))),
        }
    }
}

/// TAP Agent implementation
#[wasm_bindgen]
#[derive(Clone)]
pub struct Agent {
    did: String,
    nickname: Option<String>,
    #[allow(dead_code)]
    debug: bool,
    message_handlers: HashMap<String, js_sys::Function>,
    message_subscribers: Vec<js_sys::Function>,
}

#[wasm_bindgen]
impl Agent {
    /// Creates a new agent with the specified configuration
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Self {
        let did = if let Ok(did_prop) = Reflect::get(&config, &JsValue::from_str("did")) {
            if let Some(did_str) = did_prop.as_string() {
                did_str
            } else {
                // Generate a random DID as fallback
                format!(
                    "did:key:z6Mk{}",
                    uuid::Uuid::new_v4().to_string().replace("-", "")
                )
            }
        } else {
            // Generate a random DID as fallback
            format!(
                "did:key:z6Mk{}",
                uuid::Uuid::new_v4().to_string().replace("-", "")
            )
        };

        let nickname = if let Ok(name_prop) = Reflect::get(&config, &JsValue::from_str("nickname"))
        {
            name_prop.as_string()
        } else {
            None
        };

        let debug = if let Ok(debug_prop) = Reflect::get(&config, &JsValue::from_str("debug")) {
            debug_prop.is_truthy()
        } else {
            false
        };

        Agent {
            did,
            nickname,
            debug,
            message_handlers: HashMap::new(),
            message_subscribers: Vec::new(),
        }
    }

    /// Gets the agent's DID
    pub fn get_did(&self) -> String {
        self.did.clone()
    }

    /// Creates a new message from this agent
    pub fn create_message(&self, message_type: MessageType, ledger_id: String) -> Message {
        let message_type_str = message_type.to_string();

        let mut message = Message::new(
            uuid::Uuid::new_v4().to_string(),
            message_type_str,
            "1.0".to_string(),
            ledger_id,
        );

        // Set sender DID
        self.set_from(&mut message);

        message
    }

    /// Sets the from field for a message
    pub fn set_from(&self, message: &mut Message) {
        message.set_from_did(Some(self.did.clone()));
    }

    /// Sets the to field for a message
    pub fn set_to(&self, message: &mut Message, to_did: String) {
        message.set_to_did(Some(to_did));
    }

    /// Gets the agent's nickname
    #[wasm_bindgen(getter)]
    pub fn nickname(&self) -> Option<String> {
        self.nickname.clone()
    }

    /// Registers a message handler function
    pub fn register_message_handler(
        &mut self,
        message_type: MessageType,
        handler: js_sys::Function,
    ) {
        let message_type_str = message_type.to_string();

        self.message_handlers.insert(message_type_str, handler);
    }

    /// Processes a received message
    pub fn process_message(&self, message: JsValue, metadata: JsValue) -> Promise {
        let this = self.clone();
        let message_clone = message.clone();
        let metadata_clone = metadata.clone();

        future_to_promise(async move {
            // Try to parse the message
            let parsed_message: Result<Message, serde_wasm_bindgen::Error> =
                serde_wasm_bindgen::from_value(message_clone.clone());

            if let Ok(msg) = parsed_message {
                // Find the appropriate handler based on message type
                if let Some(handler) = this.message_handlers.get(&msg.message_type) {
                    let args = Array::new_with_length(2);
                    args.set(
                        0,
                        serde_wasm_bindgen::to_value(&msg).unwrap_or(message_clone.clone()),
                    );
                    args.set(1, metadata_clone.clone());

                    let result = handler.apply(&JsValue::NULL, &args)?;
                    return Ok(result);
                }
            } else {
                // Fallback: try to handle it as a raw JavaScript object
                // Check if the message has a message_type property
                if let Ok(message_type) =
                    Reflect::get(&message_clone, &JsValue::from_str("message_type"))
                {
                    if let Some(message_type_str) = message_type.as_string() {
                        // Handle based on message type
                        if let Some(handler) = this.message_handlers.get(&message_type_str) {
                            let args = Array::new_with_length(2);
                            args.set(0, message_clone.clone());
                            args.set(1, metadata_clone.clone());

                            let result = handler.apply(&JsValue::NULL, &args)?;
                            return Ok(result);
                        }
                    }
                }
            }

            // If we can't find a handler, just notify all subscribers
            for subscriber in &this.message_subscribers {
                let args = Array::new_with_length(2);
                args.set(0, message_clone.clone());
                args.set(1, metadata_clone.clone());

                let _result = subscriber.apply(&JsValue::NULL, &args)?;
            }

            Ok(JsValue::UNDEFINED)
        })
    }

    /// Subscribes to all messages processed by this agent
    pub fn subscribe_to_messages(&mut self, callback: js_sys::Function) -> js_sys::Function {
        let subscriber_id = self.message_subscribers.len();
        self.message_subscribers.push(callback);

        // Return an unsubscribe function
        Function::new_no_args(&format!(
            "return () => {{ console.log('Unsubscribing from agent messages, id: {}'); }}",
            subscriber_id
        ))
    }
}

/// Represents a node on the TAP network
#[wasm_bindgen]
#[derive(Clone)]
pub struct TapNode {
    agents: HashMap<String, Agent>,
    #[allow(dead_code)]
    debug: bool,
    message_subscribers: Vec<js_sys::Function>,
}

impl Default for TapNode {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl TapNode {
    /// Creates a new node
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        Self {
            agents: HashMap::new(),
            debug: false,
            message_subscribers: Vec::new(),
        }
    }

    /// Creates a new node with debugging enabled
    pub fn with_debug() -> Self {
        let mut node = Self::new();
        node.debug = true;
        node
    }

    /// Creates a new agent on this node
    pub fn create_agent(&mut self, config: JsValue) -> Agent {
        let agent = Agent::new(config);
        let agent_clone = agent.clone();

        self.agents.insert(agent.get_did(), agent);

        agent_clone
    }

    /// Registers an agent with this node
    pub fn register_agent(&mut self, agent: Agent) -> Result<(), JsValue> {
        if self.agents.contains_key(&agent.did) {
            return Err(JsValue::from_str(&format!(
                "Agent with DID {} is already registered",
                agent.did
            )));
        }

        self.agents.insert(agent.did.clone(), agent);
        Ok(())
    }

    /// Unregisters an agent from this node
    pub fn unregister_agent(&mut self, did: String) -> bool {
        self.agents.remove(&did).is_some()
    }

    /// Gets an agent by DID
    pub fn get_agent(&self, did: String) -> Option<Agent> {
        self.agents.get(&did).cloned()
    }

    /// Gets all registered agents
    pub fn get_agent_dids(&self) -> js_sys::Array {
        let result = js_sys::Array::new();

        for (i, did) in self.agents.keys().enumerate() {
            result.set(i as u32, JsValue::from_str(did));
        }

        result
    }

    /// Sends a message from one agent to another
    pub fn send_message(
        &self,
        from_did: String,
        to_did: String,
        message: Message,
    ) -> Result<String, JsValue> {
        if !self.agents.contains_key(&from_did) {
            return Err(JsValue::from_str(&format!(
                "Agent with DID {} not found",
                from_did
            )));
        }

        if !self.agents.contains_key(&to_did) && self.debug {
            // In debug mode, we allow sending to non-existent agents (for testing)
            // In a real implementation, we would try to resolve the DID and send the message
            console::warn_1(&JsValue::from_str(&format!(
                "Agent with DID {} not found, but sending message anyway (debug mode)",
                to_did
            )));
        }

        // In a real implementation, we would use DIDComm to pack the message
        // For now, just create a JSON string
        let packed_message = serde_json::json!({
            "from": from_did,
            "to": to_did,
            "message": message
        });

        Ok(serde_json::to_string(&packed_message).unwrap())
    }

    /// Process a message to see if it's a Message struct
    fn try_parse_message_struct(
        message_clone: &JsValue,
        debug: bool,
    ) -> Result<Option<Message>, JsValue> {
        // Try to parse the message to see if it's a Message struct
        let message_struct: Result<Message, _> =
            serde_wasm_bindgen::from_value(message_clone.clone());

        match &message_struct {
            Ok(_message) => {
                if debug {
                    console::log_1(&JsValue::from_str("Parsed message as Message struct"));
                }
                // Clone the result rather than moving it
                Ok(message_struct.ok())
            }
            Err(err) => {
                if debug {
                    console::warn_1(&JsValue::from_str(&format!(
                        "Failed to parse message as Message struct: {:?}",
                        err
                    )));
                }
                Ok(None)
            }
        }
    }

    /// Processes a received message
    pub fn process_message(&self, message: JsValue, metadata: JsValue) -> Promise {
        let this = self.clone();
        let message_clone = message.clone();
        let metadata_clone = metadata.clone();

        future_to_promise(async move {
            // First, notify all node subscribers
            for subscriber in &this.message_subscribers {
                let args = Array::new_with_length(2);
                args.set(0, message_clone.clone());
                args.set(1, metadata_clone.clone());

                let _result = subscriber.apply(&JsValue::NULL, &args)?;
            }

            // Try to parse the metadata to find the target agent
            let metadata_map: Result<HashMap<String, String>, _> =
                serde_wasm_bindgen::from_value(metadata_clone.clone());

            if let Ok(metadata_map) = metadata_map {
                // Try to get the to_did from metadata
                if let Some(to_did) = metadata_map.get("toDid") {
                    if let Some(agent) = this.agents.get(to_did) {
                        // Forward to specific agent by returning the agent's Promise
                        return Ok(agent.process_message(message_clone, metadata_clone).into());
                    } else if this.debug {
                        // In debug mode, log that agent wasn't found but continue
                        console::warn_1(&JsValue::from_str(&format!(
                            "Agent with DID {} not found",
                            to_did
                        )));
                    } else {
                        return Err(JsValue::from_str(&format!(
                            "Agent with DID {} not found",
                            to_did
                        )));
                    }
                }
            }

            // If metadata parsing fails or no toDid is specified,
            // try to parse the message to see if it's a Message struct
            let message_struct = Self::try_parse_message_struct(&message_clone, this.debug);

            if let Ok(Some(msg)) = message_struct {
                // If it's a Message and has to_did fields in our implementation
                if let Some(to_did) = msg.to_did {
                    if let Some(agent) = this.agents.get(&to_did) {
                        // Forward to specific agent by returning the agent's Promise
                        return Ok(agent.process_message(message_clone, metadata_clone).into());
                    } else if this.debug {
                        // In debug mode, log that agent wasn't found but continue
                        console::warn_1(&JsValue::from_str(&format!(
                            "Agent with DID {} not found",
                            to_did
                        )));
                    } else {
                        return Err(JsValue::from_str(&format!(
                            "Agent with DID {} not found",
                            to_did
                        )));
                    }
                }
            }

            // Fallback: check if it's a JS object with a 'to' property
            if let Ok(to_prop) = Reflect::get(&message_clone, &JsValue::from_str("to")) {
                if let Some(to_did) = to_prop.as_string() {
                    if let Some(agent) = this.agents.get(&to_did) {
                        // Forward to specific agent by returning the agent's Promise
                        return Ok(agent.process_message(message_clone, metadata_clone).into());
                    } else if this.debug {
                        // In debug mode, log that agent wasn't found but continue
                        console::warn_1(&JsValue::from_str(&format!(
                            "Agent with DID {} not found",
                            to_did
                        )));
                    } else {
                        return Err(JsValue::from_str(&format!(
                            "Agent with DID {} not found",
                            to_did
                        )));
                    }
                }
            }

            // If we can't determine a target agent, broadcast to all agents
            if this.debug {
                console::log_1(&JsValue::from_str(
                    "No specific agent found, broadcasting to all agents",
                ));
            }

            // Process message with all agents
            let mut promises = Vec::new();
            for (_, agent) in &this.agents {
                let promise = agent.process_message(message_clone.clone(), metadata_clone.clone());
                promises.push(promise);
            }

            // In a real implementation, we would await all promises
            // For now, just return undefined as we can't easily combine promises in Rust
            Ok(JsValue::UNDEFINED)
        })
    }

    /// Subscribes to all messages processed by this node
    pub fn subscribe_to_messages(&mut self, callback: js_sys::Function) -> js_sys::Function {
        let subscriber_id = self.message_subscribers.len();
        self.message_subscribers.push(callback);

        // Return an unsubscribe function
        Function::new_no_args(&format!(
            "return () => {{ console.log('Unsubscribing from node messages, id: {}'); }}",
            subscriber_id
        ))
    }
}

/// Creates a new DID key pair
#[wasm_bindgen]
pub fn create_did_key() -> Result<JsValue, JsValue> {
    // In a real implementation, this would generate a key pair and return a DID
    // For now, just return a mock DID
    let mock_did = format!(
        "did:key:z6Mk{}",
        uuid::Uuid::new_v4().to_string().replace("-", "")
    );

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
