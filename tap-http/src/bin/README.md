# TAP HTTP Command Line Tools

This directory contains command-line tools for working with the Transaction Authorization Protocol (TAP) via HTTP.

## TAP HTTP Server (`tap-http`)

The main HTTP server binary that provides DIDComm messaging capabilities for TAP. It creates an HTTP endpoint for receiving TAP messages and routes them to the TAP node for processing.

### Key Features

- Creates an ephemeral agent by default for easy testing
- Configurable logging with file rotation
- Structured JSON logging option
- Support for custom agent DIDs and keys
- Health check endpoint for monitoring

### Usage

```bash
# Run with default settings
tap-http

# Run with custom settings
tap-http --host 0.0.0.0 --port 8080 --endpoint /api/didcomm --logs-dir /var/log/tap
```

For complete documentation, see the main [TAP HTTP README](../README.md).

## TAP Payment Simulator (`tap-payment-simulator`)

A tool for simulating TAP payment flows by sending payment request and transfer messages to a TAP HTTP server. This is useful for testing and demonstrating the protocol.

### Key Features

- Creates an ephemeral agent for sending messages
- Simulates a complete payment flow with proper message sequence
- Configurable amount and currency
- Proper DIDComm message packing with signatures
- Follows the TAP protocol specification for message formats

### Usage

```bash
# Run with required parameters
tap-payment-simulator --url http://localhost:8000/didcomm --did did:key:z6Mk...

# Run with custom amount and currency
tap-payment-simulator --url http://localhost:8000/didcomm --did did:key:z6Mk... --amount 500 --currency EUR
```

### Flow Simulation

The payment simulator performs the following steps:

1. Creates an ephemeral agent with a unique DID
2. Generates a unique transaction ID
3. Sends a payment request message to the specified URL
4. Waits for 2 seconds
5. Sends a transfer message with the same transaction ID to complete the flow

This simulates a real-world payment flow where a customer first receives a payment request and then initiates a transfer to fulfill that request.

For complete documentation, see the main [TAP HTTP README](../README.md#creating-a-tap-payment-flow).