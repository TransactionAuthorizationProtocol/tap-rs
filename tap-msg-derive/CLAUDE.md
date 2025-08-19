# tap-msg-derive Crate

Derive macros for TAP message types, providing automatic implementations of common traits and functionality for TAP protocol messages.

## Purpose

The `tap-msg-derive` crate provides:
- `#[derive(TapMessage)]` macro for TAP message types
- Automatic trait implementations for messages
- Code generation for message validation
- Serialization and deserialization helpers
- DIDComm integration support

## Key Components

- `lib.rs` - Main derive macro implementation
- Procedural macro for TapMessage trait
- Code generation for message validation
- Trait implementations for common patterns

## Build Commands

```bash
# Build the crate
cargo build -p tap-msg-derive

# Run tests
cargo test -p tap-msg-derive

# Run specific test
cargo test -p tap-msg-derive test_name

# Check macro expansion (useful for debugging)
cargo expand --package tap-msg --example derive_macro_demo
```

## Development Guidelines

### Macro Implementation
- Use `syn` for parsing Rust syntax
- Use `quote!` for code generation
- Handle all edge cases gracefully
- Provide clear error messages for invalid usage
- Support both struct and enum message types

### Code Generation
- Generate idiomatic Rust code
- Include proper error handling
- Support all TAP message patterns
- Maintain backward compatibility
- Follow Rust naming conventions

### Documentation
- Document macro attributes and options
- Provide clear usage examples
- Explain generated code behavior
- Include troubleshooting guides

### Testing
- Test macro expansion output
- Verify generated trait implementations
- Test with various message structures
- Include regression tests
- Test error conditions and edge cases

## TapMessage Derive Macro

The `#[derive(TapMessage)]` macro automatically implements:

### Core Traits
- Message serialization/deserialization
- Validation logic
- DIDComm integration
- Error handling

### Generated Methods
- `validate()` - Message validation
- `message_type()` - Message type identification  
- `to_didcomm()` - DIDComm message conversion
- `from_didcomm()` - DIDComm message parsing

### Supported Attributes
- `#[tap_message(type = "...")]` - Set message type
- `#[tap_message(validate)]` - Enable validation
- `#[tap_message(skip)]` - Skip field in validation
- Custom validation attributes

## Usage Examples

### Basic Message Type
```rust
use tap_msg_derive::TapMessage;

#[derive(TapMessage, Serialize, Deserialize)]
#[tap_message(type = "https://tap.rsvp/schema/1.0#Transfer")]
pub struct Transfer {
    pub id: String,
    pub originator: Party,
    pub beneficiary: Party,
    pub amount: String,
    pub asset: String,
}
```

### Message with Validation
```rust
#[derive(TapMessage, Serialize, Deserialize)]
#[tap_message(type = "https://tap.rsvp/schema/1.0#Payment", validate)]
pub struct Payment {
    #[tap_message(validate = "non_empty")]
    pub amount: String,
    
    #[tap_message(validate = "caip_asset")]
    pub asset: String,
    
    #[tap_message(skip)]
    pub internal_id: Option<String>,
}
```

### Enum Message Types
```rust
#[derive(TapMessage, Serialize, Deserialize)]
pub enum TapMessageType {
    #[tap_message(type = "https://tap.rsvp/schema/1.0#Transfer")]
    Transfer(Transfer),
    
    #[tap_message(type = "https://tap.rsvp/schema/1.0#Payment")]
    Payment(Payment),
}
```

## Generated Code

The derive macro generates implementations for:

### TapMessage Trait
```rust
impl TapMessage for Transfer {
    fn message_type(&self) -> &'static str {
        "https://tap.rsvp/schema/1.0#Transfer"
    }
    
    fn validate(&self) -> Result<(), ValidationError> {
        // Generated validation logic
    }
}
```

### Serialization Support
```rust
impl From<Transfer> for DIDCommMessage {
    fn from(msg: Transfer) -> Self {
        // Generated conversion logic
    }
}
```

## Macro Attributes

### Message Type Specification
- `#[tap_message(type = "url")]` - Set the message type URL
- Required for all TAP messages
- Must be a valid TAP schema URL

### Validation Controls
- `#[tap_message(validate)]` - Enable automatic validation
- `#[tap_message(skip)]` - Skip field in validation
- Custom validation functions supported

### DIDComm Integration
- Automatic DIDComm wrapper generation
- Thread ID handling
- Parent-child message relationships

## Error Handling

The macro provides comprehensive error reporting:
- Clear compilation errors for invalid usage
- Runtime validation errors with context
- Helpful suggestions for common mistakes
- Integration with `thiserror` for error chains

## Dependencies

Core dependencies:
- `proc-macro2` - Procedural macro foundation
- `quote` - Code generation utilities  
- `syn` - Rust syntax parsing

## Testing

The derive macro includes extensive tests:
- Macro expansion verification
- Generated code compilation tests
- Runtime behavior validation
- Integration tests with tap-msg

Run tests with:
```bash
cargo test -p tap-msg-derive
```

## Advanced Usage

### Custom Validation
```rust
#[derive(TapMessage)]
#[tap_message(type = "...", validate = "custom_validator")]
pub struct CustomMessage {
    // fields
}

fn custom_validator(msg: &CustomMessage) -> Result<(), ValidationError> {
    // custom validation logic
}
```

### Conditional Compilation
```rust
#[derive(TapMessage)]
#[tap_message(type = "...")]
#[cfg(feature = "advanced")]
pub struct AdvancedMessage {
    // feature-gated message
}
```

## Debugging

To inspect generated code:
```bash
# Install cargo-expand
cargo install cargo-expand

# View expanded macros
cargo expand --package your-package
```

This helps debug macro issues and understand generated code.