# TAP-WASM Examples

This directory contains examples demonstrating how to use the TAP-WASM library.

## Browser Example

The `browser-example.html` file demonstrates how to use TAP-WASM in a browser environment.

### Running the Example

1. Build the WebAssembly package:

```bash
cd /Users/pelle/code/notabene/tap-rs/tap-wasm
wasm-pack build --target web
```

2. Start the HTTP server:

```bash
./examples/serve-example.sh
```

3. Open your browser and navigate to:

```
http://localhost:8000/examples/browser-example.html
```

### What the Example Demonstrates

The browser example demonstrates:

- Initializing the WASM module
- Creating a TAP agent
- Creating a TAP message (Transfer)
- Setting message properties
- Signing a message

## Creating Your Own Examples

To create your own examples, you can use the existing examples as a template. The key steps are:

1. Import the WASM module:

```javascript
import init, { 
  init_tap_wasm, 
  Message, 
  TapAgent, 
  MessageType 
} from '../pkg/tap_wasm.js';
```

2. Initialize the module:

```javascript
await init();
init_tap_wasm();
```

3. Use the WASM functions and classes as needed.

## Troubleshooting

- **CORS errors**: Make sure you're serving the files from an HTTP server, not opening them directly in the browser.
- **Module loading errors**: Ensure the path to the WASM module is correct.
- **WASM import errors**: Make sure you've built the WASM package with the correct target (`--target web` for browsers).