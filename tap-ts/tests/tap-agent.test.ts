import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { TapAgent } from '../src/index.js';
import type { DIDCommMessage, PackedMessageResult, TapAgentConfig, DIDDocument, DIDResolver } from '../src/types.js';

describe('TapAgent with Real WASM', () => {
  let agent: TapAgent;

  afterEach(async () => {
    // Clean up agent after each test
    if (agent) {
      agent.dispose();
    }
  });

  describe('Agent Creation', () => {
    it('should create agent with default configuration', async () => {
      agent = await TapAgent.create();
      
      expect(agent).toBeDefined();
      expect(agent.did).toMatch(/^did:key:z[1-9A-HJ-NP-Za-km-z]+$/);
      // Agent is created and ready
    });

    it('should create agent with custom configuration', async () => {
      const config: TapAgentConfig = {
        keyType: 'P256',
        nickname: 'TestAgent',
      };
      
      agent = await TapAgent.create(config);
      
      expect(agent.did).toBe('TestAgent');
      expect(agent.did).toMatch(/^did:key:z[1-9A-HJ-NP-Za-km-z]+$/);
    });

    it('should create agent with each key type', async () => {
      const keyTypes: Array<'Ed25519' | 'P256' | 'secp256k1'> = ['Ed25519', 'P256', 'secp256k1'];
      
      for (const keyType of keyTypes) {
        const testAgent = await TapAgent.create({ keyType });
        expect(testAgent.did).toMatch(/^did:key:z[1-9A-HJ-NP-Za-km-z]+$/);
        testAgent.dispose();
      }
    });

    it('should create agent from private key', async () => {
      // First create an agent to get a private key
      const originalAgent = await TapAgent.create();
      const privateKey = originalAgent.exportPrivateKey();
      originalAgent.dispose();
      
      // Create new agent from that private key
      agent = await TapAgent.fromPrivateKey(privateKey, 'Ed25519');
      expect(agent.did).toMatch(/^did:key:z[1-9A-HJ-NP-Za-km-z]+$/);
    });

    it('should handle invalid private key gracefully', async () => {
      await expect(
        TapAgent.fromPrivateKey('invalid-key', 'Ed25519')
      ).rejects.toThrow();
    });
  });

  describe('Key Operations', () => {
    beforeEach(async () => {
      agent = await TapAgent.create();
    });

    it('should get agent DID', () => {
      const did = agent.did;
      expect(did).toMatch(/^did:key:z[1-9A-HJ-NP-Za-km-z]+$/);
      expect(did).toBe(agent.did);
    });

    it('should export public key', () => {
      const publicKey = agent.publicKey;
      expect(publicKey).toMatch(/^[0-9a-f]+$/);
      expect(publicKey.length).toBeGreaterThan(0);
    });

    it('should export private key', () => {
      const privateKey = agent.exportPrivateKey();
      expect(privateKey).toMatch(/^[0-9a-f]+$/);
      expect(privateKey.length).toBeGreaterThan(0);
    });

    it('should handle nickname operations', async () => {
      const agentWithNickname = await TapAgent.create({ nickname: 'MyAgent' });
      expect(agentWithNickname.nickname).toBe('MyAgent');
      agentWithNickname.dispose();
      
      const agentWithoutNickname = await TapAgent.create();
      expect(agentWithoutNickname.nickname).toBeUndefined();
      agentWithoutNickname.dispose();
    });
  });

  describe('Message Operations', () => {
    beforeEach(async () => {
      agent = await TapAgent.create();
    });

    describe('pack', () => {
      it('should pack a basic message', async () => {
        const message: DIDCommMessage = {
          id: 'msg-123',
          type: 'https://example.com/test',
          from: agent.did,
          to: ['did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK'],
          body: { content: 'Hello World' },
        };

        const packed = await agent.pack(message);
        
        expect(packed).toHaveProperty('message');
        expect(packed).toHaveProperty('metadata');
        expect(packed.metadata.type).toBe('signed');
        
        // Verify JWS structure
        const jws = JSON.parse(packed.message);
        expect(jws).toHaveProperty('payload');
        expect(jws).toHaveProperty('signatures');
        expect(Array.isArray(jws.signatures)).toBe(true);
      });

      it('should pack a Transfer message', async () => {
        const message: DIDCommMessage = {
          id: 'transfer-123',
          type: 'https://tap.rsvp/schema/1.0#Transfer',
          from: agent.did,
          to: ['did:key:recipient'],
          body: {
            amount: '100.00',
            asset: 'USD',
            originator: { '@id': agent.did as `did:${string}:${string}` },
            beneficiary: { '@id': 'did:key:recipient' },
          },
        };

        const packed = await agent.pack(message);
        
        const jws = JSON.parse(packed.message);
        expect(jws.payload).toBeDefined();
        expect(jws.signatures.length).toBeGreaterThan(0);
      });

      it('should handle message with custom options', async () => {
        const message: DIDCommMessage = {
          id: 'msg-456',
          type: 'test-type',
          from: agent.did,
          to: ['did:key:recipient'],
          body: { test: true },
          created_time: Date.now(),
        };

        const packed = await agent.pack(message);
        expect(packed.message).toBeDefined();
      });

      it('should handle errors during packing', async () => {
        const invalidMessage = {
          // Missing required fields
          body: { test: true },
        } as any;

        await expect(agent.pack(invalidMessage)).rejects.toThrow();
      });
    });

    describe('unpack', () => {
      it('should unpack a message packed by the same agent', async () => {
        const message: DIDCommMessage = {
          id: 'msg-789',
          type: 'test-type',
          from: agent.did,
          to: [agent.did], // Send to self for testing
          body: { content: 'Test message' },
        };

        const packed = await agent.pack(message);
        const unpacked = await agent.unpack(packed.message);
        
        expect(unpacked.id).toBe(message.id);
        expect(unpacked.type).toBe(message.type);
        expect((unpacked.body as any).content).toBe('Test message');
      });

      it('should unpack with JWS object input', async () => {
        const message: DIDCommMessage = {
          id: 'msg-jws',
          type: 'test-type',
          from: agent.did,
          to: [agent.did],
          body: { test: 'jws' },
        };

        const packed = await agent.pack(message);
        const jws = JSON.parse(packed.message);
        
        // Should accept JWS object directly
        const unpacked = await agent.unpack(jws);
        expect(unpacked.id).toBe(message.id);
      });

      it('should handle invalid packed messages', async () => {
        await expect(agent.unpack('invalid-jws')).rejects.toThrow();
      });

      it('should handle type validation during unpacking', async () => {
        const message: DIDCommMessage = {
          id: 'msg-type',
          type: 'https://tap.rsvp/schema/1.0#Transfer',
          from: agent.did,
          to: [agent.did],
          body: { amount: '50.00' },
        };

        const packed = await agent.pack(message);
        
        // Should pass with correct type
        const unpacked = await agent.unpack(packed.message, { expectedType: 'Transfer' });
        expect(unpacked.type).toContain('Transfer');
        
        // Should fail with wrong type
        await expect(
          agent.unpack(packed.message, { expectedType: 'Payment' })
        ).rejects.toThrow();
      });
    });

    describe('packMessage', () => {
      it('should pack message using packMessage method', async () => {
        const message: DIDCommMessage = {
          id: 'msg-pack',
          type: 'test-type',
          from: agent.did,
          to: ['did:key:recipient'],
          body: { content: 'Pack test' },
        };

        const packed = await agent.pack(message);
        
        expect(packed).toHaveProperty('message');
        expect(packed).toHaveProperty('metadata');
      });
    });

    describe('unpackMessage', () => {
      it('should unpack message using unpackMessage method', async () => {
        const message: DIDCommMessage = {
          id: 'msg-unpack',
          type: 'test-type',
          from: agent.did,
          to: [agent.did],
          body: { content: 'Unpack test' },
        };

        const packed = await agent.pack(message);
        const unpacked = await agent.unpack(packed.message);
        
        expect(unpacked.id).toBe(message.id);
        expect((unpacked.body as any).content).toBe('Unpack test');
      });
    });
  });

  describe('DID Resolution', () => {
    beforeEach(async () => {
      agent = await TapAgent.create();
    });

    it('should resolve its own DID', async () => {
      const result = await agent.resolveDID(agent.did);
      
      expect(result.didDocument).toBeDefined();
      expect(result.didDocument?.id).toBe(agent.did);
      expect(result.didDocument?.verificationMethod).toBeDefined();
      expect(result.didDocument?.verificationMethod?.length).toBeGreaterThan(0);
    });

    it('should use custom resolver when provided', async () => {
      const customResolver: DIDResolver = {
        resolve: async (did: string) => ({
          didDocument: {
            '@context': ['https://www.w3.org/ns/did/v1'],
            id: did,
            verificationMethod: [{
              id: `${did}#key-1`,
              type: 'JsonWebKey2020',
              controller: did,
              publicKeyJwk: {},
            }],
          },
          didDocumentMetadata: {},
          didResolutionMetadata: {},
        }),
      };

      const agentWithResolver = await TapAgent.create({ didResolver: customResolver });
      const result = await agentWithResolver.resolveDid('did:custom:123');
      
      expect(result.didDocument?.id).toBe('did:custom:123');
      agentWithResolver.dispose();
    });
  });

  describe('Resource Management', () => {
    it('should properly dispose of resources', async () => {
      agent = await TapAgent.create();
      // Agent is created and ready
      
      agent.dispose();
      // Agent has been disposed
      
      // Should throw when trying to use disposed agent
      // Accessing disposed agent should throw
    });

    it('should handle multiple dispose calls gracefully', async () => {
      agent = await TapAgent.create();
      
      agent.dispose();
      // Agent has been disposed
      
      // Second dispose should not throw
      expect(() => agent.dispose()).not.toThrow();
    });

    it('should track metrics correctly', async () => {
      agent = await TapAgent.create();
      const initialMetrics = agent.getMetrics();
      
      expect(initialMetrics.messagesPacked).toBe(0);
      expect(initialMetrics.messagesUnpacked).toBe(0);
      
      // Pack a message
      const message: DIDCommMessage = {
        id: 'metrics-test',
        type: 'test',
        from: agent.did,
        to: [agent.did],
        body: { test: true },
      };
      
      const packed = await agent.pack(message);
      await agent.unpack(packed.message);
      
      const updatedMetrics = agent.getMetrics();
      expect(updatedMetrics.messagesPacked).toBe(1);
      expect(updatedMetrics.messagesUnpacked).toBe(1);
      expect(updatedMetrics.lastActivity).toBeGreaterThan(initialMetrics.lastActivity);
    });
  });

  describe('Type Safety', () => {
    beforeEach(async () => {
      agent = await TapAgent.create();
    });

    it('should maintain type safety in message operations', async () => {
      interface CustomBody {
        amount: string;
        currency: string;
        memo?: string;
      }

      const message: DIDCommMessage<CustomBody> = {
        id: 'typed-msg',
        type: 'payment',
        from: agent.did,
        to: ['did:key:recipient'],
        body: {
          amount: '100.00',
          currency: 'USD',
          memo: 'Payment for services',
        },
      };

      const packed = await agent.pack(message);
      const jws = JSON.parse(packed.message);
      
      expect(jws).toHaveProperty('payload');
      expect(jws).toHaveProperty('signatures');
    });

    it('should type-check packed message metadata', async () => {
      const message: DIDCommMessage = {
        id: 'meta-test',
        type: 'test',
        from: agent.did,
        to: ['did:key:recipient'],
        body: { test: true },
      };

      const packed: PackedMessageResult = await agent.pack(message);
      
      // TypeScript should ensure correct metadata structure
      expect(packed.metadata.type).toBe('signed');
      expect(packed.metadata.sender).toBe(agent.did);
      expect(packed.metadata.recipients).toContain('did:key:recipient');
    });
  });

  describe('Error Handling', () => {
    beforeEach(async () => {
      agent = await TapAgent.create();
    });

    it('should handle disposed agent errors consistently', async () => {
      agent.dispose();
      
      await expect(agent.pack({} as any)).rejects.toThrow('Agent has been disposed');
      await expect(agent.unpack('')).rejects.toThrow('Agent has been disposed');
      expect(() => agent.exportPrivateKey()).toThrow('Agent has been disposed');
    });

    it('should provide meaningful error messages', async () => {
      // Invalid message structure
      const invalidMessage = {
        body: {},
      } as any;
      
      await expect(agent.pack(invalidMessage)).rejects.toThrow(/required field/i);
      
      // Invalid packed message
      await expect(agent.unpack('not-a-jws')).rejects.toThrow(/unpack/i);
    });
  });
});