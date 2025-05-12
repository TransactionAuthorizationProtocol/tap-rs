/**
 * TAP Agent
 * A class for managing identities and signing/verifying messages in TAP
 */

import { DID } from '../models/types';
import { DIDCommMessageBase } from '../api/messages/base';
import { 
  DIDResolver, 
  DIDResolutionResult, 
  createDefaultResolver 
} from './resolver';
import { getWasmModule, createAgent } from '../wasm/bridge';
import { 
  ValidationError, 
  CryptoError, 
  VerificationError, 
  DIDResolutionError 
} from '../utils/errors';

/**
 * Signer interface
 * Defines the methods required for signing messages
 */
export interface Signer {
  /**
   * Sign data with the private key
   * 
   * @param data The data to sign
   * @returns Promise resolving to the signature
   */
  sign(data: Uint8Array): Promise<Uint8Array>;
  
  /**
   * Get the DID for this signer
   * 
   * @returns The DID controlled by this signer
   */
  getDID(): DID;
}

/**
 * Key material interface
 * Represents the key material for a DID
 */
export interface KeyMaterial {
  /**
   * Private key (keep secure!)
   */
  privateKey: Uint8Array;
  
  /**
   * Public key
   */
  publicKey: Uint8Array;
  
  /**
   * DID associated with this key
   */
  did: DID;
}

/**
 * TAP Agent options
 * Configuration options for creating a TAP agent
 */
export interface TAPAgentOptions {
  /**
   * DID of the agent
   */
  did: DID;
  
  /**
   * Signer for the agent
   * Used to sign messages
   */
  signer: Signer;
  
  /**
   * DID resolver
   * Used to resolve DIDs to DID Documents
   * Default: basic resolver handling did:key and did:web
   */
  resolver?: DIDResolver;
}

/**
 * TAP Agent implementation
 * Manages identities and handles message signing and verification in TAP
 */
export class TAPAgent {
  /** The agent's DID */
  private did: DID;
  
  /** The signer used for signing messages */
  private signer: Signer;
  
  /** The resolver used for resolving DIDs */
  private resolver: DIDResolver;
  
  /** The underlying WASM agent */
  private wasmAgent: any;
  
  /**
   * Create a new TAP agent
   * 
   * @param options Agent configuration options
   */
  constructor(options: TAPAgentOptions) {
    this.did = options.did;
    this.signer = options.signer;
    this.resolver = options.resolver || createDefaultResolver();
    
    // Initialize the WASM agent
    this.initWasmAgent().catch(err => {
      console.error('Failed to initialize WASM agent:', err);
    });
  }
  
  /**
   * Initialize the WASM agent
   * This happens asynchronously but we start it in the constructor
   */
  private async initWasmAgent(): Promise<void> {
    const wasm = await getWasmModule();
    this.wasmAgent = await createAgent(this.did, 'placeholder-key');
  }
  
  /**
   * Get the agent's DID
   * 
   * @returns The DID of the agent
   */
  getDID(): DID {
    return this.did;
  }
  
  /**
   * Sign a message
   * Prepares the message envelope and creates a signature
   * 
   * @param message The message to sign
   * @returns Promise resolving to the signed message
   * @throws ValidationError if the message is invalid
   * @throws CryptoError if signing fails
   */
  async sign<T = any>(message: DIDCommMessageBase<T>): Promise<DIDCommMessageBase<T>> {
    // Prepare the envelope
    message._prepareEnvelope(this.did);
    
    try {
      // If the WASM agent is available, use it to sign the message
      if (this.wasmAgent) {
        // Convert the message to a format the WASM agent can understand
        // This depends on the exact WASM API
        await this.wasmAgent.sign(message);
      } else {
        // Otherwise use the signer directly
        // This is a simplified version; real implementation would need to
        // handle header, payload, etc. according to DIDComm spec
        const messageBytes = new TextEncoder().encode(JSON.stringify(message));
        const signature = await this.signer.sign(messageBytes);
        
        // In a real implementation, we would attach the signature to the message
        // For now, we'll just show what would happen
        console.log('Message signed with signature length:', signature.length);
      }
      
      return message;
    } catch (error) {
      throw new CryptoError(`Failed to sign message: ${error}`);
    }
  }
  
  /**
   * Verify a message signature
   * 
   * @param message The message to verify
   * @returns Promise resolving to a boolean indicating if the signature is valid
   * @throws DIDResolutionError if the sender's DID cannot be resolved
   * @throws VerificationError if verification fails
   */
  async verify<T = any>(message: DIDCommMessageBase<T>): Promise<boolean> {
    // Check if message has the required fields
    if (!message.from) {
      throw new ValidationError('Message has no sender (from field)');
    }
    
    try {
      // Resolve the sender's DID
      const resolution = await this.resolver.resolve(message.from);
      
      if (!resolution.didDocument) {
        throw new DIDResolutionError(
          message.from,
          resolution.didResolutionMetadata.error || 'Unknown error'
        );
      }
      
      // If the WASM agent is available, use it to verify the message
      if (this.wasmAgent) {
        return await this.wasmAgent.verify(message);
      } else {
        // This is a placeholder for manual verification logic
        // In a real implementation, we would verify the message signature
        // using the verification methods in the DID Document
        console.log('Would verify message from:', message.from);
        console.log('Using DID Document:', resolution.didDocument);
        
        // For now, just return true (this is not secure!)
        return true;
      }
    } catch (error) {
      if (error instanceof DIDResolutionError) {
        throw error;
      }
      throw new VerificationError(`Failed to verify message: ${error}`);
    }
  }
  
  /**
   * Create a new agent with a generated key
   * Static factory method for easily creating new agents
   * 
   * @returns Promise resolving to a new TAP agent with a generated key
   */
  static async create(): Promise<TAPAgent> {
    // This is a placeholder for generating new keys
    // In a real implementation, we would use a proper key generation library
    const wasm = await getWasmModule();
    const did = await wasm.create_did_key();
    
    // Create a simple signer that uses the WASM module
    const signer: Signer = {
      async sign(data: Uint8Array): Promise<Uint8Array> {
        // This would call into the WASM module to sign with the private key
        return new Uint8Array(0); // Placeholder
      },
      
      getDID(): DID {
        return did;
      }
    };
    
    // Create and return a new agent
    return new TAPAgent({
      did,
      signer,
      resolver: createDefaultResolver()
    });
  }
}