import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import type { DIDCommMessage, PackedMessage, TapAgentConfig, DIDDocument, DIDResolver } from '../src/types.js';

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
    // Reset mock implementations
    mockWasmAgent.packMessage.mockResolvedValue({
      message: 'packed-message-content',
      metadata: {
        type: 'encrypted',
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
          keyType: 'secp256k1',
          nickname: 'test-agent' 
        };
        const agent = await TapAgent.create(config);
        
        expect(mockWasmModule.WasmTapAgent).toHaveBeenCalledWith({
          keyType: 'secp256k1',
          nickname: 'test-agent',
        });
        expect(agent).toBeInstanceOf(TapAgent);
      });

      it('should create agent with custom DID resolver', async () => {
        const mockResolver: DIDResolver = {
          resolve: vi.fn(),
        };
        
        const config: TapAgentConfig = { 
          didResolver: mockResolver 
        };
        const agent = await TapAgent.create(config);
        
        expect(agent).toBeInstanceOf(TapAgent);
        expect((agent as any).didResolver).toBe(mockResolver);
      });
    });

    describe('fromPrivateKey', () => {
      it('should create agent from existing private key with Ed25519', async () => {
        const privateKey = 'abcd1234567890abcd1234567890abcd1234567890abcd1234567890abcd1234';
        
        const agent = await TapAgent.fromPrivateKey(privateKey);
        
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
        const invalidKey = 'invalid-key';
        mockWasmModule.WasmTapAgent.fromPrivateKey.mockRejectedValueOnce(new Error('Invalid key'));
        
        await expect(TapAgent.fromPrivateKey(invalidKey)).rejects.toThrow();
      });

      it('should throw error for unsupported key type', async () => {
        const privateKey = 'abcd1234567890abcd1234567890abcd1234567890abcd1234567890abcd1234';
        
        await expect(
          TapAgent.fromPrivateKey(privateKey, 'InvalidKeyType' as any)
        ).rejects.toThrow('Unsupported key type');
      });
    });
  });

  describe('Identity Management', () => {
    let agent: any;

    beforeEach(async () => {
      agent = await TapAgent.create();
    });

    it('should return the agent DID', () => {
      expect(agent.did).toBe('did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK');
      expect(mockWasmAgent.get_did).toHaveBeenCalled();
    });

    it('should return the public key', () => {
      expect(agent.publicKey).toBe('1234567890abcd1234567890abcd1234567890abcd1234567890abcd12345678');
      expect(mockWasmAgent.exportPublicKey).toHaveBeenCalled();
    });

    it('should export private key', () => {
      const privateKey = agent.exportPrivateKey();
      
      expect(privateKey).toBe('abcd1234567890abcd1234567890abcd1234567890abcd1234567890abcd1234');
      expect(mockWasmAgent.exportPrivateKey).toHaveBeenCalled();
    });

    it('should handle private key export errors', () => {
      mockWasmAgent.exportPrivateKey.mockImplementation(() => {
        throw new Error('Key export failed');
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
          from: agent.did,
          to: ['did:key:recipient'],
          created_time: Date.now(),
          body: { amount: '100.0' },
        };

        const packed = await agent.pack(message);
        
        expect(packed).toEqual({
          message: 'packed-message-content',
          metadata: {
            type: 'encrypted',
            recipients: ['did:key:recipient'],
            sender: 'did:key:sender',
            messageType: 'Transfer',
          },
        });
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
        const packedMessage = 'packed-message-string';
        
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

    it('should resolve DID:key using built-in resolver', async () => {
      const didKey = 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK';
      
      // Mock the internal WASM DID resolution
      
      const resolutionResult = await agent.resolveDID(didKey);
      
      expect(resolutionResult.didDocument).toBeDefined();
      expect(resolutionResult.didDocument?.id).toBe(didKey);
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
          didResolutionMetadata: {},
          didDocument: customDidDoc,
          didDocumentMetadata: {},
        }),
      };

      const agentWithResolver = await TapAgent.create({ didResolver: mockResolver });
      const result = await agentWithResolver.resolveDID('did:web:example.com');
      
      expect(mockResolver.resolve).toHaveBeenCalledWith('did:web:example.com', undefined);
      expect(result.didDocument).toEqual(customDidDoc);
    });

    it('should handle DID resolution errors', async () => {
      const invalidDid = 'did:invalid:12345';
      
      await expect(agent.resolveDID(invalidDid)).rejects.toThrow();
    });
  });

  describe('Utility Methods', () => {
    let agent: any;

    beforeEach(async () => {
      agent = await TapAgent.create();
    });

    it('should create a message with proper structure', () => {
      const messageType = 'Transfer';
      const body = { amount: '100.0', asset: 'USD' };
      
      const message = agent.createMessage(messageType, body);
      
      expect(message).toEqual({
        id: 'uuid-1234-5678-9012',
        type: `https://tap.rsvp/schema/1.0#${messageType}`,
        from: agent.did,
        created_time: expect.any(Number),
        body,
      });
      expect(mockWasmModule.generateUUID).toHaveBeenCalled();
    });

    it('should create message with custom ID', () => {
      const messageType = 'Payment';
      const body = { amount: '50.0' };
      const customId = 'custom-id-123';
      
      const message = agent.createMessage(messageType, body, { id: customId });
      
      expect(message.id).toBe(customId);
      expect(mockWasmModule.generateUUID).not.toHaveBeenCalled();
    });

    it('should create message with recipients', () => {
      const messageType = 'Authorize';
      const body = { transaction_id: 'tx-123' };
      const recipients = ['did:key:recipient1', 'did:key:recipient2'];
      
      const message = agent.createMessage(messageType, body, { to: recipients });
      
      expect(message.to).toEqual(recipients);
    });
  });

  describe('Resource Management', () => {
    it('should cleanup resources when disposed', async () => {
      const agent = await TapAgent.create();
      
      agent.dispose();
      
      expect(mockWasmAgent.free).toHaveBeenCalled();
    });

    it('should handle multiple dispose calls gracefully', async () => {
      const agent = await TapAgent.create();
      
      agent.dispose();
      agent.dispose(); // Should not throw
      
      expect(mockWasmAgent.free).toHaveBeenCalledTimes(1);
    });
  });

  describe('Error Handling', () => {
    it('should wrap WASM errors with typed errors', async () => {
      mockWasmModule.WasmTapAgent.mockImplementation(() => {
        throw new Error('WASM initialization failed');
      });

      await expect(TapAgent.create()).rejects.toThrow('Failed to create TapAgent');
    });

    it('should provide helpful error messages for common failures', async () => {
      const privateKey = 'too-short';
      mockWasmModule.WasmTapAgent.fromPrivateKey.mockRejectedValueOnce(
        new Error('Invalid key length')
      );

      await expect(TapAgent.fromPrivateKey(privateKey)).rejects.toThrow(
        'Invalid private key format'
      );
    });
  });

  describe('Type Safety', () => {
    let agent: any;

    beforeEach(async () => {
      agent = await TapAgent.create();
    });

    it('should maintain type safety in message operations', async () => {
      interface TransferBody {
        amount: string;
        asset: string;
        from: string;
        to: string;
      }

      const transferBody: TransferBody = {
        amount: '100.0',
        asset: 'USD',
        from: 'account1',
        to: 'account2',
      };

      const message = agent.createMessage('Transfer', transferBody);
      
      // TypeScript should enforce that body matches TransferBody
      expect(message.body.amount).toBe('100.0');
      expect(message.body.asset).toBe('USD');
    });

    it('should type-check packed message metadata', async () => {
      const message: DIDCommMessage<{ test: string }> = {
        id: 'test',
        type: 'https://tap.rsvp/schema/1.0#Transfer',
        body: { test: 'value' },
      };

      const packed: PackedMessage = await agent.pack(message);
      
      expect(packed.message).toBeTruthy();
      expect(packed.metadata.type).toBe('encrypted');
    });
  });
});