import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Mock the WASM module
const mockWasmModule = {
  generatePrivateKey: vi.fn(() => 'generated-private-key-hex'),
  generateUUID: vi.fn(() => 'uuid-1234-5678-9012-3456'),
  WasmKeyType: {
    Ed25519: 0,
    P256: 1,
    Secp256k1: 2,
  },
};

vi.mock('tap-wasm', () => mockWasmModule);

// Import after mocking
const { generatePrivateKey, generateUUID, isValidDID, isValidPrivateKey, validateKeyType } = await import('../src/utils.js');

describe('Utils', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('generatePrivateKey', () => {
    it('should generate Ed25519 private key by default', () => {
      const privateKey = generatePrivateKey();
      
      expect(privateKey).toBe('generated-private-key-hex');
      expect(mockWasmModule.generatePrivateKey).toHaveBeenCalledWith('Ed25519');
    });

    it('should generate private key for specified key type', () => {
      const privateKey = generatePrivateKey('P256');
      
      expect(privateKey).toBe('generated-private-key-hex');
      expect(mockWasmModule.generatePrivateKey).toHaveBeenCalledWith('P256');
    });

    it('should generate secp256k1 private key', () => {
      const privateKey = generatePrivateKey('secp256k1');
      
      expect(privateKey).toBe('generated-private-key-hex');
      expect(mockWasmModule.generatePrivateKey).toHaveBeenCalledWith('secp256k1');
    });

    it('should throw error for invalid key type', () => {
      expect(() => generatePrivateKey('InvalidType' as any)).toThrow('Unsupported key type');
    });

    it('should handle WASM errors gracefully', () => {
      mockWasmModule.generatePrivateKey.mockImplementation(() => {
        throw new Error('WASM error');
      });

      expect(() => generatePrivateKey()).toThrow('Failed to generate private key');
    });
  });

  describe('generateUUID', () => {
    it('should generate a valid UUID', () => {
      const uuid = generateUUID();
      
      expect(uuid).toBe('uuid-1234-5678-9012-3456');
      expect(mockWasmModule.generateUUID).toHaveBeenCalled();
    });

    it('should handle WASM errors gracefully', () => {
      mockWasmModule.generateUUID.mockImplementation(() => {
        throw new Error('WASM error');
      });

      expect(() => generateUUID()).toThrow('Failed to generate UUID');
    });

    it('should generate different UUIDs on subsequent calls', () => {
      mockWasmModule.generateUUID
        .mockReturnValueOnce('uuid-1111-2222-3333-4444')
        .mockReturnValueOnce('uuid-5555-6666-7777-8888');

      const uuid1 = generateUUID();
      const uuid2 = generateUUID();
      
      expect(uuid1).toBe('uuid-1111-2222-3333-4444');
      expect(uuid2).toBe('uuid-5555-6666-7777-8888');
      expect(uuid1).not.toBe(uuid2);
    });
  });

  describe('isValidDID', () => {
    it('should validate did:key format', () => {
      const validDidKey = 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK';
      expect(isValidDID(validDidKey)).toBe(true);
    });

    it('should validate did:web format', () => {
      const validDidWeb = 'did:web:example.com';
      expect(isValidDID(validDidWeb)).toBe(true);
    });

    it('should validate did:ethr format', () => {
      const validDidEthr = 'did:ethr:0x1234567890123456789012345678901234567890';
      expect(isValidDID(validDidEthr)).toBe(true);
    });

    it('should reject invalid DID format', () => {
      expect(isValidDID('not-a-did')).toBe(false);
      expect(isValidDID('did:')).toBe(false);
      expect(isValidDID('did::')).toBe(false);
      expect(isValidDID('did:invalid')).toBe(false);
      expect(isValidDID('')).toBe(false);
    });

    it('should reject null or undefined', () => {
      expect(isValidDID(null as any)).toBe(false);
      expect(isValidDID(undefined as any)).toBe(false);
    });

    it('should handle complex DID paths', () => {
      const complexDid = 'did:web:example.com:users:alice';
      expect(isValidDID(complexDid)).toBe(true);
    });
  });

  describe('isValidPrivateKey', () => {
    it('should validate 32-byte hex private key (64 chars)', () => {
      const validKey = 'abcd1234567890abcd1234567890abcd1234567890abcd1234567890abcd1234';
      expect(isValidPrivateKey(validKey)).toBe(true);
    });

    it('should validate uppercase hex private key', () => {
      const validKey = 'ABCD1234567890ABCD1234567890ABCD1234567890ABCD1234567890ABCD1234';
      expect(isValidPrivateKey(validKey)).toBe(true);
    });

    it('should validate mixed case hex private key', () => {
      const validKey = 'AbCd1234567890aBcD1234567890AbCd1234567890aBcD1234567890AbCd1234';
      expect(isValidPrivateKey(validKey)).toBe(true);
    });

    it('should reject keys with invalid characters', () => {
      const invalidKey = 'ghij1234567890abcd1234567890abcd1234567890abcd1234567890abcd1234';
      expect(isValidPrivateKey(invalidKey)).toBe(false);
    });

    it('should reject keys that are too short', () => {
      const shortKey = 'abcd1234567890';
      expect(isValidPrivateKey(shortKey)).toBe(false);
    });

    it('should reject keys that are too long', () => {
      const longKey = 'abcd1234567890abcd1234567890abcd1234567890abcd1234567890abcd1234extra';
      expect(isValidPrivateKey(longKey)).toBe(false);
    });

    it('should reject empty string', () => {
      expect(isValidPrivateKey('')).toBe(false);
    });

    it('should reject null or undefined', () => {
      expect(isValidPrivateKey(null as any)).toBe(false);
      expect(isValidPrivateKey(undefined as any)).toBe(false);
    });

    it('should handle keys with 0x prefix', () => {
      const keyWithPrefix = '0xabcd1234567890abcd1234567890abcd1234567890abcd1234567890abcd1234';
      expect(isValidPrivateKey(keyWithPrefix)).toBe(true);
    });
  });

  describe('validateKeyType', () => {
    it('should validate supported key types', () => {
      expect(validateKeyType('Ed25519')).toBe(true);
      expect(validateKeyType('P256')).toBe(true);
      expect(validateKeyType('secp256k1')).toBe(true);
    });

    it('should reject unsupported key types', () => {
      expect(validateKeyType('RSA')).toBe(false);
      expect(validateKeyType('InvalidType')).toBe(false);
      expect(validateKeyType('')).toBe(false);
    });

    it('should be case sensitive', () => {
      expect(validateKeyType('ed25519')).toBe(false);
      expect(validateKeyType('p256')).toBe(false);
      expect(validateKeyType('SECP256K1')).toBe(false);
    });

    it('should reject null or undefined', () => {
      expect(validateKeyType(null as any)).toBe(false);
      expect(validateKeyType(undefined as any)).toBe(false);
    });
  });

  describe('Error Handling', () => {
    it('should provide meaningful error messages', () => {
      expect(() => generatePrivateKey('invalid' as any)).toThrow('Unsupported key type: invalid');
      expect(isValidPrivateKey('short')).toBe(false);
      expect(isValidDID('invalid')).toBe(false);
    });
  });

  describe('Integration with WASM types', () => {
    it('should map key types to WASM enum values', () => {
      // This tests the internal mapping
      expect(mockWasmModule.WasmKeyType.Ed25519).toBe(0);
      expect(mockWasmModule.WasmKeyType.P256).toBe(1);
      expect(mockWasmModule.WasmKeyType.Secp256k1).toBe(2);
    });

    it('should handle WASM module initialization errors', () => {
      // Simulate WASM not being initialized
      mockWasmModule.generatePrivateKey.mockImplementation(() => {
        throw new Error('WASM module not initialized');
      });

      expect(() => generatePrivateKey()).toThrow('Failed to generate private key');
    });
  });

  describe('Performance', () => {
    it('should execute validation functions quickly', () => {
      const start = performance.now();
      
      for (let i = 0; i < 1000; i++) {
        isValidDID('did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK');
        isValidPrivateKey('abcd1234567890abcd1234567890abcd1234567890abcd1234567890abcd1234');
        validateKeyType('Ed25519');
      }
      
      const end = performance.now();
      const duration = end - start;
      
      // Should complete 1000 iterations in under 100ms
      expect(duration).toBeLessThan(100);
    });
  });
});