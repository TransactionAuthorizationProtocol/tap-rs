/**
 * Functions for DID generation
 */

import { DIDKeyType } from './wasm-loader';
import { DID } from './types';
import { ensureWasmInitialized } from './wasm-loader';

/**
 * Create a new did:key identifier with the specified key type
 * 
 * @param keyType - The type of key to use for generation
 * @returns The generated DID and associated information
 */
export async function createDIDKey(keyType: DIDKeyType = DIDKeyType.Ed25519): Promise<{
  did: DID;
  document: any;
  didDocument: any; // Alias for backward compatibility
  privateKey: string;
  getPublicKeyHex: () => string;
  getKeyType: () => DIDKeyType;
  signData: (data: string | Uint8Array) => Promise<string>;
  verifySignature: (data: string | Uint8Array, signature: string) => Promise<boolean>;
}> {
  await ensureWasmInitialized();
  
  // Generate a did:key using uuid-based random data
  // This is a temporary implementation - a stub for compatibility
  const keyId = Math.random().toString(36).substring(2, 15);
  const did = `did:key:z6Mk${keyId}` as DID;
  
  const document = {
    id: did,
    verificationMethod: [
      {
        id: `${did}#keys-1`,
        type: 'Ed25519VerificationKey2020',
        controller: did,
        publicKeyMultibase: `z${keyId}`
      }
    ],
    authentication: [`${did}#keys-1`],
    assertionMethod: [`${did}#keys-1`]
  };

  return {
    did,
    document,
    didDocument: document, // Alias for backward compatibility
    privateKey: 'STUB_PRIVATE_KEY',
    getPublicKeyHex: () => 'DUMMY_PUBLIC_KEY_HEX',
    getKeyType: () => keyType,
    signData: (data: string | Uint8Array) => Promise.resolve('DUMMY_SIGNATURE'),
    verifySignature: (data: string | Uint8Array, signature: string) => Promise.resolve(true)
  };
}

/**
 * Create a new did:web identifier for a domain
 * 
 * @param domain - The domain to create the DID for
 * @param path - Optional path component for the DID
 * @returns The generated DID and associated information
 */
export async function createDIDWeb(domain: string, path?: string): Promise<{
  did: DID;
  document: any;
  didDocument: any; // Alias for backward compatibility
  getPublicKeyHex: () => string;
  getKeyType: () => DIDKeyType;
  signData: (data: string | Uint8Array) => Promise<string>;
  verifySignature: (data: string | Uint8Array, signature: string) => Promise<boolean>;
}> {
  await ensureWasmInitialized();
  
  // Generate a did:web using the domain
  // This is a temporary implementation - a stub for compatibility
  const encodedPath = path ? `:${encodeURIComponent(path.replace(/^\//, ''))}` : '';
  const did = `did:web:${encodeURIComponent(domain)}${encodedPath}` as DID;
  
  const document = {
    id: did,
    verificationMethod: [
      {
        id: `${did}#keys-1`,
        type: 'Ed25519VerificationKey2020',
        controller: did,
        publicKeyMultibase: `zDummyWebKeyValueForStubImplementation`
      }
    ],
    authentication: [`${did}#keys-1`],
    service: [
      {
        id: `${did}#didcomm`,
        type: 'DIDCommMessaging',
        serviceEndpoint: `https://${domain}/.well-known/did.json`
      }
    ]
  };

  return {
    did,
    document,
    didDocument: document, // Alias for backward compatibility
    getPublicKeyHex: () => 'DUMMY_WEB_PUBLIC_KEY_HEX',
    getKeyType: () => DIDKeyType.Ed25519, // Web DIDs default to Ed25519 in this stub
    signData: (data: string | Uint8Array) => Promise.resolve('DUMMY_SIGNATURE'),
    verifySignature: (data: string | Uint8Array, signature: string) => Promise.resolve(true)
  };
}