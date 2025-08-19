import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { DIDResolver } from '../src/types.js';

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

describe('DID Resolution', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should use provided DID resolver when available', async () => {
    const mockResolver: DIDResolver = {
      resolve: vi.fn().mockResolvedValue({
        didResolutionMetadata: {},
        didDocument: {
          id: 'did:example:123',
          verificationMethod: [{
            id: 'did:example:123#key-1',
            type: 'Ed25519VerificationKey2020',
            controller: 'did:example:123',
            publicKeyMultibase: 'z6Mktest...'
          }],
        },
        didDocumentMetadata: {},
      }),
    };

    const agent = await TapAgent.create({
      didResolver: mockResolver,
    });

    const result = await agent.resolveDID('did:example:123');
    
    expect(mockResolver.resolve).toHaveBeenCalledWith('did:example:123', undefined);
    expect(result.didDocument?.id).toBe('did:example:123');
  });

  it('should pass options to the resolver', async () => {
    const mockResolver: DIDResolver = {
      resolve: vi.fn().mockResolvedValue({
        didResolutionMetadata: {},
        didDocument: {
          id: 'did:web:example.com',
          verificationMethod: [],
        },
        didDocumentMetadata: {},
      }),
    };

    const agent = await TapAgent.create({
      didResolver: mockResolver,
    });

    const options = { accept: 'application/did+json' };
    await agent.resolveDID('did:web:example.com', options);
    
    expect(mockResolver.resolve).toHaveBeenCalledWith('did:web:example.com', options);
  });

  it('should throw error when no resolver is available', async () => {
    const agent = await TapAgent.create();
    
    await expect(agent.resolveDID('did:example:123')).rejects.toThrow('No DID resolver configured');
  });

  it('should validate DID format', async () => {
    const agent = await TapAgent.create();
    
    await expect(agent.resolveDID('not-a-did')).rejects.toThrow('Invalid DID format');
    await expect(agent.resolveDID('')).rejects.toThrow('Invalid DID format');
  });

  it('should handle resolver errors gracefully', async () => {
    const errorResolver: DIDResolver = {
      resolve: vi.fn().mockRejectedValue(new Error('Network error')),
    };

    const agent = await TapAgent.create({
      didResolver: errorResolver,
    });

    await expect(agent.resolveDID('did:web:example.com')).rejects.toThrow('Failed to resolve DID');
  });

  it('should handle resolver returning error metadata', async () => {
    const errorResolver: DIDResolver = {
      resolve: vi.fn().mockResolvedValue({
        didResolutionMetadata: {
          error: 'notFound',
          message: 'DID not found',
        },
        didDocumentMetadata: {},
      }),
    };

    const agent = await TapAgent.create({
      didResolver: errorResolver,
    });

    const result = await agent.resolveDID('did:web:notfound.com');
    
    expect(result.didResolutionMetadata.error).toBe('notFound');
    expect(result.didDocument).toBeUndefined();
  });
});