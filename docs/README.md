# TAP-RS Documentation

This directory contains the official documentation for the TAP-RS project, a Rust implementation of the Transaction Authorization Protocol (TAP).

## Table of Contents

### Tutorials

Step-by-step guides to get you started with TAP-RS:

- [Getting Started](./tutorials/getting_started.md) - Learn how to set up and start using TAP-RS
- [Implementing TAP Flows](./tutorials/implementing_tap_flows.md) - Comprehensive guide on implementing various TAP message flows
- [Security Best Practices](./tutorials/security_best_practices.md) - Guidelines for securing your TAP-RS implementation
- [WASM Integration](./tutorials/wasm_integration.md) - How to use TAP-RS in browser and Node.js environments through WebAssembly

### API Reference

Detailed technical documentation for each of the TAP-RS crates:

- [API Reference Index](./api/index.md) - Overview of all API documentation
- [tap-core](./api/tap-core.md) - Core TAP message types and utilities
- [tap-agent](./api/tap-agent.md) - Participant implementation for TAP
- [tap-node](./api/tap-node.md) - Node implementation for routing TAP messages
- [tap-caip](./api/tap-caip.md) - CAIP (Chain Agnostic Improvement Proposals) implementation
- [tap-http](./api/tap-http.md) - HTTP transport implementation
- [tap-wasm](./api/tap-wasm.md) - WebAssembly bindings
- [tap-ts](./api/tap-ts.md) - TypeScript wrapper library

### Examples

Comprehensive examples demonstrating TAP-RS functionality:

- [Complete Transfer Flow](./examples/complete_transfer_flow.md) - End-to-end example of a transfer flow involving multiple components

## Quick Links

- [Product Requirements Document](../prds/v1.md) - The original product requirements document for TAP-RS
- [GitHub Repository](https://github.com/notabene/tap-rs) - The main GitHub repository for the TAP-RS project

## Contributing

We welcome contributions to TAP-RS and its documentation. Please see the [Contributing Guide](../CONTRIBUTING.md) for more information.

## License

TAP-RS is licensed under the [MIT License](../LICENSE).

## Getting Help

If you need help with TAP-RS, please open an issue on the [GitHub repository](https://github.com/notabene/tap-rs).
