//! Message sender implementations for TAP Node.
//!
//! This module provides functionality for sending TAP messages to recipients
//! using various transport mechanisms.
//!
//! # Overview
//!
//! The message sender system in TAP Node implements different strategies for
//! delivering DIDComm messages to their recipients. The primary implementations are:
//!
//! - `NodeMessageSender`: A flexible sender that uses a callback function for delivery
//! - `HttpMessageSender`: Sends messages over HTTP with robust error handling and retries
//!
//! # Cross-platform Support
//!
//! The `HttpMessageSender` implementation supports multiple platforms:
//!
//! - Native environments (using reqwest)
//! - WASM environments (using web-sys)
//! - Fallback implementations for environments without these libraries
//!
//! # Usage with TapNode
//!
//! ```no_run
//! # use std::sync::Arc;
//! # use tap_node::{NodeConfig, TapNode, HttpMessageSender, MessageSender};
//! # use tap_msg::didcomm::Message;
//! # use serde_json::json;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a TapNode
//! let node = TapNode::new(NodeConfig::default());
//! 
//! // Create a sample message
//! let message = Message {
//!     id: "test-123".to_string(),
//!     typ: "https://tap.rsvp/schema/1.0#transfer".to_string(),
//!     type_: "https://tap.rsvp/schema/1.0#transfer".to_string(),
//!     body: json!({"amount": "100.00"}),
//!     from: Some("did:example:sender".to_string()),
//!     to: Some(vec!["did:example:recipient".to_string()]),
//!     created_time: Some(chrono::Utc::now().timestamp() as u64),
//!     expires_time: None,
//!     thid: None,
//!     pthid: None,
//!     attachments: None,
//!     from_prior: None,
//!     extra_headers: Default::default(),
//! };
//!
//! // Pack a message using the node's send_message method
//! let packed_message = node.send_message(
//!     "did:example:sender",
//!     "did:example:recipient", 
//!     message
//! ).await?;
//!
//! // Create an HTTP sender for external dispatch
//! let sender = HttpMessageSender::new("https://recipient-endpoint.example.com".to_string());
//!
//! // Send the packed message to the recipient node
//! sender.send(
//!     packed_message,
//!     vec!["did:example:recipient".to_string()]
//! ).await?;
//! # Ok(())
//! # }
//! ```

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
///
/// This sender allows TAP nodes to send messages to other TAP nodes over HTTP,
/// handling the necessary encoding, content types, and error handling.
///
/// # HTTP Endpoint Structure
///
/// Messages are sent to endpoints derived from the recipient's DID, using a
/// configurable base URL.
///
/// # Error Handling
///
/// The sender includes built-in error handling for common HTTP issues:
/// - Connection timeouts
/// - Request failures
/// - Invalid responses
/// - Rate limiting
///
/// # Configuration
///
/// The sender can be configured with:
/// - Base URL for the HTTP endpoint
/// - Timeout settings
/// - Retry policies
#[derive(Debug)]
pub struct HttpMessageSender {
    /// Base URL for the HTTP endpoint
    base_url: String,
    /// HTTP client (only in native environments)
    #[cfg(feature = "reqwest")]
    client: reqwest::Client,
    /// Timeout for HTTP requests in milliseconds
    #[allow(dead_code)]
    timeout_ms: u64,
    /// Maximum number of retries
    max_retries: u32,
}

impl HttpMessageSender {
    /// Create a new HttpMessageSender with the given base URL
    pub fn new(base_url: String) -> Self {
        Self::with_options(base_url, 30000, 3) // Default 30 second timeout, 3 retries
    }

    /// Create a new HttpMessageSender with custom options
    pub fn with_options(base_url: String, timeout_ms: u64, max_retries: u32) -> Self {
        #[cfg(feature = "reqwest")]
        {
            // Create a reqwest client with appropriate settings
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_millis(timeout_ms))
                .user_agent("TAP-Node/0.1")
                .build()
                .unwrap_or_default();

            Self {
                base_url,
                client,
                timeout_ms,
                max_retries,
            }
        }

        #[cfg(not(feature = "reqwest"))]
        {
            Self {
                base_url,
                timeout_ms,
                max_retries,
            }
        }
    }

    /// Helper to construct the endpoint URL for a recipient
    fn get_endpoint_url(&self, recipient_did: &str) -> String {
        // In a production implementation, this would map DID to HTTP endpoint
        // This could involve DID resolution or a lookup table

        // For now, we'll use a simple convention:
        // Append the DID to the base URL, with proper URL encoding
        let encoded_did = self.url_encode(recipient_did);
        format!("{}/api/messages/{}", self.base_url.trim_end_matches('/'), encoded_did)
    }

    /// Simple URL encoding function
    fn url_encode(&self, text: &str) -> String {
        // Simple encoding of common URL-unsafe characters
        // In a real implementation, you would use a proper URL encoding library
        text.replace(':', "%3A").replace('/', "%2F")
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "reqwest"))]
#[async_trait]
impl MessageSender for HttpMessageSender {
    async fn send(&self, packed_message: String, recipient_dids: Vec<String>) -> Result<()> {
        if recipient_dids.is_empty() {
            return Err(Error::Dispatch("No recipients specified".to_string()));
        }

        // Track failures to report them at the end
        let mut failures = Vec::new();

        // Send the message to each recipient
        for recipient in &recipient_dids {
            let endpoint = self.get_endpoint_url(recipient);
            log::info!("Sending message to {} via HTTP at {}", recipient, endpoint);

            // Retry logic
            let mut attempt = 0;
            let mut success = false;
            let mut last_error = None;

            while attempt < self.max_retries && !success {
                attempt += 1;

                // Exponential backoff for retries
                if attempt > 1 {
                    let backoff_ms = 100 * (2_u64.pow(attempt - 1));
                    tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
                }

                // Make the HTTP request
                match self.client
                    .post(&endpoint)
                    .header("Content-Type", "application/didcomm-message+json")
                    .body(packed_message.clone())
                    .send()
                    .await
                {
                    Ok(response) => {
                        // Check if the response was successful (2xx status code)
                        if response.status().is_success() {
                            log::debug!("Successfully sent message to {}", recipient);
                            success = true;
                        } else {
                            let status = response.status();
                            let body = response.text().await.unwrap_or_default();
                            log::warn!(
                                "Failed to send message to {} (attempt {}/{}): HTTP {} - {}",
                                recipient, attempt, self.max_retries, status, body
                            );
                            last_error = Some(format!("HTTP error: {} - {}", status, body));

                            // Don't retry certain status codes
                            if status.as_u16() == 404 || status.as_u16() == 400 {
                                break;  // Don't retry not found or bad request
                            }
                        }
                    },
                    Err(err) => {
                        log::warn!(
                            "Failed to send message to {} (attempt {}/{}): {}",
                            recipient, attempt, self.max_retries, err
                        );
                        last_error = Some(format!("Request error: {}", err));
                    }
                }
            }

            if !success {
                // Record the failure
                failures.push((recipient.clone(), last_error.unwrap_or_else(|| "Unknown error".to_string())));
            }
        }

        // Report failures if any
        if !failures.is_empty() {
            let failure_messages = failures
                .iter()
                .map(|(did, err)| format!("{}: {}", did, err))
                .collect::<Vec<_>>()
                .join("; ");

            return Err(Error::Dispatch(format!(
                "Failed to send message to some recipients: {}", failure_messages
            )));
        }

        Ok(())
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "reqwest")))]
#[async_trait]
impl MessageSender for HttpMessageSender {
    async fn send(&self, packed_message: String, recipient_dids: Vec<String>) -> Result<()> {
        // This is a fallback implementation when reqwest is not available
        // In a production environment, reqwest should always be available in the native configuration

        for recipient in &recipient_dids {
            let endpoint = self.get_endpoint_url(recipient);
            log::info!("Would send message to {} via HTTP at {} (reqwest not available)",
                recipient, endpoint);
            log::debug!("Message content: {}", packed_message);
        }

        log::warn!("HTTP sender is running without reqwest feature enabled. No actual HTTP requests will be made.");
        Ok(())
    }
}

// Specific implementation for WASM environments with web-sys feature
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[async_trait(?Send)]
impl MessageSender for HttpMessageSender {
    async fn send(&self, packed_message: String, recipient_dids: Vec<String>) -> Result<()> {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{Request, RequestInit, RequestMode, Response};
        
        if recipient_dids.is_empty() {
            return Err(Error::Dispatch("No recipients specified".to_string()));
        }
        
        // Track failures to report them at the end
        let mut failures = Vec::new();

        // Get the window object
        let window = web_sys::window().ok_or_else(|| {
            Error::Dispatch("Could not get window object in WASM environment".to_string())
        })?;
        
        // Send the message to each recipient
        for recipient in &recipient_dids {
            let endpoint = self.get_endpoint_url(recipient);
            log::info!("Sending message to {} via HTTP at {} (WASM)", recipient, endpoint);
            
            // Retry logic
            let mut attempt = 0;
            let mut success = false;
            let mut last_error = None;
            
            while attempt < self.max_retries && !success {
                attempt += 1;
                
                // Exponential backoff for retries
                if attempt > 1 {
                    let backoff_ms = 100 * (2_u64.pow(attempt - 1));
                    // In WASM, we need to use a Promise-based sleep
                    let promise = js_sys::Promise::new(&mut |resolve, _| {
                        let closure = Closure::once_into_js(move || {
                            resolve.call0(&JsValue::NULL).unwrap();
                        });
                        window
                            .set_timeout_with_callback_and_timeout_and_arguments_0(
                                closure.as_ref().unchecked_ref(),
                                backoff_ms as i32,
                            )
                            .unwrap();
                    });
                    
                    let _ = JsFuture::from(promise).await;
                }
                
                // Initialize a new Request
                let mut opts = RequestInit::new();
                opts.method("POST");
                opts.mode(RequestMode::Cors);
                opts.body(Some(&JsValue::from_str(&packed_message)));
                
                let request = match Request::new_with_str_and_init(&endpoint, &opts) {
                    Ok(req) => req,
                    Err(err) => {
                        let err_msg = format!("Failed to create request: {:?}", err);
                        log::warn!("{}", err_msg);
                        last_error = Some(err_msg);
                        continue;
                    }
                };
                
                // Set headers
                if let Err(err) = request.headers().set("Content-Type", "application/didcomm-message+json") {
                    let err_msg = format!("Failed to set headers: {:?}", err);
                    log::warn!("{}", err_msg);
                    last_error = Some(err_msg);
                    continue;
                }
                
                // Perform the fetch
                let resp_promise = window.fetch_with_request(&request);
                let resp_jsvalue = match JsFuture::from(resp_promise).await {
                    Ok(val) => val,
                    Err(err) => {
                        let err_msg = format!("Fetch error: {:?}", err);
                        log::warn!(
                            "Failed to send message to {} (attempt {}/{}): {}",
                            recipient, attempt, self.max_retries, err_msg
                        );
                        last_error = Some(err_msg);
                        continue;
                    }
                };
                
                // Convert the response to a Response object
                let response: Response = match resp_jsvalue.dyn_into() {
                    Ok(resp) => resp,
                    Err(err) => {
                        let err_msg = format!("Failed to convert response: {:?}", err);
                        log::warn!("{}", err_msg);
                        last_error = Some(err_msg);
                        continue;
                    }
                };
                
                // Check the status
                if response.ok() {
                    log::debug!("Successfully sent message to {}", recipient);
                    success = true;
                } else {
                    let status = response.status();
                    
                    // Try to get the response body as text
                    let body_promise = response.text();
                    let body = match JsFuture::from(body_promise).await {
                        Ok(text_jsval) => text_jsval.as_string().unwrap_or_default(),
                        Err(_) => String::from("[Could not read response body]")
                    };
                    
                    let err_msg = format!("HTTP error: {} - {}", status, body);
                    log::warn!(
                        "Failed to send message to {} (attempt {}/{}): {}",
                        recipient, attempt, self.max_retries, err_msg
                    );
                    last_error = Some(err_msg);
                    
                    // Don't retry certain status codes
                    if status == 404 || status == 400 {
                        break;  // Don't retry not found or bad request
                    }
                }
            }
            
            if !success {
                failures.push((recipient.clone(), last_error.unwrap_or_else(|| "Unknown error".to_string())));
            }
        }
        
        // Report failures if any
        if !failures.is_empty() {
            let failure_messages = failures
                .iter()
                .map(|(did, err)| format!("{}: {}", did, err))
                .collect::<Vec<_>>()
                .join("; ");
            
            return Err(Error::Dispatch(format!(
                "Failed to send message to some recipients: {}", failure_messages
            )));
        }
        
        Ok(())
    }
}

// Fallback implementation for WASM environments without web-sys feature
#[cfg(all(target_arch = "wasm32", not(feature = "wasm")))]
#[async_trait(?Send)]
impl MessageSender for HttpMessageSender {
    async fn send(&self, packed_message: String, recipient_dids: Vec<String>) -> Result<()> {
        // This is a fallback implementation when web-sys is not available in WASM
        for recipient in &recipient_dids {
            let endpoint = self.get_endpoint_url(recipient);
            log::info!(
                "Would send message to {} via HTTP at {} (WASM without web-sys)",
                recipient, endpoint
            );
            log::debug!("Message content: {}", packed_message);
        }

        log::warn!("HTTP sender is running in WASM without the web-sys feature enabled. No actual HTTP requests will be made.");
        Ok(())
    }
}