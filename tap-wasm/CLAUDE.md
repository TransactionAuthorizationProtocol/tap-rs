# tap-wasm Crate

WebAssembly bindings for the Transaction Authorization Protocol (TAP), enabling TAP functionality in web browsers and JavaScript environments.

## Purpose

The `tap-wasm` crate provides:
- WASM bindings for TAP agent functionality
- JavaScript/TypeScript compatible API
- Browser-compatible TAP message operations
- Key management and DID operations in WASM
- Integration with existing TAP agent infrastructure

## Key Components

- `wasm_agent.rs` - WASM wrapper around TapAgent
- `lib.rs` - Main WASM library exports
- `util.rs` - WASM utility functions
- `pkg/` - Generated WASM package for npm
- `examples/` - Browser integration examples

## Build Commands

```bash
# Build WASM package
wasm-pack build --target web --out-dir pkg

# Build with specific target
wasm-pack build --target nodejs --out-dir pkg-node

# Build for bundlers
wasm-pack build --target bundler --out-dir pkg-bundler

# Run WASM tests
wasm-pack test --headless --chrome

# Run WASM tests in Firefox
wasm-pack test --headless --firefox

# Run native tests
cargo test -p tap-wasm

# Run benchmarks (native only)
cargo bench -p tap-wasm
```

## Development Guidelines

### WASM API Design
- Keep APIs simple and JavaScript-friendly
- Use `JsValue` for complex types when necessary
- Provide both sync and async variants where appropriate
- Include comprehensive error handling
- Document all exported functions clearly

### Memory Management
- Use `wee_alloc` for smaller WASM binaries
- Avoid memory leaks with proper cleanup
- Handle large data transfers efficiently
- Use streaming for large message processing

### Browser Compatibility
- Test in all major browsers (Chrome, Firefox, Safari, Edge)
- Handle different WASM feature support levels
- Provide fallbacks for unsupported features
- Ensure consistent behavior across environments

### JavaScript Integration
- Provide TypeScript definitions
- Include proper error handling
- Support both Promise and callback patterns
- Enable tree-shaking for smaller bundles

## WASM Exports (Simplified API)

The WASM module now focuses exclusively on cryptographic operations that cannot be done in JavaScript:

### Agent Management
- `WasmTapAgent::new(config)` - Create new agent with auto-generated keys
- `WasmTapAgent::from_private_key(hex, key_type)` - Create from existing private key
- `get_did()` - Get agent's DID
- `export_private_key()` - Export private key as hex string
- `export_public_key()` - Export public key as hex string
- `nickname()` - Get agent's nickname if set

### Cryptographic Operations
- `pack_message(message)` - Sign and pack messages (returns Promise)
- `unpack_message(packed, expected_type)` - Verify and unpack messages (returns Promise)
- DIDComm v2 General JWS JSON format support
- Signature generation and verification

### Utility Functions
- `generate_private_key(key_type)` - Generate new private keys as hex
- `generate_uuid()` - Generate UUID v4
- `generate_uuid_v4()` - Generate UUID v4 (explicit version)
- All key types supported: Ed25519, P256, Secp256k1

### Type Conversions
- JavaScript object → TAP PlainMessage conversion
- TAP PlainMessage → JavaScript object conversion
- Automatic timestamp conversions (JS Date.now() ↔ Unix timestamps)
- Array handling for 'to' recipients
- Nested object support in message bodies
- Unicode and special character support
- Null/undefined field handling

## Features

- `console_error_panic_hook` (default) - Better error reporting in browsers
- Conditional compilation for WASM vs native builds
- Optimized for small bundle sizes

## Package Structure

The generated `pkg/` directory contains:
- `tap_wasm.js` - JavaScript bindings
- `tap_wasm_bg.wasm` - Compiled WASM binary
- `tap_wasm.d.ts` - TypeScript definitions
- `package.json` - npm package configuration

## Examples

### Browser Integration
```html
<!DOCTYPE html>
<script type="module">
import init, { WasmTapAgent } from './pkg/tap_wasm.js';

async function run() {
  await init();
  const agent = WasmTapAgent.new();
  console.log('Agent DID:', agent.get_did());
}
run();
</script>
```

### Node.js Integration
```javascript
const { WasmTapAgent } = require('./pkg-node/tap_wasm.js');

const agent = WasmTapAgent.new();
console.log('Agent DID:', agent.get_did());
```

### TypeScript Usage
```typescript
import { WasmTapAgent } from '@taprsvp/agent';

const agent = WasmTapAgent.new();
const did: string = agent.get_did();
```

## Bundle Optimization

The WASM build is optimized for size:
- Uses `wee_alloc` for smaller memory footprint
- Link-time optimization (LTO) enabled
- Dead code elimination
- Optimized for `opt-level = "s"` (size)

Target sizes:
- WASM binary: < 500KB
- JavaScript wrapper: < 50KB gzipped

## Browser Compatibility

Supported browsers:
- Chrome/Chromium 57+
- Firefox 52+
- Safari 11+
- Edge 16+

Required features:
- WebAssembly support
- ES6 modules (for modern builds)
- Crypto API (for random number generation)

## Testing

Multiple testing approaches:
- Unit tests in Rust with `wasm-bindgen-test`
- Browser integration tests with real browsers
- Node.js compatibility tests
- Performance benchmarks

### Test Coverage (60 tests total)

**WASM Bindings Tests** (11 tests)
- Agent creation with default config
- Agent creation with debug mode
- Agent creation with nickname
- Agent creation from Ed25519/P256/Secp256k1 private keys
- Private key export/import roundtrip
- Public key export
- Error handling for invalid keys

**Async Operations Tests** (9 tests)
- Promise-based pack/unpack operations
- Concurrent message operations
- Async error propagation
- Promise rejection handling

**Message Operations Tests** (10 tests)
- Pack/unpack message delegation
- All TAP message types (Transfer, Payment, Authorize, Reject, Cancel)
- Message type validation
- Error handling for invalid messages

**Utility Functions Tests** (10 tests)
- UUID v4 generation and uniqueness
- Private key generation for all key types
- Hex encoding correctness
- Key usability with agents

**Type Conversion Tests** (6 tests)
- JavaScript to Rust message conversion
- Rust to JavaScript message conversion
- Null/undefined handling
- Number conversions and timestamps
- String encoding with special characters
- Array conversions

**Additional Test Files** (14 tests)
- WASM agent tests
- Simple pack test
- Key management tests
- Private key operations

Run tests:
```bash
# Browser tests
wasm-pack test --headless --chrome

# Node.js tests  
wasm-pack test --nodejs

# All tests
cargo test -p tap-wasm
```

## Performance

The WASM implementation provides:
- Native-speed cryptographic operations
- Efficient message packing/unpacking
- Minimal JavaScript ↔ WASM overhead
- Optimized memory usage

Benchmark performance:
```bash
cargo bench -p tap-wasm --bench wasm_binding_benchmark
```

## Deployment

### npm Package
The generated package can be published to npm:
```bash
cd pkg
npm publish
```

### CDN Distribution
WASM files can be served from CDNs:
- Proper MIME types (`application/wasm`)
- CORS headers for cross-origin access
- Compression (gzip/brotli) support

### Integration Patterns
- ES6 modules for modern browsers
- CommonJS for Node.js environments
- UMD builds for legacy compatibility
- Webpack/Vite integration support