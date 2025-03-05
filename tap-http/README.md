# TAP HTTP

HTTP DIDComm server implementation for the Transaction Authorization Protocol (TAP).

## Features

- **DIDComm HTTP Endpoint**: Exposes a secure HTTP endpoint for DIDComm messaging
- **Integration with tap-node**: Seamlessly forwards messages to a tap-node instance
- **Message Validation**: Validates incoming DIDComm messages
- **Response Handling**: Proper handling of responses and acknowledgments
- **Outgoing Message Delivery**: HTTP client for sending outgoing DIDComm messages
- **Security**: Support for HTTPS/TLS and basic rate limiting

## Usage

```rust
use tap_http::server::TapHttpServer;
use tap_node::node::{TapNode, DefaultTapNode};
use tap_agent::agent::{Agent, DefaultAgent};
use tap_agent::config::AgentConfig;
use tap_agent::crypto::{DefaultMessagePacker, BasicSecretResolver};
use tap_agent::did::DefaultDIDResolver;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up DID and secret resolvers
    let did_resolver = Arc::new(DefaultDIDResolver::new());
    let secret_resolver = Arc::new(BasicSecretResolver::new());

    // Create a TAP node
    let mut node = DefaultTapNode::new(did_resolver.clone(), secret_resolver.clone());

    // Create and register an agent
    let config = AgentConfig::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string());
    let message_packer = Arc::new(DefaultMessagePacker::new(did_resolver, secret_resolver));
    let agent = Arc::new(DefaultAgent::new(config, message_packer));
    node.register_agent(agent).await?;

    // Create the HTTP server
    let server = TapHttpServer::new(node);

    // Start the server
    let addr = ([127, 0, 0, 1], 3000).into();
    println!("Starting TAP HTTP server on {}", addr);
    server.start(addr).await?;

    Ok(())
}
```

## HTTP Endpoints

### POST /didcomm

The main endpoint for receiving DIDComm messages:

```http
POST /didcomm HTTP/1.1
Host: example.com
Content-Type: application/json

{
  "protected": "eyJhbGciOiJFZERTQSIsImt...",
  "payload": "eyJ0eXBlIjoiaHR0cHM6Ly90...",
  "signature": "FW33NnvOHV0Ted9-F_...",
}
```

### Response

For successfully processed messages, the server returns:

```http
HTTP/1.1 202 Accepted
Content-Type: application/json

{
  "status": "accepted",
  "message_id": "1234-5678-90ab-cdef"
}
```

For errors:

```http
HTTP/1.1 400 Bad Request
Content-Type: application/json

{
  "error": "invalid_message",
  "message": "Invalid message format: missing signature"
}
```

## Message Flow

1. Client sends a DIDComm message to the `/didcomm` endpoint
2. Server validates the message format
3. The message is forwarded to the TAP node
4. The node routes the message to the appropriate agent
5. Agent processes the message
6. Server returns an acknowledgment response

## Security Considerations

- The server supports HTTPS for secure communication
- Incoming messages should be authenticated via DIDComm signatures
- Configure proper rate limiting to prevent abuse
- Consider running behind a reverse proxy for additional security

## Configuration

The server can be configured with the following options:

```rust
let server_config = ServerConfig {
    cors_origins: vec!["https://example.com".to_string()],
    rate_limit: Some(RateLimit {
        requests_per_minute: 100,
        burst_size: 20,
    }),
    tls_config: Some(TlsConfig {
        cert_path: "/path/to/cert.pem".to_string(),
        key_path: "/path/to/key.pem".to_string(),
    }),
};

let server = TapHttpServer::with_config(node, server_config);
```

## Examples

See the [examples directory](./examples) for more detailed usage examples.
