# tap-http API Reference

The `tap-http` crate provides HTTP transport implementations for the TAP protocol. It allows TAP nodes and agents to communicate over HTTP, which is especially useful for web-based applications and services.

## Server Components

### `TapServer`

A web server that can receive and process TAP messages.

```rust
pub struct TapServer {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new TapServer with the given node
pub fn new(node: Arc<dyn Node>) -> Self;

/// Start the server on the specified address and port
pub async fn start(&self, address: &str, port: u16) -> Result<(), Error>;

/// Stop the server
pub async fn stop(&self) -> Result<(), Error>;

/// Get a reference to the node
pub fn node(&self) -> &Arc<dyn Node>;

/// Add middleware to the server
pub fn with_middleware<M>(&mut self, middleware: M) -> &mut Self
where
    M: Middleware + Send + Sync + 'static;

/// Configure CORS for the server
pub fn with_cors(&mut self, origins: Vec<String>) -> &mut Self;

/// Configure authentication for the server
pub fn with_auth<A>(&mut self, auth: A) -> &mut Self
where
    A: Auth + Send + Sync + 'static;
```

### `ServerConfig`

Configuration options for creating a TapServer.

```rust
pub struct ServerConfig {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new default ServerConfig
pub fn new() -> Self;

/// Set the address for the server to listen on
pub fn with_address(mut self, address: String) -> Self;

/// Set the port for the server to listen on
pub fn with_port(mut self, port: u16) -> Self;

/// Configure CORS for the server
pub fn with_cors(mut self, origins: Vec<String>) -> Self;

/// Configure the maximum request body size
pub fn with_max_body_size(mut self, size: usize) -> Self;

/// Configure authentication for the server
pub fn with_auth<A>(mut self, auth: A) -> Self
where
    A: Auth + Send + Sync + 'static;
```

## Client Components

### `TapClient`

A client for sending TAP messages to a server.

```rust
pub struct TapClient {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new TapClient with the given base URL
pub fn new(base_url: String) -> Self;

/// Send a message to the server
pub async fn send_message(&self, message: Message) -> Result<Option<Message>, Error>;

/// Register an agent with the server
pub async fn register_agent(&self, did: String, endpoint: String) -> Result<(), Error>;

/// Unregister an agent from the server
pub async fn unregister_agent(&self, did: String) -> Result<(), Error>;

/// Set the authorization token for requests
pub fn with_auth_token(mut self, token: String) -> Self;

/// Set a custom HTTP client
pub fn with_client(mut self, client: Client) -> Self;
```

### `ClientConfig`

Configuration options for creating a TapClient.

```rust
pub struct ClientConfig {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new default ClientConfig
pub fn new() -> Self;

/// Set the base URL for the client
pub fn with_base_url(mut self, base_url: String) -> Self;

/// Set the authorization token for requests
pub fn with_auth_token(mut self, token: String) -> Self;

/// Set the timeout for requests
pub fn with_timeout(mut self, timeout: Duration) -> Self;

/// Configure TLS for the client
pub fn with_tls_config(mut self, config: TlsConfig) -> Self;
```

## Middleware

### `Middleware`

A trait for implementing server middleware.

```rust
pub trait Middleware: Send + Sync {
    /// Process a request before it reaches the handler
    fn process_request(&self, request: Request<Body>) -> Result<Request<Body>, Response<Body>>;
    
    /// Process a response before it is sent to the client
    fn process_response(&self, response: Response<Body>) -> Response<Body>;
}
```

#### Example Middleware Implementations

```rust
/// Logging middleware that logs requests and responses
pub struct LoggingMiddleware {
    // Internal implementation details
}

impl Middleware for LoggingMiddleware {
    fn process_request(&self, request: Request<Body>) -> Result<Request<Body>, Response<Body>> {
        println!("Request: {:?}", request);
        Ok(request)
    }
    
    fn process_response(&self, response: Response<Body>) -> Response<Body> {
        println!("Response: {:?}", response);
        response
    }
}

/// Rate limiting middleware
pub struct RateLimitingMiddleware {
    // Internal implementation details
}

impl Middleware for RateLimitingMiddleware {
    fn process_request(&self, request: Request<Body>) -> Result<Request<Body>, Response<Body>> {
        // Rate limiting logic here
        // ...
        
        Ok(request)
    }
    
    fn process_response(&self, response: Response<Body>) -> Response<Body> {
        response
    }
}
```

## Authentication

### `Auth`

A trait for implementing server authentication.

```rust
pub trait Auth: Send + Sync {
    /// Authenticate a request
    fn authenticate(&self, request: &Request<Body>) -> Result<(), Error>;
}
```

#### Example Auth Implementations

```rust
/// JWT authentication
pub struct JwtAuth {
    // Internal implementation details
}

impl Auth for JwtAuth {
    fn authenticate(&self, request: &Request<Body>) -> Result<(), Error> {
        // JWT validation logic here
        // ...
        
        Ok(())
    }
}

/// API key authentication
pub struct ApiKeyAuth {
    // Internal implementation details
}

impl Auth for ApiKeyAuth {
    fn authenticate(&self, request: &Request<Body>) -> Result<(), Error> {
        // API key validation logic here
        // ...
        
        Ok(())
    }
}
```

## Request Handlers

### `MessageHandler`

A handler for processing incoming TAP messages.

```rust
pub async fn handle_message(
    req: Request<Body>,
    node: Arc<dyn Node>,
) -> Result<Response<Body>, Error>;
```

### `AgentRegistrationHandler`

A handler for registering and unregistering agents.

```rust
pub async fn handle_agent_registration(
    req: Request<Body>,
    node: Arc<dyn Node>,
) -> Result<Response<Body>, Error>;

pub async fn handle_agent_unregistration(
    req: Request<Body>,
    node: Arc<dyn Node>,
) -> Result<Response<Body>, Error>;
```

## Error Handling

```rust
pub enum Error {
    /// Error from the core TAP library
    Core(tap_core::error::Error),
    
    /// Error from the node library
    Node(tap_node::Error),
    
    /// Error related to HTTP processing
    Http(hyper::Error),
    
    /// Error related to serialization
    Serialization(serde_json::Error),
    
    /// Error related to authentication
    Authentication(String),
    
    /// Error related to middleware
    Middleware(String),
    
    /// General error
    General(String),
}
```

## Examples

### Creating a TAP Server

```rust
use tap_http::{TapServer, ServerConfig};
use tap_node::{Node, NodeConfig};
use std::sync::Arc;

async fn create_server() -> Result<(), Box<dyn std::error::Error>> {
    // Create a node
    let node_config = NodeConfig::new()
        .with_max_agents(100)
        .with_logging(true);
    let node = Node::new(node_config);
    
    // Create a server
    let server_config = ServerConfig::new()
        .with_address("127.0.0.1".to_string())
        .with_port(8080)
        .with_cors(vec!["*".to_string()]);
    
    let mut server = TapServer::new(Arc::new(node));
    
    // Add middleware
    server.with_middleware(LoggingMiddleware::new());
    
    // Start the server
    server.start("127.0.0.1", 8080).await?;
    
    println!("TAP server started on http://127.0.0.1:8080");
    
    // Server will continue running...
    
    Ok(())
}
```

### Creating a TAP Client

```rust
use tap_http::{TapClient, ClientConfig};
use tap_core::message::TransferBody;
use didcomm::Message;
use std::time::Duration;

async fn create_client() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client_config = ClientConfig::new()
        .with_base_url("http://127.0.0.1:8080".to_string())
        .with_timeout(Duration::from_secs(30));
    
    let client = TapClient::new("http://127.0.0.1:8080".to_string());
    
    // Register an agent
    client.register_agent(
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        "http://my-endpoint.example.com".to_string()
    ).await?;
    
    // Send a message
    let message = Message::new()
        .set_type(Some("TAP_TRANSFER".to_string()))
        .set_body(Some(TransferBody::new(/* ... */)));
    
    let response = client.send_message(message).await?;
    
    // Process the response
    if let Some(resp) = response {
        println!("Received response with ID: {}", resp.id);
    }
    
    Ok(())
}
```

### Implementing a Custom Auth Provider

```rust
use tap_http::{Auth, Error};
use hyper::{Request, Body};
use std::collections::HashSet;

struct AllowedDidsAuth {
    allowed_dids: HashSet<String>,
}

impl AllowedDidsAuth {
    fn new(allowed_dids: Vec<String>) -> Self {
        Self {
            allowed_dids: allowed_dids.into_iter().collect(),
        }
    }
}

impl Auth for AllowedDidsAuth {
    fn authenticate(&self, request: &Request<Body>) -> Result<(), Error> {
        // Extract the DID from the request (e.g., from headers or JWT)
        let did = match request.headers().get("X-TAP-DID") {
            Some(did_header) => match did_header.to_str() {
                Ok(did) => did.to_string(),
                Err(_) => return Err(Error::Authentication("Invalid DID header".to_string())),
            },
            None => return Err(Error::Authentication("Missing DID header".to_string())),
        };
        
        // Check if the DID is allowed
        if self.allowed_dids.contains(&did) {
            Ok(())
        } else {
            Err(Error::Authentication(format!("DID {} not allowed", did)))
        }
    }
}

// Using the custom auth provider
async fn create_server_with_custom_auth() -> Result<(), Box<dyn std::error::Error>> {
    // Create a node
    let node = Node::new(NodeConfig::new());
    
    // Create the custom auth provider
    let auth = AllowedDidsAuth::new(vec![
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH".to_string(),
    ]);
    
    // Create a server with the custom auth
    let mut server = TapServer::new(Arc::new(node));
    server.with_auth(auth);
    
    // Start the server
    server.start("127.0.0.1", 8080).await?;
    
    println!("TAP server with custom auth started on http://127.0.0.1:8080");
    
    Ok(())
}
```

## Server Endpoints

The default TAP server exposes the following endpoints:

### `/messages`

- **POST**: Receive and process a TAP message
  - Request Body: A DIDComm message in JSON format
  - Response: 
    - 200 OK with an optional response message if successful
    - 400 Bad Request if the message is invalid
    - 401 Unauthorized if authentication fails
    - 500 Internal Server Error if processing fails

### `/agents`

- **POST**: Register an agent with the server
  - Request Body: JSON object with `did` and `endpoint` fields
  - Response:
    - 201 Created if the agent is registered successfully
    - 400 Bad Request if the request is invalid
    - 401 Unauthorized if authentication fails
    - 409 Conflict if the agent is already registered
    - 500 Internal Server Error if registration fails

- **DELETE**: Unregister an agent from the server
  - Query Parameter: `did` - the DID of the agent to unregister
  - Response:
    - 204 No Content if the agent is unregistered successfully
    - 400 Bad Request if the request is invalid
    - 401 Unauthorized if authentication fails
    - 404 Not Found if the agent is not registered
    - 500 Internal Server Error if unregistration fails

## Deployment Considerations

When deploying a TAP server in production, consider the following:

1. **TLS**: Always use HTTPS in production environments to secure message transport.
2. **Authentication**: Implement proper authentication for both the server and clients.
3. **Rate Limiting**: Add rate limiting to prevent abuse of the API.
4. **Logging**: Configure appropriate logging for monitoring and debugging.
5. **Load Balancing**: For high-traffic deployments, consider using a load balancer.
6. **Health Checks**: Implement health check endpoints for monitoring.
7. **Metrics**: Add metrics collection for monitoring performance and usage.

Example of a production-ready server configuration:

```rust
use tap_http::{TapServer, ServerConfig, middleware::RateLimitingMiddleware};
use tap_node::{Node, NodeConfig};
use std::sync::Arc;

async fn create_production_server() -> Result<(), Box<dyn std::error::Error>> {
    // Create a node with production-ready configuration
    let node_config = NodeConfig::new()
        .with_max_agents(1000)
        .with_logging(true)
        .with_composite_processor()
        .with_composite_router();
    
    let node = Node::new(node_config);
    
    // Create a server with production-ready configuration
    let server_config = ServerConfig::new()
        .with_address("0.0.0.0".to_string())  // Listen on all interfaces
        .with_port(443)  // HTTPS port
        .with_cors(vec![
            "https://example.com".to_string(),
            "https://api.example.com".to_string(),
        ])
        .with_max_body_size(1024 * 1024)  // 1 MB max body size
        .with_auth(JwtAuth::new("your-jwt-secret"));
    
    let mut server = TapServer::new(Arc::new(node));
    
    // Add middleware
    server.with_middleware(LoggingMiddleware::new());
    server.with_middleware(RateLimitingMiddleware::new(100, Duration::from_secs(60)));  // 100 req/min
    
    // Start the server with TLS
    server.start_with_tls("0.0.0.0", 443, "path/to/cert.pem", "path/to/key.pem").await?;
    
    println!("Production TAP server started on https://0.0.0.0:443");
    
    Ok(())
}
```
