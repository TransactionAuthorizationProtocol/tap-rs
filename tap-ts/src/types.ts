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

import type { Message } from "./message.ts";

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
 * Authorization request
 */
export interface AuthorizationRequest {
  /**
   * Transaction hash to authorize
   */
  transactionHash: string;
  
  /**
   * Transaction data (hex encoded)
   */
  transactionData?: string;
  
  /**
   * Source address
   */
  sourceAddress?: string;
  
  /**
   * Destination address
   */
  destinationAddress?: string;
  
  /**
   * Transaction amount
   */
  amount?: string;
  
  /**
   * Transaction fee
   */
  fee?: string;
  
  /**
   * Network name
   */
  network?: string;
  
  /**
   * Transaction reference
   */
  reference?: string;
  
  /**
   * Authorized callback URL
   */
  callbackUrl?: string;
  
  /**
   * Additional authorization data
   */
  [key: string]: unknown;
}

/**
 * Authorization response
 */
export interface AuthorizationResponse {
  /**
   * Transaction hash that was authorized
   */
  transactionHash: string;
  
  /**
   * Authorization result (string "true" or "false")
   */
  authorizationResult?: string | boolean;
  
  /**
   * Whether the transaction was approved (legacy)
   */
  approved?: boolean;
  
  /**
   * Reason for the decision
   */
  reason?: string;
}
