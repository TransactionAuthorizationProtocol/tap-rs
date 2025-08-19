# tap-http Crate

HTTP server implementation for the Transaction Authorization Protocol (TAP), providing REST API endpoints and DIDComm message handling.

## Purpose

The `tap-http` crate provides:
- HTTP/HTTPS server for TAP message transport
- REST API endpoints for TAP operations
- DIDComm v2 message handling over HTTP
- Event logging and monitoring
- Payment simulation tools
- Client libraries for HTTP transport

## Key Components

- `server.rs` - HTTP server implementation with Warp/Hyper
- `handler.rs` - HTTP request handlers and routing
- `client.rs` - HTTP client for TAP communication
- `event.rs` - Event logging and monitoring
- `config.rs` - Server configuration management
- `bin/` - Binary executables
  - `main.rs` - Main HTTP server binary
  - `tap-payment-simulator.rs` - Payment simulation tool

## Build Commands

```bash
# Build the crate
cargo build -p tap-http

# Run tests
cargo test -p tap-http

# Run specific test
cargo test -p tap-http test_name

# Run the HTTP server
cargo run -p tap-http

# Run the payment simulator
cargo run --bin tap-payment-simulator -p tap-http

# Build release version
cargo build -p tap-http --release
```

## Development Guidelines

### Server Implementation
- Use async/await for all HTTP operations
- Implement proper error handling and status codes
- Support both DIDComm v2 and plain JSON endpoints
- Include comprehensive logging and monitoring
- Follow REST API best practices

### Message Handling
- All DIDComm messages must be validated before processing
- Support both encrypted and plaintext messages
- Implement proper content negotiation
- Handle message routing and delivery
- Include timeout and retry mechanisms

### API Design
- Use consistent HTTP status codes
- Include proper error responses with details
- Support versioning in API endpoints
- Implement rate limiting and security headers
- Document all endpoints with OpenAPI/Swagger

### Testing
- Include integration tests for all endpoints
- Test with real HTTP clients and servers
- Mock external dependencies appropriately
- Test error conditions and edge cases
- Include performance and load testing

## HTTP Server Features

### DIDComm Integration
- Full DIDComm v2 message support
- Automatic message unpacking and validation
- Response correlation and threading
- Agent-to-agent message routing

### REST API Endpoints
The server provides REST endpoints for:
- Message sending and receiving
- Transaction status queries
- Agent management operations
- Health checks and monitoring

### Event Logging
- Comprehensive request/response logging
- Transaction event tracking
- Error monitoring and alerting
- Performance metrics collection

## Payment Simulator

The included payment simulator (`tap-payment-simulator`) provides:
- Automated TAP transaction flows
- Load testing capabilities
- Integration testing support
- Performance benchmarking

Usage example:
```bash
cargo run --bin tap-payment-simulator -- \
  --url http://localhost:8000/didcomm \
  --to did:key:z6Mkr1QywNiXj1WbeDu2jVkNEFzBQnfowWh6dCF5QSF6bgiW \
  -v
```

## Configuration

Server configuration supports:
- Port and bind address settings
- TLS/SSL certificate configuration
- CORS policy settings
- Rate limiting parameters
- Storage backend configuration

## Examples

The crate includes practical examples:
- `http_event_logger_demo.rs` - Event logging setup
- Basic server setup and configuration
- Client communication examples

Run examples with:
```bash
cargo run --example http_event_logger_demo -p tap-http
```

## HTTP Transport Features

### Request Handling
- Supports GET, POST, PUT, DELETE methods
- Content-Type negotiation (application/json, application/didcomm)
- Request body validation and parsing
- Response serialization and formatting

### Error Handling
- Structured error responses
- HTTP status code mapping
- Error logging and tracking
- Client-friendly error messages

### Security Features
- Request validation and sanitization
- CORS policy enforcement
- Rate limiting and throttling
- Security headers (CSP, HSTS, etc.)

## Integration with tap-node

The HTTP server integrates closely with `tap-node`:
- Uses tap-node for message processing
- Leverages tap-node storage backend
- Inherits event handling capabilities
- Maintains consistent transaction state

## Client Library

The included HTTP client provides:
- Async HTTP request/response handling
- DIDComm message transport
- Error handling and retries
- Connection pooling and reuse

## Testing

Comprehensive test coverage including:
- Unit tests for individual handlers
- Integration tests with real HTTP servers
- End-to-end message flow tests
- Performance and load tests
- Security and validation tests

Run the full test suite:
```bash
cargo test -p tap-http
```

## Deployment

The HTTP server can be deployed as:
- Standalone binary
- Docker container
- Kubernetes deployment
- Cloud function/lambda

Build for deployment:
```bash
cargo build -p tap-http --release
```

The resulting binary includes all dependencies and can run independently.