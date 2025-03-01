/**
 * did:web resolver for TAP-TS
 * 
 * This module provides did:web resolution for TAP-TS.
 */

import { TapError, ErrorType } from '../error.ts';
import { DIDDocument } from '../types.ts';
import { DIDResolver, DIDResolutionOptions, DIDResolutionResult } from './resolver.ts';

/**
 * Resolver for did:web method
 */
export class WebDIDResolver implements DIDResolver {
  /**
   * Get the DID method supported by this resolver
   * 
   * @returns The DID method name ('web')
   */
  getMethod(): string {
    return 'web';
  }
  
  /**
   * Check if this resolver supports a given DID
   * 
   * @param did - DID to check
   * @returns True if the DID starts with 'did:web:', false otherwise
   */
  canResolve(did: string): boolean {
    return did.startsWith('did:web:');
  }
  
  /**
   * Resolve a did:web to a DID document
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
          message: `Not a did:web: ${did}`,
        },
        didDocumentMetadata: {},
      };
    }
    
    try {
      // Parse the domain and path from the DID
      // Format: did:web:example.com or did:web:example.com:path:to:resource
      const didParts = did.substring(8).split(':');
      const domain = didParts[0];
      let path = '';
      
      if (didParts.length > 1) {
        path = '/' + didParts.slice(1).join('/');
      }
      
      // Construct the URL to the DID document
      const url = `https://${domain}${path}/.well-known/did.json`;
      
      // Fetch the DID document
      const response = await fetch(url, {
        headers: {
          Accept: 'application/did+json, application/json',
        },
      });
      
      if (!response.ok) {
        return {
          didDocument: { id: did },
          didResolutionMetadata: {
            error: 'notFound',
            message: `DID document not found at ${url}: ${response.status} ${response.statusText}`,
          },
          didDocumentMetadata: {},
        };
      }
      
      const contentType = response.headers.get('Content-Type') || 'application/json';
      
      // Parse the DID document
      let didDocument: DIDDocument;
      try {
        didDocument = await response.json();
      } catch (error) {
        return {
          didDocument: { id: did },
          didResolutionMetadata: {
            error: 'invalidDidDocument',
            message: `Invalid DID document JSON: ${error.message}`,
          },
          didDocumentMetadata: {},
        };
      }
      
      // Validate the DID document
      if (!didDocument.id || didDocument.id !== did) {
        return {
          didDocument: { id: did },
          didResolutionMetadata: {
            error: 'invalidDidDocument',
            message: `DID document id does not match: ${didDocument.id} !== ${did}`,
          },
          didDocumentMetadata: {},
        };
      }
      
      return {
        didDocument,
        didResolutionMetadata: {
          contentType,
        },
        didDocumentMetadata: {},
      };
    } catch (error) {
      throw new TapError({
        type: ErrorType.DID_RESOLUTION_ERROR,
        message: `Error resolving did:web: ${did}`,
        cause: error,
      });
    }
  }
}

// Create and export an instance of the resolver
export const webResolver = new WebDIDResolver();
