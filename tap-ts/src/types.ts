/**
 * Configuration options for creating a TapAgent instance
 */
export interface TapAgentConfig {
  /** The cryptographic key type to use for the agent */
  keyType?: 'Ed25519' | 'P256' | 'secp256k1';
  /** Existing private key to import (hex string) */
  privateKey?: string;
  /** Custom DID resolver implementation */
  didResolver?: DIDResolver;
  /** Optional label/nickname for the agent */
  nickname?: string;
}

/**
 * Represents a packed/encrypted message ready for transmission
 */
export interface PackedMessage {
  /** The JWE/JWS formatted message string */
  message: string;
  /** Metadata about the packed message */
  metadata: {
    /** Message encryption/signing type */
    type: 'encrypted' | 'signed' | 'plain';
    /** List of intended recipient DIDs */
    recipients?: string[];
    /** Sender DID */
    sender?: string;
    /** Message type for routing */
    messageType?: string;
  };
}

/**
 * Base structure for all DIDComm messages
 */
export interface DIDCommMessage<T = unknown> {
  /** Unique message identifier */
  id: string;
  /** Message type URI */
  type: string;
  /** Sender DID */
  from?: string;
  /** Recipient DID(s) */
  to?: string[];
  /** Message creation timestamp */
  created_time?: number;
  /** Message expiration timestamp */
  expires_time?: number;
  /** Thread identifier for message threading */
  thid?: string;
  /** Parent thread identifier */
  pthid?: string;
  /** Message body containing the actual payload */
  body: T;
  /** Additional message attachments */
  attachments?: MessageAttachment[];
  /** Custom headers */
  headers?: Record<string, unknown>;
}

/**
 * Message attachment structure
 */
export interface MessageAttachment {
  /** Attachment identifier */
  id?: string;
  /** Attachment description */
  description?: string;
  /** Attachment filename */
  filename?: string;
  /** MIME type */
  media_type?: string;
  /** Attachment data */
  data: {
    /** Data encoding (base64, json, etc.) */
    encoding?: string;
    /** Raw attachment data */
    content: string | object;
  };
}

/**
 * DID Resolution interface compatible with did-resolver package
 */
export interface DIDResolver {
  resolve(did: string, options?: DIDResolutionOptions): Promise<DIDResolutionResult>;
}

export interface DIDResolutionOptions {
  accept?: string;
  [key: string]: unknown;
}

export interface DIDResolutionResult {
  didResolutionMetadata: DIDResolutionMetadata;
  didDocument?: DIDDocument;
  didDocumentMetadata: DIDDocumentMetadata;
}

export interface DIDResolutionMetadata {
  contentType?: string;
  error?: string;
  [key: string]: unknown;
}

export interface DIDDocumentMetadata {
  created?: string;
  updated?: string;
  deactivated?: boolean;
  versionId?: string;
  nextUpdate?: string;
  nextVersionId?: string;
  equivalentId?: string[];
  canonicalId?: string;
  [key: string]: unknown;
}

/**
 * DID Document structure
 */
export interface DIDDocument {
  '@context'?: string | string[];
  id: string;
  alsoKnownAs?: string[];
  controller?: string | string[];
  verificationMethod?: VerificationMethod[];
  authentication?: (string | VerificationMethod)[];
  assertionMethod?: (string | VerificationMethod)[];
  keyAgreement?: (string | VerificationMethod)[];
  capabilityInvocation?: (string | VerificationMethod)[];
  capabilityDelegation?: (string | VerificationMethod)[];
  service?: ServiceEndpoint[];
  [key: string]: unknown;
}

export interface VerificationMethod {
  id: string;
  type: string;
  controller: string;
  publicKeyBase58?: string;
  publicKeyJwk?: JsonWebKey;
  publicKeyHex?: string;
  publicKeyMultibase?: string;
  [key: string]: unknown;
}

export interface ServiceEndpoint {
  id: string;
  type: string | string[];
  serviceEndpoint: string | string[] | object;
  [key: string]: unknown;
}

/**
 * Key types supported by the agent
 */
export type KeyType = 'Ed25519' | 'P256' | 'secp256k1';

/**
 * Error types that can be thrown by the agent
 */
export class TapAgentError extends Error {
  public readonly code: string | undefined;
  public override readonly cause: Error | undefined;
  
  constructor(
    message: string,
    code?: string,
    cause?: Error,
  ) {
    super(message);
    this.name = 'TapAgentError';
    this.code = code;
    this.cause = cause;
  }
}

export class TapAgentKeyError extends TapAgentError {
  constructor(message: string, cause?: Error) {
    super(message, 'KEY_ERROR', cause);
    this.name = 'TapAgentKeyError';
  }
}

export class TapAgentMessageError extends TapAgentError {
  constructor(message: string, cause?: Error) {
    super(message, 'MESSAGE_ERROR', cause);
    this.name = 'TapAgentMessageError';
  }
}

export class TapAgentDIDError extends TapAgentError {
  constructor(message: string, cause?: Error) {
    super(message, 'DID_ERROR', cause);
    this.name = 'TapAgentDIDError';
  }
}

/**
 * Result type for operations that may fail
 */
export type Result<T, E = TapAgentError> = { success: true; data: T } | { success: false; error: E };

/**
 * Utility type for TAP message types mapping
 * This will be extended to map to @taprsvp/types when available
 */
export interface TapMessageTypes {
  Transfer: unknown;
  Payment: unknown;
  Authorize: unknown;
  Reject: unknown;
  Settle: unknown;
  Cancel: unknown;
  Revert: unknown;
  Connect: unknown;
  Escrow: unknown;
  Capture: unknown;
  AddAgents: unknown;
  ReplaceAgent: unknown;
  RemoveAgent: unknown;
  UpdatePolicies: unknown;
  UpdateParty: unknown;
  ConfirmRelationship: unknown;
  AuthorizationRequired: unknown;
  Presentation: unknown;
  TrustPing: unknown;
  BasicMessage: unknown;
}

/**
 * Message type names as strings
 */
export type TapMessageTypeName = keyof TapMessageTypes;

/**
 * Options for message packing operations
 */
export interface PackOptions {
  /** Override default recipients */
  to?: string[];
  /** Include routing information */
  routing?: boolean;
  /** Message expiration time */
  expires_time?: number;
  /** Custom message headers */
  headers?: Record<string, unknown>;
}

/**
 * Options for message unpacking operations
 */
export interface UnpackOptions {
  /** Expected message type for validation */
  expectedType?: string;
  /** Verify message signatures */
  verifySignatures?: boolean;
  /** Maximum message age to accept (in seconds) */
  maxAge?: number;
}

/**
 * Agent statistics and metrics
 */
export interface AgentMetrics {
  /** Number of messages packed */
  messagesPacked: number;
  /** Number of messages unpacked */
  messagesUnpacked: number;
  /** Number of key operations performed */
  keyOperations: number;
  /** Agent uptime in milliseconds */
  uptime: number;
  /** Last activity timestamp */
  lastActivity: number;
}