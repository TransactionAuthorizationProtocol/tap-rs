# TAP HTTP

HTTP DIDComm server implementation for the Transaction Authorization Protocol (TAP).

## Features

- **DIDComm HTTP Endpoint**: Exposes a secure HTTP endpoint for DIDComm messaging
- **Integration with tap-node**: Seamlessly forwards messages to a tap-node instance
- **Message Validation**: Validates incoming DIDComm messages
- **Response Handling**: Proper handling of responses and errors
- **Outgoing Message Delivery**: HTTP client for sending outgoing DIDComm messages
- **Event Logging System**: Comprehensive event tracking with configurable logging destinations
- **Security**: Support for HTTPS/TLS and rate limiting (configurable)
- **Comprehensive Error Handling**: Structured error responses with appropriate HTTP status codes

## Usage

```rust
use tap_http::{TapHttpConfig, TapHttpServer};
use tap_node::{NodeConfig, TapNode};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a TAP Node for message processing
    let node = TapNode::new(NodeConfig::default());
    
    // Configure the HTTP server with custom settings
    let config = TapHttpConfig {
        host: "0.0.0.0".to_string(),    // Listen on all interfaces
        port: 8080,                     // Custom port
        didcomm_endpoint: "/api/didcomm".to_string(),  // Custom endpoint path
        request_timeout_secs: 60,       // 60-second timeout for outbound requests
        ..TapHttpConfig::default()
    };
    
    // Create and start the server
    let mut server = TapHttpServer::new(config, node);
    server.start().await?;
    
    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    
    // Gracefully stop the server
    server.stop().await?;
    
    Ok(())
}
```

## HTTP Endpoints

### POST /{didcomm_endpoint}

The main endpoint for receiving DIDComm messages. The endpoint path is configurable (default is `/didcomm`):

```http
POST /didcomm HTTP/1.1
Host: example.com
Content-Type: application/didcomm-message+json

{
  "id": "1234567890",
  "type": "https://tap.rsvp/schema/1.0#transfer",
  "body": {
    "amount": "100.00",
    "asset": "eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f",
    "transaction_id": "tx-123456"
  },
  "from": "did:example:sender",
  "to": ["did:example:recipient"],
  "created_time": 1620000000
}
```

### GET /health

Health check endpoint for monitoring system availability:

```http
GET /health HTTP/1.1
Host: example.com
```

Response:

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "status": "ok",
  "version": "0.1.0"
}
```

## Response Formats

### Success Response

For successfully processed messages:

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "status": "success",
  "message": "Message received and processed"
}
```

### Error Response

For validation and other errors:

```http
HTTP/1.1 400 Bad Request
Content-Type: application/json

{
  "status": "error",
  "error": {
    "type": "validation_error",
    "message": "Unsupported message type: https://didcomm.org/basicmessage/2.0/message, expected TAP protocol message"
  }
}
```

## Message Validation

The server performs several validation steps on incoming messages:

1. **Basic Format Validation**:
   - Ensures the message has required fields (id, type, from, to)
   - Validates message timestamps

2. **Protocol Validation**:
   - Checks that the message type is a valid TAP protocol message
   - Validates sender and recipient information

3. **TAP Node Validation**:
   - Messages are forwarded to the TAP Node for further validation
   - Authentication and signature verification is performed

## Configuration Options

The server can be configured with the following options in `TapHttpConfig`:

```rust
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
}
```

### TLS Configuration

Enable HTTPS with TLS certificates:

```rust
let config = TapHttpConfig {
    // ...other settings
    tls: Some(TlsConfig {
        cert_path: "/path/to/cert.pem".to_string(),
        key_path: "/path/to/key.pem".to_string(),
    }),
    // ...
};
```

### Event Logging

Configure event logging to track server activity:

```rust
use tap_http::event::{EventLoggerConfig, LogDestination};

let config = TapHttpConfig {
    // ...other settings
    event_logger: Some(EventLoggerConfig {
        destination: LogDestination::File {
            path: "./logs/tap-http.log".to_string(), // Default location
            max_size: Some(10 * 1024 * 1024),        // 10 MB
            rotate: true,                            // Enable rotation
        },
        structured: true,      // Use JSON format
        log_level: log::Level::Info,
    }),
    // ...
};
```

The event logging system captures:
- Server startup and shutdown
- HTTP request/response details
- DIDComm message processing
- Error events with detailed information

Custom event subscribers can also be implemented:

```rust
use std::sync::Arc;
use async_trait::async_trait;
use tap_http::event::{EventSubscriber, HttpEvent};

struct CustomEventHandler;

#[async_trait]
impl EventSubscriber for CustomEventHandler {
    async fn handle_event(&self, event: HttpEvent) {
        // Custom handling of events
        println!("Event: {:?}", event);
    }
}

// After creating the server
let custom_handler = Arc::new(CustomEventHandler);
server.event_bus().subscribe(custom_handler);
```

### Rate Limiting

Configure rate limiting to prevent abuse:

```rust
let config = TapHttpConfig {
    // ...other settings
    rate_limit: Some(RateLimitConfig {
        max_requests: 100,  // Maximum requests per window
        window_secs: 60,    // Time window in seconds
    }),
    // ...
};
```

## Client

The package also includes an HTTP client for sending DIDComm messages to other endpoints:

```rust
use tap_http::DIDCommClient;

// Create client with default timeout
let client = DIDCommClient::default();

// Send a DIDComm message
client.deliver_message("https://recipient.example.com/didcomm", packed_message).await?;
```

## Security Considerations

- Use TLS in production environments
- Configure rate limiting to prevent abuse
- Ensure proper validation and authentication of messages
- Consider running behind a reverse proxy for additional security layers

## Error Handling

The server uses a comprehensive error handling system with appropriate HTTP status codes:

- `400 Bad Request`: Format and validation errors
- `401 Unauthorized`: Authentication errors
- `429 Too Many Requests`: Rate limiting
- `500 Internal Server Error`: Server-side errors
- `503 Service Unavailable`: Configuration errors

## Command Line Usage

The tap-http package includes a binary executable that can be run from the command line:

```bash
# Install the package
cargo install --path .

# Run the HTTP server with default settings
tap-http

# Run with custom options
tap-http --host 0.0.0.0 --port 8080 --endpoint /api/didcomm
```

### Command Line Options

```
USAGE:
    tap-http [OPTIONS]

OPTIONS:
    -h, --host <HOST>            Host to bind to [default: 127.0.0.1]
    -p, --port <PORT>            Port to listen on [default: 8000]
    -e, --endpoint <ENDPOINT>    Path for the DIDComm endpoint [default: /didcomm]
    -t, --timeout <SECONDS>      Request timeout in seconds [default: 30]
    -v, --verbose                Enable verbose logging
    --help                       Print help information
    --version                    Print version information
```

### Environment Variables

You can also configure the server using environment variables:

```bash
# Set configuration options
export TAP_HTTP_HOST=0.0.0.0
export TAP_HTTP_PORT=8080
export TAP_HTTP_DIDCOMM_ENDPOINT=/api/didcomm
export TAP_HTTP_TIMEOUT=60

# Run the server (will use environment variables)
tap-http
```

## Examples

Check the examples directory for complete usage examples:

- `http_message_flow.rs`: Basic HTTP message flow
- `websocket_message_flow.rs`: WebSocket message flow example
- `event_logger_demo.rs`: Demonstration of event logging configuration

To run the examples:

```bash
# Run the HTTP message flow example
cargo run --example http_message_flow

# Run the WebSocket message flow example (with websocket feature)
cargo run --example websocket_message_flow --features websocket

# Run the event logger demo
cargo run --example event_logger_demo
```