/**
 * DID Resolver interface and registry for TAP-TS
 * 
 * This module provides DID resolution capabilities using standard npm packages.
 */

import { TapError, ErrorType } from '../error.ts';
import { DIDDocument } from '../types.ts';

// Import npm DID libraries
import { Resolver } from 'did-resolver';
import { getResolver as getKeyResolver } from 'did-method-key';
import { getResolver as getWebResolver } from 'did-method-web';
import { getResolver as getPkhResolver } from 'did-method-pkh';

/**
 * DID resolution options
 */
export interface DIDResolutionOptions {
  /** Accept timestamp */
  accept?: number;
}

/**
 * DID resolution result
 */
export interface DIDResolutionResult {
  /** The DID document */
  didDocument: DIDDocument;
  
  /** The DID resolution metadata */
  didResolutionMetadata: {
    /** Content type of the DID document */
    contentType?: string;
    
    /** Error code */
    error?: string;
    
    /** Error message */
    message?: string;
  };
  
  /** The DID document metadata */
  didDocumentMetadata: {
    /** Created timestamp */
    created?: string;
    
    /** Updated timestamp */
    updated?: string;
    
    /** Version ID */
    versionId?: string;
    
    /** Next update timestamp */
    nextUpdate?: string;
    
    /** Deactivated flag */
    deactivated?: boolean;
  };
}

/**
 * Create a singleton DID resolver instance with all supported methods
 */
function createResolver() {
  // Initialize resolvers for different DID methods
  const methodResolvers = {
    ...getKeyResolver(),
    ...getWebResolver(),
    ...getPkhResolver(),
  };
  
  return new Resolver(methodResolvers);
}

/**
 * The singleton DID resolver instance
 */
const didResolver = createResolver();

/**
 * Resolve a DID to a DID Document
 * 
 * @param did - DID to resolve
 * @param options - Resolution options
 * @returns Promise resolving to the DID resolution result
 * @throws {TapError} If the DID cannot be resolved
 */
export async function resolveDID(did: string, options?: DIDResolutionOptions): Promise<DIDResolutionResult> {
  try {
    // Call the resolver
    const result = await didResolver.resolve(did, options);
    
    return {
      didDocument: result.didDocument as DIDDocument,
      didResolutionMetadata: result.didResolutionMetadata,
      didDocumentMetadata: result.didDocumentMetadata,
    };
  } catch (error) {
    throw new TapError({
      type: ErrorType.DID_RESOLUTION_ERROR,
      message: `Error resolving DID: ${did}`,
      cause: error,
    });
  }
}

/**
 * Check if a DID is resolvable with the current resolver configuration
 * 
 * @param did - DID to check
 * @returns True if the DID can be resolved, false otherwise
 */
export function canResolveDID(did: string): boolean {
  // Parse the DID to get the method
  const match = did.match(/^did:([a-z0-9]+):.+$/);
  if (!match) {
    return false;
  }
  
  const method = match[1];
  
  // Check if the method is supported
  return ['key', 'web', 'pkh'].includes(method);
}

export default didResolver;
