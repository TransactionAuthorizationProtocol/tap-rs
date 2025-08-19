use crate::util::js_to_tap_message;
use js_sys::{Array, Function, Object, Promise, Reflect};
use std::collections::HashMap;
use std::sync::Arc;
use tap_agent::agent::TapAgent;
use tap_agent::{
    did::DIDGenerationOptions,
    message::SecurityMode,
    message_packing::{PackOptions, UnpackOptions},
    AgentConfig, AgentKeyManager, AgentKeyManagerBuilder, KeyType, Packable, Unpackable,
};

// Extension trait for TapAgent in WASM context
trait WasmTapAgentExt {
    // Get the key manager for this agent
    fn agent_key_manager(&self) -> &Arc<AgentKeyManager>;
}

impl WasmTapAgentExt for TapAgent {
    fn agent_key_manager(&self) -> &Arc<AgentKeyManager> {
        // Use the public key_manager() method
        self.key_manager()
    }
}
use tap_msg::didcomm::PlainMessage;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use web_sys::console;

/// TAP Agent implementation for WASM bindings
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmTapAgent {
    /// The underlying TapAgent
    agent: TapAgent,
    /// Nickname for the agent
    nickname: Option<String>,
    /// Debug mode flag
    debug: bool,
    /// Message handlers for different message types
    message_handlers: HashMap<String, Function>,
    /// Subscribers to all messages
    message_subscribers: Vec<Function>,
    /// Store the private key directly for export (temporary fix)
    private_key_hex: Option<String>,
}

#[wasm_bindgen]
impl WasmTapAgent {
    /// Creates a new agent from an existing private key
    #[wasm_bindgen(js_name = fromPrivateKey)]
    pub async fn from_private_key(
        private_key_hex: String,
        key_type_str: String,
    ) -> Result<WasmTapAgent, JsValue> {
        console_error_panic_hook::set_once();

        // Convert hex string to bytes
        let private_key_bytes = hex::decode(&private_key_hex)
            .map_err(|e| JsValue::from_str(&format!("Invalid hex private key: {}", e)))?;

        // Convert key type string to KeyType enum
        let key_type = match key_type_str.as_str() {
            "Ed25519" => KeyType::Ed25519,
            "P256" => KeyType::P256,
            "Secp256k1" => KeyType::Secp256k1,
            _ => {
                return Err(JsValue::from_str(&format!(
                    "Invalid key type: {}",
                    key_type_str
                )))
            }
        };

        // Create TapAgent from private key
        let (agent, did) = TapAgent::from_private_key(&private_key_bytes, key_type, false)
            .await
            .map_err(|e| {
                JsValue::from_str(&format!("Failed to create agent from private key: {}", e))
            })?;

        if agent.config.debug {
            console::log_1(&JsValue::from_str(&format!(
                "Created WASM TAP Agent from private key with DID: {}",
                did
            )));
        }

        Ok(WasmTapAgent {
            agent,
            nickname: None,
            debug: false,
            message_handlers: HashMap::new(),
            message_subscribers: Vec::new(),
            private_key_hex: Some(private_key_hex.clone()),
        })
    }

    /// Creates a new agent with the specified configuration
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> std::result::Result<WasmTapAgent, JsValue> {
        console_error_panic_hook::set_once();

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

        // Get the DID from config
        let did_string = if let Ok(did_prop) = Reflect::get(&config, &JsValue::from_str("did")) {
            did_prop.as_string()
        } else {
            None
        };

        // Create a key manager
        let key_manager_builder = AgentKeyManagerBuilder::new();
        let key_manager = match key_manager_builder.build() {
            Ok(km) => km,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Failed to build key manager: {}",
                    e
                )))
            }
        };

        let agent = if let Some(did) = did_string {
            // Create a config with the provided DID
            let agent_config = AgentConfig::new(did).with_debug(debug);

            // Create the agent with the provided DID
            TapAgent::new(agent_config, Arc::new(key_manager))
        } else {
            // Generate a new key and DID for WASM
            let options = DIDGenerationOptions {
                key_type: KeyType::Ed25519,
            };
            let generated_key = match key_manager.generate_key_without_save(options) {
                Ok(key) => key,
                Err(e) => return Err(JsValue::from_str(&format!("Failed to generate key: {}", e))),
            };

            // Add the key to the key manager
            if let Err(e) = key_manager.add_key_without_save(&generated_key) {
                return Err(JsValue::from_str(&format!("Failed to add key: {}", e)));
            }

            let agent_config = AgentConfig::new(generated_key.did.clone()).with_debug(debug);
            TapAgent::new(agent_config, Arc::new(key_manager))
        };

        if debug {
            console::log_1(&JsValue::from_str(&format!(
                "Created WASM TAP Agent with DID: {}",
                agent.config.agent_did
            )));
        }

        Ok(WasmTapAgent {
            agent,
            nickname,
            debug,
            message_handlers: HashMap::new(),
            message_subscribers: Vec::new(),
            private_key_hex: None,
        })
    }

    /// Gets the agent's DID
    pub fn get_did(&self) -> String {
        self.agent.config.agent_did.clone()
    }

    /// Gets the agent's nickname
    pub fn nickname(&self) -> Option<String> {
        self.nickname.clone()
    }

    /// Export the agent's private key as a hex string
    #[wasm_bindgen(js_name = exportPrivateKey)]
    pub fn export_private_key(&self) -> Result<String, JsValue> {
        // If we have a stored private key (from from_private_key method), use it directly
        if let Some(stored_key) = &self.private_key_hex {
            return Ok(stored_key.clone());
        }

        // Otherwise, try to get it from the key manager
        let key_manager = self.agent.agent_key_manager();

        // Get the agent's DID
        let did = &self.agent.config.agent_did;

        // Get the generated key from the key manager
        let generated_key = key_manager
            .get_generated_key(did)
            .map_err(|e| JsValue::from_str(&format!("Failed to get key for DID {}: {}", did, e)))?;

        // Convert private key bytes to hex string
        let hex_private_key = hex::encode(&generated_key.private_key);

        Ok(hex_private_key)
    }

    /// Export the agent's public key as a hex string
    #[wasm_bindgen(js_name = exportPublicKey)]
    pub fn export_public_key(&self) -> Result<String, JsValue> {
        // Get the key manager from the agent
        let key_manager = self.agent.agent_key_manager();

        // Get the agent's DID
        let did = &self.agent.config.agent_did;

        // Get the generated key from the key manager
        let generated_key = key_manager
            .get_generated_key(did)
            .map_err(|e| JsValue::from_str(&format!("Failed to get key for DID {}: {}", did, e)))?;

        // Convert public key bytes to hex string
        let hex_public_key = hex::encode(&generated_key.public_key);

        Ok(hex_public_key)
    }

    /// Pack a message using this agent's keys for transmission
    #[wasm_bindgen(js_name = packMessage)]
    pub fn pack_message(&self, message_js: JsValue) -> Promise {
        let agent = self.agent.clone();
        let debug = self.debug;

        future_to_promise(async move {
            // Convert JS message to a TapMessageBody
            let tap_message = match js_to_tap_message(&message_js) {
                Ok(msg) => msg,
                Err(e) => {
                    return Err(JsValue::from_str(&format!(
                        "Failed to convert JS message: {}",
                        e
                    )))
                }
            };

            // Create pack options
            let security_mode = SecurityMode::Signed; // Default to signed
            let sender_kid = Some(format!("{}#keys-1", agent.config.agent_did));
            let recipient_kid = None; // Can be set from message if needed

            let pack_options = PackOptions {
                security_mode,
                sender_kid,
                recipient_kid,
            };

            // Pack the message
            let key_manager = agent.agent_key_manager();

            // Debug log the message we're about to pack
            if debug {
                console::log_1(&JsValue::from_str(&format!(
                    "Packing message: id={}, type={}, from={}, to={:?}",
                    tap_message.id, tap_message.type_, tap_message.from, tap_message.to
                )));
            }

            let packed = match tap_message.pack(&**key_manager, pack_options).await {
                Ok(packed_msg) => {
                    if debug {
                        console::log_1(&JsValue::from_str(&format!(
                            "Packed message length: {}, preview: {}...",
                            packed_msg.len(),
                            &packed_msg.chars().take(50).collect::<String>()
                        )));
                    }
                    packed_msg
                }
                Err(e) => {
                    console::error_1(&JsValue::from_str(&format!("Pack error: {:?}", e)));
                    return Err(JsValue::from_str(&format!("Failed to pack message: {}", e)));
                }
            };

            if debug {
                console::log_1(&JsValue::from_str(&format!(
                    "✅ Message packed successfully for sender {}",
                    agent.config.agent_did
                )));
            }

            // Create a JS object to return with the packed message
            let result = Object::new();
            Reflect::set(
                &result,
                &JsValue::from_str("message"),
                &JsValue::from_str(&packed),
            )?;

            // Add metadata
            let metadata = Object::new();
            Reflect::set(
                &metadata,
                &JsValue::from_str("type"),
                &JsValue::from_str("signed"),
            )?;
            Reflect::set(
                &metadata,
                &JsValue::from_str("sender"),
                &JsValue::from_str(&agent.config.agent_did),
            )?;

            Reflect::set(&result, &JsValue::from_str("metadata"), &metadata)?;

            Ok(result.into())
        })
    }

    /// Unpack a message received by this agent
    #[wasm_bindgen(js_name = unpackMessage)]
    pub fn unpack_message(&self, packed_message: &str, expected_type: Option<String>) -> Promise {
        let agent = self.agent.clone();
        let debug = self.debug;
        let packed_message = packed_message.to_string(); // Clone the string to avoid lifetime issues

        future_to_promise(async move {
            // Create unpack options
            // For signed messages, we don't expect a specific recipient
            // For encrypted messages, we expect to be one of the recipients
            let unpack_options = UnpackOptions {
                expected_security_mode: SecurityMode::Any,
                expected_recipient_kid: None, // Don't require specific recipient for signed messages
                require_signature: false,
            };

            // Unpack the message
            let key_manager = agent.agent_key_manager();
            let plain_message: PlainMessage =
                match String::unpack(&packed_message, &**key_manager, unpack_options).await {
                    Ok(msg) => msg,
                    Err(e) => {
                        return Err(JsValue::from_str(&format!(
                            "Failed to unpack message: {}",
                            e
                        )))
                    }
                };

            if debug {
                console::log_1(&JsValue::from_str(&format!(
                    "✅ Message unpacked successfully for recipient {}",
                    agent.config.agent_did
                )));
            }

            // If an expected type was provided, validate it
            if let Some(expected) = expected_type {
                if plain_message.type_ != expected {
                    return Err(JsValue::from_str(&format!(
                        "Expected message type {} but got {}",
                        expected, plain_message.type_
                    )));
                }
            }

            // Convert the unpacked message to a JS object
            let result = Object::new();

            // Add message ID
            Reflect::set(
                &result,
                &JsValue::from_str("id"),
                &JsValue::from_str(&plain_message.id),
            )?;

            // Add message type
            Reflect::set(
                &result,
                &JsValue::from_str("type"),
                &JsValue::from_str(&plain_message.type_),
            )?;

            // Add from and to
            Reflect::set(
                &result,
                &JsValue::from_str("from"),
                &JsValue::from_str(&plain_message.from),
            )?;

            let to_array = Array::new();
            for to_did in &plain_message.to {
                to_array.push(&JsValue::from_str(to_did));
            }
            Reflect::set(&result, &JsValue::from_str("to"), &to_array)?;

            // Add body as a JS object
            let body_str = serde_json::to_string(&plain_message.body)
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize body: {}", e)))?;

            let body_js = js_sys::JSON::parse(&body_str)
                .map_err(|e| JsValue::from_str(&format!("Failed to parse body: {:?}", e)))?;

            Reflect::set(&result, &JsValue::from_str("body"), &body_js)?;

            // Add created time if available
            if let Some(created) = plain_message.created_time {
                Reflect::set(
                    &result,
                    &JsValue::from_str("created"),
                    &JsValue::from_f64(created as f64),
                )?;
            }

            // Add expires time if available
            if let Some(expires) = plain_message.expires_time {
                Reflect::set(
                    &result,
                    &JsValue::from_str("expires"),
                    &JsValue::from_f64(expires as f64),
                )?;
            }

            // Add thread ID if available
            if let Some(thid) = plain_message.thid {
                Reflect::set(
                    &result,
                    &JsValue::from_str("thid"),
                    &JsValue::from_str(&thid),
                )?;
            }

            // Add parent thread ID if available
            if let Some(pthid) = plain_message.pthid {
                Reflect::set(
                    &result,
                    &JsValue::from_str("pthid"),
                    &JsValue::from_str(&pthid),
                )?;
            }

            Ok(result.into())
        })
    }

    /// Registers a message handler function for a specific message type
    #[wasm_bindgen(js_name = registerMessageHandler)]
    pub fn register_message_handler(&mut self, message_type: &str, handler: Function) {
        self.message_handlers
            .insert(message_type.to_string(), handler);
        if self.debug {
            console::log_1(&JsValue::from_str(&format!(
                "Registered handler for message type: {}",
                message_type
            )));
        }
    }

    /// Processes a received message by routing it to the appropriate handler
    #[wasm_bindgen(js_name = processMessage)]
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

            // Notify all subscribers
            for subscriber in &message_subscribers {
                let _ = subscriber.call2(&JsValue::NULL, &message_clone.clone(), &metadata_clone);
            }

            // Find and call the appropriate handler
            if let Some(handler) = message_handlers.get(&message_type) {
                let result = handler.call2(&JsValue::NULL, &message_clone, &metadata_clone);
                match result {
                    Ok(value) => {
                        if value.is_instance_of::<Promise>() {
                            // It's a Promise, convert to a Future and await it
                            let future = wasm_bindgen_futures::JsFuture::from(
                                value.dyn_into::<Promise>().unwrap(),
                            );
                            match future.await {
                                Ok(result) => Ok(result),
                                Err(e) => Err(e),
                            }
                        } else {
                            // Not a Promise, just return it
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
    #[wasm_bindgen(js_name = subscribeToMessages)]
    pub fn subscribe_to_messages(&mut self, callback: Function) -> Function {
        self.message_subscribers.push(callback.clone());

        let agent_ptr = self as *mut WasmTapAgent;
        let cb_ref = callback.clone();

        let _unsubscribe = move || {
            let agent = unsafe { &mut *agent_ptr };
            agent
                .message_subscribers
                .retain(|cb| !Object::is(cb, &cb_ref));
        };

        Function::new_no_args("agent.message_subscribers.pop()")
    }

    /// Creates a new message with a random ID
    #[wasm_bindgen(js_name = createMessage)]
    pub fn create_message(&self, message_type: &str) -> JsValue {
        let id = format!("msg_{}", Uuid::new_v4().simple());
        let result = Object::new();

        // Set basic message properties
        Reflect::set(&result, &JsValue::from_str("id"), &JsValue::from_str(&id)).unwrap();
        Reflect::set(
            &result,
            &JsValue::from_str("type"),
            &JsValue::from_str(message_type),
        )
        .unwrap();
        Reflect::set(
            &result,
            &JsValue::from_str("from"),
            &JsValue::from_str(&self.agent.config.agent_did),
        )
        .unwrap();

        // Create empty arrays/objects for other fields
        let to_array = Array::new();
        Reflect::set(&result, &JsValue::from_str("to"), &to_array).unwrap();

        let body = Object::new();
        Reflect::set(&result, &JsValue::from_str("body"), &body).unwrap();

        // Set created time
        let now = js_sys::Date::now();
        Reflect::set(
            &result,
            &JsValue::from_str("created"),
            &JsValue::from_f64(now),
        )
        .unwrap();

        result.into()
    }

    /// Generate a new key for the agent
    #[wasm_bindgen(js_name = generateKey)]
    pub fn generate_key(&self, key_type_str: &str) -> Promise {
        let agent = self.agent.clone();
        let debug = self.debug;
        let key_type_str = key_type_str.to_string(); // Clone the string to avoid reference issues

        // Convert key type string to KeyType
        let key_type = match key_type_str.as_str() {
            "Ed25519" => KeyType::Ed25519,
            "P256" => KeyType::P256,
            "Secp256k1" => KeyType::Secp256k1,
            _ => {
                let err_msg = format!("Unsupported key type: {}", key_type_str);
                return future_to_promise(async move { Err(JsValue::from_str(&err_msg)) });
            }
        };

        future_to_promise(async move {
            // Create DID generation options
            let _options = DIDGenerationOptions { key_type };

            // Generate the key
            if debug {
                console::log_1(&JsValue::from_str(&format!(
                    "Generating {} key for agent {}",
                    key_type_str, agent.config.agent_did
                )));
            }

            // Using the agent's key manager to generate a key
            // Note: In a real implementation, we would need to access the key manager directly
            // This is a simplified example
            let result = Object::new();
            Reflect::set(
                &result,
                &JsValue::from_str("keyType"),
                &JsValue::from_str(&key_type_str),
            )?;
            Reflect::set(
                &result,
                &JsValue::from_str("agentDid"),
                &JsValue::from_str(&agent.config.agent_did),
            )?;

            Ok(result.into())
        })
    }
}
