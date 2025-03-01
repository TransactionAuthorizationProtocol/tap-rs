use wasm_bindgen::prelude::*;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use js_sys::{Promise, Function, Object, Reflect, Array};
use wasm_bindgen_futures::future_to_promise;

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

/// Configuration for a TAP Node
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct NodeConfig {
    pub debug: bool,
    pub network: Option<NetworkConfig>,
}

/// Network configuration for a TAP Node
#[derive(Serialize, Deserialize)]
pub struct NetworkConfig {
    pub peers: Vec<String>,
}

#[wasm_bindgen]
impl NodeConfig {
    /// Creates a new node configuration with default values
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            debug: false,
            network: None,
        }
    }

    /// Sets debug mode
    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    /// Sets the network configuration
    pub fn set_network(&mut self, peers: Vec<String>) {
        self.network = Some(NetworkConfig { peers });
    }
}

/// Configuration for a TAP Agent
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct AgentConfig {
    pub did: String,
    pub nickname: Option<String>,
    pub debug: bool,
}

#[wasm_bindgen]
impl AgentConfig {
    /// Creates a new agent configuration
    #[wasm_bindgen(constructor)]
    pub fn new(did: String) -> Self {
        Self {
            did,
            nickname: None,
            debug: false,
        }
    }

    /// Sets the agent's nickname
    pub fn set_nickname(&mut self, nickname: String) {
        self.nickname = Some(nickname);
    }

    /// Sets debug mode
    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }
}

/// TAP Message structure
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub message_type: String,
    pub version: String,
    pub ledger_id: String,
    pub authorization_request: Option<AuthorizationRequest>,
    pub authorization_response: Option<AuthorizationResponse>,
}

/// Authorization Request structure
#[derive(Serialize, Deserialize)]
pub struct AuthorizationRequest {
    pub transaction_hash: String,
    pub sender: String,
    pub receiver: String,
    pub amount: String,
}

/// Authorization Response structure
#[derive(Serialize, Deserialize)]
pub struct AuthorizationResponse {
    pub transaction_hash: String,
    pub authorization_result: bool,
    pub reason: Option<String>,
}

#[wasm_bindgen]
impl Message {
    /// Creates a new message
    #[wasm_bindgen(constructor)]
    pub fn new(message_type: MessageType, ledger_id: String) -> Self {
        let id = "msg_".to_string() + &uuid::Uuid::new_v4().to_string();
        let message_type_str = match message_type {
            MessageType::AuthorizationRequest => "TAP_AUTHORIZATION_REQUEST".to_string(),
            MessageType::AuthorizationResponse => "TAP_AUTHORIZATION_RESPONSE".to_string(),
            MessageType::Ping => "TAP_PING".to_string(),
        };

        Self {
            id,
            message_type: message_type_str,
            version: "1.0".to_string(),
            ledger_id,
            authorization_request: None,
            authorization_response: None,
        }
    }

    /// Sets the authorization request data
    pub fn set_authorization_request(&mut self, 
        transaction_hash: String, 
        sender: String, 
        receiver: String, 
        amount: String
    ) {
        self.authorization_request = Some(AuthorizationRequest {
            transaction_hash,
            sender,
            receiver,
            amount,
        });
    }

    /// Sets the authorization response data
    pub fn set_authorization_response(&mut self, 
        transaction_hash: String, 
        authorization_result: bool, 
        reason: Option<String>
    ) {
        self.authorization_response = Some(AuthorizationResponse {
            transaction_hash,
            authorization_result,
            reason,
        });
    }
}

/// TAP Agent implementation
#[wasm_bindgen]
pub struct Agent {
    did: String,
    nickname: Option<String>,
    debug: bool,
    message_handlers: HashMap<String, js_sys::Function>,
    message_subscribers: Vec<js_sys::Function>,
}

#[wasm_bindgen]
impl Agent {
    /// Creates a new agent with the specified configuration
    #[wasm_bindgen(constructor)]
    pub fn new(config: AgentConfig) -> Self {
        Self {
            did: config.did,
            nickname: config.nickname,
            debug: config.debug,
            message_handlers: HashMap::new(),
            message_subscribers: Vec::new(),
        }
    }

    /// Gets the agent's DID
    #[wasm_bindgen(getter)]
    pub fn did(&self) -> String {
        self.did.clone()
    }

    /// Gets the agent's nickname
    #[wasm_bindgen(getter)]
    pub fn nickname(&self) -> Option<String> {
        self.nickname.clone()
    }

    /// Registers a message handler function
    pub fn register_message_handler(&mut self, message_type: MessageType, handler: js_sys::Function) {
        let message_type_str = match message_type {
            MessageType::AuthorizationRequest => "TAP_AUTHORIZATION_REQUEST".to_string(),
            MessageType::AuthorizationResponse => "TAP_AUTHORIZATION_RESPONSE".to_string(),
            MessageType::Ping => "TAP_PING".to_string(),
        };

        self.message_handlers.insert(message_type_str, handler);
    }

    /// Processes a received message
    pub fn process_message(&self, message: JsValue, metadata: JsValue) -> Promise {
        let this = self.clone();
        let message_clone = message.clone();
        let metadata_clone = metadata.clone();

        future_to_promise(async move {
            // We would normally parse the message and call the appropriate handler
            // For now, let's just call all subscribers
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
        let unsubscribe = move || {
            // This would be handled by the TapNode which has a reference to the agent
            // For now, just return a placeholder
            JsValue::UNDEFINED
        };
        
        Function::new_no_args(&format!("return () => {{ console.log('Unsubscribing from agent messages, id: {}'); }}", subscriber_id))
    }

    /// Deep clones this agent (needed for async operations)
    fn clone(&self) -> Self {
        Self {
            did: self.did.clone(),
            nickname: self.nickname.clone(),
            debug: self.debug,
            message_handlers: self.message_handlers.clone(),
            message_subscribers: self.message_subscribers.clone(),
        }
    }
}

/// TAP Node implementation
#[wasm_bindgen]
pub struct TapNode {
    debug: bool,
    agents: HashMap<String, Agent>,
    message_subscribers: Vec<js_sys::Function>,
}

#[wasm_bindgen]
impl TapNode {
    /// Creates a new TAP node with the specified configuration
    #[wasm_bindgen(constructor)]
    pub fn new(config: Option<NodeConfig>) -> Self {
        let config = config.unwrap_or_else(|| NodeConfig::new());
        
        Self {
            debug: config.debug,
            agents: HashMap::new(),
            message_subscribers: Vec::new(),
        }
    }

    /// Registers an agent with this node
    pub fn register_agent(&mut self, agent: Agent) -> Result<(), JsValue> {
        if self.agents.contains_key(&agent.did) {
            return Err(JsValue::from_str(&format!("Agent with DID {} is already registered", agent.did)));
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
    pub fn send_message(&self, from_did: String, to_did: String, message: Message) -> Result<String, JsValue> {
        if !self.agents.contains_key(&from_did) {
            return Err(JsValue::from_str(&format!("Agent with DID {} not found", from_did)));
        }
        
        if !self.agents.contains_key(&to_did) && self.debug {
            // In debug mode, we allow sending to non-existent agents (for testing)
            // In a real implementation, we would try to resolve the DID and send the message
            web_sys::console::warn_1(&JsValue::from_str(
                &format!("Agent with DID {} not found, but sending message anyway (debug mode)", to_did)
            ));
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

    /// Processes a received message
    pub fn process_message(&self, message: JsValue, metadata: JsValue) -> Promise {
        let this = self.clone();
        let message_clone = message.clone();
        let metadata_clone = metadata.clone();

        future_to_promise(async move {
            // Try to get the metadata object
            let metadata_obj: HashMap<String, String> = match serde_wasm_bindgen::from_value(metadata_clone.clone()) {
                Ok(metadata) => metadata,
                Err(err) => {
                    return Err(JsValue::from_str(&format!("Failed to parse metadata: {}", err)));
                }
            };
            
            // Get the target agent
            if let Some(to_did) = metadata_obj.get("toDid") {
                if let Some(agent) = this.agents.get(to_did) {
                    // Forward the message to the agent
                    return agent.process_message(message, metadata);
                } else {
                    return Err(JsValue::from_str(&format!("Agent with DID {} not found", to_did)));
                }
            } else {
                return Err(JsValue::from_str("Missing 'toDid' in metadata"));
            }
        })
    }

    /// Subscribes to all messages processed by this node
    pub fn subscribe_to_messages(&mut self, callback: js_sys::Function) -> js_sys::Function {
        let subscriber_id = self.message_subscribers.len();
        self.message_subscribers.push(callback);
        
        // Return an unsubscribe function
        Function::new_no_args(&format!("return () => {{ console.log('Unsubscribing from node messages, id: {}'); }}", subscriber_id))
    }

    /// Deep clones this node (needed for async operations)
    fn clone(&self) -> Self {
        Self {
            debug: self.debug,
            agents: self.agents.clone(),
            message_subscribers: self.message_subscribers.clone(),
        }
    }
}

/// Creates a new DID key pair
#[wasm_bindgen]
pub fn create_did_key() -> Result<JsValue, JsValue> {
    // In a real implementation, this would generate a key pair and return a DID
    // For now, just return a mock DID
    let mock_did = format!("did:key:z6Mk{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
    
    let result = Object::new();
    Reflect::set(&result, &JsValue::from_str("did"), &JsValue::from_str(&mock_did))?;
    
    Ok(result.into())
}
