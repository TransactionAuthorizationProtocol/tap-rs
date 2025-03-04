/**
 * Message module for TAP-TS
 * 
 * @module message
 */

import * as uuid from "@std/uuid/mod.ts";
import { TapError, ErrorType } from "./error.ts";
import type { MessageMetadata } from "./types.ts";
import { wasmLoader } from "./wasm/loader.ts";

// Local interface definitions to avoid import conflicts
interface AuthorizationRequest {
  transactionHash: string;
  transactionData?: string;
  sourceAddress?: string;
  destinationAddress?: string;
  amount?: string;
  fee?: string;
  network?: string;
  reference?: string;
  callbackUrl?: string;
  [key: string]: unknown;
}

interface AuthorizationResponse {
  transactionHash: string;
  authorizationResult?: string | boolean;
  approved?: boolean;
  reason?: string;
}

/**
 * Message types for TAP
 */
export enum MessageType {
  PING = 'TAP_PING',
  PONG = 'TAP_PONG',
  AUTHORIZATION_REQUEST = 'TAP_AUTHORIZATION_REQUEST',
  AUTHORIZATION_RESPONSE = 'TAP_AUTHORIZATION_RESPONSE',
}

/**
 * Security mode for messages
 */
export enum SecurityMode {
  PLAIN = 'plain',
  SIGNED = 'signed',
  ENCRYPTED = 'encrypted',
}

/**
 * Options for creating a new message
 */
export interface MessageOptions {
  /** Message type */
  type: MessageType;
  
  /** Optional message ID (auto-generated if not provided) */
  id?: string;
  
  /** Ledger ID */
  ledgerId?: string;
  
  /** Custom data to include with the message */
  customData?: Record<string, unknown>;
  
  /** Thread ID for tracking message threads */
  threadId?: string;
  
  /** Correlation ID for tracking related messages */
  correlation?: string;
  
  /** Creation timestamp (defaults to now) */
  created?: number;
  
  /** Expiration timestamp */
  expires?: number;
  
  /** Sender DID */
  from?: string;
  
  /** Recipient DIDs */
  to?: string | string[];
  
  /** Security mode for the message */
  securityMode?: SecurityMode;
}

/**
 * TAP Message class
 * 
 * This class represents a TAP message using the DIDComm message format.
 */
export class Message {
  private wasmMessage: any;
  type: MessageType;
  id: string;
  version = "1.0";
  ledgerId?: string;
  customData?: Record<string, unknown>;
  threadId?: string;
  correlation?: string;
  created: number;
  expires?: number;
  securityMode: SecurityMode = SecurityMode.PLAIN;
  private _data: Record<string, unknown> = {};

  /**
   * Create a new TAP message
   * 
   * @param options Message options
   */
  constructor(options: MessageOptions) {
    if (!wasmLoader.moduleIsLoaded()) {
      throw new TapError({
        type: ErrorType.WASM_NOT_LOADED,
        message: "WASM module not loaded",
      });
    }
    
    // Get the WASM module
    const module = wasmLoader.getModule();
    
    this.type = options.type;
    this.id = options.id || `msg_${generateUuid()}`;
    this.ledgerId = options.ledgerId || "";
    this.customData = options.customData;
    this.threadId = options.threadId;
    this.correlation = options.correlation;
    this.created = options.created || Date.now();
    this.expires = options.expires;
    this.securityMode = options.securityMode || SecurityMode.PLAIN;
    
    // Create the WASM message
    this.wasmMessage = new module.Message(
      this.id,
      this.type,
      this.version,
      this.ledgerId || ""
    );
    
    // Set sender and recipient if provided
    if (options.from) {
      this.from(options.from);
    }
    
    if (options.to) {
      const toArray = Array.isArray(options.to) ? options.to : [options.to];
      if (toArray.length > 0) {
        // Set the first recipient (WASM binding limitation for now)
        this.to([toArray[0]]);
      }
    }
  }

  /**
   * Get the message ID
   * 
   * @returns Message ID
   */
  getId(): string {
    return this.wasmMessage.id();
  }

  /**
   * Get the message type
   * 
   * @returns Message type
   */
  getType(): MessageType {
    return this.type;
  }

  /**
   * Get the message version
   * 
   * @returns Message version
   */
  getVersion(): string {
    return this.wasmMessage.version();
  }

  /**
   * Get the ledger ID
   * 
   * @returns Ledger ID
   */
  getLedgerId(): string | undefined {
    return this.wasmMessage.ledger_id();
  }

  /**
   * Get the authorization request data (if any)
   * 
   * @returns Authorization request data or undefined
   */
  getAuthorizationRequest(): AuthorizationRequest | undefined {
    if (this.type !== MessageType.AUTHORIZATION_REQUEST) {
      return undefined;
    }
    
    const wasmData = this.wasmMessage.authorization_request();
    if (!wasmData) {
      return undefined;
    }
    
    return wasmData as AuthorizationRequest;
  }

  /**
   * Get the authorization response data (if any)
   * 
   * @returns Authorization response data or undefined
   */
  getAuthorizationResponse(): AuthorizationResponse | undefined {
    if (this.type !== MessageType.AUTHORIZATION_RESPONSE) {
      return undefined;
    }
    
    const wasmData = this.wasmMessage.authorization_response();
    if (!wasmData) {
      return undefined;
    }
    
    return wasmData as AuthorizationResponse;
  }

  /**
   * Set authorization request properties
   * 
   * @param transactionHash - Transaction hash
   * @param sourceAddress - Source address
   * @param destinationAddress - Destination address
   * @param amount - Transaction amount
   * @param additionalData - Additional data
   * @throws If the message type is not AUTHORIZATION_REQUEST
   */
  setAuthorizationRequest(
    transactionHash: string,
    sourceAddress: string,
    destinationAddress: string,
    amount: string,
    additionalData?: Record<string, unknown>
  ): void {
    if (this.type !== MessageType.AUTHORIZATION_REQUEST) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set authorization request on ${this.type} message`,
      });
    }
    
    const request: AuthorizationRequest = {
      transactionHash,
      sourceAddress,
      destinationAddress,
      amount,
      ...additionalData,
    };
    
    this.setAuthorizationRequestData(request);
  }

  /**
   * Set authorization response data
   * 
   * @param transactionHash - Transaction hash
   * @param authorizationResult - Authorization result
   * @param reason - Optional reason for the decision
   */
  setAuthorizationResponse(
    transactionHash: string,
    authorizationResult: boolean,
    reason?: string
  ): void {
    if (this.type !== MessageType.AUTHORIZATION_RESPONSE) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set authorization response on ${this.type} message`,
      });
    }
    
    const response: AuthorizationResponse = {
      transactionHash,
      authorizationResult: authorizationResult.toString(),
      approved: authorizationResult,
      reason,
    };
    
    this.setAuthorizationResponseData(response);
  }

  /**
   * Set authorization request data (compatibility method for tests)
   * 
   * @param requestData - Authorization request data
   */
  setAuthorizationRequestData(requestData: AuthorizationRequest): void {
    if (this.type !== MessageType.AUTHORIZATION_REQUEST) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set authorization request on ${this.type} message`,
      });
    }
    
    this._data = { ...this._data, ...requestData };
    this.wasmMessage.set_authorization_request(requestData);
  }

  /**
   * Get authorization request data
   * 
   * @returns Authorization request data object
   * @throws If the message type is not AUTHORIZATION_REQUEST
   */
  getAuthorizationRequestData(): AuthorizationRequest | undefined {
    if (this.type !== MessageType.AUTHORIZATION_REQUEST) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot get authorization request from ${this.type} message`,
      });
    }
    
    return this.getAuthorizationRequest();
  }

  /**
   * Set authorization response data
   * 
   * @param data - Authorization response data
   * @throws If the message type is not AUTHORIZATION_RESPONSE
   */
  setAuthorizationResponseData(data: AuthorizationResponse): void {
    if (this.type !== MessageType.AUTHORIZATION_RESPONSE) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set authorization response on ${this.type} message`,
      });
    }
    
    this._data = { ...this._data, ...data };
    
    // We need to generate the signed date in ISO format
    const signedDate = new Date().toISOString();
    
    // Set valid until to 24 hours from now by default
    const validUntil = this.expires 
      ? new Date(this.expires).toISOString() 
      : new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString();
    
    this.wasmMessage.set_authorization_response(data, signedDate, validUntil);
  }

  /**
   * Get authorization response data
   * 
   * @returns Authorization response data or undefined if not set
   * @throws If the message type is not AUTHORIZATION_RESPONSE
   */
  getAuthorizationResponseData(): AuthorizationResponse | undefined {
    if (this.type !== MessageType.AUTHORIZATION_RESPONSE) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot get authorization response from ${this.type} message`,
      });
    }
    
    return this.getAuthorizationResponse();
  }

  /**
   * Get the underlying WASM message
   * 
   * @returns WASM message
   */
  getWasmMessage(): any {
    return this.wasmMessage;
  }

  /**
   * Get or set recipient DIDs for the message
   * 
   * @param value - DIDs to set or undefined to get current value
   * @returns Current recipient DIDs if getting, this if setting
   */
  to(value?: string[] | undefined): string[] | undefined | this {
    if (value === undefined) {
      const toDid = this.wasmMessage.to_did();
      return toDid ? [toDid] : undefined;
    }
    
    // Support only the first recipient for now (WASM binding limitation)
    if (value.length > 0) {
      this.wasmMessage.set_to_did(value[0]);
    } else {
      this.wasmMessage.set_to_did(null);
    }
    
    return this;
  }

  /**
   * Get or set sender DID for the message
   * 
   * @param value - DID to set or undefined to get current value
   * @returns Current sender DID if getting, this if setting
   */
  from(value?: string | undefined): string | undefined | this {
    if (value === undefined) {
      return this.wasmMessage.from_did();
    }
    
    this.wasmMessage.set_from_did(value);
    return this;
  }

  /**
   * Sign the message using the agent's keys
   * 
   * @param agent - Agent to sign the message with
   * @returns This message for chaining
   */
  sign(agent: any): this {
    if (this.securityMode === SecurityMode.PLAIN) {
      this.securityMode = SecurityMode.SIGNED;
    }
    
    if (agent.sign_message) {
      agent.sign_message(this.wasmMessage);
    }
    
    return this;
  }

  /**
   * Encrypt the message for the specified recipients
   * 
   * @param agent - Agent to encrypt the message with
   * @param recipients - Recipients to encrypt for (defaults to message's to field)
   * @returns Encrypted message
   */
  async encrypt(agent: any, recipients?: string[]): Promise<this> {
    this.securityMode = SecurityMode.ENCRYPTED;
    
    // In the future, implement actual encryption here
    // For now, just mark it as encrypted
    
    return this;
  }

  /**
   * Verify the message signature
   * 
   * @returns True if the message signature is valid
   */
  verify(): boolean {
    // In the future, implement actual verification here
    // For now, just return true
    return true;
  }

  /**
   * Decode an encrypted message
   * 
   * @param agent - Agent to decrypt the message with
   * @returns Decrypted message
   */
  async decrypt(agent: any): Promise<this> {
    // In the future, implement actual decryption here
    // For now, just mark it as plain
    this.securityMode = SecurityMode.PLAIN;
    
    return this;
  }

  /**
   * Create a message from bytes
   * 
   * @param bytes - Message bytes
   * @returns A new Message instance
   */
  static fromBytes(bytes: Uint8Array): Message {
    if (!wasmLoader.moduleIsLoaded()) {
      throw new TapError({
        type: ErrorType.WASM_NOT_LOADED,
        message: "WASM module not loaded",
      });
    }
    
    const module = wasmLoader.getModule();
    const wasmMessage = module.Message.fromBytes(bytes);
    
    // Create a new Message instance with basic properties
    const message = new Message({
      id: wasmMessage.id(),
      type: wasmMessage.message_type() as MessageType,
      ledgerId: wasmMessage.ledger_id(),
    });
    
    // Replace the WASM message with the one we got from bytes
    message.wasmMessage = wasmMessage;
    
    return message;
  }

  /**
   * Create a message from a JSON string
   * 
   * @param json - Message JSON
   * @returns A new Message instance
   */
  static fromJSON(json: string): Message {
    if (!wasmLoader.moduleIsLoaded()) {
      throw new TapError({
        type: ErrorType.WASM_NOT_LOADED,
        message: "WASM module not loaded",
      });
    }
    
    const module = wasmLoader.getModule();
    const wasmMessage = module.Message.fromJson(json);
    
    // Create a new Message instance with basic properties
    const message = new Message({
      id: wasmMessage.id(),
      type: wasmMessage.message_type() as MessageType,
      ledgerId: wasmMessage.ledger_id(),
    });
    
    // Replace the WASM message with the one we got from JSON
    message.wasmMessage = wasmMessage;
    
    return message;
  }

  /**
   * Create a message from raw data
   * 
   * @param data - Raw message data
   * @returns A new Message instance
   */
  static fromData(data: unknown): Message {
    if (typeof data === 'string') {
      return Message.fromJSON(data);
    }
    
    if (data instanceof Uint8Array) {
      return Message.fromBytes(data);
    }
    
    // Convert object to JSON string and parse
    return Message.fromJSON(JSON.stringify(data));
  }

  /**
   * Convert to JSON
   * 
   * @returns JSON representation of the message
   */
  toJSON(): Record<string, unknown> {
    const base = {
      id: this.getId(),
      type: this.getType(),
      version: this.getVersion(),
      ledgerId: this.getLedgerId(),
      created: this.created,
      expires: this.expires,
      threadId: this.threadId,
      correlation: this.correlation,
      securityMode: this.securityMode,
    };
    
    // Add from/to if present
    const from = this.from();
    if (from) {
      Object.assign(base, { from });
    }
    
    const to = this.to();
    if (to) {
      Object.assign(base, { to });
    }
    
    // Add request/response data if present
    if (this.type === MessageType.AUTHORIZATION_REQUEST) {
      Object.assign(base, { 
        authorizationRequest: this.getAuthorizationRequest() 
      });
    } else if (this.type === MessageType.AUTHORIZATION_RESPONSE) {
      Object.assign(base, { 
        authorizationResponse: this.getAuthorizationResponse() 
      });
    }
    
    // Add custom data if present
    if (this.customData) {
      Object.assign(base, { customData: this.customData });
    }
    
    return base;
  }

  /**
   * Convert to a string (JSON)
   * 
   * @returns JSON string representation of the message
   */
  toString(): string {
    return JSON.stringify(this.toJSON());
  }

  /**
   * Convert to bytes
   * 
   * @returns Uint8Array containing the message bytes
   */
  toBytes(): Uint8Array {
    return this.wasmMessage.to_bytes();
  }
}

/**
 * Type for message handler functions
 */
export type MessageHandler = (message: Message, metadata?: MessageMetadata) => void | Promise<void>;

/**
 * Type for message subscriber functions
 */
export type MessageSubscriber = (message: Message, metadata?: MessageMetadata) => void | Promise<void>;

/**
 * Generate a UUID
 * 
 * @returns A UUID string
 */
function generateUuid(): string {
  return uuid.v4.toString().replace(/-/g, "");
}
