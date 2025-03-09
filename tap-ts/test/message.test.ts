/**
 * Tests for Message implementation
 */

import { assertEquals, assertExists, assertThrows } from "https://deno.land/std/testing/asserts.ts";
import { Message, MessageType } from "../src/message.ts";

Deno.test("Message tests", async (t) => {
  await t.step("Create basic message", () => {
    const message = new Message({
      type: MessageType.TRANSFER,
    });
    
    assertExists(message);
    assertEquals(message.type, MessageType.TRANSFER);
  });

  await t.step("Create message with all properties", () => {
    const message = new Message({
      type: MessageType.TRANSFER,
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
    assertEquals(message.type, MessageType.TRANSFER);
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
      type: MessageType.TRANSFER,
    });
    
    message.from = "did:key:alice";
    
    assertEquals(message.from, "did:key:alice");
  });

  await t.step("Create and verify transfer message", () => {
    const message = new Message({
      type: MessageType.TRANSFER,
    });
    
    // Set transfer data
    message.setTransferData({
      asset: "eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
      originator: {
        "@id": "did:key:alice",
        role: "originator"
      },
      amount: "100.00",
      agents: [
        {
          "@id": "did:key:alice",
          role: "originator"
        }
      ]
    });
    
    // Verify data
    const transferData = message.getTransferData();
    assertExists(transferData);
    if (transferData) {
      assertEquals(transferData.asset, "eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
      assertEquals(transferData.originator["@id"], "did:key:alice");
      assertEquals(transferData.amount, "100.00");
      assertEquals(transferData.agents.length, 1);
    }
  });

  await t.step("Create and verify authorize message", () => {
    const message = new Message({
      type: MessageType.AUTHORIZE,
    });
    
    message.setAuthorizeData({
      transfer_id: "test-transfer-id",
      note: "Test authorization"
    });
    
    // Verify data
    const authorizeData = message.getAuthorizeData();
    assertExists(authorizeData);
    if (authorizeData) {
      assertEquals(authorizeData.transfer_id, "test-transfer-id");
      assertEquals(authorizeData.note, "Test authorization");
    }
  });

  await t.step("Create and verify update party message", () => {
    const message = new Message({
      type: MessageType.UPDATE_PARTY,
    });
    
    // Set update party data
    message.setUpdatePartyData({
      transfer_id: "transfer-123",
      party_type: "originator",
      party: {
        "@id": "did:key:z6MkpDYxrwJw5WoD1o4YVfthJJgZfxrECpW6Da6QCWagRHLx",
        role: "new_role"
      },
      note: "Updating role after compliance check"
    });
    
    // Verify data
    const updatePartyData = message.getUpdatePartyData();
    assertExists(updatePartyData);
    if (updatePartyData) {
      assertEquals(updatePartyData.transfer_id, "transfer-123");
      assertEquals(updatePartyData.party_type, "originator");
      assertEquals(updatePartyData.party["@id"], "did:key:z6MkpDYxrwJw5WoD1o4YVfthJJgZfxrECpW6Da6QCWagRHLx");
      assertEquals(updatePartyData.party.role, "new_role");
      assertEquals(updatePartyData.note, "Updating role after compliance check");
    }
  });

  await t.step("Message serialization and deserialization", () => {
    // This test needs to be implemented with standard TAP message types
    console.log("Serialization test not yet implemented with standard TAP message types");
  });

  await t.step("Reject setting wrong data type on message", () => {
    // Test setting UpdateParty data on a Transfer message (should fail)
    const transferMessage = new Message({ type: MessageType.TRANSFER });
    
    assertThrows(() => {
      transferMessage.setUpdatePartyData({
        transfer_id: "transfer-123",
        party_type: "originator",
        party: {
          "@id": "did:key:z6MkpDYxrwJw5WoD1o4YVfthJJgZfxrECpW6Da6QCWagRHLx",
          role: "new_role"
        }
      });
    }, Error, "Cannot set UpdateParty data on");
    
    // Test setting Transfer data on an UpdateParty message (should fail)
    const updatePartyMessage = new Message({ type: MessageType.UPDATE_PARTY });
    
    assertThrows(() => {
      updatePartyMessage.setTransferData({
        asset: "eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        originator: {
          "@id": "did:key:alice",
          role: "originator"
        },
        amount: "100.00",
        agents: [
          {
            "@id": "did:key:alice",
            role: "originator"
          }
        ]
      });
    }, Error, "Cannot set Transfer data on");
  });
});
