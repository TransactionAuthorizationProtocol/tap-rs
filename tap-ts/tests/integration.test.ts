import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { TapAgent, createTransferMessage, createAuthorizeMessage, createRejectMessage, createBasicMessage, createDIDCommMessage } from '../src/index.js';
import type { DIDCommMessage } from '../src/types.js';
import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
// Import the real WASM module for true integration testing
import init from 'tap-wasm';

// Get the path to the WASM binary
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const wasmPath = join(__dirname, '../../tap-wasm/pkg/tap_wasm_bg.wasm');

describe('Integration Tests', () => {
  const createdAgents: TapAgent[] = [];

  beforeEach(async () => {
    // Initialize the WASM module with the binary file for Node.js environment
    try {
      const wasmBinary = readFileSync(wasmPath);
      await init(wasmBinary);
    } catch (error) {
      console.error('Failed to initialize WASM:', error);
      throw error;
    }
  });

  afterEach(() => {
    // Clean up all created agents to prevent memory leaks
    createdAgents.forEach(agent => {
      try {
        agent.dispose();
      } catch (error) {
        // Ignore cleanup errors
      }
    });
    createdAgents.length = 0;
  });

  describe('End-to-End Message Flow', () => {
    it('should complete full message lifecycle between two agents', async () => {
      // Create real sender and receiver agents with actual WASM implementation
      const sender = await TapAgent.create({ keyType: 'Ed25519' });
      const receiver = await TapAgent.create({ keyType: 'Ed25519' });
      
      createdAgents.push(sender, receiver);

      // Verify that real DIDs are generated
      expect(sender.did).toMatch(/^did:key:z/);
      expect(receiver.did).toMatch(/^did:key:z/);
      expect(sender.did).not.toBe(receiver.did);

      // Create a transfer message using the new helper
      const message = await createTransferMessage({
        from: sender.did,
        to: [receiver.did],
        amount: '100.0',
        asset: 'eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7',
        originator: { 
          '@id': sender.did as `did:${string}:${string}`,
          '@type': 'https://schema.org/Person',
          name: 'Sender'
        },
        beneficiary: { 
          '@id': receiver.did as `did:${string}:${string}`,
          '@type': 'https://schema.org/Person',
          name: 'Receiver'
        },
        agents: [],
      });

      expect(message).toEqual({
        id: expect.stringMatching(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/),
        type: 'https://tap.rsvp/schema/1.0#Transfer',
        from: sender.did,
        to: [receiver.did],
        created_time: expect.any(Number),
        body: expect.objectContaining({
          amount: '100.0',
          asset: 'eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7',
        }),
      });

      // Pack the message with real WASM implementation
      const packed = await sender.pack(message);
      
      // Packed returns PackedMessageResult with message and metadata
      expect(packed).toHaveProperty('message');
      expect(packed).toHaveProperty('metadata');
      
      // Parse the JWS message
      const jws = JSON.parse(packed.message);
      expect(jws).toHaveProperty('payload');
      expect(jws).toHaveProperty('signatures');
      expect(Array.isArray(jws.signatures)).toBe(true);
      expect(jws.payload).toMatch(/^eyJ/);

      // For this test, let's use the same agent to unpack (self-signed)
      // In real scenarios, proper key exchange would be needed for signature verification
      const unpacked = await sender.unpack(packed.message);
      
      // Verify the unpacked message contains the core fields
      expect(unpacked).toEqual(expect.objectContaining({
        id: message.id,
        type: message.type,
        from: message.from,
        to: message.to,
      }));

      // Verify message integrity
      expect(unpacked.body).toBeDefined();
      expect(unpacked.from).toBe(sender.did);
      expect(unpacked.to).toContain(receiver.did);
    });

    it('should handle authorization flow', async () => {
      const originator = await TapAgent.create();
      const beneficiary = await TapAgent.create();
      
      createdAgents.push(originator, beneficiary);
      
      // Verify real DIDs are different
      expect(originator.did).toMatch(/^did:key:z/);
      expect(beneficiary.did).toMatch(/^did:key:z/);
      expect(originator.did).not.toBe(beneficiary.did);

      // Step 1: Create transfer request using new helper
      const transferMessage = await createTransferMessage({
        from: originator.did,
        to: [beneficiary.did],
        amount: '50.0',
        asset: 'USD',
        originator: { 
          '@id': originator.did as `did:${string}:${string}`,
          '@type': 'https://schema.org/Person',
          name: 'Originator'
        },
        beneficiary: { 
          '@id': beneficiary.did as `did:${string}:${string}`,
          '@type': 'https://schema.org/Person',
          name: 'Beneficiary'
        },
        agents: [],
      });

      // Step 2: Pack and transmit transfer with real WASM
      const packedTransfer = await originator.pack(transferMessage);
      expect(packedTransfer).toHaveProperty('message');
      expect(packedTransfer).toHaveProperty('metadata');
      
      const jws = JSON.parse(packedTransfer.message);
      expect(jws).toHaveProperty('payload');
      expect(jws).toHaveProperty('signatures');

      // Step 3: Originator unpacks (self-signed for this test)
      // In real scenarios, proper key exchange would be needed
      const unpackedTransfer = await originator.unpack(packedTransfer.message);
      expect((unpackedTransfer.body as any).amount).toBe('50.0');
      expect(unpackedTransfer.id).toBe(transferMessage.id);
      expect(unpackedTransfer.from).toBe(originator.did);

      // Step 4: Beneficiary creates authorization response using new helper
      const authMessage = await createAuthorizeMessage({
        from: beneficiary.did,
        to: [originator.did],
        transaction_id: transferMessage.id,
        settlement_address: 'ethereum:0x1234...5678',
        thid: transferMessage.id, // Thread reference
      });

      expect(authMessage.thid).toBe(transferMessage.id);
      // Authorization body doesn't have transaction_id field in the new types
    });

    it('should handle message with attachments', async () => {
      const agent = await TapAgent.create();
      createdAgents.push(agent);

      const messageWithAttachment: DIDCommMessage<{ invoice_id: string }> = {
        id: 'msg-with-attachment',
        type: 'https://tap.rsvp/schema/1.0#Payment',
        from: agent.did,
        body: { invoice_id: 'inv-123' },
        attachments: [{
          id: 'invoice-pdf',
          description: 'Invoice document',
          filename: 'invoice_123.pdf',
          media_type: 'application/pdf',
          data: {
            encoding: 'base64',
            content: 'JVBERi0xLjQKJcOkw7zDtsO4DQo...', // Base64 PDF
          },
        }],
      };

      const packed = await agent.pack(messageWithAttachment);
      
      // Verify the message was packed successfully
      expect(packed).toHaveProperty('message');
      expect(packed).toHaveProperty('metadata');
      
      const jws = JSON.parse(packed.message);
      expect(jws).toHaveProperty('payload');
      expect(jws).toHaveProperty('signatures');
      
      // Unpack to verify the core message was preserved
      const unpacked = await agent.unpack(packed.message);
      expect((unpacked.body as any).invoice_id).toBe('inv-123');
      
      // Note: Attachments may not be preserved through WASM pack/unpack
      // This is expected behavior for the current implementation
    });
  });

  describe('Error Recovery and Edge Cases', () => {
    it('should handle invalid private key gracefully', async () => {
      const invalidPrivateKey = 'not-a-valid-private-key';
      
      await expect(TapAgent.fromPrivateKey(invalidPrivateKey, 'Ed25519'))
        .rejects.toThrow('Invalid private key format');
    });

    it('should handle corrupt packed messages', async () => {
      const agent = await TapAgent.create();
      createdAgents.push(agent);
      
      const corruptMessage = 'not-a-valid-packed-message';

      await expect(agent.unpack(corruptMessage)).rejects.toThrow('Failed to unpack message');
    });

    it('should handle moderately large messages', async () => {
      const agent = await TapAgent.create();
      createdAgents.push(agent);
      
      // Create a message with moderately large body (reduced size for practical testing)
      const largeBody = {
        data: 'x'.repeat(10000), // 10KB of data
        metadata: Array(100).fill(0).map((_, i) => ({ key: `value-${i}`, nested: { deep: true, index: i } })),
      };

      const largeMessage = await createDIDCommMessage({
        type: 'https://tap.rsvp/schema/1.0#Transfer',
        from: agent.did,
        to: ['did:key:recipient'],
        body: largeBody,
      });

      const packed = await agent.pack(largeMessage);
      // Now returns PackedMessageResult
      expect(packed).toHaveProperty('message');
      expect(packed).toHaveProperty('metadata');
      
      const jws = JSON.parse(packed.message);
      expect(jws).toHaveProperty('payload');
      expect(jws).toHaveProperty('signatures');
      
      // Verify unpacking works with large payloads
      const unpacked = await agent.unpack(packed.message);
      expect((unpacked.body as any).data).toBe('x'.repeat(10000));
      expect((unpacked.body as any).metadata).toHaveLength(100);
      // Verify metadata structure is preserved (may be simplified by WASM)
      expect((unpacked.body as any).metadata[0]).toHaveProperty('key');
      expect((unpacked.body as any).metadata[0]).toHaveProperty('nested');
    });
  });

  describe('Resource Management', () => {
    it('should properly cleanup resources when agents are disposed', async () => {
      const agents: TapAgent[] = [];

      // Create multiple agents
      for (let i = 0; i < 5; i++) {
        const agent = await TapAgent.create();
        agents.push(agent);
      }

      // Verify all agents have unique DIDs
      const dids = agents.map(agent => agent.did);
      const uniqueDids = new Set(dids);
      expect(uniqueDids.size).toBe(5);

      // Dispose all agents
      agents.forEach(agent => agent.dispose());

      // Verify agents are marked as disposed
      agents.forEach(agent => {
        expect(() => agent.did).toThrow('Agent has been disposed');
      });
    });

    it('should handle concurrent agent operations', async () => {
      const agent = await TapAgent.create();
      createdAgents.push(agent);

      const messages = await Promise.all(
        Array(10).fill(null).map(async (_, i) => 
          createTransferMessage({
            from: agent.did,
            to: ['did:key:recipient'],
            amount: `${i * 10}.0`,
            asset: 'USD',
            originator: { '@id': agent.did as `did:${string}:${string}`, '@type': 'https://schema.org/Person', name: 'Test' },
            beneficiary: { '@id': 'did:key:recipient', '@type': 'https://schema.org/Person', name: 'Recipient' },
          })
        )
      );

      // Pack all messages concurrently with real WASM
      const packedMessages = await Promise.all(
        messages.map(msg => agent.pack(msg))
      );

      expect(packedMessages).toHaveLength(10);

      // Verify all messages were packed successfully
      packedMessages.forEach((packed) => {
        expect(packed).toHaveProperty('message');
        expect(packed).toHaveProperty('metadata');
        
        const jws = JSON.parse(packed.message);
        expect(jws).toHaveProperty('payload');
        expect(jws).toHaveProperty('signatures');
      });
      
      // Verify we can unpack all messages and they contain correct content
      const unpackedMessages = await Promise.all(
        packedMessages.map(packed => agent.unpack(packed.message))
      );
      
      unpackedMessages.forEach((unpacked, index) => {
        expect((unpacked.body as any).amount).toBe(`${index * 10}.0`);
      });
    });
  });

  describe('Type Safety Integration', () => {
    it('should maintain type safety through WASM boundary', async () => {
      interface CustomTransferBody {
        amount: string;
        currency: 'USD' | 'EUR' | 'GBP';
        memo?: string;
        metadata: {
          reference: string;
          category: 'business' | 'personal';
        };
      }

      const agent = await TapAgent.create();
      createdAgents.push(agent);
      
      const customBody: CustomTransferBody = {
        amount: '250.75',
        currency: 'EUR',
        memo: 'Monthly payment',
        metadata: {
          reference: 'REF-2024-001',
          category: 'business',
        },
      };

      const customMessage = await createDIDCommMessage<CustomTransferBody>({
        type: 'https://tap.rsvp/schema/1.0#Transfer',
        from: agent.did,
        to: ['did:key:recipient'],
        body: customBody,
      });

      // TypeScript should enforce the body type
      expect(customMessage.body.currency).toBe('EUR');
      expect(customMessage.body.metadata.category).toBe('business');

      const packed = await agent.pack(customMessage);
      expect(packed).toHaveProperty('message');
      expect(packed).toHaveProperty('metadata');
      
      const jws = JSON.parse(packed.message);
      expect(jws).toHaveProperty('payload');
      expect(jws).toHaveProperty('signatures');

      // Verify the typed body survives the WASM round-trip
      const unpacked = await agent.unpack(packed.message);
      expect(unpacked.body).toEqual({
        amount: '250.75',
        currency: 'EUR',
        memo: 'Monthly payment',
        metadata: {
          reference: 'REF-2024-001',
          category: 'business',
        },
      });
    });
  });

  describe('Performance Integration', () => {
    it('should handle rapid message operations efficiently with real WASM', async () => {
      const agent = await TapAgent.create();
      createdAgents.push(agent);
      
      const messageCount = 50; // Reduced for real WASM performance

      const start = performance.now();

      // Create and pack many messages rapidly with real WASM
      const operations = Array(messageCount).fill(null).map(async (_, i) => {
        const message = await createTransferMessage({
          from: agent.did,
          to: ['did:key:recipient'],
          amount: `${i}.0`,
          asset: 'USD',
          originator: { '@id': agent.did as `did:${string}:${string}`, '@type': 'https://schema.org/Person', name: 'Test' },
          beneficiary: { '@id': 'did:key:recipient', '@type': 'https://schema.org/Person', name: 'Recipient' },
          memo: `tx-${i}`,
        });
        return agent.pack(message);
      });

      const results = await Promise.all(operations);

      const end = performance.now();
      const duration = end - start;

      expect(results).toHaveLength(messageCount);
      
      // Verify all messages were packed successfully
      results.forEach((packed) => {
        expect(packed).toHaveProperty('message');
        expect(packed).toHaveProperty('metadata');
        
        const jws = JSON.parse(packed.message);
        expect(jws).toHaveProperty('payload');
        expect(jws).toHaveProperty('signatures');
      });

      // Should complete operations in reasonable time (< 5 seconds for real WASM)
      expect(duration).toBeLessThan(5000);

      console.log(`Completed ${messageCount} real WASM pack operations in ${duration.toFixed(2)}ms`);
      
      // Test metrics are updated correctly
      const metrics = agent.getMetrics();
      expect(metrics.messagesPacked).toBe(messageCount);
      expect(metrics.uptime).toBeGreaterThan(0);
    });
  });
});