/**
 * Tests for Agent implementation
 */

import { assertEquals, assertExists, assertThrows } from "https://deno.land/std/testing/asserts.ts";
import { Agent } from "../src/agent.ts";
import { Message, MessageType } from "../src/message.ts";

Deno.test("Agent - Create agent", () => {
  const agent = new Agent({
    did: "did:example:123",
  });
  
  assertExists(agent);
  assertEquals(agent.did, "did:example:123");
  assertEquals(agent.isReady, true);
});

Deno.test("Agent - Create message", () => {
  const agent = new Agent({
    did: "did:example:123",
  });
  
  const message = agent.createMessage({
    type: MessageType.PING,
  });
  
  assertExists(message);
  assertEquals(message.type, MessageType.PING); // Fix the second message type assertion
});

Deno.test("Agent - Handle authorization request", async () => {
  const agent = new Agent({
    did: "did:example:123",
  });
  
  // Register handler for authorization request
  let requestReceived = false;
  agent.registerHandler(MessageType.AUTHORIZATION_REQUEST, async (message) => {
    requestReceived = true;
    
    // Verify the message
    assertEquals((message as Message).type, MessageType.AUTHORIZATION_REQUEST);
    
    const requestData = (message as Message).getAuthorizationRequestData();
    assertEquals(requestData?.transactionHash, "0x1234567890abcdef");
    assertEquals(requestData?.sender, "0xSender");
    assertEquals(requestData?.receiver, "0xReceiver");
    assertEquals(requestData?.amount, "100");
    assertEquals(requestData?.asset, "BTC");
  });
  
  // Create an authorization request message
  const message = new Message({
    type: MessageType.AUTHORIZATION_REQUEST,
    ledgerId: "test-ledger",
  });
  
  message.setAuthorizationRequestData({
    transactionHash: "0x1234567890abcdef",
    sender: "0xSender",
    receiver: "0xReceiver",
    amount: "100",
    asset: "BTC",
  });
  
  // Process the message
  await agent.processMessage(message);
  
  // Verify the handler was called
  assertEquals(requestReceived, true);
});

Deno.test("Agent - Handle authorization response", async () => {
  const agent = new Agent({
    did: "did:example:123",
  });
  
  // Register handler for authorization response
  let responseReceived = false;
  agent.registerHandler(MessageType.AUTHORIZATION_RESPONSE, async (message) => {
    responseReceived = true;
    
    // Verify the message
    assertEquals((message as Message).type, MessageType.AUTHORIZATION_RESPONSE);
    
    const responseData = (message as Message).getAuthorizationResponseData();
    assertEquals(responseData?.transactionHash, "0x1234567890abcdef");
    assertEquals(responseData?.approved, true);
    assertEquals(responseData?.reason, "Test approval");
  });
  
  // Create an authorization response message
  const message = new Message({
    type: MessageType.AUTHORIZATION_RESPONSE,
    ledgerId: "test-ledger",
  });
  
  message.setAuthorizationResponseData({
    transactionHash: "0x1234567890abcdef",
    approved: true,
    reason: "Test approval",
  });
  
  // Process the message
  await agent.processMessage(message);
  
  // Verify the handler was called
  assertEquals(responseReceived, true);
});

Deno.test("Agent - Subscribe to messages", async () => {
  const agent = new Agent({
    did: "did:example:123",
  });
  
  // Subscribe to messages
  let lastMessage: Message | null = null;
  agent.subscribe((message) => {
    lastMessage = message;
  });
  
  // Create a ping message
  const message = new Message({
    type: MessageType.PING,
    ledgerId: "test-ledger",
  });
  
  // Process the message
  await agent.processMessage(message);
  
  // Verify the subscriber was called
  assertExists(lastMessage);
  assertEquals((lastMessage as Message).type, MessageType.PING);
});

Deno.test("Agent - Handler registration and unregistration", () => {
  const agent = new Agent({
    did: "did:example:123",
  });
  
  // Initially, there should be no handler
  assertEquals(agent.hasHandler(MessageType.PING), false);
  
  // Register a handler
  agent.registerHandler(MessageType.PING, async () => {
    // Do nothing for the test
  });
  
  // Now there should be a handler
  assertEquals(agent.hasHandler(MessageType.PING), true);
  
  // Unregister the handler
  const result = agent.unregisterHandler(MessageType.PING);
  assertEquals(result, true);
  
  // Now there should be no handler again
  assertEquals(agent.hasHandler(MessageType.PING), false);
});

Deno.test("Agent - Message handling", async () => {
  const agent = new Agent({
    did: "did:example:123",
  });
  
  // Register a handler
  let handlerMessage: Message | null = null;
  agent.registerHandler(MessageType.PING, async (message) => {
    handlerMessage = message;
  });
  
  // Create a ping message
  const message = new Message({
    type: MessageType.PING,
    ledgerId: "test-ledger",
  });
  
  // Process the message
  await agent.processMessage(message);
  
  // Verify the handler was called
  assertExists(handlerMessage);
  assertEquals((handlerMessage as Message).type, MessageType.PING);
});
