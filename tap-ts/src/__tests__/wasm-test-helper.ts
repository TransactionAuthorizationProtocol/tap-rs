/**
 * Helper functions for testing with the WASM module
 */

import * as tapWasm from 'tap-wasm';
import { DIDKeyType } from '../wasm-loader';
import { vi } from 'vitest';

/**
 * Creates a mock DID key for testing
 */
export class MockDIDKey {
  did: string;
  didDocument: string;
  keyType: string;

  constructor(keyType: string = 'Ed25519') {
    this.keyType = keyType;
    
    // Generate a deterministic DID based on key type
    if (keyType === 'Ed25519') {
      this.did = 'did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp';
    } else if (keyType === 'P256') {
      this.did = 'did:key:zDnaerDaTF5BXEavCrfRZEk316dpbLsfPDZ3WJ5hRTPFR7v';
    } else if (keyType === 'Secp256k1') {
      this.did = 'did:key:zQ3shokFTS3brHcDQrn82RUDfCZESWL1ZdCEJwekUDPQiYBme';
    } else {
      this.did = 'did:key:zGenericMockDIDKey';
    }
    
    // Create a realistic DID document
    this.didDocument = JSON.stringify({
      id: this.did,
      verificationMethod: [{
        id: `${this.did}#key1`,
        type: `${keyType}VerificationKey2020`,
        controller: this.did,
        publicKeyMultibase: 'z12345'
      }],
      keyAgreement: [`${this.did}#keyAgreement`]
    });
  }
  
  // For compatibility with the original WASM functions
  getPublicKeyHex(): string {
    return '0x1234';
  }
  
  getPrivateKeyHex(): string {
    return '0x5678';
  }
  
  getPublicKeyBase64(): string {
    return 'YWJjZA==';
  }
  
  getPrivateKeyBase64(): string {
    return 'ZWZnaA==';
  }
  
  getKeyType(): string {
    return this.keyType;
  }
  
  // WASM style functions (snake_case)
  get_public_key_hex(): string {
    return this.getPublicKeyHex();
  }
  
  get_private_key_hex(): string {
    return this.getPrivateKeyHex();
  }
  
  get_public_key_base64(): string {
    return this.getPublicKeyBase64();
  }
  
  get_private_key_base64(): string {
    return this.getPrivateKeyBase64();
  }
  
  get_key_type(): string {
    return this.getKeyType();
  }
}

/**
 * Creates a mock DID web for testing
 */
export class MockDIDWeb extends MockDIDKey {
  constructor(domain: string, keyType: string = 'Ed25519') {
    super(keyType);
    this.did = `did:web:${domain}`;
    this.didDocument = JSON.stringify({
      id: this.did,
      verificationMethod: [{
        id: `${this.did}#key1`,
        type: `${keyType}VerificationKey2020`,
        controller: this.did,
        publicKeyMultibase: 'z12345'
      }]
    });
  }
}

/**
 * Mock implementation of TapAgent for testing
 */
export class MockTapAgent {
  private _nickname: string;
  private _did: string;
  
  constructor(config?: any) {
    this._nickname = config?.nickname || 'Mock Agent';
    this._did = 'did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp';
  }
  
  get_did(): string {
    return this._did;
  }
  
  nickname(): string {
    return this._nickname;
  }
  
  create_message(messageType: number): any {
    return {
      id: () => 'mock-id',
      message_type: () => messageType,
      set_from_did: () => {},
      set_to_did: () => {},
      from_did: () => this._did,
      to_did: () => 'did:key:recipient',
      set_transfer_body: () => {},
      set_payment_request_body: () => {},
      set_authorize_body: () => {},
      set_reject_body: () => {},
      set_settle_body: () => {},
      set_cancel_body: () => {},
      set_revert_body: () => {},
      get_transfer_body: () => ({
        asset: 'eip155:1/erc20:mock-token',
        amount: '100.0',
        originator: { '@id': this._did, '@type': 'Party', role: 'originator' },
        beneficiary: { '@id': 'did:key:beneficiary', '@type': 'Party', role: 'beneficiary' },
        agents: []
      }),
      get_payment_request_body: () => ({
        asset: 'eip155:1/erc20:mock-token',
        amount: '100.0',
        merchant: { '@id': this._did, '@type': 'Party', role: 'merchant' },
        customer: { '@id': 'did:key:customer', '@type': 'Party', role: 'customer' }
      }),
      get_didcomm_message: () => ({ body: {} })
    };
  }
  
  set_from(): void {}
  set_to(): void {}
  sign_message(): void {}
  verify_message(): boolean { return true; }
  process_message(): Promise<any> { return Promise.resolve({}); }
  subscribe_to_messages(): void {}
}

/**
 * Setup for tests using WASM
 * This uses vite's vi.mock to mock the WASM module
 */
export async function setupWasmTests(): Promise<void> {
  // Mock the tap-wasm module
  vi.mock('tap-wasm', () => {
    return {
      // Mock DID key types
      DIDKeyType: {
        Ed25519: 'Ed25519',
        P256: 'P256',
        Secp256k1: 'Secp256k1'
      },
      
      // Mock message types
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
        Unknown: 13,
        Cancel: 14,
        Revert: 15
      },
      
      // Mock initialization functions
      init: vi.fn(),
      init_tap_wasm: vi.fn(),
      init_tap_msg: vi.fn(),
      
      // Mock DID key creation
      create_did_key: vi.fn().mockImplementation((keyType) => {
        let kt = 'Ed25519';
        
        if (keyType === 'P256') {
          kt = 'P256';
        } else if (keyType === 'Secp256k1') {
          kt = 'Secp256k1';
        }
        
        return new MockDIDKey(kt);
      }),
      
      // Mock DID web creation
      create_did_web: vi.fn().mockImplementation((domain, keyType) => {
        let kt = 'Ed25519';
        
        if (keyType === 'P256') {
          kt = 'P256';
        } else if (keyType === 'Secp256k1') {
          kt = 'Secp256k1';
        }
        
        return new MockDIDWeb(domain, kt);
      }),
      
      // Mock TapAgent
      TapAgent: vi.fn().mockImplementation((config) => new MockTapAgent(config)),
      
      // Add default export for __wbg_init
      default: vi.fn().mockResolvedValue({})
    };
  });
  
  // Give a little time for everything to initialize
  await new Promise(resolve => setTimeout(resolve, 10));
}