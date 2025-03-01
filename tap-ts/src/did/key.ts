/**
 * did:key resolver for TAP-TS
 * 
 * This module provides did:key resolution for TAP-TS.
 */

import { TapError, ErrorType } from '../error.ts';
import { DIDDocument } from '../types.ts';
import { DIDResolver, DIDResolutionOptions, DIDResolutionResult } from './resolver.ts';

/**
 * Resolver for did:key method
 */
export class KeyDIDResolver implements DIDResolver {
  /**
   * Get the DID method supported by this resolver
   * 
   * @returns The DID method name ('key')
   */
  getMethod(): string {
    return 'key';
  }
  
  /**
   * Check if this resolver supports a given DID
   * 
   * @param did - DID to check
   * @returns True if the DID starts with 'did:key:', false otherwise
   */
  canResolve(did: string): boolean {
    return did.startsWith('did:key:');
  }
  
  /**
   * Resolve a did:key to a DID document
   * 
   * @param did - DID to resolve
   * @param _options - Resolution options (unused for did:key)
   * @returns Promise resolving to the DID resolution result
   */
  async resolve(did: string, _options?: DIDResolutionOptions): Promise<DIDResolutionResult> {
    if (!this.canResolve(did)) {
      return {
        didDocument: { id: did },
        didResolutionMetadata: {
          error: 'invalidDid',
          message: `Not a did:key: ${did}`,
        },
        didDocumentMetadata: {},
      };
    }
    
    try {
      // Parse the multibase-encoded public key from the DID
      const keyId = `${did}#${did.substring(8)}`;
      
      // Create a basic DID document
      const didDocument: DIDDocument = {
        id: did,
        '@context': [
          'https://www.w3.org/ns/did/v1',
          'https://w3id.org/security/suites/ed25519-2020/v1',
        ],
        verificationMethod: [
          {
            id: keyId,
            type: 'Ed25519VerificationKey2020',
            controller: did,
            publicKeyMultibase: did.substring(8),
          },
        ],
        authentication: [keyId],
        assertionMethod: [keyId],
        capabilityDelegation: [keyId],
        capabilityInvocation: [keyId],
        keyAgreement: [keyId],
      };
      
      return {
        didDocument,
        didResolutionMetadata: {
          contentType: 'application/did+json',
        },
        didDocumentMetadata: {},
      };
    } catch (error) {
      throw new TapError({
        type: ErrorType.DID_RESOLUTION_ERROR,
        message: `Error resolving did:key: ${did}`,
        cause: error,
      });
    }
  }
}

// Create and export an instance of the resolver
export const keyResolver = new KeyDIDResolver();
