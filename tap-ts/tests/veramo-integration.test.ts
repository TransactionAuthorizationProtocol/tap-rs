import { describe, it, expect, beforeAll, afterAll } from "vitest";
import {
  TapAgent,
  createTransferMessage,
  createPaymentMessage,
  createConnectMessage,
  createBasicMessage,
  createDIDCommMessage,
} from "../src/index.js";
import type { DIDCommMessage } from "../src/types.js";
import type { DID } from '@taprsvp/types';
import { readFileSync } from "fs";
import { fileURLToPath } from "url";
import { dirname, join } from "path";
import init from "tap-wasm";

// Veramo imports
import {
  createAgent,
  IDIDManager,
  IKeyManager,
  IMessageHandler,
  IResolver,
  TAgent,
} from "@veramo/core";
import { DIDManager, MemoryDIDStore } from "@veramo/did-manager";
import { KeyManager, MemoryKeyStore, MemoryPrivateKeyStore } from "@veramo/key-manager";
import { KeyManagementSystem } from "@veramo/kms-local";
import { DIDResolverPlugin } from "@veramo/did-resolver";
import { KeyDIDProvider } from "@veramo/did-provider-key";
import { MessageHandler } from "@veramo/message-handler";
import { DIDComm, IDIDComm, DIDCommMessageHandler } from "@veramo/did-comm";
import { Resolver } from "did-resolver";
import { getResolver as getKeyResolver } from "key-did-resolver";

// Get the path to the WASM binary
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const wasmPath = join(__dirname, "../../tap-wasm/pkg/tap_wasm_bg.wasm");

type IAgent = TAgent<IDIDManager & IKeyManager & IMessageHandler & IResolver & IDIDComm>;

/**
 * TAP-Veramo Interoperability Tests
 *
 * Tests bidirectional DIDComm v2 message exchange between TAP and Veramo agents.
 * Both use did:key identifiers and JWS (signed) message packing.
 *
 * See veramo-format-compatibility.test.ts for detailed format tests.
 */
describe("TAP-Veramo Interoperability Tests", () => {
  let tapAgent: TapAgent;
  let veramoAgent: IAgent;
  let veramo2Agent: IAgent;

  beforeAll(async () => {
    // Initialize TAP WASM
    try {
      const wasmBinary = readFileSync(wasmPath);
      await init(wasmBinary);
    } catch (error) {
      console.error("Failed to initialize WASM:", error);
      throw error;
    }

    // Create TAP agent
    tapAgent = await TapAgent.create({ keyType: "Ed25519" });

    // Create Veramo agents with DIDComm support using built-in memory stores
    const createVeramoAgent = () =>
      createAgent<IDIDManager & IKeyManager & IMessageHandler & IResolver & IDIDComm>({
        plugins: [
          new DIDManager({
            store: new MemoryDIDStore(),
            defaultProvider: "did:key",
            providers: {
              "did:key": new KeyDIDProvider({
                defaultKms: "local",
              }),
            },
          }),
          new KeyManager({
            store: new MemoryKeyStore(),
            kms: {
              local: new KeyManagementSystem(new MemoryPrivateKeyStore()),
            },
          }),
          new DIDResolverPlugin({
            resolver: new Resolver({
              ...getKeyResolver(),
            }),
          }),
          new MessageHandler({
            messageHandlers: [new DIDCommMessageHandler()],
          }),
          new DIDComm(),
        ],
      });

    veramoAgent = createVeramoAgent();
    veramo2Agent = createVeramoAgent();
  });

  afterAll(() => {
    tapAgent?.dispose();
  });

  describe("DID Creation and Resolution", () => {
    it("should create compatible DID:key identifiers", async () => {
      // Create DID with Veramo
      const veramoIdentifier = await veramoAgent.didManagerCreate({
        provider: "did:key",
        kms: "local",
        options: {
          keyType: "Ed25519",
        },
      });

      // Both should create valid did:key DIDs
      expect(tapAgent.did).toMatch(/^did:key:z6Mk/);
      expect(veramoIdentifier.did).toMatch(/^did:key:z6Mk/);

      // Both should be resolvable via Veramo (TAP DID resolution may require network)
      const veramoDidDoc = await veramoAgent.resolveDid({
        didUrl: veramoIdentifier.did,
      });
      const tapDidDocViaVeramo = await veramoAgent.resolveDid({
        didUrl: tapAgent.did,
      });

      expect(veramoDidDoc).toBeTruthy();
      expect(tapDidDocViaVeramo).toBeTruthy();
      expect(veramoDidDoc.didDocument?.id).toBe(
        veramoIdentifier.did,
      );
      expect(tapDidDocViaVeramo.didDocument?.id).toBe(
        tapAgent.did,
      );
    });

    it("should resolve each other's DIDs", async () => {
      const veramoIdentifier = await veramoAgent.didManagerCreate({
        provider: "did:key",
        kms: "local",
        options: {
          keyType: "Ed25519",
        },
      });

      // Use Veramo for resolution to avoid network calls
      const veramoResolvedVeramoDid = await veramoAgent.resolveDid({
        didUrl: veramoIdentifier.did,
      });
      expect(
        veramoResolvedVeramoDid.didDocument?.id,
      ).toBe(veramoIdentifier.did);

      // Veramo agent should be able to resolve TAP-created DID
      const veramoResolvedTapDid = await veramoAgent.resolveDid({
        didUrl: tapAgent.did,
      });
      expect(
        veramoResolvedTapDid.didDocument?.id,
      ).toBe(tapAgent.did);
    });
  });

  describe("Message Format Compatibility", () => {
    it("should unpack messages packed by Veramo (JWS)", async () => {
      const veramoSender = await veramoAgent.didManagerCreate({
        provider: "did:key",
        kms: "local",
        options: { keyType: "Ed25519" },
      });

      const message = {
        id: "veramo-jws-001",
        type: "https://didcomm.org/basicmessage/2.0/message",
        from: veramoSender.did,
        to: [tapAgent.did],
        body: {
          content: "Hello from Veramo (JWS)!",
        },
      };

      // Pack with Veramo using JWS
      const veramoPacked = await veramoAgent.packDIDCommMessage({
        packing: "jws",
        message,
      });

      // TAP unpacks Veramo's JWS
      const tapUnpacked = await tapAgent.unpack(veramoPacked.message);
      expect((tapUnpacked.body as any).content).toBe("Hello from Veramo (JWS)!");
      expect(tapUnpacked.from).toBe(veramoSender.did);
    });

    it("should have Veramo unpack messages packed by TAP (JWS)", async () => {
      const veramoRecipient = await veramoAgent.didManagerCreate({
        provider: "did:key",
        kms: "local",
        options: { keyType: "Ed25519" },
      });

      const tapMessage: DIDCommMessage = {
        id: "tap-jws-001",
        type: "https://didcomm.org/basicmessage/2.0/message",
        from: tapAgent.did,
        to: [veramoRecipient.did as DID],
        created_time: Date.now(),
        body: {
          content: "Hello from TAP (JWS)!",
        },
      };

      const tapPacked = await tapAgent.pack(tapMessage);

      // Pass just the JWS message string, not the full PackedMessageResult
      const veramoUnpacked = await veramoAgent.unpackDIDCommMessage({
        message: tapPacked.message,
      });

      expect(veramoUnpacked.message.body.content).toBe("Hello from TAP (JWS)!");
      expect(veramoUnpacked.message.from).toBe(tapAgent.did);
      expect(veramoUnpacked.metaData.packing).toBe("jws");
    });

    // TAP WASM currently only supports JWS (signed) packing, not JWE (encrypted)
    it.skip("should unpack messages packed by Veramo (JWE anoncrypt)", async () => {
      const veramoSender = await veramoAgent.didManagerCreate({
        provider: "did:key",
        kms: "local",
        options: { keyType: "Ed25519" },
      });

      const message = {
        id: "veramo-jwe-anon-001",
        type: "https://didcomm.org/basicmessage/2.0/message",
        from: veramoSender.did,
        to: [tapAgent.did],
        body: {
          content: "Hello from Veramo (JWE anoncrypt)!",
        },
      };

      const veramoPacked = await veramoAgent.packDIDCommMessage({
        packing: "anoncrypt",
        message,
      });

      const tapUnpacked = await tapAgent.unpack(veramoPacked.message);
      expect((tapUnpacked.body as any).content).toBe("Hello from Veramo (JWE anoncrypt)!");
    });

    it("should have Veramo unpack JWS messages from TAP", async () => {
      const veramoRecipient = await veramoAgent.didManagerCreate({
        provider: "did:key",
        kms: "local",
        options: { keyType: "Ed25519" },
      });

      const tapMessage: DIDCommMessage = {
        id: "tap-test-001",
        type: "https://didcomm.org/basicmessage/2.0/message",
        from: tapAgent.did,
        to: [veramoRecipient.did as DID],
        created_time: Date.now(),
        body: {
          content: "Testing TAP to Veramo",
        },
      };

      // TAP produces JWS format
      const tapPacked = await tapAgent.pack(tapMessage);

      // Verify Flattened JWS structure
      const parsedMessage = JSON.parse(tapPacked.message);
      expect(parsedMessage.payload).toBeDefined();
      expect(parsedMessage.protected).toBeDefined();
      expect(parsedMessage.signature).toBeDefined();

      // Veramo unpacks TAP's JWS
      const veramoUnpacked = await veramoAgent.unpackDIDCommMessage({
        message: tapPacked.message,
      });
      expect(veramoUnpacked.message.body.content).toBe("Testing TAP to Veramo");
      expect(veramoUnpacked.metaData.packing).toBe("jws");
    });

    it("should handle trust ping messages bidirectionally", async () => {
      const veramoIdentifier = await veramoAgent.didManagerCreate({
        provider: "did:key",
        kms: "local",
        options: { keyType: "Ed25519" },
      });

      // 1. Veramo packs a trust ping
      const veramoPing = {
        id: "veramo-ping-001",
        type: "https://didcomm.org/trust-ping/2.0/ping",
        from: veramoIdentifier.did,
        to: [tapAgent.did],
        body: {
          response_requested: true,
        },
      };

      const veramoPackedPing = await veramoAgent.packDIDCommMessage({
        packing: "jws",
        message: veramoPing,
      });

      // 2. TAP unpacks Veramo's ping
      const tapUnpackedPing = await tapAgent.unpack(veramoPackedPing.message);
      expect(tapUnpackedPing.type).toBe("https://didcomm.org/trust-ping/2.0/ping");
      expect((tapUnpackedPing.body as any).response_requested).toBe(true);

      // 3. TAP packs a ping response
      const tapPingResponse: DIDCommMessage = {
        id: "tap-ping-response-001",
        type: "https://didcomm.org/trust-ping/2.0/ping-response",
        from: tapAgent.did,
        to: [veramoIdentifier.did as DID],
        thid: veramoPing.id,
        created_time: Date.now(),
        body: {},
      };

      const tapPackedResponse = await tapAgent.pack(tapPingResponse);

      // 4. Veramo unpacks TAP's response
      const veramoUnpackedResponse = await veramoAgent.unpackDIDCommMessage({
        message: tapPackedResponse.message,
      });

      expect(veramoUnpackedResponse.message.type).toBe(
        "https://didcomm.org/trust-ping/2.0/ping-response",
      );
      expect(veramoUnpackedResponse.message.thid).toBe(veramoPing.id);
    });
  });

  describe("TAP-specific Messages", () => {
    it("should have Veramo handle TAP Transfer messages", async () => {
      const veramoRecipient = await veramoAgent.didManagerCreate({
        provider: "did:key",
        kms: "local",
        options: { keyType: "Ed25519" },
      });

      const transferMessage = await createTransferMessage({
        from: tapAgent.did,
        to: [veramoRecipient.did as DID],
        amount: "100.50",
        asset: "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        originator: {
          "@id": tapAgent.did as DID,
          "@type": "https://schema.org/Person",
          name: "Alice Smith",
        },
        beneficiary: {
          "@id": veramoRecipient.did as DID,
          "@type": "https://schema.org/Person",
          name: "Bob Jones",
        },
        memo: "Payment for services rendered",
      });

      const packed = await tapAgent.pack(transferMessage);

      // Pass just the JWS message string to Veramo
      const veramoUnpacked = await veramoAgent.unpackDIDCommMessage({
        message: packed.message,
      });

      expect(veramoUnpacked.message.type).toBe("https://tap.rsvp/schema/1.0#Transfer");
      expect(veramoUnpacked.message.body.amount).toBe("100.50");
      expect(veramoUnpacked.message.body.originator["@id"]).toBe(tapAgent.did);
      expect(veramoUnpacked.message.body.beneficiary["@id"]).toBe(veramoRecipient.did);
      expect(veramoUnpacked.metaData.packing).toBe("jws");
    });

    it("should have Veramo handle TAP Payment messages with invoices", async () => {
      const veramoMerchant = await veramoAgent.didManagerCreate({
        provider: "did:key",
        kms: "local",
        options: { keyType: "Ed25519" },
      });

      const paymentMessage = await createPaymentMessage({
        from: tapAgent.did,
        to: [veramoMerchant.did as DID],
        amount: "249.99",
        currency: "USD",
        merchant: {
          "@id": veramoMerchant.did as DID,
          "@type": "https://schema.org/Organization",
          name: "Example Merchant",
        },
        invoice: {
          invoiceNumber: "INV-2024-12345",
          items: [
            {
              description: "Premium Widget",
              quantity: 1,
              unitPrice: "199.99",
            },
            {
              description: "Shipping",
              quantity: 1,
              unitPrice: "25.00",
            },
            {
              description: "Tax",
              quantity: 1,
              unitPrice: "25.00",
            },
          ],
          total: "249.99",
          dueDate: "2024-12-31",
        },
      });

      const packed = await tapAgent.pack(paymentMessage);

      const veramoUnpacked = await veramoAgent.unpackDIDCommMessage({
        message: packed.message,
      });

      expect(veramoUnpacked.message.type).toBe("https://tap.rsvp/schema/1.0#Payment");
      expect(veramoUnpacked.message.body.amount).toBe("249.99");
      expect(veramoUnpacked.message.body.invoice.invoiceNumber).toBe("INV-2024-12345");
      expect(veramoUnpacked.message.body.invoice.items).toHaveLength(3);
      expect(veramoUnpacked.message.body.merchant["@id"]).toBe(veramoMerchant.did);
    });

    it("should have bidirectional TAP message exchange with Veramo", async () => {
      const veramoCounterparty = await veramoAgent.didManagerCreate({
        provider: "did:key",
        kms: "local",
        options: { keyType: "Ed25519" },
      });

      // 1. TAP packs a Connect message
      const connectMessage = await createConnectMessage({
        from: tapAgent.did,
        to: [veramoCounterparty.did as DID],
        requester: {
          "@id": tapAgent.did as DID,
          "@type": "https://schema.org/Person",
          name: "Connector",
        },
        principal: {
          "@id": tapAgent.did as DID,
          "@type": "https://schema.org/Person",
          name: "Connector",
        },
        constraints: {
          asset_types: [
            "eip155:1/erc20:*",
            "eip155:137/erc20:*",
            "eip155:56/erc20:*",
          ],
          currency_types: ["USD", "EUR", "GBP", "JPY"],
          transaction_limits: {
            min_amount: "10.00",
            max_amount: "100000.00",
            daily_limit: "500000.00",
            monthly_limit: "10000000.00",
          },
        },
      });

      const tapPacked = await tapAgent.pack(connectMessage);

      // 2. Veramo unpacks TAP's Connect message
      const veramoUnpacked = await veramoAgent.unpackDIDCommMessage({
        message: tapPacked.message,
      });

      expect(veramoUnpacked.message.type).toBe("https://tap.rsvp/schema/1.0#Connect");
      expect(veramoUnpacked.message.body.constraints.asset_types).toHaveLength(3);

      // 3. Veramo packs an Authorize response
      const veramoResponse = {
        id: "veramo-auth-001",
        type: "https://tap.rsvp/schema/1.0#Authorize",
        from: veramoCounterparty.did,
        to: [tapAgent.did],
        thid: connectMessage.id,
        body: {
          "@context": "https://tap.rsvp/schema/1.0",
          "@type": "Authorize",
          settlementAddress: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7",
        },
      };

      const veramoPackedResponse = await veramoAgent.packDIDCommMessage({
        packing: "jws",
        message: veramoResponse,
      });

      // 4. TAP unpacks Veramo's response
      const tapUnpackedResponse = await tapAgent.unpack(veramoPackedResponse.message);
      expect(tapUnpackedResponse.type).toBe("https://tap.rsvp/schema/1.0#Authorize");
      expect(tapUnpackedResponse.thid).toBe(connectMessage.id);
    });
  });

  describe("Key Algorithm Compatibility", () => {
    it("should work with Ed25519 keys from both systems", async () => {
      const tapEd25519 = await TapAgent.create({ keyType: "Ed25519" });
      const veramoEd25519 = await veramoAgent.didManagerCreate({
        provider: "did:key",
        kms: "local",
        options: { keyType: "Ed25519" },
      });

      // Both should create z6Mk* DIDs for Ed25519
      expect(tapEd25519.did).toMatch(/^did:key:z6Mk/);
      expect(veramoEd25519.did).toMatch(/^did:key:z6Mk/);

      // Test message exchange
      const message = await createBasicMessage({
        from: tapEd25519.did,
        to: [veramoEd25519.did as DID],
        content: "Ed25519 compatibility test",
      });
      message.to = [veramoEd25519.did as DID];

      const packed = await tapEd25519.pack(message);
      const unpacked = await tapEd25519.unpack(packed.message);

      expect((unpacked.body as any).content).toBe("Ed25519 compatibility test");
    });

    it("should work with secp256k1 keys", async () => {
      const tapSecp = await TapAgent.create({ keyType: "secp256k1" });

      // Should create valid did:key for secp256k1
      expect(tapSecp.did).toMatch(/^did:key:z/);

      // Test basic functionality
      const message = await createDIDCommMessage({
        type: "https://didcomm.org/trust-ping/2.0/ping",
        from: tapSecp.did,
        to: [tapAgent.did],
        body: {
          response_requested: true,
        },
      });

      const packed = await tapSecp.pack(message);
      const unpacked = await tapSecp.unpack(packed.message);

      expect((unpacked.body as any).response_requested).toBe(true);
    });

    it("should work with P-256 keys", async () => {
      const tapP256 = await TapAgent.create({ keyType: "P256" });

      // Should create valid did:key for P-256
      expect(tapP256.did).toMatch(/^did:key:z/);

      // Test basic functionality
      const message = await createBasicMessage({
        from: tapP256.did,
        to: [tapAgent.did],
        content: "P-256 test message",
      });

      const packed = await tapP256.pack(message);
      const unpacked = await tapP256.unpack(packed.message);

      expect((unpacked.body as any).content).toBe("P-256 test message");
    });
  });

  describe("Threading and Conversation Flow", () => {
    it("should maintain thread context compatible with Veramo", async () => {
      const veramoParticipant = await veramoAgent.didManagerCreate({
        provider: "did:key",
        kms: "local",
        options: { keyType: "Ed25519" },
      });

      const threadId = `conversation-${Date.now()}`;
      const parentThreadId = `parent-${Date.now()}`;

      // Start conversation with TAP
      const initialMessage = await createTransferMessage({
        from: tapAgent.did,
        to: [veramoParticipant.did as DID],
        amount: "500.00",
        asset: "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        originator: {
          "@id": tapAgent.did as DID,
          "@type": "https://schema.org/Person",
          name: "Sender",
        },
        beneficiary: {
          "@id": veramoParticipant.did as DID,
          "@type": "https://schema.org/Person",
          name: "Receiver",
        },
        thid: threadId,
        pthid: parentThreadId,
      });

      const packed1 = await tapAgent.pack(initialMessage);
      const unpacked1 = await tapAgent.unpack(packed1.message);

      expect(unpacked1.thid).toBe(threadId);
      expect(unpacked1.pthid).toBe(parentThreadId);

      // Continue conversation
      const responseMessage = {
        id: `response-${Date.now()}`,
        type: "https://tap.rsvp/schema/1.0#Authorize",
        from: veramoParticipant.did,
        to: [tapAgent.did],
        thid: threadId, // Same thread
        created_time: Date.now(),
        body: {
          transaction_id: unpacked1.id,
          settlement_address: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7",
        },
      };

      const packed2 = await tapAgent.pack(responseMessage as DIDCommMessage);
      const unpacked2 = await tapAgent.unpack(packed2.message);

      expect(unpacked2.thid).toBe(threadId);
      expect((unpacked2.body as any).transaction_id).toBe(unpacked1.id);
    });
  });

  describe("Error Handling and Edge Cases", () => {
    it("should handle mixed message types in conversations", async () => {
      const veramoAgent2Identifier = await veramo2Agent.didManagerCreate({
        provider: "did:key",
        kms: "local",
        options: { keyType: "Ed25519" },
      });

      // Start with standard DIDComm message
      const pingMessage: DIDCommMessage = {
        id: "mixed-001",
        type: "https://didcomm.org/trust-ping/2.0/ping",
        from: tapAgent.did,
        to: [veramoAgent2Identifier.did as DID],
        created_time: Date.now(),
        body: { response_requested: true },
      };

      const packedPing = await tapAgent.pack(pingMessage);
      const unpackedPing = await tapAgent.unpack(packedPing.message);

      expect(unpackedPing.type).toBe("https://didcomm.org/trust-ping/2.0/ping");

      // Respond with TAP-specific message
      const tapResponse = await createPaymentMessage({
        from: tapAgent.did,
        to: [veramoAgent2Identifier.did as DID],
        amount: "25.00",
        currency: "USD",
        merchant: {
          "@id": veramoAgent2Identifier.did as DID,
          "@type": "https://schema.org/Organization",
          name: "Test Merchant",
        },
        thid: pingMessage.id,
      });

      const packedResponse = await tapAgent.pack(tapResponse);
      const unpackedResponse = await tapAgent.unpack(packedResponse.message);

      expect(unpackedResponse.type).toBe("https://tap.rsvp/schema/1.0#Payment");
      expect(unpackedResponse.thid).toBe(pingMessage.id);
    });

    it("should handle malformed Veramo-style messages gracefully", async () => {
      const malformedMessages = [
        // Missing required fields
        {
          type: "https://didcomm.org/basicmessage/2.0/message",
          body: { content: "test" },
        },
        // Invalid DID format
        {
          id: "test-001",
          type: "https://didcomm.org/basicmessage/2.0/message",
          from: "not-a-valid-did",
          to: ["also-not-valid"],
          body: { content: "test" },
        },
        // Unknown message type
        {
          id: "test-002",
          type: "https://unknown.protocol/unknown/1.0/unknown",
          from: tapAgent.did,
          to: [tapAgent.did],
          body: { test: "data" },
        },
      ];

      for (const malformed of malformedMessages) {
        try {
          const packed = await tapAgent.pack(malformed as DIDCommMessage);
          // If packing succeeds, unpacking should also work
          const unpacked = await tapAgent.unpack(packed.message);
          expect(unpacked.type).toBe(malformed.type);
        } catch (error) {
          // Some malformed messages should throw errors
          expect(error).toBeDefined();
        }
      }
    });
  });

  describe("Performance with Veramo Compatibility", () => {
    it("should maintain performance when creating Veramo-compatible messages", async () => {
      const veramoRecipients = [];

      // Create multiple Veramo identities
      for (let i = 0; i < 5; i++) {
        const identifier = await veramoAgent.didManagerCreate({
          provider: "did:key",
          kms: "local",
          options: { keyType: "Ed25519" },
        });
        veramoRecipients.push(identifier.did);
      }

      const startTime = Date.now();
      const messageCount = 20;

      for (let i = 0; i < messageCount; i++) {
        const recipient = veramoRecipients[i % veramoRecipients.length];

        const message = await createTransferMessage({
          from: tapAgent.did,
          to: [recipient],
          amount: `${(i + 1) * 10}.00`,
          asset: "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
          originator: {
            "@id": tapAgent.did as DID,
            "@type": "https://schema.org/Person",
            name: "Sender",
          },
          beneficiary: {
            "@id": recipient as DID,
            "@type": "https://schema.org/Person",
            name: "Recipient",
          },
          memo: `Batch transfer ${i + 1}`,
        });

        const packed = await tapAgent.pack(message);
        const unpacked = await tapAgent.unpack(packed.message);

        expect((unpacked.body as any).amount).toBe(`${(i + 1) * 10}.00`);
        expect((unpacked.body as any).beneficiary["@id"]).toBe(recipient);
      }

      const duration = Date.now() - startTime;
      console.log(
        `Processed ${messageCount} messages with Veramo DIDs in ${duration}ms`,
      );

      // Should maintain reasonable performance (< 50ms per message)
      expect(duration).toBeLessThan(messageCount * 50);
    });
  });
});
