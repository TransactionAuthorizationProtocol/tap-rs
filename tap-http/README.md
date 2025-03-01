# tap-http

A HTTP server implementation for the Transaction Authorization Protocol (TAP) that uses the DIDComm v2 protocol for message exchange.

## Features

- Exposes a HTTP endpoint to receive DIDComm messages (`/didcomm`)
- Forwards received messages to a TAP Node for processing
- Outbound message delivery to external endpoints
- Implements response handling according to DIDComm conventions
- Provides health checking endpoint (`/health`)
- Configurable for TLS and rate limiting

## Usage

### Server Setup

```rust
use tap_http::{TapHttpConfig, TapHttpServer};
use tap_node::{TapNode, NodeConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Configure the HTTP server
    let config = TapHttpConfig {
        host: "127.0.0.1".to_string(),
        port: 8000,
        didcomm_endpoint: "/didcomm".to_string(),
        ..TapHttpConfig::default()
    };

    // Create a TAP Node to handle message processing
    let node = TapNode::new(NodeConfig::default());
    
    // Create and start the HTTP server
    let mut server = TapHttpServer::new(config, node);
    server.start().await?;
    
    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    println!("Shutting down server...");
    
    // Stop the server
    server.stop().await?;
    
    Ok(())
}
```

### Outbound Message Delivery

You can also use the included `DIDCommClient` to deliver DIDComm messages to external endpoints:

```rust
use tap_http::DIDCommClient;

async fn send_message(endpoint: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create a client with a custom timeout (in seconds)
    let client = DIDCommClient::new(Some(30));
    
    // Deliver the DIDComm message
    client.deliver_message(endpoint, message).await?;
    
    Ok(())
}
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check endpoint that returns status and version information |
| `/didcomm` | POST | Receives DIDComm messages for processing by the TAP Node |

## Security Considerations

- TLS encryption should be configured in production environments
- Authentication can be enabled for secure environments
- Rate limiting is available to prevent abuse
- Set appropriate timeouts for outbound message delivery

## Testing

Run the tests with:

```bash
cargo test --package tap-http
```

## License

MIT
