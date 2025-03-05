/**
 * Basic usage example of the TAP-TS library
 */

import {
  Agent,
  TapNode,
  Message,
  MessageType,
  wasmLoader,
  resolveDID,
  canResolveDID,
} from "../src/mod.ts";

// Load the WASM module
console.log("Loading WASM module...");
await wasmLoader.load();
console.log("WASM module loaded");

// Example DIDs
const aliceDID = "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH";
const bobDID = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";

// Resolve DIDs and print metadata
if (canResolveDID(aliceDID)) {
  console.log(`Can resolve DID: ${aliceDID}`);
  const resolution = await resolveDID(aliceDID);
  console.log("DID Resolution result:", {
    id: resolution.didDocument.id,
    metadata: resolution.didResolutionMetadata,
  });
} else {
  console.log(`Cannot resolve DID: ${aliceDID}`);
}

// Create a TAP node
console.log("Creating TAP node...");
const node = new TapNode({
  debug: true,
  network: {
    peers: ["https://example.com/tap"],
  },
});

// Subscribe to messages on the node
const nodeUnsubscribe = node.subscribeToMessages((message, metadata) => {
  console.log("Node received message:", message.id);
  console.log("Message metadata:", metadata);
});

// Create and register agents
console.log("Creating agents...");
const aliceAgent = new Agent({
  did: aliceDID,
  nickname: "Alice",
});

const bobAgent = new Agent({
  did: bobDID,
  nickname: "Bob",
});

// Register the agents with the node
console.log("Registering agents...");
node.registerAgent(aliceAgent);
node.registerAgent(bobAgent);

// Subscribe to messages on Bob's agent
const bobUnsubscribe = bobAgent.subscribeToMessages((message, metadata) => {
  console.log("Bob received message:", message.id);
  console.log("From:", metadata.fromDid);
  
  // If it's a transfer message, automatically authorize it
  if (message.type === MessageType.TRANSFER) {
    console.log("Transfer message received, sending authorize response...");
    
    const transferData = message.getTransferData();
    if (transferData) {
      // Create an authorize response message
      const response = new Message({
        type: MessageType.AUTHORIZE,
        correlation: message.id,
      });
      
      // Set authorize data
      response.setAuthorizeData({
        transfer_id: message.id,
        note: "Automatic authorization",
      });
      
      // Send the response
      bobAgent.sendMessage(metadata.fromDid || "", response)
        .then(() => console.log("Authorization sent"))
        .catch((error) => console.error("Error sending authorization:", error));
    }
  }
});

// Create a TAP transfer message
console.log("Creating transfer message...");
const transferMessage = new Message({
  type: MessageType.TRANSFER,
});

// Set the transfer data following TAIP-3
transferMessage.setTransferData({
  asset: "eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
  originator: {
    "@id": aliceDID,
    role: "originator"
  },
  amount: "100.00",
  beneficiary: {
    "@id": bobDID,
    role: "beneficiary"
  },
  agents: [
    {
      "@id": aliceDID,
      role: "originator"
    },
    {
      "@id": bobDID,
      role: "beneficiary"
    }
  ],
  memo: "Example transfer"
});

// Send the message from Alice to Bob
console.log("Sending message from Alice to Bob...");
try {
  await aliceAgent.sendMessage(bobDID, transferMessage);
  console.log("Message sent");
} catch (error) {
  console.error("Error sending message:", error);
}

// Wait for a bit to allow message processing
console.log("Waiting for message processing...");
await new Promise((resolve) => setTimeout(resolve, 1000));

// Clean up
nodeUnsubscribe();
bobUnsubscribe();

console.log("Example complete");
