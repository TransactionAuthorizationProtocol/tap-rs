/**
 * @taprsvp/agent - TypeScript wrapper for TAP WASM Agent
 * 
 * Browser-optimized agent for TAP message packing/unpacking with flexible key management.
 * This package provides cryptographic operations via WASM while you create messages using @taprsvp/types.
 * 
 * @example Basic Usage
 * ```typescript
 * import { TapAgent, generatePrivateKey, createTransferMessage } from '@taprsvp/agent';
 * import type { Transfer } from '@taprsvp/types';
 * 
 * // Create a new agent
 * const agent = await TapAgent.create({
 *   keyType: 'Ed25519',
 *   nickname: 'my-agent'
 * });
 * 
 * // Create a TAP-compliant message using types
 * const transfer: Transfer = createTransferMessage({
 *   from: agent.did,
 *   to: ['did:key:recipient'],
 *   amount: '100.0',
 *   asset: 'eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48',
 *   originator: { '@id': agent.did },
 *   beneficiary: { '@id': 'did:key:recipient' }
 * });
 * 
 * // Pack (sign) the message
 * const packed = await agent.pack(transfer);
 * console.log('Packed message:', packed.message);
 * ```
 * 
 * @example Key Management
 * ```typescript
 * import { TapAgent, generatePrivateKey } from '@taprsvp/agent';
 * 
 * // Generate a new private key
 * const privateKey = generatePrivateKey('Ed25519');
 * 
 * // Create agent from existing key
 * const agent = await TapAgent.fromPrivateKey(privateKey, 'Ed25519');
 * 
 * // Export keys for storage
 * const exportedKey = agent.exportPrivateKey();
 * localStorage.setItem('tapAgent.key', exportedKey);
 * ```
 * 
 * @example Custom DID Resolver
 * ```typescript
 * import { TapAgent } from '@taprsvp/agent';
 * import { Resolver } from 'did-resolver';
 * import { getResolver as getWebResolver } from 'web-did-resolver';
 * 
 * const didResolver = new Resolver({
 *   ...getWebResolver(),
 * });
 * 
 * const agent = await TapAgent.create({
 *   didResolver
 * });
 * 
 * const didDoc = await agent.resolveDID('did:web:example.com');
 * ```
 */

// Main exports
export { TapAgent } from './tap-agent.js';
export {
  generatePrivateKey,
  generateUUID,
  isValidDID,
  isValidPrivateKey,
  validateKeyType,
} from './utils.js';
export {
  createTransferMessage,
  createPaymentMessage,
  createAuthorizeMessage,
  createRejectMessage,
  createSettleMessage,
  createConnectMessage,
  createBasicMessage,
  createDIDCommMessage,
} from './message-helpers.js';

// Type exports
export type {
  TapAgentConfig,
  DIDCommMessage,
  PackedMessageResult,
  PackedMessage,
  JWSMessage,
  JWEMessage,
  KeyType,
  DIDDocument,
  DIDResolver,
  DIDResolutionResult,
  DIDResolutionOptions,
  DIDResolutionMetadata,
  DIDDocumentMetadata,
  VerificationMethod,
  ServiceEndpoint,
  MessageAttachment,
  PackOptions,
  UnpackOptions,
  TapMessageTypes,
  TapMessageTypeName,
  AgentMetrics,
} from './types.js';

// Error exports
export {
  TapAgentError,
  TapAgentKeyError,
  TapAgentMessageError,
  TapAgentDIDError,
  isJWS,
  isJWE,
} from './types.js';

// Type mapping utilities (for advanced use)
export {
  validateTapMessageType,
  extractMessageTypeName,
  messageTypeToUri,
  validateMessageStructure,
} from './type-mapping.js';

/**
 * Package version
 */
export const VERSION = '0.1.0';

/**
 * Supported TAP message types
 */
export const SUPPORTED_MESSAGE_TYPES = [
  'Transfer',
  'Payment',
  'Authorize', 
  'Reject',
  'Settle',
  'Cancel',
  'Revert',
  'Connect',
  'Escrow',
  'Capture',
  'AddAgents',
  'ReplaceAgent',
  'RemoveAgent',
  'UpdatePolicies',
  'UpdateParty',
  'ConfirmRelationship',
  'AuthorizationRequired',
  'Presentation',
  'TrustPing',
  'BasicMessage',
] as const;

/**
 * Supported cryptographic key types
 */
export const SUPPORTED_KEY_TYPES = [
  'Ed25519',
  'P256', 
  'secp256k1',
] as const;

/**
 * Default configuration values
 */
export const DEFAULT_CONFIG = {
  keyType: 'Ed25519' as const,
  maxMessageAge: 3600, // 1 hour in seconds
  retryAttempts: 3,
  retryBaseDelay: 100, // milliseconds
} as const;