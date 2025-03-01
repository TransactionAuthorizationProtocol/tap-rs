/**
 * Tests for Agent implementation
 */

import { assertEquals, assertExists, assertThrows } from "@std/assert/mod.ts";
import { Agent, Message, MessageType, wasmLoader, TapError, ErrorType } from "../src/mod.ts";

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

Deno.test("Agent tests", async (t) => {
  await setup();

  // Sample DIDs
  const aliceDID = "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH";
  
  // Agent instance
  let agent: Agent;

  await t.step("Create Agent instance", () => {
    agent = new Agent({
      did: aliceDID,
      nickname: "Alice",
    });

    assertExists(agent);
    assertEquals(agent.did, aliceDID);
    assertEquals(agent.nickname, "Alice");
  });

  await t.step("Agent properties", () => {
    // Check default properties
    assertEquals(agent.did, aliceDID);
    assertEquals(agent.nickname, "Alice");
    assertEquals(agent.isReady, true);
  });

  await t.step("Create message with Agent", () => {
    const message = agent.createMessage({
      type: MessageType.PING,
    });
    
    assertExists(message);
    assertEquals(message.type, MessageType.PING);
    assertEquals(message.from, aliceDID);
  });

  await t.step("Create authorization request message", () => {
    const message = agent.createAuthorizationRequest({
      transactionHash: "0x1234567890abcdef",
      sender: "0xAliceSender",
      receiver: "0xBobReceiver",
      amount: "100.0",
      asset: "BTC",
    });
    
    assertExists(message);
    assertEquals(message.type, MessageType.AUTHORIZATION_REQUEST);
    assertEquals(message.from, aliceDID);
    
    const requestData = message.getAuthorizationRequestData();
    assertExists(requestData);
    assertEquals(requestData?.transactionHash, "0x1234567890abcdef");
    assertEquals(requestData?.sender, "0xAliceSender");
    assertEquals(requestData?.receiver, "0xBobReceiver");
    assertEquals(requestData?.amount, "100.0");
    assertEquals(requestData?.asset, "BTC");
  });

  await t.step("Create authorization response message", () => {
    // Create a request first
    const request = agent.createAuthorizationRequest({
      transactionHash: "0x1234567890abcdef",
      sender: "0xAliceSender",
      receiver: "0xBobReceiver",
      amount: "100.0",
      asset: "BTC",
    });
    
    // Create a response to the request
    const response = agent.createAuthorizationResponse({
      requestId: request.id,
      transactionHash: "0x1234567890abcdef",
      approved: true,
      reason: "Test approval",
    });
    
    assertExists(response);
    assertEquals(response.type, MessageType.AUTHORIZATION_RESPONSE);
    assertEquals(response.from, aliceDID);
    assertEquals(response.correlation, request.id);
    
    const responseData = response.getAuthorizationResponseData();
    assertExists(responseData);
    assertEquals(responseData?.transactionHash, "0x1234567890abcdef");
    assertEquals(responseData?.approved, true);
    assertEquals(responseData?.reason, "Test approval");
  });

  await t.step("Message subscription", async () => {
    let receivedMessages = 0;
    let lastMessage: Message | null = null;
    
    // Subscribe to messages
    const unsubscribe = agent.subscribeToMessages((message) => {
      receivedMessages++;
      lastMessage = message;
    });
    
    // Process a message
    const message = new Message({
      type: MessageType.PING,
      to: [aliceDID],
    });
    
    await agent.processMessage(message);
    
    // Check message receipt
    assertEquals(receivedMessages, 1);
    assertExists(lastMessage);
    assertEquals(lastMessage?.type, MessageType.PING);
    
    // Clean up subscription
    unsubscribe();
    
    // Process another message after unsubscribing
    const message2 = new Message({
      type: MessageType.PING,
      to: [aliceDID],
    });
    
    await agent.processMessage(message2);
    
    // Check that the count didn't increase
    assertEquals(receivedMessages, 1);
  });

  await t.step("Message handlers", async () => {
    let handlerCalled = false;
    let handlerMessage: Message | null = null;
    
    // Register a handler for PING messages
    agent.registerMessageHandler(MessageType.PING, (message) => {
      handlerCalled = true;
      handlerMessage = message;
      return Promise.resolve();
    });
    
    // Process a message
    const message = new Message({
      type: MessageType.PING,
      to: [aliceDID],
    });
    
    await agent.processMessage(message);
    
    // Check handler execution
    assertEquals(handlerCalled, true);
    assertExists(handlerMessage);
    assertEquals(handlerMessage?.type, MessageType.PING);
    
    // Reset for next test
    handlerCalled = false;
    handlerMessage = null;
    
    // Process a different message type
    const message2 = new Message({
      type: MessageType.AUTHORIZATION_REQUEST,
      to: [aliceDID],
    });
    
    await agent.processMessage(message2);
    
    // Check that the handler wasn't called for this type
    assertEquals(handlerCalled, false);
  });

  await teardown();
});
