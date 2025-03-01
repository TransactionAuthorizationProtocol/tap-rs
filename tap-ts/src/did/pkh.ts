/**
 * did:pkh resolver for TAP-TS
 * 
 * This module provides did:pkh resolution for TAP-TS.
 * did:pkh is a DID method for blockchain account addresses.
 */

import { TapError, ErrorType } from '../error.ts';
import { DIDDocument } from '../types.ts';
import { DIDResolver, DIDResolutionOptions, DIDResolutionResult } from './resolver.ts';

/**
 * Supported blockchains for did:pkh
 */
enum BlockchainType {
  ETHEREUM = 'eip155',
  BITCOIN = 'bip122',
  SOLANA = 'solana',
}

/**
 * Resolver for did:pkh method
 */
export class PkhDIDResolver implements DIDResolver {
  /**
   * Get the DID method supported by this resolver
   * 
   * @returns The DID method name ('pkh')
   */
  getMethod(): string {
    return 'pkh';
  }
  
  /**
   * Check if this resolver supports a given DID
   * 
   * @param did - DID to check
   * @returns True if the DID starts with 'did:pkh:', false otherwise
   */
  canResolve(did: string): boolean {
    return did.startsWith('did:pkh:');
  }
  
  /**
   * Resolve a did:pkh to a DID document
   * 
   * @param did - DID to resolve
   * @param _options - Resolution options
   * @returns Promise resolving to the DID resolution result
   */
  async resolve(did: string, _options?: DIDResolutionOptions): Promise<DIDResolutionResult> {
    if (!this.canResolve(did)) {
      return {
        didDocument: { id: did },
        didResolutionMetadata: {
          error: 'invalidDid',
          message: `Not a did:pkh: ${did}`,
        },
        didDocumentMetadata: {},
      };
    }
    
    try {
      // Parse the blockchain and address from the DID
      // Format: did:pkh:eip155:1:0x...
      const parts = did.substring(8).split(':');
      
      if (parts.length < 3) {
        return {
          didDocument: { id: did },
          didResolutionMetadata: {
            error: 'invalidDid',
            message: `Invalid did:pkh format: ${did}`,
          },
          didDocumentMetadata: {},
        };
      }
      
      const blockchain = parts[0];
      const chainId = parts[1];
      const address = parts[2];
      
      // Create verification method ID
      const vmId = `${did}#${blockchain}-${chainId}`;
      
      // Determine the verification method type based on the blockchain
      let vmType: string;
      switch (blockchain) {
        case BlockchainType.ETHEREUM:
          vmType = 'EcdsaSecp256k1RecoveryMethod2020';
          break;
        case BlockchainType.BITCOIN:
          vmType = 'EcdsaSecp256k1VerificationKey2019';
          break;
        case BlockchainType.SOLANA:
          vmType = 'Ed25519VerificationKey2018';
          break;
        default:
          vmType = 'BlockchainVerificationMethod2021';
      }
      
      // Create a basic DID document
      const didDocument: DIDDocument = {
        id: did,
        '@context': [
          'https://www.w3.org/ns/did/v1',
          'https://w3id.org/security/suites/secp256k1recovery-2020/v2',
        ],
        verificationMethod: [
          {
            id: vmId,
            type: vmType,
            controller: did,
            blockchainAccountId: `${blockchain}:${chainId}:${address}`,
          },
        ],
        authentication: [vmId],
        assertionMethod: [vmId],
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
        message: `Error resolving did:pkh: ${did}`,
        cause: error,
      });
    }
  }
}

// Create and export an instance of the resolver
export const pkhResolver = new PkhDIDResolver();
