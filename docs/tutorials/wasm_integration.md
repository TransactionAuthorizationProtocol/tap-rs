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
import { TAPAgent, Message, MessageType } from '@taprsvp/tap-agent';

// Create a participant using the static factory method
// IMPORTANT: Always use the async create() method, NOT the constructor directly
async function setupParticipant() {
  // This ensures WASM is properly initialized before agent creation
  const participant = await TAPAgent.create({
    nickname: 'Browser Wallet Participant',
    // You can provide a custom key resolver or use the default
  });

  console.log('Participant DID:', participant.did);
  return participant;
}

// Call the async function
setupParticipant();
```

### Creating and Processing TAP Messages

```typescript
// Create a transfer message
async function createTransferMessage(participant, beneficiaryDid: string, amount: string, asset: string) {
  // Ensure we have a valid TAPAgent instance
  if (!participant) {
    participant = await TAPAgent.create({ nickname: "Default Participant" });
  }
  
  // Using the transfer helper method directly on the agent
  const transfer = participant.transfer({
    asset: asset, // e.g., "eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F"
    amount: amount, // e.g., "100.0"
    originator: {
      '@id': participant.did,
      role: "originator"
    },
    beneficiary: {
      '@id': beneficiaryDid,
      role: "beneficiary"
    },
    memo: "Payment from web application"
  });
  
  return transfer;
}

// Process an incoming message
async function processMessage(participantPromise, messageJson: string) {
  try {
    // Ensure we have a valid TAPAgent instance
    const participant = await participantPromise;
    
    // Parse the message
    const message = await participant.parseMessage(messageJson);
    
    console.log('Received message type:', message.type);
    
    switch(message.type) {
      case "https://tap.rsvp/schema/1.0#Transfer":
        console.log('Transfer request:', message.body);
        
        // Process transfer request
        // ...
        
        // Create an authorize response
        const authorize = participant.authorize({
          reason: "Transfer authorized by web application"
        });
        
        // Link it to the original message
        authorize.setThreadId(message.id);
        
        return authorize;
        
      case "https://tap.rsvp/schema/1.0#Authorize":
        console.log('Authorization received:', message.body);
        
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
import { TAPAgent } from '@taprsvp/tap-agent';
import Web3 from 'web3';

class WalletTapIntegration {
  private participant: TAPAgent | null = null;
  private web3: Web3;
  private initialized: Promise<void>;
  
  constructor() {
    // Initialize Web3
    if (window.ethereum) {
      this.web3 = new Web3(window.ethereum);
    } else {
      throw new Error("No Ethereum provider found");
    }
    
    // Initialize TAPAgent asynchronously
    this.initialized = this.init();
  }
  
  // Async initialization method
  private async init() {
    try {
      // Create TAP participant using the static factory method
      this.participant = await TAPAgent.create({
        nickname: 'Web Wallet'
      });
      console.log("TAP agent initialized with DID:", this.participant.did);
    } catch (error) {
      console.error("Failed to initialize TAP agent:", error);
      throw error;
    }
  }
  
  // Connect wallet and get accounts
  async connect() {
    // Wait for TAP initialization to complete
    await this.initialized;
    
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
    // Wait for TAP initialization to complete
    await this.initialized;
    
    if (!this.participant) {
      throw new Error("TAP agent not initialized");
    }
    
    // Use the transfer helper method on the TAPAgent
    const transfer = this.participant.transfer({
      asset: asset,
      amount: amount,
      originator: {
        '@id': this.participant.did,
        role: "originator"
      },
      beneficiary: {
        '@id': beneficiaryDid,
        role: "beneficiary"
      },
      memo: "Transfer initiated from web wallet"
    });
    
    return transfer;
  }
  
  // Execute an on-chain transaction after receiving authorization
  async executeTransaction(authorizeMessage: any) {
    // Wait for TAP initialization to complete
    await this.initialized;
    
    if (!this.participant) {
      throw new Error("TAP agent not initialized");
    }
    
    try {
      // Get the original transfer from the thread ID
      const transferData = this.getOriginalTransfer(authorizeMessage.thid);
      
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
      const toAddress = this.didToAddress(transferData.beneficiary['@id']);
      
      // Send the transaction
      const tx = await tokenContract.methods.transfer(toAddress, amount.toString()).send({
        from: fromAddress
      });
      
      // Create settle message
      const settle = this.participant.settle({
        settlementId: tx.transactionHash,
        status: "completed",
        note: "Settlement transaction completed on blockchain"
      });
      
      // Link to the original transfer
      settle.setThreadId(authorizeMessage.thid);
      
      return settle;
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
async function main() {
  try {
    const tapWallet = new WalletTapIntegration();
    const account = await tapWallet.connect();
    console.log('Connected account:', account);
  } catch (error) {
    console.error("Failed to initialize wallet integration:", error);
  }
}

main();
```

## Handling WASM Loading

When using TAP-RS in a browser environment, you need to handle WASM loading correctly. The static TAPAgent.create() method helps with this by ensuring WASM is properly initialized before creating agent instances:

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
      <div id="loading">Initializing TAP agent...</div>
      <button id="createMessage" style="display: none;">Create Transfer Message</button>
      <pre id="messageOutput"></pre>
    </div>
  </div>
  
  <script type="module">
    // With bundlers like webpack or Parcel
    import { TAPAgent } from '@taprsvp/tap-agent';
    
    // Initialize after the WASM is loaded
    async function initialize() {
      try {
        // Create the agent using the static factory method
        // This ensures WASM is properly initialized
        const participant = await TAPAgent.create({
          nickname: 'Browser Demo Participant'
        });
        
        // Show the button and hide loading message once initialized
        document.getElementById('loading').style.display = 'none';
        document.getElementById('createMessage').style.display = 'block';
        
        document.getElementById('createMessage').addEventListener('click', async () => {
          // Use the transfer helper method on the agent
          const transfer = participant.transfer({
            asset: "eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F",
            amount: "10.0",
            originator: {
              '@id': participant.did,
              role: "originator"
            },
            beneficiary: {
              '@id': "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
              role: "beneficiary"
            },
            memo: "Demo transfer"
          });
          
          // Pack the message for transport
          const packed = await transfer.pack();
          document.getElementById('messageOutput').textContent = JSON.stringify(packed, null, 2);
        });
        
        console.log('TAP-RS WASM loaded successfully');
        console.log('Participant DID:', participant.did);
      } catch (error) {
        console.error('Failed to initialize TAP-RS:', error);
        document.getElementById('loading').textContent = 'Error loading TAP agent: ' + error.message;
      }
    }
    
    // Initialize the application
    initialize();
  </script>
</body>
</html>
```

## Usage in Node.js

Using TAP-RS in Node.js is similar to browser usage, but make sure to use the async factory pattern:

```javascript
// JavaScript/Node.js example
const { TAPAgent } = require('@taprsvp/tap-agent');

async function main() {
  try {
    // Create a participant using the async factory method
    const participant = await TAPAgent.create({
      nickname: 'Node.js Participant'
    });
    
    console.log('TAP agent initialized with DID:', participant.did);
    
    // Create a simple transfer message
    const transfer = participant.transfer({
      asset: "eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F",
      amount: "100.0",
      originator: {
        '@id': participant.did,
        role: "originator"
      },
      beneficiary: {
        '@id': "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        role: "beneficiary"
      },
      memo: "Node.js test transfer"
    });
    
    // Convert to JSON
    const json = transfer.toJSON();
    console.log('Transfer message:');
    console.log(json);
    
    // Pack the message for transport
    const packed = await transfer.pack();
    console.log('Packed message:');
    console.log(packed);
    
    // You can also unpack messages received from elsewhere
    const unpacked = await participant.unpackMessage(packed.message);
    console.log('Unpacked message:');
    console.log(unpacked);
  } catch (error) {
    console.error('Error:', error);
  }
}

// Run the async function
main();
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

3. **Async Nature**: WASM initialization is inherently asynchronous. The `TAPAgent.create()` static factory method helps manage this asynchronous nature correctly, but you need to design your application flow to account for this.

Example of handling the asynchronous loading with the factory pattern:

```typescript
// Better approach using the factory method and async/await
import { TAPAgent } from '@taprsvp/tap-agent';

// Queue for messages received before initialization
const pendingMessages = [];
let agent = null;

// Function to initialize the agent
async function initializeAgent() {
  try {
    // Create an agent with the static factory method
    agent = await TAPAgent.create({
      nickname: 'Async Loading Example'
    });
    
    console.log("Agent initialized with DID:", agent.did);
    
    // Process any messages that arrived before initialization
    await Promise.all(pendingMessages.map(processMessage));
    pendingMessages.length = 0; // Clear the queue
    
    return agent;
  } catch (error) {
    console.error("Failed to initialize agent:", error);
    throw error;
  }
}

// Start initialization immediately
const initPromise = initializeAgent();

// Message handler
async function handleIncomingMessage(message) {
  if (agent) {
    // Agent is already initialized
    await processMessage(message);
  } else {
    // Wait for agent initialization and then process
    pendingMessages.push(message);
    
    // Ensure initialization is happening
    initPromise.catch(error => {
      console.error("Agent initialization failed:", error);
    });
  }
}

async function processMessage(message) {
  try {
    // Make sure agent is initialized
    if (!agent) {
      agent = await initPromise;
    }
    
    // Now process the message with the agent
    const unpacked = await agent.unpackMessage(message);
    console.log("Processing message:", unpacked);
    
    // Handle different message types
    if (unpacked.type === "https://tap.rsvp/schema/1.0#Transfer") {
      // Handle transfer message
      // ...
    }
  } catch (error) {
    console.error("Error processing message:", error);
  }
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

const participant = new Agent({
  nickname: 'Custom Key Participant',
  keyManager,
  keyResolver,
});
```

## Next Steps

- Explore [Implementing TAP Flows](./implementing_tap_flows.md) for building complete message flows
- Review [Security Best Practices](./security_best_practices.md) for securing your WASM implementation
- Check out the [API Reference](../api/index.md) for detailed information on the TypeScript API

For questions or support, please open an issue on the [TAP-RS GitHub repository](https://github.com/notabene/tap-rs).
