# TAP Agent Examples

This directory contains examples demonstrating how to use the TAP Agent crate to implement the Transaction Authorization Protocol (TAP).

## Available Examples

### 1. Basic Transfer Flow (`transfer_flow.rs`)

A simple example demonstrating a complete TAIP-3 transfer flow with TAIP-4 authorization between two agents:
- Originator agent initiates a transfer request
- Beneficiary agent authorizes the transfer
- Originator agent settles the transfer

Run with:
```bash
cargo run --example transfer_flow
```

### 2. Multi-Agent Flow (`multi_agent_flow.rs`)

A more complex scenario with multiple agents participating in the authorization process:
- Originator VASP initiates a transfer request
- Beneficiary VASP and Wallet API both process the request
- Demonstrates rejection handling and recovery
- Shows how multiple agents can collaborate in the authorization process
- Includes settlement with transaction ID confirmation

Run with:
```bash
cargo run --example multi_agent_flow
```

### 3. Secure Transfer Flow (`secure_transfer_flow.rs`)

A comprehensive example with proper security considerations and error handling:
- Proper key management and DID resolution
- Message validation and error handling
- Security mode selection based on message type
- Risk assessment and authorization decision
- Expiry time validation
- Complete transfer flow with proper validation at each step

Run with:
```bash
cargo run --example secure_transfer_flow
```

## Key Concepts Demonstrated

These examples demonstrate several key concepts of the TAP protocol:

1. **Agent Setup**: Creating agents with proper DID resolution and key management
2. **Message Flow**: Implementing the complete TAIP-3 transfer flow with TAIP-4 authorization
3. **Security**: Using appropriate security modes for different message types
4. **Validation**: Validating messages at each step of the process
5. **Error Handling**: Properly handling errors and rejections
6. **Multi-Agent Collaboration**: Showing how multiple agents can participate in the authorization process

## Implementation Notes

The examples follow Rust best practices:
- Proper error handling with the `Result` type
- Clear separation of concerns
- Comprehensive documentation
- Type safety and validation

## Running the Examples

All examples can be run directly using Cargo:

```bash
cargo run --example <example_name>
```

For instance, to run the basic transfer flow example:

```bash
cargo run --example transfer_flow
```

## Additional Resources

For more information about the TAP protocol, refer to the following specifications:
- [TAIP-3: Asset Transfer](https://github.com/TransactionAuthorizationProtocol/TAIPs/blob/main/TAIPs/taip-3.md)
- [TAIP-4: Transaction Authorization Protocol](https://github.com/TransactionAuthorizationProtocol/TAIPs/blob/main/TAIPs/taip-4.md)
- [TAIP-5: Transaction Agents](https://github.com/TransactionAuthorizationProtocol/TAIPs/blob/main/TAIPs/taip-5.md)
- [TAIP-6: Transaction Parties](https://github.com/TransactionAuthorizationProtocol/TAIPs/blob/main/TAIPs/taip-6.md)
