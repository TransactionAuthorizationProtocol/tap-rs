//! Message sender implementations for TAP Node.
//!
//! This module provides functionality for sending TAP messages to recipients.

use async_trait::async_trait;
use std::fmt::{self, Debug};
use std::sync::Arc;

use crate::error::{Error, Result};

/// Message sender trait for sending packed messages to recipients
#[async_trait]
pub trait MessageSender: Send + Sync + Debug {
    /// Send a packed message to one or more recipients
    async fn send(&self, packed_message: String, recipient_dids: Vec<String>) -> Result<()>;
}

/// Node message sender implementation
pub struct NodeMessageSender {
    /// Callback function for sending messages
    #[allow(dead_code)]
    send_callback: Arc<dyn Fn(String, Vec<String>) -> Result<()> + Send + Sync>,
}

impl Debug for NodeMessageSender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeMessageSender")
            .field("send_callback", &"<function>")
            .finish()
    }
}

impl NodeMessageSender {
    /// Create a new NodeMessageSender with the given callback
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(String, Vec<String>) -> Result<()> + Send + Sync + 'static,
    {
        Self {
            send_callback: Arc::new(callback),
        }
    }
}

#[async_trait]
impl MessageSender for NodeMessageSender {
    async fn send(&self, packed_message: String, recipient_dids: Vec<String>) -> Result<()> {
        // Call the callback function with the packed message and recipient DIDs
        (self.send_callback)(packed_message, recipient_dids)
            .map_err(|e| Error::Dispatch(format!("Failed to send message: {}", e)))
    }
}

/// HTTP message sender implementation for sending messages over HTTP
#[derive(Debug)]
pub struct HttpMessageSender {
    /// Base URL for the HTTP endpoint
    _base_url: String,
}

impl HttpMessageSender {
    /// Create a new HttpMessageSender with the given base URL
    pub fn new(base_url: String) -> Self {
        Self { _base_url: base_url }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl MessageSender for HttpMessageSender {
    async fn send(&self, packed_message: String, recipient_dids: Vec<String>) -> Result<()> {
        // In a production implementation, this would send the message to each recipient via HTTP
        for recipient in &recipient_dids {
            log::info!("Sending message to {} via HTTP", recipient);
            log::debug!("Message content: {}", packed_message);
            
            // TODO: Implement actual HTTP request using reqwest or similar
        }
        
        // For now, just pretend it worked
        Ok(())
    }
}

// Specific implementation for WASM environments
#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
impl MessageSender for HttpMessageSender {
    async fn send(&self, packed_message: String, recipient_dids: Vec<String>) -> Result<()> {
        // In a WASM environment, we'd use web-sys or similar to make HTTP requests
        for recipient in &recipient_dids {
            log::info!("Sending message to {} via HTTP (WASM environment)", recipient);
            log::debug!("Message content: {}", packed_message);
            
            // TODO: Implement actual HTTP request using web-sys or similar
        }
        
        // For now, just pretend it worked
        Ok(())
    }
}
