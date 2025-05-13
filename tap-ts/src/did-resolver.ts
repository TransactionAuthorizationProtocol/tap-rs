/**
 * DID Resolver module for tap-ts
 * 
 * This module provides a unified interface for resolving DIDs using different resolver implementations.
 */

import { Resolver } from 'did-resolver';
import { getResolver as getKeyResolver } from 'key-did-resolver';
import { getResolver as getEthrResolver } from 'ethr-did-resolver';
import { getResolver as getPkhResolver } from 'pkh-did-resolver';
import { getResolver as getWebResolver } from 'web-did-resolver';
import { DID } from './types';

/**
 * Interface for the resolver configuration options
 */
export interface ResolverOptions {
  // Base options
  resolvers?: {
    key?: boolean;
    ethr?: boolean;
    pkh?: boolean;
    web?: boolean;
  };
  
  // Ethr DID resolver options
  ethrOptions?: {
    networks?: Array<{
      name: string;
      rpcUrl: string;
      registry?: string;
    }>;
  };
  
  // PKH DID resolver options
  pkhOptions?: any;
  
  // Custom resolvers to include directly
  customResolvers?: Record<string, any>;
}

/**
 * Default options with sensible defaults
 */
const DEFAULT_OPTIONS: ResolverOptions = {
  resolvers: {
    key: true,
    ethr: true, 
    pkh: true,
    web: true
  },
  ethrOptions: {
    networks: [
      {
        name: 'mainnet',
        rpcUrl: 'https://mainnet.infura.io/v3/7238211010344719ad14a89db874158c'
      },
      {
        name: 'goerli',
        rpcUrl: 'https://goerli.infura.io/v3/7238211010344719ad14a89db874158c'
      }
    ]
  },
  pkhOptions: {}
};

/**
 * Create a unified DID resolver with the specified options
 */
export function createResolver(options: ResolverOptions = {}): Resolver {
  const mergedOptions = { ...DEFAULT_OPTIONS, ...options };
  const methods: Record<string, any> = {};
  
  // Add enabled resolvers
  if (mergedOptions.resolvers?.key) {
    Object.assign(methods, getKeyResolver());
  }
  
  if (mergedOptions.resolvers?.ethr) {
    // Cast to any to avoid TypeScript issues with the ethr-did-resolver package
    Object.assign(methods, getEthrResolver(mergedOptions.ethrOptions as any));
  }
  
  if (mergedOptions.resolvers?.pkh) {
    // PKH resolver doesn't need options
    Object.assign(methods, getPkhResolver());
  }
  
  if (mergedOptions.resolvers?.web) {
    Object.assign(methods, getWebResolver());
  }
  
  // Add custom resolvers
  if (mergedOptions.customResolvers) {
    Object.assign(methods, mergedOptions.customResolvers);
  }
  
  return new Resolver(methods);
}

/**
 * DID Resolver implementation that uses the did-resolver library
 */
export class StandardDIDResolver {
  private resolver: Resolver;
  
  constructor(options: ResolverOptions = {}) {
    this.resolver = createResolver(options);
  }
  
  /**
   * Resolve a DID to its DID Document
   */
  async resolve(did: DID): Promise<any> {
    try {
      const resolution = await this.resolver.resolve(did);
      return resolution.didDocument;
    } catch (error) {
      console.error(`Failed to resolve DID ${did}:`, error);
      throw new Error(`Failed to resolve DID ${did}: ${error}`);
    }
  }
}

// Export default resolver instance for convenience
export const defaultResolver = new StandardDIDResolver();