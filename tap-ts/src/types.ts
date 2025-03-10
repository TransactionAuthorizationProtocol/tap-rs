/**
 * Type definitions for TAP-TS
 * 
 * This module contains type definitions used across the TAP-TS library.
 */


/**
 * DID Document type
 */
export interface DIDDocument {
  /** The DID that the document is about */
  id: string;
  
  /** Context for the DID Document */
  '@context'?: string | string[];
  
  /** Alternative identifiers for this DID */
  alsoKnownAs?: string[];
  
  /** Controller DIDs */
  controller?: string | string[];
  
  /** Verification methods associated with this DID */
  verificationMethod?: VerificationMethod[];
  
  /** Authentication verification method references */
  authentication?: (string | VerificationMethod)[];
  
  /** Assertion verification method references */
  assertionMethod?: (string | VerificationMethod)[];
  
  /** Key agreement verification method references */
  keyAgreement?: (string | VerificationMethod)[];
  
  /** Capability invocation verification method references */
  capabilityInvocation?: (string | VerificationMethod)[];
  
  /** Capability delegation verification method references */
  capabilityDelegation?: (string | VerificationMethod)[];
  
  /** Service endpoints */
  service?: Service[];
}

/**
 * Verification Method in a DID Document
 */
export interface VerificationMethod {
  /** ID of the verification method */
  id: string;
  
  /** DID that the verification method belongs to */
  controller: string;
  
  /** Type of the verification method */
  type: string;
  
  /** Public key in JWK format */
  publicKeyJwk?: Record<string, unknown>;
  
  /** Public key in multibase format */
  publicKeyMultibase?: string;
  
  /** Public key in hex format */
  publicKeyHex?: string;
  
  /** Public key in base64 format */
  publicKeyBase64?: string;
  
  /** Public key in PEM format */
  publicKeyPem?: string;
}

/**
 * Service endpoint in a DID Document
 */
export interface Service {
  /** ID of the service */
  id: string;
  
  /** Type of the service */
  type: string;
  
  /** Service endpoint URI */
  serviceEndpoint: string | string[] | Record<string, unknown>;
  
  /** Additional properties */
  [key: string]: unknown;
}

/**
 * Agent configuration
 */
export interface AgentConfig {
  /** Agent DID */
  did: string;
  
  /** Optional agent ID */
  id?: string;
  
  /** Optional agent nickname */
  nickname?: string;
  
  /** Debug mode */
  debug?: boolean;
}

/**
 * Node configuration
 */
export interface NodeConfig {
  /** Optional node ID */
  id?: string;
  
  /** Debug mode flag */
  debug?: boolean;
  
  /** Network configuration */
  network?: NetworkConfig;
}

/**
 * Network configuration for a node
 */
export interface NetworkConfig {
  /** List of peer DIDs */
  peers?: string[];
}

import { Message } from "./message.ts";

/**
 * Message metadata
 */
export interface MessageMetadata {
  /**
   * Additional message metadata
   */
  [key: string]: unknown;
}

/**
 * Message callback function type
 */
export type MessageCallback = (message: Message, metadata?: MessageMetadata) => Promise<void>;

/**
 * Message subscriber function type
 */
export type MessageSubscriber = (message: Message, metadata?: MessageMetadata) => void;


/**
 * Options for creating a new agent
 */
export interface AgentOptions {
  /** DID for the agent */
  did: string;
  
  /** Optional nickname for the agent */
  nickname?: string;
  
  /** Optional agent ID (auto-generated if not provided) */
  id?: string;
  
  /** Optional private key as JWK for signing messages */
  privateKeyJwk?: Record<string, unknown>;
}

/**
 * Metadata for a message
 */
export interface MessageMetadata {
  /** The DID of the sender */
  senderDid?: string;
  
  /** The timestamp when the message was received */
  receivedTimestamp?: number;
  
  /** Transport-specific metadata */
  transport?: Record<string, unknown>;
}

/**
 * Storage options
 */
export interface StorageOptions {
  /** The type of storage to use */
  type: 'memory' | 'indexeddb' | 'localstorage';
  
  /** The name of the storage (for persistent storage) */
  name?: string;
}

/**
 * ConfirmRelationship message data (TAIP-9)
 */
export interface ConfirmRelationshipData {
  /** ID of the transfer being confirmed */
  transfer_id: string;

  /** ID of the agent confirming the relationship */
  agent_id: string;

  /** ID of the entity for which the relationship is being confirmed */
  for: string;

  /** Optional role of the entity in the relationship */
  role?: string;

  /** Additional metadata */
  [key: string]: unknown;
}

/**
 * Keypair for signing and encrypting messages
 */
export interface Keypair {
  /** The public key in JWK format */
  publicKeyJwk: Record<string, unknown>;
  
  /** The private key in JWK format */
  privateKeyJwk: Record<string, unknown>;
  
  /** The key ID */
  kid: string;
  
  /** The key type */
  kty: string;
  
  /** The key algorithm */
  alg: string;
}

/**
 * DID Resolution Result
 */
export interface DidResolutionResult {
  /** The DID resolution metadata */
  didResolutionMetadata: Record<string, unknown>;
  
  /** The DID document */
  didDocument: {
    id: string;
    verificationMethod?: Array<{
      id: string;
      type: string;
      controller: string;
      publicKeyJwk?: Record<string, unknown>;
      publicKeyMultibase?: string;
    }>;
    authentication?: Array<string | Record<string, unknown>>;
    assertionMethod?: Array<string | Record<string, unknown>>;
    keyAgreement?: Array<string | Record<string, unknown>>;
    capabilityInvocation?: Array<string | Record<string, unknown>>;
    capabilityDelegation?: Array<string | Record<string, unknown>>;
    service?: Array<{
      id: string;
      type: string;
      serviceEndpoint: string | Record<string, unknown>;
    }>;
  };
  
  /** The DID document metadata */
  didDocumentMetadata: Record<string, unknown>;
}

/**
 * Authorization result
 */
export interface AuthorizationResult {
  /** Whether the authorization was approved */
  approved: boolean;
  
  /** Optional reason for the decision */
  reason?: string;
  
  /** Optional signature or proof of the decision */
  proof?: Record<string, unknown>;
}

/**
 * Options for sending messages
 */
export interface SendMessageOptions {
  /** Whether to wait for a response */
  waitForResponse?: boolean;
  
  /** Timeout in milliseconds for waiting for a response */
  responseTimeout?: number;
  
  /** Transport-specific options */
  transportOptions?: Record<string, unknown>;
}
