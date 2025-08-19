/**
 * TAP Agent TypeScript wrapper for WASM implementation
 */

import initWasm, { WasmTapAgent, generateUUID } from 'tap-wasm';
import type {
  TapAgentConfig,
  DIDCommMessage,
  PackedMessage,
  KeyType,
  DIDDocument,
  DIDResolver,
  DIDResolutionResult,
  PackOptions,
  UnpackOptions,
  TapMessageTypeName,
  AgentMetrics,
  TAPMessageUnion,
} from './types.js';
import {
  TapAgentError,
  TapAgentKeyError,
  TapAgentMessageError,
  TapAgentDIDError,
} from './types.js';
import {
  validateKeyType,
  normalizePrivateKey,
  isValidDID,
  isMessageWithinAgeLimit,
} from './utils.js';
import {
  convertToWasmMessage,
  convertFromWasmMessage,
  validateMessageStructure,
  messageTypeToUri,
  mergeMessages,
} from './type-mapping.js';

/**
 * TypeScript wrapper for TAP WASM Agent providing browser-optimized
 * message packing/unpacking with flexible key management
 */
export class TapAgent {
  private wasmAgent: WasmTapAgent;
  private didResolver: DIDResolver | undefined;
  private isDisposed = false;
  private metrics: AgentMetrics;
  private readonly createdAt: number;

  /**
   * Private constructor - use static factory methods instead
   */
  private constructor(wasmAgent: WasmTapAgent, config?: TapAgentConfig) {
    this.wasmAgent = wasmAgent;
    this.didResolver = config?.didResolver;
    this.createdAt = Date.now();
    this.metrics = {
      messagesPacked: 0,
      messagesUnpacked: 0,
      keyOperations: 0,
      uptime: 0,
      lastActivity: this.createdAt,
    };
  }

  /**
   * Create a new TAP agent with generated keys
   * @param config - Optional agent configuration
   * @returns Promise resolving to new TapAgent instance
   */
  public static async create(config?: TapAgentConfig): Promise<TapAgent> {
    try {
      // Initialize WASM module if not already done
      await initWasm();

      const keyType = config?.keyType ?? 'Ed25519';
      
      if (!validateKeyType(keyType)) {
        throw new TapAgentKeyError(`Unsupported key type: ${keyType}`);
      }

      // Create WASM agent configuration
      const wasmConfig: Record<string, unknown> = {
        keyType,
      };

      if (config?.nickname) {
        wasmConfig.nickname = config.nickname;
      }

      const wasmAgent = new WasmTapAgent(wasmConfig);
      return new TapAgent(wasmAgent, config);
    } catch (error) {
      if (error instanceof TapAgentError) {
        throw error;
      }
      throw new TapAgentError('Failed to create TapAgent', 'CREATION_ERROR', error as Error);
    }
  }

  /**
   * Create a TAP agent from an existing private key
   * @param privateKey - Hex-encoded private key
   * @param keyType - Key type (default: Ed25519)
   * @returns Promise resolving to new TapAgent instance
   */
  public static async fromPrivateKey(
    privateKey: string,
    keyType: KeyType = 'Ed25519'
  ): Promise<TapAgent> {
    try {
      // Initialize WASM module if not already done
      await initWasm();

      if (!validateKeyType(keyType)) {
        throw new TapAgentKeyError(`Unsupported key type: ${keyType}`);
      }

      const normalizedKey = normalizePrivateKey(privateKey);
      const wasmAgent = await WasmTapAgent.fromPrivateKey(normalizedKey, keyType);
      
      return new TapAgent(wasmAgent);
    } catch (error) {
      if (error instanceof TapAgentError) {
        throw error;
      }
      throw new TapAgentKeyError('Failed to create agent from private key', error as Error);
    }
  }

  /**
   * Get the agent's DID
   */
  public get did(): string {
    this.ensureNotDisposed();
    try {
      return this.wasmAgent.get_did();
    } catch (error) {
      throw new TapAgentError('Failed to get agent DID', 'DID_ACCESS_ERROR', error as Error);
    }
  }

  /**
   * Get the agent's public key
   */
  public get publicKey(): string {
    this.ensureNotDisposed();
    try {
      return this.wasmAgent.exportPublicKey();
    } catch (error) {
      throw new TapAgentKeyError('Failed to export public key', error as Error);
    }
  }

  /**
   * Export the agent's private key
   * @returns Hex-encoded private key
   */
  public exportPrivateKey(): string {
    this.ensureNotDisposed();
    try {
      this.metrics.keyOperations++;
      this.updateLastActivity();
      return this.wasmAgent.exportPrivateKey();
    } catch (error) {
      throw new TapAgentKeyError('Failed to export private key', error as Error);
    }
  }

  /**
   * Pack a message for transmission
   * @param message - TAP message or DIDComm message to pack
   * @param options - Optional packing options
   * @returns Promise resolving to packed message
   */
  public async pack(
    message: TAPMessageUnion,
    options?: PackOptions
  ): Promise<PackedMessage> {
    this.ensureNotDisposed();
    
    try {
      // Validate message structure
      validateMessageStructure(message);

      // Apply options to message
      let processedMessage = message;
      if (options) {
        // Create a more flexible override object that can handle both TAP and generic DIDComm fields
        const overrides: any = {};
        
        if (options.to) {
          overrides.to = options.to;
        }
        
        if (options.expires_time) {
          overrides.expires_time = options.expires_time;
        }

        if (options.headers && Object.keys(options.headers).length > 0) {
          // Add custom headers to message (implementation specific)
          overrides.headers = options.headers;
        }

        processedMessage = mergeMessages(message, overrides);
      }

      // Convert to WASM format and pack
      const wasmMessage = convertToWasmMessage(processedMessage);
      const packed = await this.wasmAgent.packMessage(wasmMessage);

      this.metrics.messagesPacked++;
      this.updateLastActivity();

      return packed as PackedMessage;
    } catch (error) {
      if (error instanceof TapAgentError) {
        throw error;
      }
      throw new TapAgentMessageError('Failed to pack message', error as Error);
    }
  }

  /**
   * Unpack a received message
   * @param packedMessage - Packed message string
   * @param options - Optional unpacking options
   * @returns Promise resolving to unpacked TAP message or DIDComm message
   */
  public async unpack(
    packedMessage: string,
    options?: UnpackOptions
  ): Promise<TAPMessageUnion> {
    this.ensureNotDisposed();
    
    try {
      if (!packedMessage || typeof packedMessage !== 'string') {
        throw new TapAgentMessageError('Invalid packed message format');
      }

      // Unpack using WASM
      const wasmMessage = await this.wasmAgent.unpackMessage(
        packedMessage,
        options?.expectedType
      );

      // Convert from WASM format
      const message = convertFromWasmMessage(wasmMessage);

      // Apply options validation
      if (options?.maxAge && message.created_time) {
        if (!isMessageWithinAgeLimit(message.created_time, options.maxAge)) {
          throw new TapAgentMessageError('Message too old');
        }
      }

      // Verify signatures if requested (delegated to WASM)
      if (options?.verifySignatures === true) {
        // Signature verification is handled by the WASM layer
        // This option is mainly for documentation and future extensibility
      }

      this.metrics.messagesUnpacked++;
      this.updateLastActivity();

      // Return the message as-is - type checking will be done by the caller
      // The isTAPMessage type guard can be used by consumers to narrow the type
      return message;
    } catch (error) {
      if (error instanceof TapAgentError) {
        throw error;
      }
      throw new TapAgentMessageError('Failed to unpack message', error as Error);
    }
  }

  /**
   * Resolve a DID document
   * @param did - DID to resolve
   * @param options - Optional resolution options
   * @returns Promise resolving to DID resolution result
   */
  public async resolveDID(did: string, options?: Record<string, unknown>): Promise<DIDResolutionResult> {
    this.ensureNotDisposed();

    try {
      if (!isValidDID(did)) {
        throw new TapAgentDIDError(`Invalid DID format: ${did}`);
      }

      // Use custom resolver if available
      if (this.didResolver) {
        return await this.didResolver.resolve(did, options);
      }

      // Use built-in DID:key resolver for performance
      if (did.startsWith('did:key:')) {
        return await this.resolveDidKey(did);
      }

      throw new TapAgentDIDError(`No resolver available for DID method: ${did}`);
    } catch (error) {
      if (error instanceof TapAgentError) {
        throw error;
      }
      throw new TapAgentDIDError('Failed to resolve DID', error as Error);
    }
  }

  /**
   * Create a new message with proper structure
   * @param messageType - TAP message type
   * @param body - Message body
   * @param options - Optional message options
   * @returns New DIDComm message
   */
  public createMessage<T = unknown>(
    messageType: TapMessageTypeName | string,
    body: T,
    options?: {
      id?: string;
      to?: string[];
      thid?: string;
      pthid?: string;
      expires_time?: number;
    }
  ): DIDCommMessage<T> {
    this.ensureNotDisposed();

    try {
      const message: DIDCommMessage<T> = {
        id: options?.id ?? generateUUID(),
        type: messageTypeToUri(messageType),
        from: this.did,
        created_time: Date.now(),
        body,
      };

      if (options?.to) {
        message.to = options.to;
      }

      if (options?.thid) {
        message.thid = options.thid;
      }

      if (options?.pthid) {
        message.pthid = options.pthid;
      }

      if (options?.expires_time) {
        message.expires_time = options.expires_time;
      }

      // Validate the created message
      validateMessageStructure(message);

      return message;
    } catch (error) {
      if (error instanceof TapAgentError) {
        throw error;
      }
      throw new TapAgentMessageError('Failed to create message', error as Error);
    }
  }

  /**
   * Get agent metrics and statistics
   * @returns Current agent metrics
   */
  public getMetrics(): AgentMetrics {
    this.ensureNotDisposed();
    
    return {
      ...this.metrics,
      uptime: Date.now() - this.createdAt,
    };
  }

  /**
   * Dispose of the agent and cleanup resources
   */
  public dispose(): void {
    if (this.isDisposed) {
      return;
    }

    try {
      this.wasmAgent.free();
    } catch (error) {
      // Ignore cleanup errors
    }

    this.isDisposed = true;
  }

  /**
   * Built-in DID:key resolver implementation
   * @param did - DID:key to resolve
   * @returns Promise resolving to DID resolution result
   * @private
   */
  private async resolveDidKey(did: string): Promise<DIDResolutionResult> {
    try {
      // Extract the multibase key from the DID
      const keyId = did.replace('did:key:', '');
      
      // For now, create a basic DID document
      // In a full implementation, this would decode the multibase key
      // and create the appropriate verification methods
      const didDocument: DIDDocument = {
        id: did,
        verificationMethod: [
          {
            id: `${did}#key-1`,
            type: 'Ed25519VerificationKey2020',
            controller: did,
            publicKeyMultibase: keyId,
          },
        ],
        authentication: [`${did}#key-1`],
        assertionMethod: [`${did}#key-1`],
        keyAgreement: [`${did}#key-1`],
      };

      return {
        didResolutionMetadata: {},
        didDocument,
        didDocumentMetadata: {},
      };
    } catch (error) {
      return {
        didResolutionMetadata: {
          error: 'notFound',
        },
        didDocumentMetadata: {},
      };
    }
  }

  /**
   * Ensure the agent is not disposed
   * @private
   */
  private ensureNotDisposed(): void {
    if (this.isDisposed) {
      throw new TapAgentError('Agent has been disposed');
    }
  }

  /**
   * Update the last activity timestamp
   * @private
   */
  private updateLastActivity(): void {
    this.metrics.lastActivity = Date.now();
  }
}

// Re-export types for convenience
export type {
  TapAgentConfig,
  DIDCommMessage,
  PackedMessage,
  KeyType,
  PackOptions,
  UnpackOptions,
} from './types.js';