//! Configuration for the TAP HTTP server.

use crate::event::EventLoggerConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for the TAP HTTP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TapHttpConfig {
    /// The host address to bind to.
    pub host: String,

    /// The port to bind to.
    pub port: u16,

    /// The endpoint path for receiving DIDComm messages.
    pub didcomm_endpoint: String,

    /// Optional rate limiting configuration.
    pub rate_limit: Option<RateLimitConfig>,

    /// Optional TLS configuration.
    pub tls: Option<TlsConfig>,

    /// Default timeout for outbound HTTP requests in seconds.
    pub request_timeout_secs: u64,

    /// Optional event logger configuration.
    /// If not provided, no event logging will be performed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_logger: Option<EventLoggerConfig>,
}

/// Configuration for rate limiting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum number of requests per window.
    pub max_requests: u32,

    /// Time window in seconds.
    pub window_secs: u64,
}

/// Configuration for TLS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Path to the certificate file.
    pub cert_path: String,

    /// Path to the key file.
    pub key_path: String,
}

impl Default for TapHttpConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8000,
            didcomm_endpoint: "/didcomm".to_string(),
            rate_limit: None,
            tls: None,
            request_timeout_secs: 30,
            event_logger: Some(EventLoggerConfig::default()),
        }
    }
}

impl TapHttpConfig {
    /// Returns the full server address as a string (e.g., "127.0.0.1:8000").
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Returns the full URL for the DIDComm endpoint.
    pub fn didcomm_url(&self, secure: bool) -> String {
        let protocol = if secure || self.tls.is_some() {
            "https"
        } else {
            "http"
        };
        format!(
            "{}://{}:{}{}",
            protocol, self.host, self.port, self.didcomm_endpoint
        )
    }

    /// Returns the request timeout as a Duration.
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.request_timeout_secs)
    }
}
