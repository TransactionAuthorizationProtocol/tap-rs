//! Agent implementation for TAP
//!
//! This module provides the TAP Agent implementation for handling messages.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tap_core::message::{TapMessage, TapMessageType, Validate};

use crate::config::AgentConfig;
use crate::crypto::{DefaultMessagePacker, MessagePacker};
#[cfg(not(target_arch = "wasm32"))]
use crate::did::WebResolver;
use crate::did::{DidResolver, KeyResolver, MultiResolver, PkhResolver};
use crate::error::{Error, Result};
use crate::policy::{DefaultPolicyHandler, PolicyHandler};

/// The primary trait for a TAP Agent
#[async_trait]
pub trait Agent: Send + Sync {
    /// Returns the DID of the agent
    fn did(&self) -> &str;

    /// Returns the name of the agent, if any
    fn name(&self) -> Option<&str>;

    /// Sends a TAP message to a recipient
    async fn send_message(&self, message: &TapMessage, recipient: &str) -> Result<String>;

    /// Receives a packed TAP message
    async fn receive_message(&self, packed_message: &str) -> Result<TapMessage>;

    /// Creates a new TAP message with this agent as the sender
    async fn create_message(
        &self,
        message_type: TapMessageType,
        body: Option<serde_json::Value>,
    ) -> Result<TapMessage>;
}

/// TAP Agent implementation
pub struct TapAgent {
    /// Configuration for the agent
    config: AgentConfig,
    /// Agent's DID
    did: String,
    /// Agent's name
    name: Option<String>,
    /// DID resolver for resolving DIDs
    #[allow(dead_code)]
    resolver: Arc<dyn DidResolver>,
    /// Message packer for packing and unpacking messages
    message_packer: Arc<dyn MessagePacker>,
    /// Policy handler for evaluating policies
    #[allow(dead_code)]
    policy_handler: Arc<dyn PolicyHandler>,
}

impl TapAgent {
    /// Creates a new TapAgent with the specified configuration
    pub fn new(
        config: AgentConfig,
        did: String,
        name: Option<String>,
        resolver: Arc<dyn DidResolver>,
        message_packer: Arc<dyn MessagePacker>,
        policy_handler: Arc<dyn PolicyHandler>,
    ) -> Self {
        Self {
            config,
            did,
            name,
            resolver,
            message_packer,
            policy_handler,
        }
    }

    /// Returns the DID of the agent
    pub fn did(&self) -> &str {
        &self.did
    }

    /// Returns the name of the agent, if any
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the configuration of the agent
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Extracts the body of a TAP message as a specific type
    pub fn extract_body<T: serde::de::DeserializeOwned>(
        &self,
        message: &TapMessage,
    ) -> Result<Option<T>> {
        if let Some(body) = &message.body {
            let parsed = serde_json::from_value(body.clone())
                .map_err(|e| Error::Other(format!("Failed to parse message body: {}", e)))?;
            Ok(Some(parsed))
        } else {
            Ok(None)
        }
    }

    /// Creates a new TapAgent with default components
    pub fn with_defaults(config: AgentConfig, did: String, name: Option<String>) -> Result<Self> {
        // Create a multi-resolver with default resolvers
        let key_resolver = KeyResolver;
        let pkh_resolver = PkhResolver;

        let mut multi_resolver = MultiResolver::new();
        multi_resolver.add_resolver(key_resolver);

        // Only add web resolver if not in WASM environment
        #[cfg(not(target_arch = "wasm32"))]
        {
            let web_resolver = WebResolver;
            multi_resolver.add_resolver(web_resolver);
        }

        multi_resolver.add_resolver(pkh_resolver);

        let resolver = Arc::new(multi_resolver);

        // Create a default message packer
        let message_packer = Arc::new(DefaultMessagePacker::new(did.clone(), resolver.clone()));

        // Create a default policy handler
        let policy_handler = Arc::new(DefaultPolicyHandler::new());

        Ok(Self::new(
            config,
            did,
            name,
            resolver,
            message_packer,
            policy_handler,
        ))
    }
}

#[async_trait]
impl Agent for TapAgent {
    /// Gets the DID of the agent
    fn did(&self) -> &str {
        &self.did
    }

    /// Gets the name of the agent, if set
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sends a TAP message to a recipient
    async fn send_message(&self, message: &TapMessage, recipient: &str) -> Result<String> {
        // Validate the message before sending
        message.validate().map_err(Error::Core)?;

        // Pack the message for the recipient
        let packed = self.message_packer.pack_message(message, recipient).await?;

        Ok(packed)
    }

    /// Receives a packed TAP message
    async fn receive_message(&self, packed_message: &str) -> Result<TapMessage> {
        // Unpack the message
        let message = self.message_packer.unpack_message(packed_message).await?;

        // Check if the sender exists in metadata
        if let Some(from_val) = message.metadata.get("from") {
            if let Some(_sender) = from_val.as_str() {
                // In a real implementation, we would validate the sender's DID
                // For now, just check if the message can be unpacked
                let _unpacked = self.message_packer.unpack_message(packed_message).await?;
                Ok(message)
            } else {
                Err(Error::Validation(
                    "Invalid 'from' field in message".to_string(),
                ))
            }
        } else {
            // No from field, assume it's a valid message for now
            Ok(message)
        }
    }

    /// Creates a new TAP message with this agent as the sender
    async fn create_message(
        &self,
        message_type: TapMessageType,
        body: Option<serde_json::Value>,
    ) -> Result<TapMessage> {
        let mut message = TapMessage::new(message_type);

        if let Some(body_value) = body {
            message.body = Some(body_value);
        }

        // Add sender information to metadata
        let mut metadata = HashMap::new();
        metadata.insert(
            "from".to_string(),
            serde_json::Value::String(self.did.clone()),
        );
        message.metadata = metadata;

        Ok(message)
    }
}
