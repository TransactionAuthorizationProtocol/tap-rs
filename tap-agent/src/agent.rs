use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize as SerdeSerialize};
use tap_core::message::tap_message_trait::TapMessageBody;

use crate::config::AgentConfig;
use crate::crypto::{MessagePacker, SecurityMode};
use crate::error::{Error, Result};
use crate::policy::PolicyHandler;

/// A trait for sending messages to recipients
#[async_trait]
pub trait MessageSender: Send + Sync + Debug {
    /// Send a packed message to one or more recipients
    async fn send(&self, packed_message: String, recipient_dids: Vec<String>) -> Result<()>;
}

/// Agent trait defining the core functionality of a TAP Agent
#[async_trait]
pub trait Agent: Send + Sync + Debug {
    /// Get the agent's DID
    fn get_agent_did(&self) -> &str;

    /// Send a serialized message to a recipient
    async fn send_serialized_message(
        &self, 
        message: &(dyn erased_serde::Serialize + Sync), 
        to: &str
    ) -> Result<String>;

    /// Receive and unpack a serialized message
    async fn receive_serialized_message(
        &self,
        packed_message: &str,
    ) -> Result<serde_json::Value>;
}

/// Default implementation of a TAP Agent
#[derive(Debug)]
pub struct DefaultAgent {
    /// Agent configuration
    config: AgentConfig,

    /// Message packer for packing and unpacking messages
    message_packer: Arc<dyn MessagePacker>,

    /// Policy handler for evaluating policies
    policy_handler: Arc<dyn PolicyHandler>,
}

impl DefaultAgent {
    /// Create a new DefaultAgent
    pub fn new(
        config: AgentConfig,
        message_packer: Arc<dyn MessagePacker>,
        policy_handler: Arc<dyn PolicyHandler>,
    ) -> Self {
        Self {
            config,
            message_packer,
            policy_handler,
        }
    }
    
    /// Send a TAP message to a recipient
    pub async fn send_message<T: TapMessageBody + SerdeSerialize + Send + Sync>(
        &self,
        message: &T,
        to: &str,
    ) -> Result<String> {
        // Check policy before sending
        self.policy_handler.evaluate_outgoing(message).await?;

        // Use message packer to pack the message
        let security_mode = if let Some(mode) = &self.config.security_mode {
            match mode.as_str() {
                "PLAIN" => SecurityMode::Plain,
                "AUTHCRYPT" => SecurityMode::Authcrypt,
                "ANONCRYPT" => SecurityMode::Anoncrypt,
                _ => SecurityMode::Plain,
            }
        } else {
            SecurityMode::Plain
        };

        let packed = self
            .message_packer
            .pack_message(message, to, Some(self.get_agent_did()), security_mode)
            .await?;

        Ok(packed)
    }

    /// Receive and unpack a TAP message
    pub async fn receive_message<T: TapMessageBody + DeserializeOwned + Send + Sync>(
        &self,
        packed_message: &str,
    ) -> Result<T> {
        // Unpack the message
        let (body_value, _sender) = self.message_packer.unpack_message(packed_message).await?;

        // Deserialize the body
        let message: T = serde_json::from_value(body_value.clone()).map_err(|e| {
            Error::SerializationError(format!("Failed to deserialize message body: {}", e))
        })?;

        // Apply policy
        self.policy_handler.evaluate_incoming(&body_value).await?;

        Ok(message)
    }
}

#[async_trait]
impl Agent for DefaultAgent {
    fn get_agent_did(&self) -> &str {
        &self.config.agent_did
    }

    async fn send_serialized_message(
        &self, 
        message: &(dyn erased_serde::Serialize + Sync), 
        to: &str
    ) -> Result<String> {
        // Use message packer to pack the message
        let security_mode = if let Some(mode) = &self.config.security_mode {
            match mode.as_str() {
                "PLAIN" => SecurityMode::Plain,
                "AUTHCRYPT" => SecurityMode::Authcrypt,
                "ANONCRYPT" => SecurityMode::Anoncrypt,
                _ => SecurityMode::Plain,
            }
        } else {
            SecurityMode::Plain
        };

        let packed = self
            .message_packer
            .pack_message(message, to, Some(self.get_agent_did()), security_mode)
            .await?;

        Ok(packed)
    }

    async fn receive_serialized_message(
        &self,
        packed_message: &str,
    ) -> Result<serde_json::Value> {
        // Unpack the message
        let (body_value, _sender) = self.message_packer.unpack_message(packed_message).await?;

        // Apply policy
        self.policy_handler.evaluate_incoming(&body_value).await?;

        Ok(body_value)
    }
}
