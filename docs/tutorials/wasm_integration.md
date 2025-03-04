# WASM Integration for TAP-RS

This tutorial explains how to use the TAP-RS library in browser and Node.js environments through WebAssembly (WASM).

## Overview

The TAP-RS project provides WASM bindings through two main crates:

1. **tap-wasm**: Core WASM bindings for the TAP message types and functions
2. **tap-ts**: TypeScript wrapper library that provides a more ergonomic API

This integration allows you to:
- Create and process TAP messages in browser applications
- Integrate with web-based wallets and DeFi applications
- Use TAP in Node.js applications

## Prerequisites

Before starting with WASM integration, make sure you have:

- Node.js 14+ installed
- npm or yarn
- Basic knowledge of TypeScript/JavaScript
- Understanding of the basic TAP message types and flows

## Installation

### Using the npm Package

The simplest way to use TAP-RS in a web or Node.js application is via the npm package:

```bash
# Install the TAP-RS TypeScript package
npm install @tap-rs/tap-ts

# Or with yarn
yarn add @tap-rs/tap-ts
```

### Building from Source

If you need to build from source:

1. Make sure you have Rust and wasm-pack installed:

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
cargo install wasm-pack
```

2. Build the WASM package:

```bash
# Clone the repository
git clone https://github.com/notabene/tap-rs.git
cd tap-rs

# Build the WASM package
cd tap-wasm
wasm-pack build --target bundler

# Build the TypeScript package
cd ../tap-ts
npm install
npm run build
```

## Basic Usage in Web Applications

### Importing the TAP-TS Library

```typescript
// Import the TAP-TS library
import { Agent, Message, MessageType } from '@tap-rs/tap-ts';

// Create an agent
const agent = new Agent({
  nickname: 'Browser Wallet Agent',
  // You can provide a custom key resolver or use the default
});

console.log('Agent DID:', agent.did);
```

### Creating and Processing TAP Messages

```typescript
// Create a transfer message
function createTransferMessage(beneficiaryDid: string, amount: string, asset: string) {
  const transfer = new Message({
    type: MessageType.TRANSFER,
  });
  
  transfer.setTransferData({
    asset: asset, // e.g., "eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F"
    amount: amount, // e.g., "100.0"
    originatorDid: agent.did,
    beneficiaryDid: beneficiaryDid,
    memo: "Payment from web application"
  });
  
  return transfer;
}

// Process an incoming message
function processMessage(messageJson: string) {
  try {
    const message = Message.fromJson(messageJson);
    
    console.log('Received message type:', message.type);
    
    switch(message.type) {
      case MessageType.TRANSFER:
        const transferData = message.getTransferData();
        console.log('Transfer request:', transferData);
        
        // Process transfer request
        // ...
        
        // Create an authorize response
        const authorize = new Message({
          type: MessageType.AUTHORIZE,
          correlation: message.id,
        });
        
        authorize.setAuthorizeData({
          note: "Transfer authorized by web application"
        });
        
        return authorize;
        
      case MessageType.AUTHORIZE:
        const authorizeData = message.getAuthorizeData();
        console.log('Authorization received:', authorizeData);
        
        // Handle authorization
        // ...
        break;
        
      // Handle other message types
    }
  } catch (error) {
    console.error('Error processing message:', error);
  }
}
```

## Integration with Web Wallets

Here's how to integrate TAP-RS with a web wallet:

```typescript
import { Agent, Message, MessageType } from '@tap-rs/tap-ts';
import Web3 from 'web3';

class WalletTapIntegration {
  private agent: Agent;
  private web3: Web3;
  
  constructor() {
    // Initialize Web3
    if (window.ethereum) {
      this.web3 = new Web3(window.ethereum);
    } else {
      throw new Error("No Ethereum provider found");
    }
    
    // Create TAP agent
    this.agent = new Agent({
      nickname: 'Web Wallet'
    });
  }
  
  // Connect wallet and get accounts
  async connect() {
    try {
      const accounts = await window.ethereum.request({ method: 'eth_requestAccounts' });
      return accounts[0];
    } catch (error) {
      console.error('Failed to connect wallet:', error);
      throw error;
    }
  }
  
  // Create a transfer message for sending funds
  async createTransfer(beneficiaryDid: string, asset: string, amount: string) {
    const transfer = new Message({
      type: MessageType.TRANSFER,
    });
    
    transfer.setTransferData({
      asset: asset,
      amount: amount,
      originatorDid: this.agent.did,
      beneficiaryDid: beneficiaryDid,
      memo: "Transfer initiated from web wallet"
    });
    
    return transfer;
  }
  
  // Execute an on-chain transaction after receiving authorization
  async executeTransaction(authorizeMessage: Message) {
    try {
      const authorizeData = authorizeMessage.getAuthorizeData();
      const transferData = this.getOriginalTransfer(authorizeMessage.correlation);
      
      if (!transferData) {
        throw new Error("Original transfer data not found");
      }
      
      // Parse the asset ID to get token contract address
      const assetParts = transferData.asset.split(':');
      const tokenAddress = assetParts[assetParts.length - 1];
      
      // Get the current connected account
      const accounts = await this.web3.eth.getAccounts();
      const fromAddress = accounts[0];
      
      // Create contract instance for ERC-20
      const erc20Abi = [/* ERC-20 ABI */];
      const tokenContract = new this.web3.eth.Contract(erc20Abi, tokenAddress);
      
      // Convert amount to the proper format (with decimals)
      const decimals = await tokenContract.methods.decimals().call();
      const amount = this.web3.utils.toBN(parseFloat(transferData.amount) * 10**decimals);
      
      // Get the on-chain address for the beneficiary DID
      const toAddress = this.didToAddress(transferData.beneficiaryDid);
      
      // Send the transaction
      const tx = await tokenContract.methods.transfer(toAddress, amount.toString()).send({
        from: fromAddress
      });
      
      // Create receipt message
      const receipt = new Message({
        type: MessageType.RECEIPT,
        correlation: authorizeMessage.correlation,
      });
      
      receipt.setReceiptData({
        settlementId: tx.transactionHash,
        note: "Settlement transaction completed"
      });
      
      return receipt;
    } catch (error) {
      console.error('Transaction execution failed:', error);
      throw error;
    }
  }
  
  // Helper method to convert DID to Ethereum address
  private didToAddress(did: string): string {
    // This is a simplified example
    // In a real implementation, you would need to properly resolve the DID
    if (did.startsWith('did:pkh:eip155:1:')) {
      return '0x' + did.substring('did:pkh:eip155:1:'.length);
    }
    
    throw new Error("Unsupported DID method for address conversion");
  }
  
  // Get original transfer data for a given message ID
  private getOriginalTransfer(messageId: string) {
    // In a real application, you would store and retrieve this from a database
    // This is just a placeholder
    return this.storedTransfers[messageId];
  }
  
  // Storage for transfers (in a real app, use a proper database)
  private storedTransfers: {[key: string]: any} = {};
}

// Usage
const tapWallet = new WalletTapIntegration();
tapWallet.connect().then(account => {
  console.log('Connected account:', account);
});
```

## Handling WASM Loading

When using TAP-RS in a browser environment, you need to handle WASM loading correctly:

```html
<!DOCTYPE html>
<html>
<head>
  <title>TAP-RS WASM Example</title>
</head>
<body>
  <div id="app">
    <h1>TAP-RS WASM Example</h1>
    <div>
      <button id="createMessage">Create Transfer Message</button>
      <pre id="messageOutput"></pre>
    </div>
  </div>
  
  <script type="module">
    // With bundlers like webpack or Parcel
    import { Agent, Message, MessageType } from '@tap-rs/tap-ts';
    
    // Initialize after the WASM is loaded
    async function initialize() {
      try {
        const agent = new Agent({
          nickname: 'Browser Demo Agent'
        });
        
        document.getElementById('createMessage').addEventListener('click', () => {
          const transfer = new Message({
            type: MessageType.TRANSFER,
          });
          
          transfer.setTransferData({
            asset: "eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F",
            amount: "10.0",
            originatorDid: agent.did,
            beneficiaryDid: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
            memo: "Demo transfer"
          });
          
          const json = transfer.toJson();
          document.getElementById('messageOutput').textContent = json;
        });
        
        console.log('TAP-RS WASM loaded successfully');
        console.log('Agent DID:', agent.did);
      } catch (error) {
        console.error('Failed to initialize TAP-RS:', error);
      }
    }
    
    // Initialize the application
    initialize();
  </script>
</body>
</html>
```

## Usage in Node.js

Using TAP-RS in Node.js is similar to browser usage:

```javascript
// JavaScript/Node.js example
const tap = require('@tap-rs/tap-ts');

// Create an agent
const agent = new tap.Agent({
  nickname: 'Node.js Agent'
});

// Create a simple transfer message
const transfer = new tap.Message({
  type: tap.MessageType.TRANSFER,
});

transfer.setTransferData({
  asset: "eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F",
  amount: "100.0",
  originatorDid: agent.did,
  beneficiaryDid: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  memo: "Node.js test transfer"
});

// Convert to JSON
const json = transfer.toJson();
console.log(json);

// Parse from JSON
const parsedMessage = tap.Message.fromJson(json);
console.log(parsedMessage.getTransferData());
```

## Bundling in Modern Web Applications

### With Webpack

If you're using Webpack, you'll need to configure it to handle WASM imports:

```javascript
// webpack.config.js
module.exports = {
  // ... other webpack configuration ...
  experiments: {
    asyncWebAssembly: true,
  },
  module: {
    rules: [
      {
        test: /\.wasm$/,
        type: 'webassembly/async',
      },
      // ... other rules ...
    ],
  },
};
```

### With Vite

For Vite-based projects:

```javascript
// vite.config.js
import { defineConfig } from 'vite';

export default defineConfig({
  // ... other configuration ...
  optimizeDeps: {
    exclude: ['@tap-rs/tap-ts'],
  },
});
```

## Performance Considerations

WASM modules have some performance characteristics to be aware of:

1. **Initial Load Time**: WASM modules may take longer to load initially compared to pure JavaScript. Consider using a loading indicator during initialization.

2. **Memory Management**: WASM modules manage memory differently. Large message processing might require more memory.

3. **Async Nature**: All WASM functions are inherently asynchronous during loading. Design your application flow to account for this.

Example of handling the asynchronous loading:

```typescript
// Loading state management
let tapInitialized = false;
let pendingMessages = [];

// Initialize TAP-RS
import('@tap-rs/tap-ts').then(tap => {
  const agent = new tap.Agent({
    nickname: 'Async Loading Example'
  });
  
  // Process any pending messages
  pendingMessages.forEach(msg => processMessage(msg));
  pendingMessages = [];
  tapInitialized = true;
}).catch(error => {
  console.error('Failed to load TAP-RS:', error);
});

// Message handler
function handleIncomingMessage(message) {
  if (tapInitialized) {
    processMessage(message);
  } else {
    pendingMessages.push(message);
  }
}

function processMessage(message) {
  // Process with TAP-RS
  // ...
}
```

## Debugging WASM Issues

If you encounter issues with the WASM integration:

1. **Check console errors**: Most WASM loading or execution errors will appear in the browser console.

2. **Verify WASM file loading**: Ensure your bundler correctly handles .wasm files and they're being served with the correct MIME type (application/wasm).

3. **Memory issues**: If you encounter memory-related errors, verify you're not keeping too many large objects in memory.

4. **Cross-origin issues**: WASM files must be served from the same origin or with proper CORS headers.

## Advanced Usage

### Implementing Custom Key Management

```typescript
import { Agent, KeyManager, KeyResolver } from '@tap-rs/tap-ts';

// Create a custom key manager that integrates with a web wallet
class WalletKeyManager implements KeyManager {
  private wallet: any; // Your wallet interface
  
  constructor(wallet) {
    this.wallet = wallet;
  }
  
  async sign(data: Uint8Array): Promise<Uint8Array> {
    // Use wallet to sign data
    const signature = await this.wallet.sign(data);
    return new Uint8Array(signature);
  }
  
  async getPublicKey(): Promise<Uint8Array> {
    // Get public key from wallet
    const pubKey = await this.wallet.getPublicKey();
    return new Uint8Array(pubKey);
  }
  
  // Other required methods...
}

// Create a custom resolver that can handle your DIDs
class CustomKeyResolver implements KeyResolver {
  async resolveKey(did: string): Promise<Uint8Array | null> {
    // Implement custom DID resolution logic
    if (did.startsWith('did:pkh:')) {
      // Extract public key from blockchain address
      // ...
      return publicKey;
    }
    
    // Default to standard resolution for other DIDs
    return null;
  }
}

// Use custom implementations
const wallet = getWalletInstance(); // Your wallet implementation
const keyManager = new WalletKeyManager(wallet);
const keyResolver = new CustomKeyResolver();

const agent = new Agent({
  nickname: 'Custom Key Agent',
  keyManager,
  keyResolver,
});
```

## Next Steps

- Explore [Implementing TAP Flows](./implementing_tap_flows.md) for building complete message flows
- Review [Security Best Practices](./security_best_practices.md) for securing your WASM implementation
- Check out the [API Reference](../api/index.md) for detailed information on the TypeScript API

For questions or support, please open an issue on the [TAP-RS GitHub repository](https://github.com/notabene/tap-rs).
