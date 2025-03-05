/**
 * TAP-TS Main Module
 * 
 * This is the main entry point for the TAP-TS library.
 */

// Export modules
export {
  Agent
} from "./agent.ts";

export {
  Message,
  MessageType,
  SecurityMode
} from "./message.ts";

// Export MessageHandler as a type
export type { 
  MessageHandler 
} from "./message.ts";

export {
  TapNode
} from "./node.ts";

export * from "./error.ts";

// Export types
export type {
  DIDDocument,
  VerificationMethod,
  Service,
  AgentConfig,
  NodeConfig,
  NetworkConfig,
  MessageMetadata,
  AgentOptions,
  // Export only the non-duplicate type
  MessageSubscriber,
  StorageOptions,
  Keypair,
  DidResolutionResult,
  AuthorizationResult,
  SendMessageOptions,
} from "./types.ts";

export * from "./wasm/loader.ts";
