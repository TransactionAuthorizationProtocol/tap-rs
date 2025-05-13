# DID Resolver Integration for TAP-TS

This document explains how to use the DID Resolver functionality in the TAP-TS package.

## Table of Contents

- [Overview](#overview)
- [Default Configuration](#default-configuration)
- [Customizing Resolvers](#customizing-resolvers)
- [Implementing Custom Resolvers](#implementing-custom-resolvers)
- [Supported DID Methods](#supported-did-methods)
- [API Reference](#api-reference)
- [Examples](#examples)

## Overview

The TAP-TS package includes support for resolving Decentralized Identifiers (DIDs) using the `did-resolver` library and several resolver implementations. This functionality allows TAP agents to verify the authenticity of messages and interact with entities identified by various DID methods.

## Default Configuration

By default, TAP-TS includes and enables the following DID resolvers:

- `key-did-resolver`: Resolves `did:key` method DIDs
- `ethr-did-resolver`: Resolves `did:ethr` method DIDs
- `pkh-did-resolver`: Resolves `did:pkh` method DIDs
- `web-did-resolver`: Resolves `did:web` method DIDs

When you create a new `TAPAgent` instance without specifying resolver options, it will use all of these resolvers with default configuration.

```typescript
import { TAPAgent } from '@taprsvp/tap-agent';

// Create an agent with default resolver configuration
const agent = new TAPAgent({
  nickname: 'Default Agent'
});
```

## Customizing Resolvers

You can customize which resolvers are enabled and their configuration by providing `resolverOptions` when creating a `TAPAgent`.

```typescript
import { TAPAgent, ResolverOptions } from '@taprsvp/tap-agent';

// Custom resolver options
const resolverOptions: ResolverOptions = {
  resolvers: {
    key: true,    // Enable did:key resolver
    ethr: true,   // Enable did:ethr resolver
    pkh: false,   // Disable did:pkh resolver
    web: false    // Disable did:web resolver
  },
  ethrOptions: {
    // Configure the ethr-did-resolver
    networks: [
      {
        name: 'mainnet',
        rpcUrl: 'https://mainnet.infura.io/v3/YOUR_INFURA_KEY'
      },
      {
        name: 'sepolia',
        rpcUrl: 'https://sepolia.infura.io/v3/YOUR_INFURA_KEY'
      }
    ]
  }
};

// Create an agent with custom resolver configuration
const agent = new TAPAgent({
  nickname: 'Custom Resolver Agent',
  resolverOptions
});
```

## Implementing Custom Resolvers

You can also implement your own DID resolver by creating a class that implements the `DIDResolver` interface:

```typescript
import { TAPAgent, DIDResolver, DID } from '@taprsvp/tap-agent';

class MyCustomResolver implements DIDResolver {
  async resolve(did: DID): Promise<any> {
    // Your custom resolution logic here
    if (did.startsWith('did:custom:')) {
      return {
        id: did,
        '@context': 'https://www.w3.org/ns/did/v1',
        // Other DID Document properties
      };
    }
    
    throw new Error(`Cannot resolve DID: ${did}`);
  }
}

// Create an agent with your custom resolver
const agent = new TAPAgent({
  nickname: 'Agent with Custom Resolver',
  didResolver: new MyCustomResolver()
});
```

## Supported DID Methods

The following DID methods are supported out of the box:

- `did:key`: Self-contained DIDs with embedded Ed25519 keys
- `did:ethr`: Ethereum address-based DIDs
- `did:pkh`: Public Key Hash DIDs for various blockchains (BTC, ETH, etc.)
- `did:web`: Web domain-based DIDs

## API Reference

### `StandardDIDResolver`

A DID resolver implementation that uses the `did-resolver` library and supports multiple DID methods.

```typescript
import { StandardDIDResolver, ResolverOptions } from '@taprsvp/tap-agent';

// Create a resolver with default options
const defaultResolver = new StandardDIDResolver();

// Create a resolver with custom options
const customResolver = new StandardDIDResolver({
  resolvers: {
    key: true,
    ethr: true,
    pkh: false,
    web: false
  },
  ethrOptions: {
    // ethr-did-resolver options
    networks: [/* ... */]
  }
});

// Resolve a DID
const didDocument = await resolver.resolve('did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK');
```

### `createResolver`

A utility function to create a `did-resolver` Resolver instance with custom options.

```typescript
import { createResolver, ResolverOptions } from '@taprsvp/tap-agent';

const options: ResolverOptions = {
  resolvers: {/* ... */},
  ethrOptions: {/* ... */},
  customResolvers: {
    myMethod: async (did, parsed, resolver, options) => {
      // Custom resolver logic
      return { didDocument: { id: did } };
    }
  }
};

const resolver = createResolver(options);
```

### `ResolverOptions`

Interface for configuring the DID resolver.

```typescript
interface ResolverOptions {
  // Which resolvers to enable
  resolvers?: {
    key?: boolean;   // did:key
    ethr?: boolean;  // did:ethr
    pkh?: boolean;   // did:pkh
    web?: boolean;   // did:web
  };
  
  // Options for ethr-did-resolver
  ethrOptions?: {
    networks?: Array<{
      name: string;
      rpcUrl: string;
      registry?: string;
    }>;
  };
  
  // Options for pkh-did-resolver
  pkhOptions?: any;
  
  // Custom resolvers to include directly
  customResolvers?: Record<string, any>;
}
```

## Examples

For more detailed examples, see the [resolver-example.ts](src/examples/resolver-example.ts) file in the repository.