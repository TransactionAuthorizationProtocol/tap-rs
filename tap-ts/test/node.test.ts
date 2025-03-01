/**
 * Tests for TapNode implementation
 */

import { assertEquals, assertExists, assertThrows } from "@std/assert/mod.ts";
import { TapNode, Agent, Message, MessageType, wasmLoader, TapError, ErrorType } from "../src/mod.ts";

// Setup and teardown
const setup = async () => {
  // Load the WASM module before tests
  if (!wasmLoader.moduleIsLoaded()) {
    await wasmLoader.load();
  }
};

const teardown = () => {
  // Nothing to clean up yet
};

Deno.test("TapNode tests", async (t) => {
  await setup();

  // Sample DIDs
  const aliceDID = "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH";
  const bobDID = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";

  // Create test agents
  let aliceAgent: Agent;
  let bobAgent: Agent;
  let node: TapNode;

  await t.step("Create TapNode instance", () => {
    node = new TapNode({
      debug: true,
      network: {
        peers: ["https://example.com/tap"],
      },
    });

    assertExists(node);
    assertEquals(node.getAgentDIDs().length, 0);
  });

  await t.step("Create and register agents", () => {
    aliceAgent = new Agent({
      did: aliceDID,
      nickname: "Alice",
    });

    bobAgent = new Agent({
      did: bobDID,
      nickname: "Bob",
    });

    assertExists(aliceAgent);
    assertEquals(aliceAgent.did, aliceDID);
    assertEquals(aliceAgent.nickname, "Alice");

    assertExists(bobAgent);
    assertEquals(bobAgent.did, bobDID);
    assertEquals(bobAgent.nickname, "Bob");

    // Register agents
    node.registerAgent(aliceAgent);
    node.registerAgent(bobAgent);

    // Check registration
    assertEquals(node.getAgentDIDs().length, 2);
    assertEquals(node.getAgentDIDs().includes(aliceDID), true);
    assertEquals(node.getAgentDIDs().includes(bobDID), true);
  });

  await t.step("Get agent by DID", () => {
    const retrievedAlice = node.getAgent(aliceDID);
    const retrievedBob = node.getAgent(bobDID);
    const nonExistent = node.getAgent("did:key:nonexistent");

    assertExists(retrievedAlice);
    assertEquals(retrievedAlice?.did, aliceDID);
    
    assertExists(retrievedBob);
    assertEquals(retrievedBob?.did, bobDID);
    
    assertEquals(nonExistent, undefined);
  });

  await t.step("Cannot register agent with same DID twice", () => {
    const duplicateAgent = new Agent({
      did: aliceDID,
      nickname: "Duplicate Alice",
    });

    assertThrows(
      () => node.registerAgent(duplicateAgent),
      TapError,
      ErrorType.AGENT_ALREADY_EXISTS
    );
  });

  await t.step("Unregister agent", () => {
    // Unregister Alice
    const unregistered = node.unregisterAgent(aliceDID);
    assertEquals(unregistered, true);
    
    // Check registration
    assertEquals(node.getAgentDIDs().length, 1);
    assertEquals(node.getAgentDIDs().includes(aliceDID), false);
    assertEquals(node.getAgentDIDs().includes(bobDID), true);
    
    // Try to unregister non-existent agent
    const nonExistent = node.unregisterAgent("did:key:nonexistent");
    assertEquals(nonExistent, false);
  });

  await t.step("Message exchange", async () => {
    // Re-register Alice
    node.registerAgent(aliceAgent);
    
    // Setup message receipt tracking
    let bobReceivedMessage = false;
    const unsubscribe = bobAgent.subscribeToMessages((message) => {
      bobReceivedMessage = true;
      assertEquals(message.type, MessageType.PING);
    });
    
    // Create a message
    const message = new Message({
      type: MessageType.PING,
    });
    
    // Send the message
    await node.sendMessage(aliceDID, bobDID, message);
    
    // Short wait to allow processing
    await new Promise((resolve) => setTimeout(resolve, 100));
    
    // Check message receipt
    assertEquals(bobReceivedMessage, true);
    
    // Clean up subscription
    unsubscribe();
  });

  await t.step("Message subscription", async () => {
    let nodeReceivedMessages = 0;
    
    // Subscribe to all messages
    const unsubscribe = node.subscribeToMessages(() => {
      nodeReceivedMessages++;
    });
    
    // Create and send messages
    const message1 = new Message({ type: MessageType.PING });
    const message2 = new Message({ type: MessageType.PING });
    
    await node.sendMessage(aliceDID, bobDID, message1);
    await node.sendMessage(bobDID, aliceDID, message2);
    
    // Short wait to allow processing
    await new Promise((resolve) => setTimeout(resolve, 100));
    
    // Check message receipt
    assertEquals(nodeReceivedMessages, 2);
    
    // Clean up subscription
    unsubscribe();
  });

  await teardown();
});
