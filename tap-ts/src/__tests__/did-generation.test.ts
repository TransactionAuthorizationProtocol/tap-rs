/**
 * Tests for DID generation functionality
 */

import { describe, it, expect, vi, beforeAll } from 'vitest';
import { TAPAgent, DIDKeyType, createDIDKey, createDIDWeb } from '../index';

// Mock the tap-wasm module
vi.mock('tap-wasm', () => {
  const mockModule = {
    init_tap_wasm: vi.fn(),
    init: vi.fn(),
    MessageType: {
      Transfer: 0,
      PaymentRequest: 1,
      Authorize: 3,
      Reject: 4,
      Settle: 5,
      Presentation: 2,
      AddAgents: 6,
      ReplaceAgent: 7,
      RemoveAgent: 8,
      UpdatePolicies: 9,
      UpdateParty: 10,
      ConfirmRelationship: 11,
      Error: 12,
      Unknown: 13
    },
    DIDKeyType: {
      Ed25519: 'Ed25519',
      P256: 'P256',
      Secp256k1: 'Secp256k1'
    },
    TapAgent: vi.fn().mockImplementation(() => ({
      get_did: vi.fn().mockReturnValue('did:key:mockagent'),
      nickname: vi.fn().mockReturnValue('Mock Agent'),
      subscribe_to_messages: vi.fn(),
      create_message: vi.fn(),
      set_from: vi.fn(),
      set_to: vi.fn(),
      sign_message: vi.fn(),
      verify_message: vi.fn().mockReturnValue(true),
      process_message: vi.fn().mockResolvedValue({})
    })),
    create_did_key: vi.fn().mockImplementation((keyType) => ({
      did: `did:key:mock${keyType || 'Ed25519'}`,
      didDocument: JSON.stringify({
        id: `did:key:mock${keyType || 'Ed25519'}`,
        verificationMethod: [{
          id: `did:key:mock${keyType || 'Ed25519'}#key1`,
          type: `${keyType || 'Ed25519'}VerificationKey2020`,
          controller: `did:key:mock${keyType || 'Ed25519'}`,
          publicKeyMultibase: 'z12345'
        }],
        keyAgreement: [`did:key:mock${keyType || 'Ed25519'}#keyAgreement`]
      }),
      getPublicKeyHex: vi.fn().mockReturnValue('0x1234'),
      getPrivateKeyHex: vi.fn().mockReturnValue('0x5678'),
      getPublicKeyBase64: vi.fn().mockReturnValue('YWJjZA=='),
      getPrivateKeyBase64: vi.fn().mockReturnValue('ZWZnaA=='),
      getKeyType: vi.fn().mockReturnValue(keyType || 'Ed25519')
    })),
    create_did_web: vi.fn().mockImplementation((domain, keyType) => ({
      did: `did:web:${domain}`,
      didDocument: JSON.stringify({
        id: `did:web:${domain}`,
        verificationMethod: [{
          id: `did:web:${domain}#key1`,
          type: `${keyType || 'Ed25519'}VerificationKey2020`,
          controller: `did:web:${domain}`,
          publicKeyMultibase: 'z12345'
        }]
      }),
      getPublicKeyHex: vi.fn().mockReturnValue('0x1234'),
      getPrivateKeyHex: vi.fn().mockReturnValue('0x5678'),
      getPublicKeyBase64: vi.fn().mockReturnValue('YWJjZA=='),
      getPrivateKeyBase64: vi.fn().mockReturnValue('ZWZnaA=='),
      getKeyType: vi.fn().mockReturnValue(keyType || 'Ed25519')
    }))
  };
  
  // Add default export for __wbg_init
  const mockDefault = vi.fn().mockResolvedValue({});
  mockModule.default = mockDefault;
  
  return mockModule;
});

// Mock the commander library to prevent CLI execution
vi.mock('commander', () => ({
  program: {
    name: vi.fn().mockReturnThis(),
    description: vi.fn().mockReturnThis(),
    version: vi.fn().mockReturnThis(),
    command: vi.fn().mockReturnThis(),
    action: vi.fn().mockReturnThis(),
    option: vi.fn().mockReturnThis(),
    requiredOption: vi.fn().mockReturnThis(),
    parse: vi.fn(),
    outputHelp: vi.fn()
  }
}));

// Mock prompts library to prevent CLI interaction
vi.mock('inquirer', () => ({
  default: {
    prompt: vi.fn().mockResolvedValue({})
  }
}));

// Mock fs to prevent file system operations
vi.mock('fs', () => ({
  writeFileSync: vi.fn(),
  existsSync: vi.fn().mockReturnValue(true),
  mkdirSync: vi.fn()
}));

// Mock the CLI module to prevent execution
vi.mock('../cli/did-generator', () => ({}));
vi.mock('../cli/index', () => ({}));

// Initialize before tests
beforeAll(async () => {
  // Allow time for mock initialization
  await new Promise(resolve => setTimeout(resolve, 0));
});

describe('DID Generation', () => {
  it('should create a TAPAgent with automatically generated DID', async () => {
    const agent = new TAPAgent({
      nickname: 'Auto DID Agent',
      debug: true
    });
    
    // Wait for the agent to initialize
    await new Promise(resolve => setTimeout(resolve, 0));
    
    // Check the DID
    const did = agent.did;
    expect(did).toBeDefined();
    expect(did).toBe('did:key:mockagent');
  });
  
  it('should generate a did:key with Ed25519', async () => {
    const agent = new TAPAgent();
    const didKey = await agent.generateDID(DIDKeyType.Ed25519);
    
    expect(didKey).toBeDefined();
    expect(didKey.did).toBe('did:key:mockEd25519');
    expect(didKey.getKeyType()).toBe('Ed25519');
    expect(didKey.getPublicKeyHex()).toBe('0x1234');
    expect(didKey.getPrivateKeyHex()).toBe('0x5678');
    
    // Check DID document
    const didDoc = JSON.parse(didKey.didDocument);
    expect(didDoc.id).toBe(didKey.did);
    expect(didDoc.verificationMethod).toHaveLength(1);
    expect(didDoc.keyAgreement).toHaveLength(1);
  });
  
  it('should generate a did:key with P-256', async () => {
    const didKey = await createDIDKey(DIDKeyType.P256);
    
    expect(didKey).toBeDefined();
    expect(didKey.did).toBe('did:key:mockP256');
    expect(didKey.getKeyType()).toBe('P256');
  });
  
  it('should generate a did:key with Secp256k1', async () => {
    const didKey = await createDIDKey(DIDKeyType.Secp256k1);
    
    expect(didKey).toBeDefined();
    expect(didKey.did).toBe('did:key:mockSecp256k1');
    expect(didKey.getKeyType()).toBe('Secp256k1');
  });
  
  it('should generate a did:web', async () => {
    const domain = 'example.com';
    const didKey = await createDIDWeb(domain, DIDKeyType.Ed25519);
    
    expect(didKey).toBeDefined();
    expect(didKey.did).toBe(`did:web:${domain}`);
    expect(didKey.getKeyType()).toBe('Ed25519');
    
    // Check DID document
    const didDoc = JSON.parse(didKey.didDocument);
    expect(didDoc.id).toBe(didKey.did);
    expect(didDoc.verificationMethod).toHaveLength(1);
  });
  
  it('should generate and list DIDs through agent', async () => {
    const agent = new TAPAgent();
    
    // Generate a few DIDs
    await agent.generateDID(DIDKeyType.Ed25519);
    await agent.generateDID(DIDKeyType.P256);
    await agent.generateWebDID('example.org', DIDKeyType.Secp256k1);
    
    // List DIDs
    const dids = await agent.listDIDs();
    expect(dids).toBeDefined();
    expect(dids.length).toBeGreaterThanOrEqual(1);
    expect(dids[0]).toBe(agent.did);
  });
  
  it('should get key info', async () => {
    const agent = new TAPAgent();
    const keyInfo = agent.getKeysInfo();
    
    expect(keyInfo).toBeDefined();
    expect(keyInfo.did).toBe(agent.did);
  });
});