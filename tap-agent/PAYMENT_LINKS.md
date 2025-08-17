# Payment Link Support in TAP Agent

This document describes the payment link functionality added to the `tap-agent` crate, implementing Out-of-Band (OOB) messages containing signed Payment messages according to TAIP-14 and TAIP-2.

## Overview

Payment links allow merchants to create shareable URLs that contain Payment requests. These links use the DIDComm Out-of-Band invitation specification to embed signed TAP messages in URLs that can be shared via QR codes, email, or direct links.

## Key Components

### 1. Out-of-Band Messages (`oob.rs`)

- `OutOfBandInvitation`: Main structure following DIDComm v2 OOB specification
- `OutOfBandBuilder`: Builder pattern for creating OOB invitations
- Support for signed message attachments using JWS
- URL encoding/decoding with `_oob` parameter
- Validation according to TAP goal codes (`tap.payment`, `tap.connect`, etc.)

### 2. Payment Links (`payment_link.rs`)

- `PaymentLink`: Complete payment link with URL and metadata
- `PaymentLinkBuilder`: Builder for creating payment links with signing
- `PaymentLinkConfig`: Configuration for service URLs and metadata
- `PaymentLinkInfo`: Information extracted from payment link URLs
- Integration with any TAP message type (not just payments)

### 3. Agent Integration

New methods added to the `Agent` trait:

- `create_oob_invitation()`: Generic OOB creation for any TAP message
- `create_payment_link()`: Specific for Payment messages  
- `parse_oob_invitation()`: Parse OOB URLs
- `process_oob_invitation()`: Extract and verify attached messages

## Usage Examples

### Basic Payment Link Creation

```rust
use tap_agent::{Agent, TapAgent, PaymentLinkConfig};
use tap_msg::message::Payment;

// Create payment message
let payment = Payment { /* ... payment details ... */ };

// Create payment link with default config
let payment_url = agent.create_payment_link(&payment, None).await?;

// Create with custom configuration
let config = PaymentLinkConfig::new()
    .with_service_url("https://pay.mystore.com")
    .with_metadata("order_id", json!("12345"))
    .with_goal("Complete your purchase");

let payment_url = agent.create_payment_link(&payment, Some(config)).await?;
```

### Processing Payment Links

```rust
// Parse OOB invitation from URL
let oob_invitation = agent.parse_oob_invitation(&payment_url)?;

// Extract the signed payment message
let plain_message = agent.process_oob_invitation(&oob_invitation).await?;

// Parse the payment from the message body
let payment: Payment = serde_json::from_value(plain_message.body)?;
```

### Generic OOB Message Creation

```rust
// Create OOB for any TAP message type
let transfer_url = agent.create_oob_invitation(
    &transfer_message,
    "tap.transfer", 
    "Complete asset transfer",
    "https://my-service.com/transfer"
).await?;
```

## Configuration

### Default Service URL

The default service URL is `https://flow-connect.notabene.dev/payin` but can be customized:

```rust
use tap_agent::payment_link::DEFAULT_PAYMENT_SERVICE_URL;

let config = PaymentLinkConfig::new()
    .with_service_url("https://custom-payment-service.com");
```

### Metadata Support

Payment links support arbitrary metadata that gets included in the OOB invitation:

```rust
let config = PaymentLinkConfig::new()
    .with_metadata("theme", json!("dark"))
    .with_metadata("return_url", json!("https://store.com/success"))
    .with_metadata("expires_at", json!("2025-12-31T23:59:59Z"));
```

## URL Format

Payment links follow the DIDComm Out-of-Band specification:

```
https://service.com/path?_oob=eyJ0eXAiOiJKV1QiLCJhbGciOiJFZERTQSJ9...
```

Where `_oob` contains the base64url-encoded OOB invitation with the signed Payment message attached.

## Security Considerations

- All TAP messages in payment links are signed using JWS
- Message verification happens during `process_oob_invitation()`
- Service URLs should use HTTPS for production
- Payment links can include expiration times for security

## Testing

The implementation includes comprehensive tests for:

- OOB invitation creation and validation
- URL encoding/decoding round trips
- Payment link configuration
- Error handling for invalid URLs

Run tests with:
```bash
cargo test --package tap-agent --lib
```

## Examples

See the example files:
- `examples/simple_payment_link.rs` - Configuration examples
- `examples/payment_link_flow.rs` - Full payment flow (may need dependency updates)

## Integration with TAP Node

This payment link functionality is designed to work with both standalone TAP agents and TAP Node deployments. The OOB messages can be processed by any TAP-compatible system that supports DIDComm messaging.