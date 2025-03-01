/**
 * Tests for Message implementation
 */

import { assertEquals, assertExists, assertThrows } from "@std/assert/mod.ts";
import { Message, MessageType, TapError, ErrorType } from "../src/mod.ts";

Deno.test("Message tests", async (t) => {
  await t.step("Create basic message", () => {
    const message = new Message({
      type: MessageType.PING,
    });
    
    assertExists(message);
    assertEquals(message.type, MessageType.PING);
    assertExists(message.id);
    assertEquals(message.id.startsWith("msg_"), true);
    assertEquals(message.version, "1.0");
  });

  await t.step("Create message with all properties", () => {
    const message = new Message({
      type: MessageType.PING,
      id: "custom_id",
      from: "did:key:alice",
      to: ["did:key:bob"],
      created: 123456789,
      expires: 987654321,
      threadId: "thread_123",
      correlation: "corr_123",
      customData: {
        test: "value",
      },
    });
    
    assertExists(message);
    assertEquals(message.id, "custom_id");
    assertEquals(message.type, MessageType.PING);
    assertEquals(message.from, "did:key:alice");
    assertEquals(message.to?.length, 1);
    assertEquals(message.to?.[0], "did:key:bob");
    assertEquals(message.created, 123456789);
    assertEquals(message.expires, 987654321);
    assertEquals(message.threadId, "thread_123");
    assertEquals(message.correlation, "corr_123");
    assertEquals(message.customData?.test, "value");
  });

  await t.step("Create message with from set after construction", () => {
    const message = new Message({
      type: MessageType.PING,
    });
    
    message.from = "did:key:alice";
    
    assertEquals(message.from, "did:key:alice");
  });

  await t.step("Create and verify authorization request", () => {
    const message = new Message({
      type: MessageType.AUTHORIZATION_REQUEST,
    });
    
    // Set authorization request data
    message.setAuthorizationRequestData({
      transactionHash: "0x1234567890abcdef",
      sender: "0xAliceSender",
      receiver: "0xBobReceiver",
      amount: "100.0",
      asset: "BTC",
    });
    
    // Verify data
    const requestData = message.getAuthorizationRequestData();
    assertExists(requestData);
    assertEquals(requestData.transactionHash, "0x1234567890abcdef");
    assertEquals(requestData.sender, "0xAliceSender");
    assertEquals(requestData.receiver, "0xBobReceiver");
    assertEquals(requestData.amount, "100.0");
    assertEquals(requestData.asset, "BTC");
  });

  await t.step("Create and verify authorization response", () => {
    const message = new Message({
      type: MessageType.AUTHORIZATION_RESPONSE,
    });
    
    // Set authorization response data
    message.setAuthorizationResponseData({
      transactionHash: "0x1234567890abcdef",
      approved: true,
      reason: "Test approval",
    });
    
    // Verify data
    const responseData = message.getAuthorizationResponseData();
    assertExists(responseData);
    assertEquals(responseData.transactionHash, "0x1234567890abcdef");
    assertEquals(responseData.approved, true);
    assertEquals(responseData.reason, "Test approval");
  });

  await t.step("Message serialization and deserialization", () => {
    const originalMessage = new Message({
      type: MessageType.AUTHORIZATION_REQUEST,
      from: "did:key:alice",
      to: ["did:key:bob"],
    });
    
    // Set authorization request data
    originalMessage.setAuthorizationRequestData({
      transactionHash: "0x1234567890abcdef",
      sender: "0xAliceSender",
      receiver: "0xBobReceiver",
      amount: "100.0",
      asset: "BTC",
    });
    
    // Serialize to JSON
    const json = originalMessage.toJSON();
    
    // Deserialize from JSON
    const deserializedMessage = Message.fromJSON(json);
    
    // Verify deserialized message
    assertEquals(deserializedMessage.type, originalMessage.type);
    assertEquals(deserializedMessage.id, originalMessage.id);
    assertEquals(deserializedMessage.from, originalMessage.from);
    assertEquals(deserializedMessage.to, originalMessage.to);
    
    // Verify request data
    const requestData = deserializedMessage.getAuthorizationRequestData();
    assertExists(requestData);
    assertEquals(requestData.transactionHash, "0x1234567890abcdef");
    assertEquals(requestData.sender, "0xAliceSender");
    assertEquals(requestData.receiver, "0xBobReceiver");
    assertEquals(requestData.amount, "100.0");
    assertEquals(requestData.asset, "BTC");
  });

  await t.step("Reject setting wrong data type on message", () => {
    // Create an authorization request message
    const requestMessage = new Message({
      type: MessageType.AUTHORIZATION_REQUEST,
    });
    
    // Try to set authorization response data on a request message
    assertThrows(
      () => {
        // @ts-expect-error - This is intentionally wrong for the test
        requestMessage.setAuthorizationResponseData({
          transactionHash: "0x1234567890abcdef",
          approved: true,
        });
      },
      TapError,
      ErrorType.INVALID_MESSAGE_TYPE
    );
    
    // Create an authorization response message
    const responseMessage = new Message({
      type: MessageType.AUTHORIZATION_RESPONSE,
    });
    
    // Try to set authorization request data on a response message
    assertThrows(
      () => {
        // @ts-expect-error - This is intentionally wrong for the test
        responseMessage.setAuthorizationRequestData({
          transactionHash: "0x1234567890abcdef",
          sender: "0xSender",
          receiver: "0xReceiver",
        });
      },
      TapError,
      ErrorType.INVALID_MESSAGE_TYPE
    );
  });
});
