/**
 * DID resolver implementation
 * Provides interfaces and helpers for resolving DIDs
 */

import { DID } from '../models/types';
import { DIDResolutionError } from '../utils/errors';

/**
 * DID Document verification method
 * Represents a verification method in a DID Document
 */
export interface VerificationMethod {
  /** Verification method ID */
  id: string;
  
  /** Verification method type */
  type: string;
  
  /** Controller DID */
  controller: string;
  
  /** Public key in JWK format or other key material */
  publicKeyJwk?: Record<string, any>;
  
  /** Public key in multibase format */
  publicKeyMultibase?: string;
}

/**
 * DID Document
 * A document containing verification methods and services for a DID
 */
export interface DIDDocument {
  /** The DID this document describes */
  id: string;
  
  /** Verification methods */
  verificationMethod?: VerificationMethod[];
  
  /** Authentication methods */
  authentication?: (string | VerificationMethod)[];
  
  /** Key agreement methods */
  keyAgreement?: (string | VerificationMethod)[];
  
  /** Service endpoints */
  service?: {
    /** Service ID */
    id: string;
    
    /** Service type */
    type: string;
    
    /** Service endpoint */
    serviceEndpoint: string | string[] | Record<string, any>;
  }[];
  
  /** Other properties */
  [key: string]: any;
}

/**
 * DID resolution result
 * Result of resolving a DID to a DID Document
 */
export interface DIDResolutionResult {
  /** The resolved DID Document */
  didDocument: DIDDocument | null;
  
  /** Resolution metadata */
  didResolutionMetadata: {
    /** Content type of the result */
    contentType?: string;
    
    /** Error code if resolution failed */
    error?: string;
    
    /** Error message if resolution failed */
    message?: string;
  };
  
  /** DID Document metadata */
  didDocumentMetadata: {
    /** When the DID Document was created */
    created?: string;
    
    /** When the DID Document was last updated */
    updated?: string;
    
    /** Whether the DID is deactivated */
    deactivated?: boolean;
    
    /** Version ID of the DID Document */
    versionId?: string;
    
    /** When the DID Document will next be updated */
    nextUpdate?: string;
    
    /** When the DID Document was last updated */
    nextVersionId?: string;
    
    [key: string]: any;
  };
}

/**
 * DID resolver interface
 * Defines the methods for resolving DIDs to DID Documents
 */
export interface DIDResolver {
  /**
   * Resolve a DID to a DID Document
   * 
   * @param did The DID to resolve
   * @returns Promise resolving to a DID resolution result
   */
  resolve(did: DID): Promise<DIDResolutionResult>;
}

/**
 * Memory DID resolver
 * A simple in-memory DID resolver for testing and development
 */
export class MemoryDIDResolver implements DIDResolver {
  private documents: Map<string, DIDDocument> = new Map();
  
  /**
   * Register a DID Document
   * 
   * @param document The DID Document to register
   */
  register(document: DIDDocument): void {
    this.documents.set(document.id, document);
  }
  
  /**
   * Resolve a DID to a DID Document
   * 
   * @param did The DID to resolve
   * @returns Promise resolving to a DID resolution result
   */
  async resolve(did: DID): Promise<DIDResolutionResult> {
    const document = this.documents.get(did);
    
    if (document) {
      return {
        didDocument: document,
        didResolutionMetadata: {
          contentType: 'application/did+json'
        },
        didDocumentMetadata: {}
      };
    } else {
      return {
        didDocument: null,
        didResolutionMetadata: {
          error: 'notFound',
          message: `DID ${did} not found`
        },
        didDocumentMetadata: {}
      };
    }
  }
}

/**
 * Create a key-based resolver
 * Resolves DIDs created with the did:key method
 * 
 * @returns A DID resolver for the did:key method
 */
export function createKeyResolver(): DIDResolver {
  // This is a basic implementation; in a real app, you'd use a proper did:key resolver
  return {
    async resolve(did: DID): Promise<DIDResolutionResult> {
      // Only handle did:key
      if (!did.startsWith('did:key:')) {
        return {
          didDocument: null,
          didResolutionMetadata: {
            error: 'methodNotSupported',
            message: `Method not supported: ${did.split(':')[1]}`
          },
          didDocumentMetadata: {}
        };
      }
      
      // Extract the key from the DID
      const keyPart = did.split(':')[2];
      
      // Create a basic DID Document
      const document: DIDDocument = {
        id: did,
        verificationMethod: [
          {
            id: `${did}#${keyPart}`,
            type: 'Ed25519VerificationKey2020',
            controller: did,
            publicKeyMultibase: keyPart
          }
        ],
        authentication: [`${did}#${keyPart}`],
        keyAgreement: [`${did}#${keyPart}`]
      };
      
      return {
        didDocument: document,
        didResolutionMetadata: {
          contentType: 'application/did+json'
        },
        didDocumentMetadata: {}
      };
    }
  };
}

/**
 * Create a web-based resolver
 * Resolves DIDs created with the did:web method
 * 
 * @returns A DID resolver for the did:web method
 */
export function createWebResolver(): DIDResolver {
  // This is a basic implementation; in a real app, you'd use HTTP to fetch the DID Document
  return {
    async resolve(did: DID): Promise<DIDResolutionResult> {
      // Only handle did:web
      if (!did.startsWith('did:web:')) {
        return {
          didDocument: null,
          didResolutionMetadata: {
            error: 'methodNotSupported',
            message: `Method not supported: ${did.split(':')[1]}`
          },
          didDocumentMetadata: {}
        };
      }
      
      // In a real implementation, we would fetch the DID Document from the web
      // For now, we'll just return a placeholder document
      const domain = did.split(':')[2];
      
      const document: DIDDocument = {
        id: did,
        verificationMethod: [
          {
            id: `${did}#key-1`,
            type: 'JsonWebKey2020',
            controller: did,
            publicKeyJwk: {
              kty: 'EC',
              crv: 'P-256',
              x: 'PLACEHOLDER-X',
              y: 'PLACEHOLDER-Y'
            }
          }
        ],
        authentication: [`${did}#key-1`],
        service: [
          {
            id: `${did}#tap-service`,
            type: 'TAPService',
            serviceEndpoint: `https://${domain}/tap`
          }
        ]
      };
      
      return {
        didDocument: document,
        didResolutionMetadata: {
          contentType: 'application/did+json'
        },
        didDocumentMetadata: {}
      };
    }
  };
}

/**
 * Create a composite resolver
 * Combines multiple resolvers to handle different DID methods
 * 
 * @param resolvers A map of method names to resolvers
 * @returns A DID resolver that delegates to the appropriate resolver based on method
 */
export function createCompositeResolver(
  resolvers: Record<string, DIDResolver>
): DIDResolver {
  return {
    async resolve(did: DID): Promise<DIDResolutionResult> {
      const method = did.split(':')[1];
      const resolver = resolvers[method];
      
      if (resolver) {
        return resolver.resolve(did);
      } else {
        return {
          didDocument: null,
          didResolutionMetadata: {
            error: 'methodNotSupported',
            message: `Method not supported: ${method}`
          },
          didDocumentMetadata: {}
        };
      }
    }
  };
}

/**
 * Create a default resolver
 * Creates a resolver that can handle common DID methods
 * 
 * @returns A DID resolver that can handle common DID methods
 */
export function createDefaultResolver(): DIDResolver {
  return createCompositeResolver({
    key: createKeyResolver(),
    web: createWebResolver()
  });
}