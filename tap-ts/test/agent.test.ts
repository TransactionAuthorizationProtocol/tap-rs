/**
 * Tests for Agent implementation
 */

/// <reference path="../deno.d.ts" />
import { assertEquals, assertExists, assertThrows } from "https://deno.land/std@0.177.0/testing/asserts.ts";
import { Agent } from "../src/agent.ts";
import { Message, MessageType } from "../src/message.ts";

// @ts-ignore: Deno namespace is available at runtime
Deno.test("Agent - Create agent", () => {
  const agent = new Agent({
    did: "did:example:123",
  });
  
  assertExists(agent);
  assertEquals(agent.did, "did:example:123");
  assertEquals(agent.isReady, true);
});

// @ts-ignore: Deno namespace is available at runtime
Deno.test("Agent - Create message", () => {
  const agent = new Agent({
    did: "did:example:123",
  });
  
  const message = agent.createMessage(MessageType.TRANSFER);
  
  assertExists(message);
  assertEquals(message.type, MessageType.TRANSFER); 
});

// @ts-ignore: Deno namespace is available at runtime
Deno.test("Agent - Handle transfer message", async () => {
  const agent = new Agent({
    did: "did:example:123",
  });
  
  // Register handler for transfer message
  let requestReceived = false;
  agent.registerHandler(MessageType.TRANSFER, async (message) => {
    requestReceived = true;
    
    // Verify the message
    assertEquals((message as Message).type, MessageType.TRANSFER);
    
    const transferData = (message as Message).getTransferData();
    assertEquals(transferData?.asset, "eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
    assertEquals(transferData?.originator["@id"], "did:key:alice");
    assertEquals(transferData?.amount, "100.00");
  });
  
  // Create a transfer message
  const message = new Message({
    type: MessageType.TRANSFER,
  });
  
  message.setTransferData({
    asset: "eip155:1/erc20:0x1234567890abcdef",
    originator: {
      "@id": "did:example:originator",
      role: "originator"
    },
    amount: "100",
    agents: [
      {
        "@id": "did:example:originator",
        role: "originator"
      }
    ]
  });
  
  // Process the message
  await agent.processMessage(message);
  
  // Verify the handler was called
  assertEquals(requestReceived, true);
});

// @ts-ignore: Deno namespace is available at runtime
Deno.test("Agent - Handle authorize message", async () => {
  const agent = new Agent({
    did: "did:example:123",
  });
  
  // Register handler for authorize message
  let responseReceived = false;
  agent.registerHandler(MessageType.AUTHORIZE, async (message) => {
    responseReceived = true;
    
    // Verify the message
    assertEquals((message as Message).type, MessageType.AUTHORIZE);
    
    const authorizeData = (message as Message).getAuthorizeData();
    assertEquals(authorizeData?.transfer_id, "test-transfer-id");
    assertEquals(authorizeData?.note, "Test authorization");
  });
  
  // Create an authorize message
  const message = new Message({
    type: MessageType.AUTHORIZE,
  });
  
  message.setAuthorizeData({
    transfer_id: "mocked-transfer-id",
    note: "mocked-note"
  });
  
  // Process the message
  await agent.processMessage(message);
  
  // Verify the handler was called
  assertEquals(responseReceived, true);
});

// @ts-ignore: Deno namespace is available at runtime
Deno.test("Agent - Subscribe to messages", async () => {
  const agent = new Agent({
    did: "did:example:123",
  });
  
  // Subscribe to messages
  let lastMessage: Message | null = null;
  agent.subscribe((message) => {
    lastMessage = message;
  });
  
  // Create an error message for testing subscription
  const message = new Message({
    type: MessageType.ERROR,
  });
  
  // Process the message
  await agent.processMessage(message);
  
  // Verify the subscriber was called
  assertExists(lastMessage);
  assertEquals(lastMessage && (lastMessage as Message).type, MessageType.ERROR);
});

// @ts-ignore: Deno namespace is available at runtime
Deno.test("Agent - Handle cancel message", async () => {
  const agent = new Agent({
    did: "did:example:123",
  });
  
  // Register handler for cancel message
  let cancelReceived = false;
  agent.registerHandler(MessageType.CANCEL, async (message) => {
    cancelReceived = true;
    
    // Verify the message
    assertEquals((message as Message).type, MessageType.CANCEL);
    
    const cancelData = (message as Message).getCancelData();
    assertEquals(cancelData?.transfer_id, "mocked-transfer-id");
    assertEquals(cancelData?.reason, "User requested cancellation");
    assertEquals(cancelData?.note, "Test cancellation");
  });
  
  // Create a cancel message
  const message = new Message({
    type: MessageType.CANCEL,
  });
  
  const timestamp = new Date().toISOString();
  message.setCancelData({
    transfer_id: "mocked-transfer-id",
    reason: "User requested cancellation",
    note: "Test cancellation",
    timestamp,
    metadata: { test: "value" }
  });
  
  // Process the message
  await agent.processMessage(message);
  
  // Verify the handler was called
  assertEquals(cancelReceived, true);
});

// @ts-ignore: Deno namespace is available at runtime
Deno.test("Agent - Handle revert message", async () => {
  const agent = new Agent({
    did: "did:example:123",
  });
  
  // Register handler for revert message
  let revertReceived = false;
  agent.registerHandler(MessageType.REVERT, async (message) => {
    revertReceived = true;
    
    // Verify the message
    assertEquals((message as Message).type, MessageType.REVERT);
    
    const revertData = (message as Message).getRevertData();
    assertEquals(revertData?.transfer_id, "mocked-transfer-id");
    assertEquals(revertData?.settlement_address, "eip155:1:0x1234567890123456789012345678901234567890");
    assertEquals(revertData?.reason, "Failed compliance check");
    assertEquals(revertData?.note, "Test revert");
  });
  
  // Create a revert message
  const message = new Message({
    type: MessageType.REVERT,
  });
  
  const timestamp = new Date().toISOString();
  message.setRevertData({
    transfer_id: "mocked-transfer-id",
    settlement_address: "eip155:1:0x1234567890123456789012345678901234567890",
    reason: "Failed compliance check",
    note: "Test revert",
    timestamp,
    metadata: { test: "value" }
  });
  
  // Process the message
  await agent.processMessage(message);
  
  // Verify the handler was called
  assertEquals(revertReceived, true);
});

// @ts-ignore: Deno namespace is available at runtime
Deno.test("Agent - Create message with Cancel and Revert data", () => {
  const agent = new Agent({
    did: "did:example:123",
  });
  
  // Test creating a Cancel message
  const timestamp = new Date().toISOString();
  const cancelMessage = agent.createMessage(MessageType.CANCEL, {
    cancelData: {
      transfer_id: "test-transfer-id",
      reason: "User requested cancellation",
      note: "Cancel via agent",
      timestamp,
      metadata: { test: "value" }
    }
  });
  
  assertExists(cancelMessage);
  assertEquals(cancelMessage.type, MessageType.CANCEL);
  const cancelData = cancelMessage.getCancelData();
  assertExists(cancelData);
  assertEquals(cancelData?.transfer_id, "test-transfer-id");
  assertEquals(cancelData?.reason, "User requested cancellation");
  
  // Test creating a Revert message
  const revertMessage = agent.createMessage(MessageType.REVERT, {
    revertData: {
      transfer_id: "test-transfer-id",
      settlement_address: "eip155:1:0x1234567890123456789012345678901234567890",
      reason: "Failed compliance check",
      note: "Revert via agent",
      timestamp,
      metadata: { test: "value" }
    }
  });
  
  assertExists(revertMessage);
  assertEquals(revertMessage.type, MessageType.REVERT);
  const revertData = revertMessage.getRevertData();
  assertExists(revertData);
  assertEquals(revertData?.transfer_id, "test-transfer-id");
  assertEquals(revertData?.settlement_address, "eip155:1:0x1234567890123456789012345678901234567890");
  assertEquals(revertData?.reason, "Failed compliance check");
});

// @ts-ignore: Deno namespace is available at runtime
Deno.test("Agent - Handler registration and unregistration", () => {
  const agent = new Agent({
    did: "did:example:123",
  });
  
  // Initially, there should be no handler for ERROR type
  assertEquals(agent.hasHandler(MessageType.ERROR), false);
  
  // Register a handler
  agent.registerHandler(MessageType.ERROR, async () => {
    // Do nothing for the test
  });
  
  // Now there should be a handler
  assertEquals(agent.hasHandler(MessageType.ERROR), true);
  
  // Unregister the handler
  const result = agent.unregisterAllHandlers(MessageType.ERROR);
  assertEquals(result, true);
  
  // Now there should be no handler again
  assertEquals(agent.hasHandler(MessageType.ERROR), false);
});

// @ts-ignore: Deno namespace is available at runtime
Deno.test("Agent - Message handling", async () => {
  const agent = new Agent({
    did: "did:example:123",
  });
  
  // Register a handler
  let handlerMessage: Message | null = null;
  agent.registerHandler(MessageType.ERROR, async (message) => {
    handlerMessage = message;
  });
  
  // Create an error message
  const message = new Message({
    type: MessageType.ERROR,
  });
  
  // Process the message
  await agent.processMessage(message);
  
  // Verify the handler was called
  assertExists(handlerMessage);
  assertEquals(handlerMessage && (handlerMessage as Message).type, MessageType.ERROR);
});
