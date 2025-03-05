# Security Best Practices for TAP-RS

This document outlines the essential security best practices for implementing and deploying the Transaction Authorization Protocol (TAP) using the TAP-RS library.

## DID Management

### Securing Private Keys

The security of your TAP implementation fundamentally depends on how private keys are managed:

```rust
// NEVER DO THIS - hardcoded keys are a serious security risk
let hardcoded_key = "abcdef1234567890..."; // SECURITY RISK

// DO THIS INSTEAD - use a secure key management approach
use tap_msg::did::KeyPair;

async fn secure_key_management() -> Result<KeyPair, Box<dyn std::error::Error>> {
    // Option 1: Generate a new key pair securely
    let new_key = KeyPair::generate_ed25519().await?;
    
    // Option 2: Load from a secure storage system with proper access controls
    // let stored_key = SecureStorage::load_key("agent-key-id").await?;
    // let key_pair = KeyPair::from_stored_key(stored_key)?;
    
    // Always use memory protection when possible
    // (TAP-RS does not expose the key in memory outside the KeyPair struct)
    
    Ok(new_key)
}
```

### Key Rotation Practices

Regularly rotating keys is a security best practice:

1. Create a new key pair
2. Register the new DID if necessary
3. Begin using the new key for signing new messages
4. Maintain the old key temporarily to verify messages from parties who haven't updated

```rust
async fn rotate_keys(agent: &mut Agent) -> Result<(), Box<dyn std::error::Error>> {
    // Generate a new key pair
    let new_key_pair = KeyPair::generate_ed25519().await?;
    
    // Update the agent's key
    agent.update_key_pair(Arc::new(new_key_pair)).await?;
    
    // Notify your counterparties about the key change
    // ... notification logic ...
    
    Ok(())
}
```

## Message Security

### Encryption

Always use encrypted DIDComm messages in production:

```rust
async fn send_encrypted_message(
    agent: &Agent,
    recipient_did: &str,
    message_body: impl TapMessageBody,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create the DIDComm message
    let didcomm_message = message_body.to_didcomm()?
        .set_from(Some(agent.did().to_string()))
        .set_to(Some(vec![recipient_did.to_string()]))
        .set_created_time(Some(chrono::Utc::now().to_rfc3339()));
    
    // Encrypt the message using DIDComm encryption
    let encrypted = agent.encrypt_message(didcomm_message, &[recipient_did]).await?;
    
    // Send the encrypted message
    // ... transport logic ...
    
    Ok(())
}
```

### Message Validation

Always validate all incoming messages before processing:

```rust
async fn validate_message(
    message: &Message
) -> Result<(), ValidationError> {
    // Check required fields
    if message.from.is_none() {
        return Err(ValidationError::MissingField("from".to_string()));
    }
    
    // Validate the message type
    if message.type_.is_none() || !message.type_.as_ref().unwrap().starts_with("TAP_") {
        return Err(ValidationError::InvalidType);
    }
    
    // Verify timestamps to prevent replay attacks
    if let Some(created_time) = &message.created_time {
        let created = chrono::DateTime::parse_from_rfc3339(created_time)
            .map_err(|_| ValidationError::InvalidTimestamp)?;
        
        let now = chrono::Utc::now();
        let time_diff = now.signed_duration_since(created.with_timezone(&chrono::Utc));
        
        // Reject messages older than 15 minutes
        if time_diff > chrono::Duration::minutes(15) {
            return Err(ValidationError::MessageTooOld);
        }
    }
    
    // Additional validation based on message type...
    
    Ok(())
}
```

### Preventing Replay Attacks

Implement measures to prevent message replay attacks:

```rust
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

struct MessageTracker {
    processed_ids: Mutex<HashSet<String>>,
}

impl MessageTracker {
    fn new() -> Self {
        Self {
            processed_ids: Mutex::new(HashSet::new()),
        }
    }
    
    fn check_and_mark_processed(&self, message_id: &str) -> bool {
        let mut ids = self.processed_ids.lock().unwrap();
        if ids.contains(message_id) {
            return false; // Already processed
        }
        
        ids.insert(message_id.to_string());
        true // First time seeing this message
    }
}

// Use in your message processor
async fn process_message(
    message: Message,
    tracker: Arc<MessageTracker>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if this message has been processed before
    if !tracker.check_and_mark_processed(&message.id) {
        return Err("Message already processed - potential replay attack".into());
    }
    
    // Continue with normal processing...
    Ok(())
}
```

## Network and Transport Security

### TLS for HTTP Transport

Always use TLS (HTTPS) when using HTTP as a transport:

```rust
use tap_http::{HttpServer, HttpConfig};

async fn create_secure_server() -> Result<(), Box<dyn std::error::Error>> {
    let config = HttpConfig::new()
        .with_tls_cert_path("/path/to/cert.pem")
        .with_tls_key_path("/path/to/key.pem");
    
    let server = HttpServer::new(config);
    server.start().await?;
    
    Ok(())
}
```

### Client Authentication

Implement client authentication for sensitive operations:

```rust
async fn authenticate_request(
    request: &HttpRequest,
    agent_registry: &AgentRegistry,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Extract authentication information (e.g., from headers or body)
    let auth_token = request.headers().get("Authorization")
        .ok_or("Missing Authorization header")?;
    
    // Validate the token (this is a simplified example)
    // In production, use a proper authentication scheme like OAuth or API keys
    if auth_token.starts_with("Bearer ") {
        let token = &auth_token["Bearer ".len()..];
        
        // Verify the token against your authentication system
        // ...
        
        return Ok(true);
    }
    
    Ok(false)
}
```

## Input Validation

### Asset ID Validation

Always validate asset IDs and other critical fields:

```rust
use tap_caip::AssetId;

fn validate_transfer_request(
    asset_id: &str,
    amount: &str,
) -> Result<(), ValidationError> {
    // Validate Asset ID format
    let asset = AssetId::parse(asset_id)
        .map_err(|_| ValidationError::InvalidAssetId)?;
    
    // Validate amount format
    let amount_value = amount.parse::<f64>()
        .map_err(|_| ValidationError::InvalidAmount)?;
    
    if amount_value <= 0.0 {
        return Err(ValidationError::InvalidAmount);
    }
    
    // Additional validation...
    
    Ok(())
}
```

### Safe Deserialization

Be cautious with deserialization from untrusted sources:

```rust
use serde_json::Value;

fn safely_deserialize_message(raw_message: &str) -> Result<Message, Box<dyn std::error::Error>> {
    // First, parse as a generic JSON Value
    let json_value: Value = serde_json::from_str(raw_message)?;
    
    // Validate structure before fully deserializing
    if !json_value.is_object() {
        return Err("Message must be a JSON object".into());
    }
    
    // Check for required fields to avoid panics later
    if !json_value.get("id").is_some() {
        return Err("Message missing required 'id' field".into());
    }
    
    // Check size limits to prevent DoS attacks
    if raw_message.len() > 1_000_000 {  // 1MB limit
        return Err("Message exceeds size limit".into());
    }
    
    // Now deserialize to the actual type
    let message: Message = serde_json::from_value(json_value)?;
    
    Ok(message)
}
```

## Production Deployment Security

### Securing Node Configuration

Protect your node configuration:

```rust
use tap_node::{Node, NodeConfig};

fn create_production_node() -> Node {
    let config = NodeConfig::default()
        .with_log_level("info")  // Don't log sensitive data in production
        .with_max_connections(100)  // Limit connections to prevent DoS
        .with_request_timeout(30)  // Set timeouts to prevent hanging connections
        .with_rate_limiting(true);  // Enable rate limiting
    
    Node::new(config)
}
```

### Audit Logging

Implement comprehensive audit logging:

```rust
use tap_msg::logging::{AuditLogger, LogLevel, LogEvent};

fn setup_audit_logging() -> AuditLogger {
    let logger = AuditLogger::new()
        .with_file_output("/var/log/tap/audit.log")
        .with_level(LogLevel::Info)
        .with_retention_days(90);  // Keep logs for compliance
    
    logger
}

async fn log_message_event(
    logger: &AuditLogger,
    message_id: &str,
    event_type: &str,
    agent_id: &str,
) {
    let event = LogEvent::new()
        .with_timestamp(chrono::Utc::now())
        .with_event_type(event_type)
        .with_message_id(message_id)
        .with_agent_id(agent_id);
    
    logger.log(event).await;
}
```

## WASM Security Considerations

When using TAP-RS in browser environments through WASM, consider these additional security considerations:

### Secure Browser Storage

```typescript
// Store keys securely in the browser
async function securelyStoreKey(key: string, name: string) {
    // Use the browser's built-in crypto APIs
    const encoder = new TextEncoder();
    const keyData = encoder.encode(key);
    
    // Get encryption key from password or other secure source
    const encryptionKey = await window.crypto.subtle.generateKey(
        { name: "AES-GCM", length: 256 },
        true,
        ["encrypt", "decrypt"]
    );
    
    // Encrypt the key before storing
    const iv = window.crypto.getRandomValues(new Uint8Array(12));
    const encrypted = await window.crypto.subtle.encrypt(
        { name: "AES-GCM", iv },
        encryptionKey,
        keyData
    );
    
    // Store safely
    localStorage.setItem(`key_${name}_iv`, Array.from(iv).join(','));
    localStorage.setItem(`key_${name}`, Array.from(new Uint8Array(encrypted)).join(','));
}
```

### Browser Security Headers

Ensure your web server sets appropriate security headers:

```
Content-Security-Policy: default-src 'self'; script-src 'self'
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
Strict-Transport-Security: max-age=31536000; includeSubDomains
```

## Security Testing

Regularly test your TAP-RS implementation for security vulnerabilities:

1. **Fuzzing**: Use tools like `cargo-fuzz` to find potential issues with message parsing:

```rust
// Example fuzz target for message parsing
#[fuzz]
fn fuzz_message_parsing(data: &[u8]) {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = didcomm::Message::receive(s);
    }
}
```

2. **Penetration Testing**: Regularly perform security audits and penetration tests.

3. **Dependency Scanning**: Use tools like `cargo-audit` to check for vulnerabilities in dependencies.

## Conclusion

Security in TAP-RS implementations requires a multi-layered approach. By following these best practices, you can mitigate many common security risks and build a more robust TAP implementation.

Always remember:
- Secure your keys
- Validate all inputs
- Encrypt sensitive data
- Implement proper authentication and authorization
- Keep your dependencies updated
- Log security-relevant events
- Regularly test for vulnerabilities

For questions or security concerns, please open an issue on the [TAP-RS GitHub repository](https://github.com/notabene/tap-rs).
