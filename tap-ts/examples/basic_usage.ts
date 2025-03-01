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
  
  // If it's an authorization request, automatically respond
  if (message.type === MessageType.AUTHORIZATION_REQUEST) {
    console.log("Authorization request received, sending response...");
    
    // Create a response message
    const response = new Message({
      type: MessageType.AUTHORIZATION_RESPONSE,
      correlation: message.id,
    });
    
    // Set authorization response data
    response.setAuthorizationResponseData({
      transactionHash: message.getAuthorizationRequestData()?.transactionHash || "",
      approved: true,
      reason: "Automatic approval",
    });
    
    // Send the response
    bobAgent.sendMessage(metadata.fromDid || "", response)
      .then(() => console.log("Response sent"))
      .catch((error) => console.error("Error sending response:", error));
  }
});

// Create a TAP authorization request message
console.log("Creating authorization request message...");
const authRequest = new Message({
  type: MessageType.AUTHORIZATION_REQUEST,
});

// Set the authorization request data
authRequest.setAuthorizationRequestData({
  transactionHash: "0x1234567890abcdef",
  sender: "0xAliceSender",
  receiver: "0xBobReceiver",
  amount: "100.0",
  asset: "BTC",
});

// Send the message from Alice to Bob
console.log("Sending message from Alice to Bob...");
try {
  await aliceAgent.sendMessage(bobDID, authRequest);
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
