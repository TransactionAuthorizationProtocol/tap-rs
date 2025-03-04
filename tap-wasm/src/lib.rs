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

/// Set up panic hook for better error messages when debugging in browser
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    Ok(())
}

/// The type of TAP Messages
#[derive(Debug, Clone, Copy, PartialEq)]
#[wasm_bindgen]
pub enum MessageType {
    AuthorizationRequest,
    AuthorizationResponse,
    TransferRequest,
    Unknown,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageType::AuthorizationRequest => write!(f, "TAP_AUTHORIZATION_REQUEST"),
            MessageType::AuthorizationResponse => write!(f, "TAP_AUTHORIZATION_RESPONSE"),
            MessageType::TransferRequest => write!(f, "TAP_TRANSFER_REQUEST"),
            MessageType::Unknown => write!(f, "UNKNOWN"),
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
    /// Ledger ID
    ledger_id: String,
    /// Additional fields for authorization request/response
    additional_fields: HashMap<String, serde_json::Value>,
}

#[wasm_bindgen]
impl Message {
    /// Creates a new message with the specified types and fields
    #[wasm_bindgen(constructor)]
    pub fn new(id: String, message_type: String, version: String, ledger_id: String) -> Message {
        // Create a new DIDComm message
        let didcomm_message = DIDCommMessage {
            id: id.clone(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: format!("https://tap.org/protocols/{}/{}", message_type, version),
            body: serde_json::json!({}),
            from: None,
            to: None,
            thid: None,
            pthid: None,
            extra_headers: Default::default(),
            created_time: None,
            expires_time: None,
            from_prior: None,
            attachments: None,
        };

        // Create our TAP message wrapper
        Message {
            didcomm_message,
            message_type,
            version,
            ledger_id,
            additional_fields: HashMap::new(),
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

    /// Gets the ledger ID
    pub fn ledger_id(&self) -> String {
        self.ledger_id.clone()
    }

    /// Sets the ledger ID
    pub fn set_ledger_id(&mut self, ledger_id: String) {
        self.ledger_id = ledger_id;
    }

    /// Sets the authorization request data
    pub fn set_authorization_request(&mut self, private_payload: JsValue) -> Result<(), JsValue> {
        // Convert the JavaScript value to a serde_json::Value
        let value: serde_json::Value =
            serde_wasm_bindgen::from_value(private_payload).map_err(|e| {
                JsValue::from_str(&format!("Failed to parse authorization request: {}", e))
            })?;

        // Store the authorization request in the additional fields
        self.additional_fields
            .insert("authorization_request".to_string(), value);

        // Set the DIDComm message body with the authorization request
        self.didcomm_message.body = serde_json::json!({
            "type": "authorization_request",
            "ledger_id": self.ledger_id,
            "data": self.additional_fields.get("authorization_request")
        });

        Ok(())
    }

    /// Sets the authorization response data
    pub fn set_authorization_response(
        &mut self,
        private_payload: JsValue,
        signed_date: String,
        valid_until: Option<String>,
    ) -> Result<(), JsValue> {
        // Convert the JavaScript value to a serde_json::Value
        let value: serde_json::Value =
            serde_wasm_bindgen::from_value(private_payload).map_err(|e| {
                JsValue::from_str(&format!("Failed to parse authorization response: {}", e))
            })?;

        // Store the authorization response in the additional fields
        self.additional_fields
            .insert("authorization_response".to_string(), value);
        self.additional_fields.insert(
            "signed_date".to_string(),
            serde_json::Value::String(signed_date.clone()),
        );

        let valid_until_value = valid_until.clone().map(serde_json::Value::String);
        if let Some(valid_until_str) = valid_until_value.clone() {
            self.additional_fields
                .insert("valid_until".to_string(), valid_until_str);
        }

        // Set the DIDComm message body with the authorization response
        self.didcomm_message.body = serde_json::json!({
            "type": "authorization_response",
            "ledger_id": self.ledger_id,
            "signed_date": signed_date,
            "valid_until": valid_until,
            "data": self.additional_fields.get("authorization_response")
        });

        Ok(())
    }

    /// Gets the authorization request as a JavaScript object
    pub fn authorization_request(&self) -> JsValue {
        if let Some(value) = self.additional_fields.get("authorization_request") {
            match serde_wasm_bindgen::to_value(value) {
                Ok(js_value) => js_value,
                Err(_) => JsValue::null(),
            }
        } else {
            // Try to extract from DIDComm message body
            if let Some(body) = self.didcomm_message.body.as_object() {
                if let Some(data) = body.get("data") {
                    match serde_wasm_bindgen::to_value(data) {
                        Ok(js_value) => js_value,
                        Err(_) => JsValue::null(),
                    }
                } else {
                    JsValue::null()
                }
            } else {
                JsValue::null()
            }
        }
    }

    /// Gets the authorization response as a JavaScript object
    pub fn authorization_response(&self) -> JsValue {
        if let Some(value) = self.additional_fields.get("authorization_response") {
            match serde_wasm_bindgen::to_value(value) {
                Ok(js_value) => js_value,
                Err(_) => JsValue::null(),
            }
        } else {
            // Try to extract from DIDComm message body
            if let Some(body) = self.didcomm_message.body.as_object() {
                if let Some(data) = body.get("data") {
                    match serde_wasm_bindgen::to_value(data) {
                        Ok(js_value) => js_value,
                        Err(_) => JsValue::null(),
                    }
                } else {
                    JsValue::null()
                }
            } else {
                JsValue::null()
            }
        }
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
    pub fn create_message(&self, message_type: MessageType, ledger_id: String) -> Message {
        let id = format!("msg_{}", generate_uuid_v4());

        Message::new(id, message_type.to_string(), "1.0".to_string(), ledger_id)
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
            "ledger_id": self.ledger_id,
            "additional_fields": self.additional_fields
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

                let ledger_id = match wrapper.get("ledger_id") {
                    Some(ledger_val) => match ledger_val.as_str() {
                        Some(str) => str.to_string(),
                        None => return Err(JsValue::from_str("'ledger_id' field is not a string")),
                    },
                    None => return Err(JsValue::from_str("'ledger_id' field is missing")),
                };

                // Extract additional fields
                let additional_fields = match wrapper.get("additional_fields") {
                    Some(fields_val) => match fields_val.as_object() {
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
                    ledger_id,
                    additional_fields,
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

        // Check if we have access to the signature in additional_fields
        let signature = match self.additional_fields.get("signature") {
            Some(sig) => sig,
            None => return Err(JsValue::from_str("Message has no signature, cannot verify")),
        };

        // In a real implementation, we would use the DIDComm library to validate the signature
        // against the public key of the sender's DID

        // For now, we'll just check if the signature contains the expected pattern
        let sig_str = match signature.get("signature") {
            Some(serde_json::Value::String(s)) => s,
            _ => return Err(JsValue::from_str("Invalid signature format")),
        };

        let expected_pattern = format!("signed_by_{}_with_didcomm", from_did);
        let is_valid = sig_str == &expected_pattern;

        if debug {
            console::log_1(&JsValue::from_str(&format!(
                "Message verification result: {}",
                is_valid
            )));
        }

        Ok(is_valid)
    }
}

/// TAP Agent implementation
#[wasm_bindgen]
#[derive(Clone)]
pub struct Agent {
    did: String,
    nickname: Option<String>,
    debug: bool,
    message_handlers: HashMap<String, js_sys::Function>,
    message_subscribers: Vec<js_sys::Function>,
    secrets_resolver: Arc<BasicSecretResolver>,
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
                format!("did:key:z6Mk{}", uuid::Uuid::new_v4().as_simple())
            }
        } else {
            format!("did:key:z6Mk{}", uuid::Uuid::new_v4().as_simple())
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

        Agent {
            did,
            nickname,
            debug,
            message_handlers: HashMap::new(),
            message_subscribers: Vec::new(),
            secrets_resolver: Arc::new(secrets_resolver),
        }
    }

    /// Gets the agent's DID
    pub fn get_did(&self) -> String {
        self.did.clone()
    }

    /// Creates a new message from this agent
    pub fn create_message(&self, message_type: MessageType, ledger_id: String) -> Message {
        let id = format!("msg_{}", generate_uuid_v4());

        let mut message = Message::new(id, message_type.to_string(), "1.0".to_string(), ledger_id);

        message.set_from_did(Some(self.did.clone()));

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
            message.set_from_did(Some(self.did.clone()));
        }

        // For a complete implementation, we would use the didcomm library's signing capabilities
        // through the secrets_resolver. Here's a placeholder that simulates the signing process.

        // Check if we have a secret for this DID
        let secrets_map = self.secrets_resolver.get_secrets_map();
        if !secrets_map.contains_key(&self.did) {
            return Err(JsValue::from_str(&format!(
                "No secret found for DID: {}",
                self.did
            )));
        }

        // In a real implementation, we would sign the message with the DID's secret
        // For now, we're just adding a signature field to simulate the signing process
        let signature = format!("signed_by_{}_with_didcomm", self.did);
        let value: serde_json::Value = serde_json::json!({
            "signature": signature,
            "signed_time": chrono::Utc::now().to_rfc3339(),
            "key_id": self.did.clone(),
        });

        message
            .additional_fields
            .insert("signature".to_string(), value);

        if self.debug {
            console::log_1(&JsValue::from_str(&format!(
                "Message signed by {}",
                self.did
            )));
        }

        Ok(())
    }

    /// Processes a received message
    pub fn process_message(&self, message: JsValue, metadata: JsValue) -> Promise {
        let agent_clone = self.clone();

        future_to_promise(async move {
            let message_obj = match TapNode::try_parse_message_struct(&message, agent_clone.debug) {
                Ok(Some(msg)) => msg,
                Ok(None) => {
                    let message_str = match js_sys::JSON::stringify(&message) {
                        Ok(str) => match str.as_string() {
                            Some(s) => s,
                            None => {
                                return Err(JsValue::from_str(
                                    "Failed to convert message to string",
                                ))
                            }
                        },
                        Err(_) => {
                            return Err(JsValue::from_str("Failed to stringify message object"))
                        }
                    };

                    match Message::message_from_json(&message_str) {
                        Ok(msg) => msg,
                        Err(e) => return Err(e),
                    }
                }
                Err(e) => return Err(e),
            };

            let message_type = message_obj.message_type();

            let meta_obj = if !metadata.is_null() && !metadata.is_undefined() {
                metadata
            } else {
                js_sys::Object::new().into()
            };

            for subscriber in &agent_clone.message_subscribers {
                let _ = subscriber.call2(&JsValue::NULL, &message.clone(), &meta_obj);
            }

            if let Some(handler) = agent_clone.message_handlers.get(&message_type) {
                // Convert the result of calling the handler to a JsFuture if it's a Promise
                let result = handler.call2(&JsValue::NULL, &message, &meta_obj);
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
                if agent_clone.debug {
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

        let agent_ptr = self as *mut Agent;
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
    agents: HashMap<String, Agent>,
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
                if let (Ok(id), Ok(message_type), Ok(version), Ok(ledger_id)) = (
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
                    js_sys::Reflect::get(message, &JsValue::from_str("ledger_id")).and_then(|v| {
                        v.as_string()
                            .ok_or(JsValue::from_str("ledger_id is not a string"))
                    }),
                ) {
                    let mut msg = Message::new(
                        id.to_string(),
                        message_type.to_string(),
                        version.to_string(),
                        ledger_id.to_string(),
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

                    if let Ok(auth_req) =
                        js_sys::Reflect::get(message, &JsValue::from_str("auth_request"))
                    {
                        if !auth_req.is_null() && !auth_req.is_undefined() {
                            if let Err(e) = msg.set_authorization_request(auth_req) {
                                if debug {
                                    console::log_1(&JsValue::from_str(&format!(
                                        "Error setting authorization request: {}",
                                        e.as_string().unwrap_or_default()
                                    )));
                                }
                            }
                        }
                    }

                    if let Ok(auth_resp) =
                        js_sys::Reflect::get(message, &JsValue::from_str("auth_response"))
                    {
                        if !auth_resp.is_null() && !auth_resp.is_undefined() {
                            let signed_date =
                                js_sys::Reflect::get(&auth_resp, &JsValue::from_str("signed_date"))
                                    .ok()
                                    .and_then(|v| v.as_string())
                                    .unwrap_or_else(|| {
                                        let date = js_sys::Date::new_0();
                                        date.to_iso_string().as_string().unwrap_or_default()
                                    });

                            let valid_until =
                                js_sys::Reflect::get(&auth_resp, &JsValue::from_str("valid_until"))
                                    .ok()
                                    .and_then(|v| v.as_string());

                            if let Err(e) =
                                msg.set_authorization_response(auth_resp, signed_date, valid_until)
                            {
                                if debug {
                                    console::log_1(&JsValue::from_str(&format!(
                                        "Error setting authorization response: {}",
                                        e.as_string().unwrap_or_default()
                                    )));
                                }
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

    /// Adds a new agent to the node
    pub fn add_agent(&mut self, agent: Agent) {
        let did = agent.get_did();
        self.agents.insert(did, agent);
    }

    /// Gets an agent by DID
    pub fn get_agent(&self, did: &str) -> Option<Agent> {
        self.agents.get(did).cloned()
    }

    /// Gets all agents in the node
    pub fn get_agents(&self) -> Array {
        let result = Array::new();
        for (i, agent) in self.agents.values().enumerate() {
            result.set(i as u32, JsValue::from(agent.clone()));
        }
        result
    }

    /// Processes a message through the appropriate agent
    pub fn process_message(&self, message: JsValue, metadata: JsValue) -> Promise {
        let this = self.clone();
        let message_clone = message.clone();
        let metadata_clone = metadata.clone();

        future_to_promise(async move {
            let to_did = if let Ok(Some(msg)) =
                TapNode::try_parse_message_struct(&message_clone, this.debug)
            {
                msg.to_did()
            } else if let Ok(to_prop) = Reflect::get(&message_clone, &JsValue::from_str("to_did")) {
                to_prop.as_string()
            } else {
                None
            };

            if let Some(did) = to_did {
                if let Some(agent) = this.agents.get(&did) {
                    // Convert Promise to a Future that can be awaited
                    let promise = agent.process_message(message_clone, metadata_clone);
                    let future = wasm_bindgen_futures::JsFuture::from(promise);
                    match future.await {
                        Ok(result) => Ok(result),
                        Err(e) => Err(e),
                    }
                } else {
                    if this.debug {
                        console::log_1(&JsValue::from_str(&format!(
                            "No agent found with DID: {}",
                            did
                        )));
                    }
                    Ok(JsValue::FALSE)
                }
            } else {
                for agent in this.agents.values() {
                    // Convert Promise to a Future that can be awaited
                    let promise =
                        agent.process_message(message_clone.clone(), metadata_clone.clone());
                    let future = wasm_bindgen_futures::JsFuture::from(promise);
                    match future.await {
                        Ok(result) => {
                            if result.is_truthy() {
                                return Ok(JsValue::TRUE);
                            }
                        }
                        Err(e) => {
                            if this.debug {
                                console::log_1(&JsValue::from_str(&format!(
                                    "Error processing message: {}",
                                    e.as_string().unwrap_or_default()
                                )));
                            }
                        }
                    }
                }

                Ok(JsValue::FALSE)
            }
        })
    }
}

/// Creates a new DID key pair
#[wasm_bindgen]
pub fn create_did_key() -> Result<JsValue, JsValue> {
    let uuid_str = uuid::Uuid::new_v4().as_simple().to_string();

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
