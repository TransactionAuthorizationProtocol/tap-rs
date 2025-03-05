/**
 * Tests for TapNode implementation
 */

import { assertEquals, assertExists, assertThrows } from "https://deno.land/std/testing/asserts.ts";
import { TapNode } from "../src/node.ts";
import { Agent } from "../src/agent.ts";
import { Message, MessageType } from "../src/message.ts";
import { TapError, ErrorType } from "../src/error.ts";

Deno.test("TapNode - Create node", () => {
  const node = new TapNode({
    id: "test-node",
  });
  
  assertExists(node);
  assertEquals(node.id, "test-node");
});

Deno.test("TapNode - Register agent", () => {
  const node = new TapNode();
  
  const agent = new Agent({
    did: "did:example:123",
    id: "test-agent",
  });
  
  node.registerAgent(agent);
  
  // The agent should be in the node's agent map
  const agents = node.getAgents();
  assertEquals(agents.size, 1);
  assertEquals(agents.has(agent.id), true);
  
  // Get agent by ID
  const retrievedAgent = agents.get(agent.id);
  assertExists(retrievedAgent);
  assertEquals(retrievedAgent.did, agent.did);
});

Deno.test("TapNode - Register multiple agents", () => {
  const node = new TapNode();
  
  const aliceDID = "did:example:alice";
  const bobDID = "did:example:bob";
  
  const aliceAgent = new Agent({
    did: aliceDID,
    id: "alice",
  });
  
  const bobAgent = new Agent({
    did: bobDID,
    id: "bob",
  });
  
  node.registerAgent(aliceAgent);
  node.registerAgent(bobAgent);
  
  // Both agents should be in the node's agent map
  const agents = node.getAgents();
  assertEquals(agents.size, 2);
  
  // Get agents by ID
  const retrievedAlice = agents.get(aliceAgent.id);
  const retrievedBob = agents.get(bobAgent.id);
  
  assertExists(retrievedAlice);
  assertExists(retrievedBob);
  
  assertEquals(retrievedAlice.did, aliceDID);
  assertEquals(retrievedBob.did, bobDID);
});

Deno.test("TapNode - Unregister agent", () => {
  const node = new TapNode();
  
  const aliceDID = "did:example:alice";
  const bobDID = "did:example:bob";
  
  const aliceAgent = new Agent({
    did: aliceDID,
    id: "alice",
  });
  
  const bobAgent = new Agent({
    did: bobDID,
    id: "bob",
  });
  
  // Register both agents
  node.registerAgent(aliceAgent);
  node.registerAgent(bobAgent);
  
  // Unregister alice
  node.unregisterAgent(aliceAgent.id);
  
  // Check if alice was removed
  const agents = node.getAgents();
  assertEquals(agents.size, 1);
  
  // Alice should be gone, Bob should still be there
  assertEquals(node.getAgentDIDs().includes(aliceDID), false);
  assertEquals(node.getAgentDIDs().includes(bobDID), true);
  
  // Try to unregister a non-existent agent
  try {
    node.unregisterAgent("non-existent");
  } catch (e) {
    assertEquals((e as TapError).type, ErrorType.AGENT_NOT_FOUND);
  }
});

Deno.test("TapNode - Send message between agents", async () => {
  const node = new TapNode();
  
  const aliceDID = "did:example:alice";
  const bobDID = "did:example:bob";
  
  const aliceAgent = new Agent({
    did: aliceDID,
    id: "alice",
  });
  
  const bobAgent = new Agent({
    did: bobDID,
    id: "bob",
  });
  
  // Register a message handler for Bob
  let messageReceived = false;
  bobAgent.registerHandler(MessageType.TRANSFER, async () => {
    messageReceived = true;
  });
  
  // Register both agents
  node.registerAgent(aliceAgent);
  node.registerAgent(bobAgent);
  
  // Create a message
  const message = new Message({
    type: MessageType.TRANSFER,
  });
  
  // Send message from Alice to Bob
  await node.sendMessage(aliceDID, bobDID, message);
  
  // Bob should have received the message
  assertEquals(messageReceived, true);
});

Deno.test("TapNode - Subscribe to node messages", async () => {
  const node = new TapNode();
  
  const aliceDID = "did:example:alice";
  const bobDID = "did:example:bob";
  
  const aliceAgent = new Agent({
    did: aliceDID,
    id: "alice",
  });
  
  const bobAgent = new Agent({
    did: bobDID,
    id: "bob",
  });
  
  // Register both agents
  node.registerAgent(aliceAgent);
  node.registerAgent(bobAgent);
  
  // Subscribe to node messages
  let messageCount = 0;
  const unsubscribe = node.subscribeToMessages(() => {
    messageCount++;
  });
  
  // Create and send two messages
  const message1 = new Message({
    type: MessageType.TRANSFER,
  });
  
  const message2 = new Message({
    type: MessageType.TRANSFER,
  });
  
  await node.sendMessage(aliceDID, bobDID, message1);
  await node.sendMessage(bobDID, aliceDID, message2);
  
  // We should have received both messages
  assertEquals(messageCount, 2);
  
  // Unsubscribe and send another message
  unsubscribe();
  
  const message3 = new Message({
    type: MessageType.TRANSFER,
  });
  
  await node.sendMessage(aliceDID, bobDID, message3);
  
  // The message count should not have changed
  assertEquals(messageCount, 2);
});
