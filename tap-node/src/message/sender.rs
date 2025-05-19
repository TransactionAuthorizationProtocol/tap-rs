//! PlainMessage sender implementations for TAP Node.
//!
//! This module provides functionality for sending TAP messages to recipients
//! using various transport mechanisms.
//!
//! # Overview
//!
//! The message sender system in TAP Node implements different strategies for
//! delivering DIDComm messages to their recipients. The primary implementations are:
//!
//! - `NodePlainMessageSender`: A flexible sender that uses a callback function for delivery
//! - `HttpPlainMessageSender`: Sends messages over HTTP with robust error handling and retries
//!
//! # Cross-platform Support
//!
//! The `HttpPlainMessageSender` implementation supports multiple platforms:
//!
//! - Native environments (using reqwest)
//! - WASM environments (using web-sys)
//! - Fallback implementations for environments without these libraries
//!
//! # Usage with TapNode
//!
//! ```no_run
//! # use std::sync::Arc;
//! # use tap_node::{NodeConfig, TapNode, HttpPlainMessageSender, PlainMessageSender};
//! # use tap_msg::didcomm::PlainMessage;
//! # use serde_json::json;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a TapNode
//! let node = TapNode::new(NodeConfig::default());
//!
//! // Create a sample message
//! let message = PlainMessage {
//!     id: "test-123".to_string(),
//!     typ: "https://tap.rsvp/schema/1.0#transfer".to_string(),
//!     type_: "https://tap.rsvp/schema/1.0#transfer".to_string(),
//!     body: json!({"amount": "100.00"}),
//!     from: "did:example:sender".to_string(),
//!     to: vec!["did:example:recipient".to_string()],
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
//! let sender = HttpPlainMessageSender::new("https://recipient-endpoint.example.com".to_string());
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
#[cfg(all(not(target_arch = "wasm32"), feature = "websocket"))]
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::sync::Arc;

use crate::error::{Error, Result};

/// PlainMessage sender trait for sending packed messages to recipients
#[async_trait]
pub trait PlainMessageSender: Send + Sync + Debug {
    /// Send a packed message to one or more recipients
    async fn send(&self, packed_message: String, recipient_dids: Vec<String>) -> Result<()>;
}

/// Node message sender implementation
pub struct NodePlainMessageSender {
    /// Callback function for sending messages
    send_callback: Arc<dyn Fn(String, Vec<String>) -> Result<()> + Send + Sync>,
}

impl Debug for NodePlainMessageSender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodePlainMessageSender")
            .field("send_callback", &"<function>")
            .finish()
    }
}

impl NodePlainMessageSender {
    /// Create a new NodePlainMessageSender with the given callback
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
impl PlainMessageSender for NodePlainMessageSender {
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
/// PlainMessages are sent to endpoints derived from the recipient's DID, using a
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
pub struct HttpPlainMessageSender {
    /// Base URL for the HTTP endpoint
    base_url: String,
    /// HTTP client (only in native environments)
    #[cfg(feature = "reqwest")]
    client: reqwest::Client,
    /// Timeout for HTTP requests in milliseconds
    #[allow(dead_code)] // Used for future timeout configuration
    timeout_ms: u64,
    /// Maximum number of retries
    max_retries: u32,
}

impl HttpPlainMessageSender {
    /// Create a new HttpPlainMessageSender with the given base URL
    pub fn new(base_url: String) -> Self {
        Self::with_options(base_url, 30000, 3) // Default 30 second timeout, 3 retries
    }

    /// Create a new HttpPlainMessageSender with custom options
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
        format!(
            "{}/api/messages/{}",
            self.base_url.trim_end_matches('/'),
            encoded_did
        )
    }

    /// Simple URL encoding function
    fn url_encode(&self, text: &str) -> String {
        // Simple encoding of common URL-unsafe characters
        // In a real implementation, you would use a proper URL encoding library
        text.replace(':', "%3A").replace('/', "%2F")
    }
}

/// WebSocket message sender implementation for sending messages over WebSockets
///
/// This sender enables real-time bidirectional communication between TAP nodes,
/// providing a persistent connection that can be used for both sending and receiving
/// messages. WebSockets are particularly useful for scenarios requiring:
///
/// - Low-latency message delivery
/// - Bidirectional communication
/// - Connection state awareness
/// - Reduced overhead compared to repeated HTTP requests
///
/// # Connection Management
///
/// The WebSocket sender manages a pool of connections to recipient endpoints,
/// keeping them alive and reconnecting as needed. This makes it suitable for
/// high-frequency message exchanges between known parties.
///
/// # Error Handling
///
/// The sender handles various WebSocket-specific error conditions:
/// - Connection failures
/// - PlainMessage delivery failures
/// - Connection drops and reconnection
/// - Protocol errors
///
/// # Cross-platform Support
///
/// Like the HTTP sender, the WebSocket sender supports:
/// - Native environments (using tokio_tungstenite)
/// - WASM environments (using web-sys WebSocket API)
/// - Fallback implementations for environments without these libraries
#[derive(Debug)]
pub struct WebSocketPlainMessageSender {
    /// Base URL for WebSocket endpoints (ws:// or wss://)
    base_url: String,
    /// Active connections (only in native environments)
    #[cfg(all(not(target_arch = "wasm32"), feature = "websocket"))]
    connections: std::sync::Mutex<HashMap<String, tokio::sync::mpsc::Sender<String>>>,
    /// WebSocket task handles (only in native environments)
    #[cfg(all(not(target_arch = "wasm32"), feature = "websocket"))]
    task_handles: std::sync::Mutex<HashMap<String, tokio::task::JoinHandle<()>>>,
}

impl WebSocketPlainMessageSender {
    /// Create a new WebSocketPlainMessageSender with the given base URL
    pub fn new(base_url: String) -> Self {
        Self::with_options(base_url)
    }

    /// Create a new WebSocketPlainMessageSender with custom options
    pub fn with_options(base_url: String) -> Self {
        #[cfg(all(not(target_arch = "wasm32"), feature = "websocket"))]
        {
            Self {
                base_url,
                connections: std::sync::Mutex::new(HashMap::new()),
                task_handles: std::sync::Mutex::new(HashMap::new()),
            }
        }

        #[cfg(not(all(not(target_arch = "wasm32"), feature = "websocket")))]
        {
            Self { base_url }
        }
    }

    /// Helper to construct the WebSocket endpoint URL for a recipient
    fn get_endpoint_url(&self, recipient_did: &str) -> String {
        // In a production implementation, this would map DID to WebSocket endpoint
        // This could involve DID resolution or a lookup table

        // Convert http(s):// to ws(s)://
        let ws_base_url = if self.base_url.starts_with("https://") {
            self.base_url.replace("https://", "wss://")
        } else if self.base_url.starts_with("http://") {
            self.base_url.replace("http://", "ws://")
        } else {
            self.base_url.clone()
        };

        // Append the DID to the base URL, with proper URL encoding
        let encoded_did = self.url_encode(recipient_did);
        format!(
            "{}/ws/messages/{}",
            ws_base_url.trim_end_matches('/'),
            encoded_did
        )
    }

    /// Simple URL encoding function
    fn url_encode(&self, text: &str) -> String {
        // Simple encoding of common URL-unsafe characters
        // In a real implementation, you would use a proper URL encoding library
        text.replace(':', "%3A").replace('/', "%2F")
    }

    /// Ensures a connection exists for the given recipient
    #[cfg(all(not(target_arch = "wasm32"), feature = "websocket"))]
    async fn ensure_connection(
        &self,
        recipient: &str,
    ) -> Result<tokio::sync::mpsc::Sender<String>> {
        use futures::sink::SinkExt;
        use futures::stream::StreamExt;
        use tokio_tungstenite::connect_async;
        use tokio_tungstenite::tungstenite::protocol::Message;

        // Check if we already have an active connection and return it if we do
        {
            // Scope the lock to ensure it's released before any await points
            let connections = self.connections.lock().unwrap();
            if let Some(connection) = connections.get(recipient) {
                return Ok(connection.clone());
            }
        }

        // Otherwise, create a new connection
        let endpoint = self.get_endpoint_url(recipient);
        log::info!(
            "Creating new WebSocket connection to {} at {}",
            recipient,
            endpoint
        );

        // Create a channel for sending messages to the WebSocket task
        let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

        // Connect to the WebSocket with default timeout (30 seconds)
        let (ws_stream, _) = match tokio::time::timeout(
            std::time::Duration::from_millis(30000),
            connect_async(&endpoint),
        )
        .await
        {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => {
                return Err(Error::Dispatch(format!(
                    "Failed to connect to WebSocket endpoint {}: {}",
                    endpoint, e
                )));
            }
            Err(_) => {
                return Err(Error::Dispatch(format!(
                    "Connection to WebSocket endpoint {} timed out",
                    endpoint
                )));
            }
        };

        log::debug!("WebSocket connection established to {}", recipient);

        // Split the WebSocket stream
        let (mut write, mut read) = ws_stream.split();

        // Create a task that will:
        // 1. Listen for messages from the channel and send them to the WebSocket
        // 2. Listen for messages from the WebSocket and handle them
        let recipient_clone = recipient.to_string();
        let handle = tokio::spawn(async move {
            // Process messages from the channel to send over WebSocket
            loop {
                tokio::select! {
                    // Handle outgoing messages
                    Some(message) = rx.recv() => {
                        log::debug!("Sending message to {} via WebSocket", recipient_clone);
                        if let Err(e) = write.send(Message::Text(message)).await {
                            log::error!("Failed to send WebSocket message to {}: {}", recipient_clone, e);
                            // Try to reconnect? For now we'll just log the error
                        }
                    }

                    // Handle incoming messages
                    result = read.next() => {
                        match result {
                            Some(Ok(message)) => {
                                // Process incoming message - for now just log it
                                if let Message::Text(text) = message {
                                    log::debug!("Received WebSocket message from {}: {}", recipient_clone, text);
                                }
                            }
                            Some(Err(e)) => {
                                log::error!("WebSocket error from {}: {}", recipient_clone, e);
                                // Connection likely dropped, exit the loop
                                break;
                            }
                            None => {
                                // WebSocket closed
                                log::info!("WebSocket connection to {} closed", recipient_clone);
                                break;
                            }
                        }
                    }
                }
            }

            // WebSocket loop ended - clean up and possibly reconnect
            log::info!("WebSocket connection to {} terminated", recipient_clone);
        });

        // Store the sender and task handle (using separate scopes to avoid holding multiple locks)
        {
            // Get mutable access to the connections map
            let mut connections = self.connections.lock().unwrap();
            connections.insert(recipient.to_string(), tx.clone());
        }

        {
            // Get mutable access to the task handles map
            let mut task_handles = self.task_handles.lock().unwrap();
            task_handles.insert(recipient.to_string(), handle);
        }

        Ok(tx)
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "websocket"))]
#[async_trait]
impl PlainMessageSender for WebSocketPlainMessageSender {
    async fn send(&self, packed_message: String, recipient_dids: Vec<String>) -> Result<()> {
        if recipient_dids.is_empty() {
            return Err(Error::Dispatch("No recipients specified".to_string()));
        }

        // Track failures to report them at the end
        let mut failures = Vec::new();

        // Send the message to each recipient
        for recipient in &recipient_dids {
            log::info!("Sending message to {} via WebSocket", recipient);

            // Ensure we have a connection
            match self.ensure_connection(recipient).await {
                Ok(sender) => {
                    // Send the message through the channel to the WebSocket task
                    if let Err(e) = sender.send(packed_message.clone()).await {
                        let err_msg = format!("Failed to send message to WebSocket task: {}", e);
                        log::error!("{}", err_msg);
                        failures.push((recipient.clone(), err_msg));
                    }
                }
                Err(e) => {
                    let err_msg = format!("Failed to establish WebSocket connection: {}", e);
                    log::error!("{}", err_msg);
                    failures.push((recipient.clone(), err_msg));
                }
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
                "Failed to send message to some recipients via WebSocket: {}",
                failure_messages
            )));
        }

        Ok(())
    }
}

// Specific implementation for WASM environments with web-sys feature
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[async_trait(?Send)]
impl PlainMessageSender for WebSocketPlainMessageSender {
    async fn send(&self, packed_message: String, recipient_dids: Vec<String>) -> Result<()> {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{MessageEvent, WebSocket};

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
            log::info!(
                "Sending message to {} via WebSocket at {} (WASM)",
                recipient,
                endpoint
            );

            // Create a promise to set up a WebSocket connection and send the message
            let (resolve, reject) = js_sys::Promise::new_resolver();
            let promise_resolver = resolve.clone();
            let promise_rejecter = reject.clone();

            // Create a new WebSocket
            let ws = match WebSocket::new(&endpoint) {
                Ok(ws) => ws,
                Err(err) => {
                    let err_msg = format!("Failed to create WebSocket: {:?}", err);
                    log::error!("{}", err_msg);
                    failures.push((recipient.clone(), err_msg));
                    continue;
                }
            };

            // Set up event handlers
            let onopen_callback = Closure::once(Box::new(move |_: web_sys::Event| {
                promise_resolver.resolve(&JsValue::from(true));
            }) as Box<dyn FnOnce(web_sys::Event)>);

            let onerror_callback = Closure::once(Box::new(move |e: web_sys::Event| {
                let err_msg = format!("WebSocket error: {:?}", e);
                promise_rejecter.reject(&JsValue::from_str(&err_msg));
            }) as Box<dyn FnOnce(web_sys::Event)>);

            let message_clone = packed_message.clone();
            let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
                if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                    log::debug!("Received message: {}", String::from(txt));
                }
            }) as Box<dyn FnMut(MessageEvent)>);

            // Register event handlers
            ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
            ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
            ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));

            // Wait for the connection to be established
            match JsFuture::from(js_sys::Promise::race(&js_sys::Array::of2(
                &js_sys::Promise::resolve(&promise_resolver),
                &js_sys::Promise::new(&mut |resolve, _| {
                    let timeout_closure = Closure::once_into_js(move || {
                        resolve.call0(&JsValue::NULL).unwrap();
                    });
                    window
                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                            timeout_closure.as_ref().unchecked_ref(),
                            30000, // Default 30 second timeout
                        )
                        .unwrap();
                }),
            )))
            .await
            {
                Ok(_) => {
                    // Connection established, send the message
                    if let Err(err) = ws.send_with_str(&message_clone) {
                        let err_msg = format!("Failed to send WebSocket message: {:?}", err);
                        log::error!("{}", err_msg);
                        failures.push((recipient.clone(), err_msg));
                    }
                }
                Err(err) => {
                    let err_msg = format!("WebSocket connection failed: {:?}", err);
                    log::error!("{}", err_msg);
                    failures.push((recipient.clone(), err_msg));
                }
            }

            // Keep the callbacks alive
            onopen_callback.forget();
            onerror_callback.forget();
            onmessage_callback.forget();
        }

        // Report failures if any
        if !failures.is_empty() {
            let failure_messages = failures
                .iter()
                .map(|(did, err)| format!("{}: {}", did, err))
                .collect::<Vec<_>>()
                .join("; ");

            return Err(Error::Dispatch(format!(
                "Failed to send message to some recipients via WebSocket: {}",
                failure_messages
            )));
        }

        Ok(())
    }
}

// Fallback implementation for environments without WebSocket support
#[cfg(not(any(
    all(not(target_arch = "wasm32"), feature = "websocket"),
    all(target_arch = "wasm32", feature = "wasm")
)))]
#[async_trait]
impl PlainMessageSender for WebSocketPlainMessageSender {
    async fn send(&self, packed_message: String, recipient_dids: Vec<String>) -> Result<()> {
        // This is a fallback implementation when neither tokio_tungstenite nor web-sys is available
        for recipient in &recipient_dids {
            let endpoint = self.get_endpoint_url(recipient);
            log::info!(
                "Would send message to {} via WebSocket at {} (WebSocket not available)",
                recipient,
                endpoint
            );
            log::debug!("PlainMessage content: {}", packed_message);
        }

        log::warn!("WebSocket sender is running without WebSocket features enabled. No actual WebSocket connections will be made.");
        Ok(())
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "reqwest"))]
#[async_trait]
impl PlainMessageSender for HttpPlainMessageSender {
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
                match self
                    .client
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
                                recipient,
                                attempt,
                                self.max_retries,
                                status,
                                body
                            );
                            last_error = Some(format!("HTTP error: {} - {}", status, body));

                            // Don't retry certain status codes
                            if status.as_u16() == 404 || status.as_u16() == 400 {
                                break; // Don't retry not found or bad request
                            }
                        }
                    }
                    Err(err) => {
                        log::warn!(
                            "Failed to send message to {} (attempt {}/{}): {}",
                            recipient,
                            attempt,
                            self.max_retries,
                            err
                        );
                        last_error = Some(format!("Request error: {}", err));
                    }
                }
            }

            if !success {
                // Record the failure
                failures.push((
                    recipient.clone(),
                    last_error.unwrap_or_else(|| "Unknown error".to_string()),
                ));
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
                "Failed to send message to some recipients: {}",
                failure_messages
            )));
        }

        Ok(())
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "reqwest")))]
#[async_trait]
impl PlainMessageSender for HttpPlainMessageSender {
    async fn send(&self, packed_message: String, recipient_dids: Vec<String>) -> Result<()> {
        // This is a fallback implementation when reqwest is not available
        // In a production environment, reqwest should always be available in the native configuration

        for recipient in &recipient_dids {
            let endpoint = self.get_endpoint_url(recipient);
            log::info!(
                "Would send message to {} via HTTP at {} (reqwest not available)",
                recipient,
                endpoint
            );
            log::debug!("PlainMessage content: {}", packed_message);
        }

        log::warn!("HTTP sender is running without reqwest feature enabled. No actual HTTP requests will be made.");
        Ok(())
    }
}

// Specific implementation for WASM environments with web-sys feature
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[async_trait(?Send)]
impl PlainMessageSender for HttpPlainMessageSender {
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
            log::info!(
                "Sending message to {} via HTTP at {} (WASM)",
                recipient,
                endpoint
            );

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
                if let Err(err) = request
                    .headers()
                    .set("Content-Type", "application/didcomm-message+json")
                {
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
                            recipient,
                            attempt,
                            self.max_retries,
                            err_msg
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
                        Err(_) => String::from("[Could not read response body]"),
                    };

                    let err_msg = format!("HTTP error: {} - {}", status, body);
                    log::warn!(
                        "Failed to send message to {} (attempt {}/{}): {}",
                        recipient,
                        attempt,
                        self.max_retries,
                        err_msg
                    );
                    last_error = Some(err_msg);

                    // Don't retry certain status codes
                    if status == 404 || status == 400 {
                        break; // Don't retry not found or bad request
                    }
                }
            }

            if !success {
                failures.push((
                    recipient.clone(),
                    last_error.unwrap_or_else(|| "Unknown error".to_string()),
                ));
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
                "Failed to send message to some recipients: {}",
                failure_messages
            )));
        }

        Ok(())
    }
}

// Fallback implementation for WASM environments without web-sys feature
#[cfg(all(target_arch = "wasm32", not(feature = "wasm")))]
#[async_trait(?Send)]
impl PlainMessageSender for HttpPlainMessageSender {
    async fn send(&self, packed_message: String, recipient_dids: Vec<String>) -> Result<()> {
        // This is a fallback implementation when web-sys is not available in WASM
        for recipient in &recipient_dids {
            let endpoint = self.get_endpoint_url(recipient);
            log::info!(
                "Would send message to {} via HTTP at {} (WASM without web-sys)",
                recipient,
                endpoint
            );
            log::debug!("PlainMessage content: {}", packed_message);
        }

        log::warn!("HTTP sender is running in WASM without the web-sys feature enabled. No actual HTTP requests will be made.");
        Ok(())
    }
}
