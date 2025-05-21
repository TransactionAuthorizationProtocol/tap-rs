/**
 * TypeScript typings for the WASM interface
 */

/**
 * WasmTapAgent interface - represents the TAP Agent WASM bindings
 */
export interface WasmTapAgent {
  /**
   * Get the agent's DID
   */
  get_did(): string;

  /**
   * Get the agent's nickname
   */
  nickname(): string | undefined;

  /**
   * Pack a message for transmission
   */
  packMessage(message: any): Promise<{ message: string, metadata: any }>;

  /**
   * Unpack a received message
   */
  unpackMessage(packedMessage: string, expectedType?: string): Promise<any>;

  /**
   * Register a message handler for a specific message type
   */
  registerMessageHandler(messageType: string, handler: (message: any, metadata?: any) => Promise<any>): void;

  /**
   * Process a received message
   */
  processMessage(message: any, metadata: any): Promise<any>;

  /**
   * Subscribe to all messages
   */
  subscribeToMessages(callback: (message: any, metadata?: any) => void): any;

  /**
   * Create a new message with the specified type
   */
  createMessage(messageType: string): any;

  /**
   * Generate a new key with the specified type
   */
  generateKey(keyType: string): Promise<any>;
}

/**
 * TapNode interface - represents a TAP node in the WASM bindings
 */
export interface TapNode {
  /**
   * Add an agent to this node
   */
  add_agent(agent: WasmTapAgent): void;

  /**
   * Get an agent from this node by DID
   */
  get_agent(did: string): WasmTapAgent | undefined;

  /**
   * List all agents in this node
   */
  list_agents(): any;

  /**
   * Remove an agent from this node
   */
  remove_agent(did: string): boolean;
}

/**
 * Key type enumeration in WASM
 */
export enum WasmKeyType {
  /**
   * Ed25519 key type
   */
  Ed25519 = 0,

  /**
   * P-256 key type
   */
  P256 = 1,

  /**
   * Secp256k1 key type
   */
  Secp256k1 = 2,
}

/**
 * Message type enumeration in WASM
 */
export enum WasmMessageType {
  /**
   * Transfer message
   */
  Transfer = 0,

  /**
   * Payment message
   */
  Payment = 1,

  /**
   * Presentation message
   */
  Presentation = 2,

  /**
   * Authorize message
   */
  Authorize = 3,

  /**
   * Reject message
   */
  Reject = 4,

  /**
   * Settle message
   */
  Settle = 5,

  /**
   * Cancel message
   */
  Cancel = 6,

  /**
   * Revert message
   */
  Revert = 7,

  /**
   * AddAgents message
   */
  AddAgents = 8,

  /**
   * ReplaceAgent message
   */
  ReplaceAgent = 9,

  /**
   * RemoveAgent message
   */
  RemoveAgent = 10,

  /**
   * UpdatePolicies message
   */
  UpdatePolicies = 11,

  /**
   * UpdateParty message
   */
  UpdateParty = 12,

  /**
   * ConfirmRelationship message
   */
  ConfirmRelationship = 13,

  /**
   * Connect message
   */
  Connect = 14,

  /**
   * AuthorizationRequired message
   */
  AuthorizationRequired = 15,

  /**
   * Complete message
   */
  Complete = 16,

  /**
   * Error message
   */
  Error = 17,

  /**
   * Unknown message type
   */
  Unknown = 18,
}