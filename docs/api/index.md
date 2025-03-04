# TAP-RS API Reference

This section contains the API reference documentation for the TAP-RS library. Each component of the library has its own dedicated documentation page.

## Core Components

- [tap-core](./tap-core.md) - Core TAP message types and utilities
- [tap-agent](./tap-agent.md) - Agent implementation for TAP
- [tap-node](./tap-node.md) - Node implementation for routing TAP messages

## External Standards

- [tap-caip](./tap-caip.md) - CAIP (Chain Agnostic Improvement Proposals) implementation

## Transport and Integration

- [tap-http](./tap-http.md) - HTTP transport implementation for TAP

## Cross-Platform Support

- [tap-wasm](./tap-wasm.md) - WebAssembly bindings for TAP
- [tap-ts](./tap-ts.md) - TypeScript wrapper library

## Future Development

- [Future Components](./future_components.md) - Planned API components and implementation timeline

## Related Documentation

- [Complete Transfer Flow Example](../examples/complete_transfer_flow.md) - End-to-end example integrating multiple components

## Usage Guidelines

When using the TAP-RS API, consider the following guidelines:

1. **Error Handling**: All functions that can fail return a `Result` type. Always handle these errors appropriately.
2. **Async Functions**: Many operations are asynchronous and require an async runtime like Tokio.
3. **Security**: Always follow the [Security Best Practices](../tutorials/security_best_practices.md) when implementing TAP-RS.
4. **Versioning**: TAP-RS follows semantic versioning. Be aware of breaking changes when upgrading.
5. **Message Flow**: Understand the TAP message flow before implementing your application. See the [Implementing TAP Flows](../tutorials/implementing_tap_flows.md) tutorial for more information.
