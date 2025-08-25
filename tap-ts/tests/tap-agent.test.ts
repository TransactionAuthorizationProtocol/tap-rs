import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import type { DIDCommMessage, PackedMessage, PackedMessageResult, TapAgentConfig, DIDDocument, DIDResolver } from '../src/types.js';

// Shared mock instance that all tests will use
const mockWasmAgent = {
  free: vi.fn(),
  get_did: vi.fn(() => 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK'),
  exportPrivateKey: vi.fn(() => 'abcd1234567890abcd1234567890abcd1234567890abcd1234567890abcd1234'),
  exportPublicKey: vi.fn(() => '1234567890abcd1234567890abcd1234567890abcd1234567890abcd12345678'),
  packMessage: vi.fn(),
  unpackMessage: vi.fn(),
};

const mockWasmModule = {
  WasmTapAgent: Object.assign(vi.fn(() => mockWasmAgent), {
    fromPrivateKey: vi.fn().mockResolvedValue(mockWasmAgent),
  }),
  generatePrivateKey: vi.fn(() => 'generatedPrivateKey123'),
  generateUUID: vi.fn(() => 'uuid-1234-5678-9012'),
  WasmKeyType: {
    Ed25519: 0,
    P256: 1,
    Secp256k1: 2,
  },
  default: vi.fn().mockResolvedValue({}), // Mock WASM initialization
};

// Mock the tap-wasm import
vi.mock('tap-wasm', () => mockWasmModule);

// Import the class we're testing after mocking
const { TapAgent } = await import('../src/tap-agent.js');

describe('TapAgent', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Reset mock implementations - WASM returns wrapped format, TapAgent parses it
    mockWasmAgent.packMessage.mockResolvedValue({
      message: JSON.stringify({
        payload: 'eyJpZCI6Im1zZy0xMjMiLCJ0eXBlIjoiaHR0cHM6Ly90YXAucnN2cC9zY2hlbWEvMS4wI1RyYW5zZmVyIn0',
        signatures: [{
          protected: 'eyJhbGciOiJFZERTQSIsImtpZCI6ImRpZDprZXk6ejZNa2hhWGdCWkR2b3REa0w1MjU3ZmFpenRpR2lDMlF0S0xHcGJubkVHdGEyZG9LIn0',
          signature: 'mock-signature-value'
        }]
      }),
      metadata: {
        type: 'jws',
        recipients: ['did:key:recipient'],
        sender: 'did:key:sender',
        messageType: 'Transfer',
      },
    });
    mockWasmAgent.unpackMessage.mockResolvedValue({
      id: 'msg-123',
      type: 'https://tap.rsvp/schema/1.0#Transfer', // WASM uses 'type'
      from: 'did:key:sender',
      to: ['did:key:recipient'],
      created_time: Date.now(),
      body: { amount: '100.0' },
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('Static Factory Methods', () => {
    describe('create', () => {
      it('should create a new agent with default Ed25519 key', async () => {
        const agent = await TapAgent.create();
        
        expect(mockWasmModule.WasmTapAgent).toHaveBeenCalledWith({
          keyType: 'Ed25519',
        });
        expect(agent).toBeInstanceOf(TapAgent);
        expect(agent.did).toBe('did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK');
      });

      it('should create a new agent with specified key type', async () => {
        const config: TapAgentConfig = { keyType: 'P256' };
        const agent = await TapAgent.create(config);
        
        expect(mockWasmModule.WasmTapAgent).toHaveBeenCalledWith({
          keyType: 'P256',
        });
        expect(agent).toBeInstanceOf(TapAgent);
      });

      it('should create a new agent with nickname', async () => {
        const config: TapAgentConfig = {
          keyType: 'Ed25519',
          nickname: 'test-agent',
        };
        const agent = await TapAgent.create(config);
        
        expect(mockWasmModule.WasmTapAgent).toHaveBeenCalledWith({
          keyType: 'Ed25519',
          nickname: 'test-agent',
        });
        expect(agent).toBeInstanceOf(TapAgent);
      });

      it('should create agent with custom DID resolver', async () => {
        const mockResolver: DIDResolver = {
          resolve: vi.fn().mockResolvedValue({
            didDocument: {
              id: 'did:web:example.com',
              verificationMethod: [],
            },
            didResolutionMetadata: {},
            didDocumentMetadata: {},
          }),
        };

        const config: TapAgentConfig = {
          didResolver: mockResolver,
        };

        const agent = await TapAgent.create(config);
        expect(agent).toBeInstanceOf(TapAgent);
      });
    });

    describe('fromPrivateKey', () => {
      it('should create agent from existing private key with Ed25519', async () => {
        const privateKey = 'abcd1234567890abcd1234567890abcd1234567890abcd1234567890abcd1234';
        const agent = await TapAgent.fromPrivateKey(privateKey, 'Ed25519');
        
        expect(mockWasmModule.WasmTapAgent.fromPrivateKey).toHaveBeenCalledWith(
          privateKey,
          'Ed25519'
        );
        expect(agent).toBeInstanceOf(TapAgent);
      });

      it('should create agent from existing private key with specified key type', async () => {
        const privateKey = 'abcd1234567890abcd1234567890abcd1234567890abcd1234567890abcd1234';
        const agent = await TapAgent.fromPrivateKey(privateKey, 'P256');
        
        expect(mockWasmModule.WasmTapAgent.fromPrivateKey).toHaveBeenCalledWith(
          privateKey,
          'P256'
        );
        expect(agent).toBeInstanceOf(TapAgent);
      });

      it('should throw error for invalid private key', async () => {
        const invalidPrivateKey = 'not-a-valid-private-key';
        mockWasmModule.WasmTapAgent.fromPrivateKey.mockRejectedValue(
          new Error('Invalid private key format')
        );

        await expect(TapAgent.fromPrivateKey(invalidPrivateKey, 'Ed25519'))
          .rejects.toThrow('Invalid private key format');
      });

      it('should throw error for unsupported key type', async () => {
        const privateKey = 'abcd1234567890abcd1234567890abcd1234567890abcd1234567890abcd1234';
        
        await expect(TapAgent.fromPrivateKey(privateKey, 'InvalidKeyType' as any))
          .rejects.toThrow('Unsupported key type');
      });
    });
  });

  describe('Identity Management', () => {
    let agent: any;

    beforeEach(async () => {
      agent = await TapAgent.create();
    });

    it('should return the agent DID', () => {
      const did = agent.did;
      
      expect(did).toBe('did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK');
      expect(mockWasmAgent.get_did).toHaveBeenCalled();
    });

    it('should return the public key', () => {
      const publicKey = agent.publicKey;
      
      expect(publicKey).toBe('1234567890abcd1234567890abcd1234567890abcd1234567890abcd12345678');
      expect(mockWasmAgent.exportPublicKey).toHaveBeenCalled();
    });

    it('should export private key', () => {
      const privateKey = agent.exportPrivateKey();
      
      expect(privateKey).toBe('abcd1234567890abcd1234567890abcd1234567890abcd1234567890abcd1234');
      expect(mockWasmAgent.exportPrivateKey).toHaveBeenCalled();
    });

    it('should handle private key export errors', () => {
      mockWasmAgent.exportPrivateKey.mockImplementation(() => {
        throw new Error('Export failed');
      });

      expect(() => agent.exportPrivateKey()).toThrow('Failed to export private key');
    });
  });

  describe('Message Operations', () => {
    let agent: any;

    beforeEach(async () => {
      agent = await TapAgent.create();
    });

    describe('pack', () => {
      it('should pack a message successfully', async () => {
        const message: DIDCommMessage<{ amount: string }> = {
          id: 'msg-123',
          type: 'https://tap.rsvp/schema/1.0#Transfer',
          from: 'did:key:sender',
          to: ['did:key:recipient'],
          created_time: Date.now(),
          body: { amount: '100.0' },
        };

        const packed = await agent.pack(message);
        
        // Now returns PackedMessageResult
        expect(packed).toHaveProperty('message');
        expect(packed).toHaveProperty('metadata');
        
        // Parse the JWS from the message
        const jws = JSON.parse(packed.message);
        expect(jws).toHaveProperty('payload');
        expect(jws).toHaveProperty('signatures');
        expect(jws.payload).toBe('eyJpZCI6Im1zZy0xMjMiLCJ0eXBlIjoiaHR0cHM6Ly90YXAucnN2cC9zY2hlbWEvMS4wI1RyYW5zZmVyIn0');
        expect(jws.signatures[0].signature).toBe('mock-signature-value');
        
        // Verify the message was converted to WASM format
        expect(mockWasmAgent.packMessage).toHaveBeenCalledWith(
          expect.objectContaining({
            id: message.id,
            type: message.type, // WASM uses 'type' field
            from: message.from,
            to: message.to,
            body: message.body,
          })
        );
      });

      it('should pack message with custom options', async () => {
        const message: DIDCommMessage<{ amount: string }> = {
          id: 'msg-123',
          type: 'https://tap.rsvp/schema/1.0#Transfer',
          from: agent.did,
          to: ['did:key:recipient'],
          created_time: Date.now(),
          body: { amount: '100.0' },
        };

        const options = {
          to: ['did:key:custom-recipient'],
          expires_time: Date.now() + 3600000,
        };

        const packed = await agent.pack(message, options);
        
        expect(packed).toBeDefined();
        expect(packed).toHaveProperty('message');
        
        const jws = JSON.parse(packed.message);
        expect(jws).toHaveProperty('payload');
        expect(jws).toHaveProperty('signatures');
        
        // Verify the message includes the custom options and is converted to WASM format
        expect(mockWasmAgent.packMessage).toHaveBeenCalledWith(
          expect.objectContaining({
            id: message.id,
            type: message.type, // WASM uses 'type' field
            to: options.to,
            expires_time: options.expires_time,
            body: message.body,
          })
        );
      });

      it('should handle packing errors', async () => {
        const message: DIDCommMessage<{ amount: string }> = {
          id: 'msg-123',
          type: 'https://tap.rsvp/schema/1.0#Transfer',
          from: agent.did,
          to: ['did:key:recipient'],
          created_time: Date.now(),
          body: { amount: '100.0' },
        };

        mockWasmAgent.packMessage.mockRejectedValue(new Error('Packing failed'));

        await expect(agent.pack(message)).rejects.toThrow('Failed to pack message');
      });
    });

    describe('unpack', () => {
      it('should unpack a message successfully', async () => {
        const packedMessage = JSON.stringify({
          payload: 'eyJpZCI6Im1zZy0xMjMifQ',
          signatures: [{
            protected: 'eyJhbGciOiJFZERTQSJ9',
            signature: 'signature-value'
          }]
        });
        
        const unpacked = await agent.unpack(packedMessage);
        
        expect(unpacked).toEqual({
          id: 'msg-123',
          type: 'https://tap.rsvp/schema/1.0#Transfer',
          from: 'did:key:sender',
          to: ['did:key:recipient'],
          created_time: expect.any(Number),
          body: { amount: '100.0' },
        });
        expect(mockWasmAgent.unpackMessage).toHaveBeenCalledWith(packedMessage, undefined);
      });

      it('should unpack a JWS object directly', async () => {
        const packedMessage = {
          payload: 'eyJpZCI6Im1zZy0xMjMifQ',
          signatures: [{
            protected: 'eyJhbGciOiJFZERTQSJ9',
            signature: 'signature-value'
          }]
        };
        
        const unpacked = await agent.unpack(packedMessage);
        
        expect(unpacked).toEqual({
          id: 'msg-123',
          type: 'https://tap.rsvp/schema/1.0#Transfer',
          from: 'did:key:sender',
          to: ['did:key:recipient'],
          created_time: expect.any(Number),
          body: { amount: '100.0' },
        });
        expect(mockWasmAgent.unpackMessage).toHaveBeenCalledWith(JSON.stringify(packedMessage), undefined);
      });

      it('should unpack message with expected type', async () => {
        const packedMessage = 'packed-message-string';
        const options = { expectedType: 'Transfer' };
        
        const unpacked = await agent.unpack(packedMessage, options);
        
        expect(unpacked).toBeDefined();
        expect(mockWasmAgent.unpackMessage).toHaveBeenCalledWith(packedMessage, 'Transfer');
      });

      it('should handle unpacking errors', async () => {
        const packedMessage = 'invalid-packed-message';
        mockWasmAgent.unpackMessage.mockRejectedValue(new Error('Unpacking failed'));

        await expect(agent.unpack(packedMessage)).rejects.toThrow('Failed to unpack message');
      });

      it('should validate message age when maxAge option is provided', async () => {
        const oldTimestamp = Date.now() - 7200000; // 2 hours ago
        mockWasmAgent.unpackMessage.mockResolvedValue({
          id: 'msg-123',
          type: 'https://tap.rsvp/schema/1.0#Transfer', // WASM uses 'type'
          created_time: oldTimestamp,
          body: { amount: '100.0' },
        });

        const packedMessage = 'packed-message-string';
        const options = { maxAge: 3600 }; // 1 hour max age

        await expect(agent.unpack(packedMessage, options)).rejects.toThrow('Message too old');
      });
    });
  });

  describe('DID Resolution', () => {
    let agent: any;

    beforeEach(async () => {
      agent = await TapAgent.create();
    });

    it('should throw error when no resolver is provided', async () => {
      const didKey = 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK';
      
      await expect(agent.resolveDID(didKey)).rejects.toThrow('No DID resolver configured');
    });

    it('should use custom resolver when provided', async () => {
      const customDidDoc: DIDDocument = {
        id: 'did:web:example.com',
        verificationMethod: [{
          id: 'did:web:example.com#key-1',
          type: 'Ed25519VerificationKey2020',
          controller: 'did:web:example.com',
          publicKeyMultibase: 'z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK',
        }],
      };

      const mockResolver: DIDResolver = {
        resolve: vi.fn().mockResolvedValue({
          didDocument: customDidDoc,
          didResolutionMetadata: {},
          didDocumentMetadata: {},
        }),
      };

      const agent = await TapAgent.create({ didResolver: mockResolver });
      const result = await agent.resolveDID('did:web:example.com');

      expect(result.didDocument).toEqual(customDidDoc);
      expect(mockResolver.resolve).toHaveBeenCalledWith('did:web:example.com', undefined);
    });

    it('should handle DID resolution errors', async () => {
      const mockResolver: DIDResolver = {
        resolve: vi.fn().mockRejectedValue(new Error('Resolution failed')),
      };

      const agent = await TapAgent.create({ didResolver: mockResolver });
      
      await expect(agent.resolveDID('did:web:example.com'))
        .rejects.toThrow('Failed to resolve DID');
    });
  });

  describe('Utility Methods', () => {
    let agent: any;

    beforeEach(async () => {
      agent = await TapAgent.create();
    });

    it('should generate UUID for message IDs', async () => {
      const uuid = await agent.generateUUID();
      
      expect(uuid).toBe('uuid-1234-5678-9012');
      expect(mockWasmModule.generateUUID).toHaveBeenCalled();
    });

    it('should provide agent metrics', async () => {
      // Pack a message to update metrics
      const message: DIDCommMessage<{ content: string }> = {
        id: 'msg-123',
        type: 'https://didcomm.org/basicmessage/2.0/message',
        from: agent.did,
        to: ['did:key:recipient'],
        body: { content: 'test' },
      };

      await agent.pack(message);
      
      const metrics = agent.getMetrics();
      
      expect(metrics).toEqual({
        messagesPacked: 1,
        messagesUnpacked: 0,
        keyOperations: 0,
        uptime: expect.any(Number),
        lastActivity: expect.any(Number),
      });
      expect(metrics.uptime).toBeGreaterThanOrEqual(0);
    });
  });

  describe('Resource Management', () => {
    it('should cleanup resources when disposed', async () => {
      const agent: any = await TapAgent.create();
      
      agent.dispose();
      
      expect(mockWasmAgent.free).toHaveBeenCalled();
      
      // Should throw error when accessing after disposal
      expect(() => agent.did).toThrow('Agent has been disposed');
    });

    it('should handle multiple dispose calls gracefully', async () => {
      const agent: any = await TapAgent.create();
      
      agent.dispose();
      agent.dispose(); // Second call should not throw
      
      expect(mockWasmAgent.free).toHaveBeenCalledTimes(1);
    });
  });

  describe('Error Handling', () => {
    let agent: any;

    beforeEach(async () => {
      agent = await TapAgent.create();
    });

    it('should wrap WASM errors with typed errors', async () => {
      mockWasmAgent.get_did.mockImplementation(() => {
        throw new Error('WASM operation failed');
      });

      expect(() => agent.did).toThrow('Failed to get agent DID');
    });

    it('should provide helpful error messages for common failures', async () => {
      const message: DIDCommMessage<{ content: string }> = {
        id: '',  // Invalid ID
        type: 'https://didcomm.org/basicmessage/2.0/message',
        from: agent.did,
        to: [],  // Invalid recipients
        body: { content: 'test' },
      };

      await expect(agent.pack(message)).rejects.toThrow();
    });
  });

  describe('Type Safety', () => {
    let agent: any;

    beforeEach(async () => {
      agent = await TapAgent.create();
    });

    it('should maintain type safety in message operations', async () => {
      interface CustomBody {
        amount: string;
        currency: 'USD' | 'EUR';
        memo?: string;
      }

      const message: DIDCommMessage<CustomBody> = {
        id: 'msg-123',
        type: 'https://tap.rsvp/schema/1.0#Transfer',
        from: agent.did,
        to: ['did:key:recipient'],
        body: {
          amount: '100.50',
          currency: 'USD',
          memo: 'Payment for services',
        },
      };

      const packed = await agent.pack(message);
      
      expect(packed).toBeDefined();
      expect(packed).toHaveProperty('message');
      expect(packed).toHaveProperty('metadata');
      
      // Parse the JWS to check its structure
      const jws = JSON.parse(packed.message);
      expect(jws).toHaveProperty('payload');
      expect(jws).toHaveProperty('signatures');
    });

    it('should type-check packed message metadata', async () => {
      const message: DIDCommMessage<{ content: string }> = {
        id: 'msg-123',
        type: 'https://didcomm.org/basicmessage/2.0/message',
        from: agent.did,
        to: ['did:key:recipient'],
        body: { content: 'test' },
      };

      const packedResult: PackedMessageResult = await agent.pack(message);
      const packed: PackedMessage = JSON.parse(packedResult.message);
      
      // TypeScript should ensure PackedMessage is either JWS or JWE
      if ('signatures' in packed) {
        // JWS format
        expect(packed.payload).toBeDefined();
        expect(packed.signatures).toBeInstanceOf(Array);
      } else {
        // JWE format
        expect(packed.protected).toBeDefined();
        expect(packed.ciphertext).toBeDefined();
      }
    });
  });
});