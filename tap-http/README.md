# TAP HTTP

HTTP DIDComm server implementation for the Transaction Authorization Protocol (TAP), providing secure message exchange via standard HTTP endpoints.

## Features

- **DIDComm HTTP Endpoint**: Exposes a secure HTTP endpoint for DIDComm messaging
- **Integration with tap-node**: Seamlessly forwards messages to a tap-node instance
- **Ephemeral Agent Support**: Creates an ephemeral agent with did:key by default
- **Message Validation**: Validates incoming DIDComm messages
- **Response Handling**: Proper handling of responses and errors
- **Outgoing Message Delivery**: HTTP client for sending outgoing DIDComm messages
- **Event Logging System**: Comprehensive event tracking with configurable logging destinations
- **Security**: Support for HTTPS/TLS and rate limiting (configurable)
- **Comprehensive Error Handling**: Structured error responses with appropriate HTTP status codes
- **Payment Flow Simulator**: Included CLI tool for simulating TAP payment flows
- **Persistent Storage**: SQLite database using async SQLx for message audit trail and transaction tracking

## Usage

```rust
use tap_http::{TapHttpConfig, TapHttpServer};
use tap_node::{NodeConfig, TapNode};
use tap_agent::DefaultAgent;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a TAP Agent - either load from stored keys or create ephemeral
    let agent = DefaultAgent::from_stored_or_ephemeral(None, true);
    println!("Server using agent with DID: {}", agent.get_agent_did());
    
    // Create a TAP Node configuration with storage
    let mut node_config = NodeConfig::default();
    node_config.storage_path = Some("tap-http.db".into());
    
    // Create a TAP Node for message processing
    let mut node = TapNode::new(node_config);
    node.init_storage().await?;
    node.register_agent(Arc::new(agent)).await?;
    
    // Configure the HTTP server with custom settings
    let config = TapHttpConfig {
        host: "0.0.0.0".to_string(),                  // Listen on all interfaces
        port: 8080,                                   // Custom port
        didcomm_endpoint: "/api/didcomm".to_string(), // Custom endpoint path
        request_timeout_secs: 60,                     // 60-second timeout for requests
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
Content-Type: application/didcomm-encrypted+json

eyJhbGciOiJFQ0RILUVTK0EyNTZLVyIsImFwdiI6InpRbFpBQ0pZVFpnZUNidFhvd0xkX18zdWNmQmstLW0za2NXekQyQ0kiLCJlbmMiOiJBMjU2R0NNIiwiZXBrIjp7ImNydiI6IlAtMjU2Iiwia3R5IjoiRUMiLCJ4IjoiZ1RxS2ZaQk45bXpLNHZhX1l2TXQ2c0VkNEw0X1Q3aS1PVmtvMGFaVHUwZyIsInkiOiJQOHdyeFFDYmFZckdPdTRXWGM0R05WdWkyLWVpbEhYNUNHZXo5dk9FX2ZrIn0sInByb3RlY3RlZCI6ImV5SmtjeUk2ZXlKQmJXRjZiMjVCZEhSeWFXSmhkR1ZVWVdkZmMxOGdUVVZCTUZVMVRFMVlXRkF5UmtkRWFEVmFkejA5SW4xOSIsInR5cCI6ImFwcGxpY2F0aW9uL2RpZGNvbW0tZW5jcnlwdGVkK2pzb24ifQ...
```

When unpacked, the above message would contain a TAP protocol message like:

```json
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

## Response Formats and Status Codes

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

## Message Validation and Processing

The server performs several validation steps on incoming messages:

1. **Message Unpacking**:
   - Decrypts and verifies message signatures using the TAP Agent
   - Handles different security modes (Plain, Signed, AuthCrypt)
   - Validates cryptographic integrity

2. **DID Verification**:
   - Resolves DIDs using the TAP Agent's resolver
   - Validates sender's verification methods
   - Checks service endpoints for routing

3. **Protocol Validation**:
   - Validates message types against TAP protocol schemas
   - Verifies required fields (id, type, from, to)
   - Validates message timestamps and sequence

4. **TAP Node Processing**:
   - Forwards valid messages to the TAP Node for business logic processing
   - Returns responses from the node to the sender
   - Logs process events and errors

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
    
    /// Event logger configuration (for tracking server events).
    pub event_logger: Option<EventLoggerConfig>,
    
    /// CORS configuration for cross-origin requests.
    pub cors: Option<CorsConfig>,
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
        include_payloads: false, // Don't log sensitive message payloads
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

## DIDComm Client

The package includes an HTTP client for sending DIDComm messages to other endpoints:

```rust
use tap_http::DIDCommClient;
use tap_agent::{Agent, DefaultAgent};
use tap_msg::message::Transfer;

// Create a TAP Agent
let (agent, agent_did) = DefaultAgent::new_ephemeral()?;

// Create client with custom timeout
let client = DIDCommClient::new(std::time::Duration::from_secs(30));

// Create a message
let transfer = Transfer {
    // Message fields...
    transaction_id: uuid::Uuid::new_v4().to_string(),
};

// Pack a message using the agent
let recipient_did = "did:web:example.com";
let (packed_message, _) = agent.send_message(&transfer, vec![recipient_did], false).await?;

// Send the packed message to a recipient's endpoint
let response = client.deliver_message(
    "https://recipient.example.com/didcomm",
    &packed_message
).await?;

// Process the response
println!("Delivery status: {}", response.status());
```

You can also use the built-in delivery functionality of the TAP Agent:

```rust
// The third parameter (true) enables automatic delivery
let (_, delivery_results) = agent.send_message(&transfer, vec![recipient_did], true).await?;

// Check delivery results
for result in delivery_results {
    if let Some(status) = result.status {
        println!("Delivery status: {}", status);
    }
}
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

The tap-http package includes binary executables that can be run from the command line:

# TAP HTTP Server

## Installation

The TAP HTTP server can be installed in several ways:

```bash
# From crates.io (recommended for most users)
cargo install tap-http

# From the repository (if you have it cloned)
cargo install --path tap-rs/tap-http

# Run the HTTP server with default settings (creates ephemeral agent)
tap-http

# Run with custom options
tap-http --host 0.0.0.0 --port 8080 --endpoint /api/didcomm

# Run with stored key (uses default from ~/.tap/keys.json)
tap-http --use-stored-key

# Run with a specific stored key by its DID
tap-http --use-stored-key --agent-did did:key:z6Mk...

# Run with custom logging options
tap-http --logs-dir /var/log/tap --structured-logs
```

## Docker

The easiest way to run `tap-http` is with Docker. All persistent state (keys, databases, logs) is stored in a single volume at `/data/tap`.

### Quick Start

```bash
# Build and run with docker compose
docker compose up -d

# View logs
docker compose logs -f tap-http

# Stop
docker compose down
```

### Build the Image Manually

```bash
docker build -t tap-http .
```

### Run with Docker

```bash
# Run with a named volume for persistent storage
docker run -d \
  --name tap-http \
  -p 8000:8000 \
  -v tap-data:/data/tap \
  tap-http

# Run with a host directory for easy inspection of data
docker run -d \
  --name tap-http \
  -p 8000:8000 \
  -v ./tap-data:/data/tap \
  tap-http

# Pass additional CLI flags
docker run -d \
  --name tap-http \
  -p 8000:8000 \
  -v tap-data:/data/tap \
  tap-http --verbose --structured-logs
```

### Persistent Storage

The container stores all state under `/data/tap`, which maps to:

| Path | Contents |
|------|----------|
| `/data/tap/keys.json` | Agent key store |
| `/data/tap/logs/` | Event log files |
| `/data/tap/<did>/transactions.db` | Per-agent SQLite databases |

Back up this volume to preserve keys and transaction history across container recreations.

### Environment Variables

Configure the container with environment variables via `docker compose` or `docker run -e`:

```bash
docker run -d \
  --name tap-http \
  -p 9000:9000 \
  -v tap-data:/data/tap \
  -e TAP_HTTP_PORT=9000 \
  -e TAP_STRUCTURED_LOGS=true \
  -e TAP_AGENT_DID=did:key:z6Mk... \
  -e TAP_AGENT_KEY=<base64-private-key> \
  tap-http
```

See the full list of environment variables in the [Environment Variables](#environment-variables-for-tap-http) section below.

### Inspecting Data

```bash
# Access the SQLite database from the host
docker run --rm -v tap-data:/data alpine ls -la /data/tap/

# Query the database directly
docker exec tap-http sqlite3 /data/tap/*/transactions.db \
  "SELECT message_id, message_type, direction FROM messages LIMIT 10;"

# Tail event logs
docker exec tap-http tail -f /data/tap/logs/tap-http.log
```

## Command Line Options for tap-http

After installation, you can use the `tap-http` command to run a TAP HTTP server:

```
USAGE:
    tap-http [OPTIONS]

OPTIONS:
    -h, --host <HOST>            Host to bind to [default: 127.0.0.1]
    -p, --port <PORT>            Port to listen on [default: 8000]
    -e, --endpoint <ENDPOINT>    Path for the DIDComm endpoint [default: /didcomm]
    -t, --timeout <SECONDS>      Request timeout in seconds [default: 30]
    --use-stored-key             Use a key from the local key store (~/.tap/keys.json)
    --agent-did <DID>            Specific DID to use from key store (when --use-stored-key is set)
    --generate-key               Generate a new key and save it to the key store
    --key-type <TYPE>            Key type for generation [default: ed25519] [possible values: ed25519, p256, secp256k1]
    --logs-dir <DIR>             Directory for event logs [default: ./logs]
    --structured-logs            Use structured JSON logging [default: true]
    --db-path <PATH>             Path to the database file [default: tap-http.db]
    --rate-limit <RATE>          Rate limit in requests per minute [default: 60]
    --tls-cert <PATH>            Path to TLS certificate file
    --tls-key <PATH>             Path to TLS private key file
    -v, --verbose                Enable verbose logging
    --help                       Print help information
    --version                    Print version information
```

### Environment Variables for tap-http

You can also configure the server using environment variables:

```bash
# Server configuration
export TAP_HTTP_HOST=0.0.0.0
export TAP_HTTP_PORT=8080
export TAP_HTTP_DIDCOMM_ENDPOINT=/api/didcomm
export TAP_HTTP_TIMEOUT=60

# Agent configuration
export TAP_USE_STORED_KEY=true
export TAP_AGENT_DID=did:key:z6Mk...
export TAP_GENERATE_KEY=false
export TAP_KEY_TYPE=ed25519

# Logging configuration
export TAP_LOGS_DIR=/var/log/tap
export TAP_STRUCTURED_LOGS=true
export TAP_LOG_LEVEL=info

# Storage configuration
export TAP_NODE_DB_PATH=/var/lib/tap/tap-http.db

# Security configuration
export TAP_RATE_LIMIT=100
export TAP_TLS_CERT=/path/to/cert.pem
export TAP_TLS_KEY=/path/to/key.pem

# Run the server (will use environment variables)
tap-http
```

### TAP Payment Flow Simulator

The package also includes a payment flow simulator that can be used to test the TAP HTTP server:

```bash
# Run the payment simulator
tap-payment-simulator --url http://localhost:8000/didcomm --did did:key:z6Mk...

# Run with custom amount and currency
tap-payment-simulator --url http://localhost:8000/didcomm --did did:key:z6Mk... --amount 500 --currency EUR
```

## Command Line Options for tap-payment-simulator

The payment simulator is installed together with the `tap-http` package. You can use it to test your TAP HTTP server:

```
USAGE:
    tap-payment-simulator --url <server-url> --did <server-agent-did> [OPTIONS]

REQUIRED ARGUMENTS:
    --url <URL>                 URL of the TAP HTTP server's DIDComm endpoint
    --did <DID>                 DID of the server's agent

OPTIONS:
    --amount <AMOUNT>           Amount to transfer [default: 100.00]
    --currency <CURRENCY>       Currency code [default: USD]
    -v, --verbose               Enable verbose logging
    --help                      Print help information
    --version                   Print version information
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

## Creating a TAP Payment Flow

Using the tap-payment-simulator tool, you can easily test a complete TAP payment flow:

1. Install the HTTP server and payment simulator (if not already installed):
   ```bash
   cargo install tap-http
   ```
   This installs both `tap-http` and `tap-payment-simulator` binaries.

2. Start the tap-http server with an ephemeral agent:
   ```bash
   tap-http --verbose
   ```
   The server will display the generated DID on startup:
   ```
   TAP HTTP Server started with agent DID: did:key:z6Mk...
   ```

2. In another terminal, run the payment simulator to send messages to the server:
   ```bash
   tap-payment-simulator --url http://localhost:8000/didcomm --did did:key:z6Mk...
   ```
   The payment simulator will also display its agent DID:
   ```
   Payment simulator using agent DID: did:key:z6Mk...
   ```

3. The simulator will:
   - Create its own ephemeral agent
   - Send a payment request message to the server
   - Wait for 2 seconds
   - Send a transfer message to the server
   - Both messages will use the same transaction ID to create a complete payment flow

4. Check the server logs to see the received messages and how they were processed:
   ```
   tail -f ./logs/tap-http.log
   ```

This simulates a complete payment flow between two agents, demonstrating how the TAP protocol works in practice.

## Integration with tap-agent Features

The TAP HTTP server leverages all the key features of the TAP Agent:

### Key Management

The server can use any of the TAP Agent's key management approaches:

- **Ephemeral keys** for testing and development (default)
- **Stored keys** from the local key store (`~/.tap/keys.json`) - shared with `tap-agent-cli`
- **Generated keys** created at startup and optionally saved

To generate and manage keys for use with `tap-http`, you can use the `tap-agent-cli` tool:

```bash
# Install the tap-agent CLI
cargo install tap-agent

# Generate and save a key for later use with tap-http
tap-agent-cli generate --save

# View your stored keys
tap-agent-cli keys list

# Then use a stored key with tap-http
tap-http --use-stored-key

# Use a specific stored key
tap-http --use-stored-key --agent-did did:key:z6Mk...
```

### DID Resolution

The server uses the TAP Agent's DID resolution capabilities:

- Support for `did:key` and `did:web` by default
- Custom DID method resolvers can be added
- Automatic endpoint discovery for message routing

### Secure Messaging

All security modes from the TAP Agent are supported:

- **Plain** - No security (for testing)
- **Signed** - Digital signatures for integrity
- **AuthCrypt** - Encrypted messages for confidentiality

### Service Endpoint Handling

The server acts as a service endpoint for incoming messages:

1. Configure the URL in your DID document's service section
2. Other agents can discover this endpoint via DID resolution
3. Messages will be automatically routed to your endpoint

## Persistent Storage

The TAP HTTP server includes built-in SQLite storage using async SQLx for:

- **Message Audit Trail**: All incoming and outgoing messages are logged
- **Transaction Tracking**: Transfer and Payment messages are tracked separately
- **Automatic Schema Management**: Database migrations run automatically on startup
- **JSON Column Support**: Message content is stored as validated JSON

### Storage Configuration

By default, the server creates a database file at `tap-http.db` in the current directory. You can customize this location:

```bash
# Via command line
tap-http --db-path /var/lib/tap/tap-http.db

# Via environment variable
export TAP_NODE_DB_PATH=/var/lib/tap/tap-http.db
tap-http
```

### Database Schema

The storage system maintains two tables:

1. **messages** - Complete audit trail of all messages:
   - message_id (unique identifier)
   - message_type (TAP message type)
   - from_did, to_did (sender and recipient DIDs)
   - direction (incoming/outgoing)
   - message_json (full message content as JSON column type)
   - created_at (timestamp)

2. **transactions** - Business logic for Transfer and Payment messages:
   - type (transfer/payment)
   - reference_id (message ID)
   - status (pending/completed/failed/cancelled)
   - from_did, to_did
   - thread_id
   - created_at, updated_at

### Querying the Database

You can query the database directly using SQLite tools:

```bash
# View recent messages
sqlite3 tap-http.db "SELECT message_id, message_type, direction, created_at FROM messages ORDER BY created_at DESC LIMIT 10;"

# Count messages by type
sqlite3 tap-http.db "SELECT message_type, COUNT(*) FROM messages GROUP BY message_type;"

# View pending transactions
sqlite3 tap-http.db "SELECT * FROM transactions WHERE status = 'pending';"
```

## Performance and Scaling

The TAP HTTP server is designed for performance:

- **Async Processing** - Uses Tokio runtime for efficient concurrency
- **Connection Pooling** - Reuses connections for outgoing requests
- **Minimal Copies** - Efficient handling of message payloads
- **Horizontal Scaling** - Can be deployed across multiple instances
- **Efficient Storage** - SQLite with async SQLx connection pooling and WAL mode

For high-volume deployments, consider:

- Running behind a load balancer
- Using a Redis-backed rate limiter
- Implementing a message queue for async processing
- Setting up proper monitoring and alerts
- Regular database maintenance and archival