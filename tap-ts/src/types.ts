/**
 * Type definitions for TAP-TS
 * 
 * This module contains type definitions used across the TAP-TS library.
 */

/**
 * Message type enum
 */
export enum MessageType {
  /** Authorization request message */
  AUTHORIZATION_REQUEST = 'TAP_AUTHORIZATION_REQUEST',
  
  /** Authorization response message */
  AUTHORIZATION_RESPONSE = 'TAP_AUTHORIZATION_RESPONSE',
  
  /** Ping message for testing */
  PING = 'TAP_PING',
}

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
  /** DID of the agent */
  did: string;
  
  /** Optional nickname for the agent */
  nickname?: string;
  
  /** Debug mode flag */
  debug?: boolean;
}

/**
 * Node configuration
 */
export interface NodeConfig {
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

/**
 * Authorization request
 */
export interface AuthorizationRequest {
  /** Transaction hash */
  transactionHash: string;
  
  /** Sender address */
  sender: string;
  
  /** Receiver address */
  receiver: string;
  
  /** Transaction amount */
  amount: string;
}

/**
 * Authorization response
 */
export interface AuthorizationResponse {
  /** Transaction hash that this response is for */
  transactionHash: string;
  
  /** Authorization result (true=approved, false=rejected) */
  authorizationResult: boolean;
  
  /** Optional reason for the decision */
  reason?: string;
}

/**
 * Message metadata
 */
export interface MessageMetadata {
  /** Sender DID */
  fromDid?: string;
  
  /** Recipient DID */
  toDid?: string;
  
  /** Created timestamp */
  created?: number;
  
  /** Expires timestamp */
  expires?: number;
  
  /** Additional metadata */
  [key: string]: unknown;
}
