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
  PING = 'ping',
  PONG = 'pong',
  AUTHORIZATION_REQUEST = 'authorization_request',
  AUTHORIZATION_RESPONSE = 'authorization_response',
}

/**
 * Options for creating a new message
 */
export interface MessageOptions {
  /** Message type */
  type: MessageType;
  
  /** Optional message ID (auto-generated if not provided) */
  id?: string;
  
  /** Optional sender DID */
  from?: string;
  
  /** Optional recipient DIDs */
  to?: string[];
  
  /** Optional creation timestamp (defaults to now) */
  created?: number;
  
  /** Optional expiration timestamp */
  expires?: number;
  
  /** Optional thread ID for message threading */
  threadId?: string;
  
  /** Optional correlation ID for related messages */
  correlation?: string;
  
  /** Optional custom data */
  customData?: Record<string, unknown>;
  
  /** Optional ledger ID */
  ledgerId?: string;
}

/**
 * TAP Message class
 */
export class Message {
  private wasmMessage: any;
  
  /** Message type */
  type: MessageType;
  
  /** Message ID */
  id: string;
  
  /** Message version */
  version = "1.0";
  
  /** Ledger ID */
  ledgerId?: string;
  
  /** Custom data */
  customData?: Record<string, unknown>;
  
  /** Thread ID for message threading */
  threadId?: string;
  
  /** Correlation ID for related messages */
  correlation?: string;
  
  /** Creation timestamp */
  created: number;
  
  /** Expiration timestamp */
  expires?: number;
  
  /** Authorization request data */
  private authorizationRequest?: AuthorizationRequest;
  
  /** Authorization response data */
  private _data: Record<string, unknown> = {};
  
  /**
   * Create a new Message instance
   * 
   * @param options - Options for creating a new message
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
    
    // Convert the message type to a WASM message type
    let wasmMessageType;
    switch (options.type) {
      case MessageType.PING:
        wasmMessageType = module.MessageType.Ping;
        break;
      case MessageType.PONG:
        wasmMessageType = module.MessageType.Pong;
        break;
      case MessageType.AUTHORIZATION_REQUEST:
        wasmMessageType = module.MessageType.AuthorizationRequest;
        break;
      case MessageType.AUTHORIZATION_RESPONSE:
        wasmMessageType = module.MessageType.AuthorizationResponse;
        break;
      default:
        wasmMessageType = module.MessageType.Unknown;
        break;
    }
    
    this.type = options.type;
    this.id = options.id || `msg_${generateUuid()}`;
    this.ledgerId = options.ledgerId;
    this.customData = options.customData;
    this.threadId = options.threadId;
    this.correlation = options.correlation;
    this.created = options.created || Date.now();
    this.expires = options.expires;
    
    this.wasmMessage = new module.Message(wasmMessageType, options.ledgerId);
    
    if (options.from) {
      this.from = options.from;
    }
    
    if (options.to) {
      this.to = options.to;
    }
  }
  
  /**
   * Get the message ID
   * 
   * @returns Message ID
   */
  get getId(): string {
    return this.id;
  }
  
  /**
   * Get the message type
   * 
   * @returns Message type
   */
  get getType(): MessageType {
    return this.type;
  }
  
  /**
   * Get the message version
   * 
   * @returns Message version
   */
  get getVersion(): string {
    return this.version;
  }
  
  /**
   * Get the ledger ID
   * 
   * @returns Ledger ID
   */
  get getLedgerId(): string | undefined {
    return this.ledgerId;
  }
  
  /**
   * Get the authorization request data (if any)
   * 
   * @returns Authorization request data or undefined
   */
  get getAuthorizationRequest(): AuthorizationRequest | undefined {
    const request = this.wasmMessage.authorization_request;
    if (!request) {
      return undefined;
    }
    
    return {
      transactionHash: request.transaction_hash,
      sourceAddress: request.source_address,
      destinationAddress: request.destination_address,
      amount: request.amount,
    };
  }
  
  /**
   * Get the authorization response data (if any)
   * 
   * @returns Authorization response data or undefined
   */
  get getAuthorizationResponse(): AuthorizationResponse | undefined {
    const response = this.wasmMessage.authorization_response;
    if (!response) {
      return undefined;
    }
    
    return {
      transactionHash: response.transaction_hash,
      authorizationResult: response.authorization_result,
      reason: response.reason,
    };
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
        type: ErrorType.MESSAGE_INVALID,
        message: `Cannot set authorization request data on message type ${this.type}`,
      });
    }
    
    // Prepare request data
    if (!this._data) {
      this._data = {};
    }
    
    this._data.authorization_request = {
      transactionHash,
      sourceAddress,
      destinationAddress,
      amount,
      ...additionalData
    };
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
    this.wasmMessage.set_authorization_response(
      transactionHash,
      authorizationResult,
      reason
    );
  }
  
  /**
   * Set authorization request data (compatibility method for tests)
   * 
   * @param requestData - Authorization request data
   */
  setAuthorizationRequestData(requestData: AuthorizationRequest): void {
    this.setAuthorizationRequest(
      requestData.transactionHash || '',
      requestData.sourceAddress || '',
      requestData.destinationAddress || '',
      requestData.amount || ''
    );
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
        type: ErrorType.MESSAGE_INVALID,
        message: `Cannot get authorization request data from message type ${this.type}`,
      });
    }
    
    if (!this._data || !this._data.authorization_request) {
      return undefined;
    }
    
    const request = this._data.authorization_request as Record<string, unknown>;
    
    return {
      transactionHash: request.transaction_hash as string,
      sourceAddress: request.source_address as string,
      destinationAddress: request.destination_address as string,
      amount: request.amount as string,
      // Include any additional fields from the request
      ...Object.fromEntries(
        Object.entries(request)
          .filter(([key]) => !['transaction_hash', 'source_address', 'destination_address', 'amount'].includes(key))
          .map(([key, value]) => [key, value])
      )
    };
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
        type: ErrorType.MESSAGE_INVALID,
        message: `Cannot set authorization response data on message type ${this.type}`,
      });
    }
    
    // Store data in the message
    if (!this._data) {
      this._data = {};
    }
    
    if (!this._data.authorization_response) {
      this._data.authorization_response = {};
    }
    
    const responseData = this._data.authorization_response as Record<string, unknown>;
    
    responseData.transactionHash = data.transactionHash;
    responseData.authorizationResult = data.authorizationResult !== undefined 
      ? String(data.authorizationResult)
      : (data.approved ? "true" : "false");
    responseData.reason = data.reason || "";
    
    // Set the approved property for backward compatibility
    if (data.approved !== undefined) {
      responseData.approved = data.approved;
    } else if (data.authorizationResult !== undefined) {
      // Convert string "true"/"false" to boolean if needed
      responseData.approved = 
        typeof data.authorizationResult === 'string'
          ? data.authorizationResult === 'true'
          : Boolean(data.authorizationResult);
    }
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
        type: ErrorType.MESSAGE_INVALID,
        message: `Cannot get authorization response data from message type ${this.type}`,
      });
    }
    
    if (!this._data || !this._data.authorization_response) {
      return undefined;
    }
    
    const responseData = this._data.authorization_response as Record<string, unknown>;
    
    return {
      transactionHash: responseData.transactionHash as string,
      authorizationResult: responseData.authorizationResult as string,
      approved: responseData.approved !== undefined 
        ? Boolean(responseData.approved)
        : (responseData.authorizationResult as string) === 'true',
      reason: responseData.reason as string,
    };
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
   * Recipient DIDs for the message
   */
  get to(): string[] | undefined {
    if (!this._data.to) {
      return undefined;
    }
    
    return Array.isArray(this._data.to) ? this._data.to : [this._data.to as string];
  }
  
  /**
   * Set recipient DIDs for the message
   */
  set to(value: string[] | undefined) {
    if (!value) {
      delete this._data.to;
      return;
    }
    
    this._data.to = value;
  }
  
  /**
   * Sender DID for the message
   */
  get from(): string | undefined {
    return this._data.from as string;
  }
  
  /**
   * Set sender DID for the message
   */
  set from(value: string | undefined) {
    if (!value) {
      delete this._data.from;
      return;
    }
    
    this._data.from = value;
  }
  
  /**
   * Create a message from raw data
   * 
   * @param data - Raw message data
   * @returns A new Message instance
   */
  static fromJSON(data: unknown): Message {
    // Parse string data if necessary
    if (typeof data === 'string') {
      try {
        data = JSON.parse(data);
      } catch (error) {
        throw new TapError({
          type: ErrorType.MESSAGE_INVALID,
          message: 'Failed to parse message JSON',
          cause: error,
        });
      }
    }
    
    // Basic type check
    if (!data || typeof data !== 'object') {
      throw new TapError({
        type: ErrorType.MESSAGE_INVALID,
        message: 'Invalid message format: data is not an object',
      });
    }
    
    const typed = data as Record<string, unknown>;
    
    // Check required fields
    if (!typed.message_type || typeof typed.message_type !== 'string') {
      throw new TapError({
        type: ErrorType.MESSAGE_INVALID,
        message: 'Invalid message format: missing or invalid message_type',
      });
    }
    
    if (!typed.ledger_id || typeof typed.ledger_id !== 'string') {
      throw new TapError({
        type: ErrorType.MESSAGE_INVALID,
        message: 'Invalid message format: missing or invalid ledger_id',
      });
    }
    
    // Create a new message
    let messageType: MessageType;
    switch (typed.message_type) {
      case MessageType.AUTHORIZATION_REQUEST:
        messageType = MessageType.AUTHORIZATION_REQUEST;
        break;
      case MessageType.AUTHORIZATION_RESPONSE:
        messageType = MessageType.AUTHORIZATION_RESPONSE;
        break;
      case MessageType.PING:
        messageType = MessageType.PING;
        break;
      default:
        throw new TapError({
          type: ErrorType.MESSAGE_INVALID,
          message: `Invalid message type: ${typed.message_type}`,
        });
    }
    
    const message = new Message({
      type: messageType,
      ledgerId: typed.ledger_id as string,
    });
    
    // Set authorization request data if present
    if (typed.authorization_request && typeof typed.authorization_request === 'object') {
      const req = typed.authorization_request as Record<string, unknown>;
      
      if (req.transaction_hash && typeof req.transaction_hash !== 'string') {
        throw new TapError({
          type: ErrorType.MESSAGE_INVALID,
          message: 'Invalid authorization request: invalid transaction_hash',
        });
      }
      
      if (req.source_address && typeof req.source_address !== 'string') {
        throw new TapError({
          type: ErrorType.MESSAGE_INVALID,
          message: 'Invalid authorization request: invalid source_address',
        });
      }
      
      if (req.destination_address && typeof req.destination_address !== 'string') {
        throw new TapError({
          type: ErrorType.MESSAGE_INVALID,
          message: 'Invalid authorization request: invalid destination_address',
        });
      }
      
      if (req.amount && typeof req.amount !== 'string') {
        throw new TapError({
          type: ErrorType.MESSAGE_INVALID,
          message: 'Invalid authorization request: invalid amount',
        });
      }
      
      message.setAuthorizationRequest(
        req.transaction_hash as string,
        req.source_address as string,
        req.destination_address as string,
        req.amount as string
      );
    }
    
    // Set authorization response data if present
    if (typed.authorization_response && typeof typed.authorization_response === 'object') {
      const res = typed.authorization_response as Record<string, unknown>;
      
      if (res.transaction_hash && typeof res.transaction_hash !== 'string') {
        throw new TapError({
          type: ErrorType.MESSAGE_INVALID,
          message: 'Invalid authorization response: invalid transaction_hash',
        });
      }
      
      if (res.authorization_result && typeof res.authorization_result !== 'string') {
        throw new TapError({
          type: ErrorType.MESSAGE_INVALID,
          message: 'Invalid authorization response: invalid authorization_result',
        });
      }
      
      message.setAuthorizationResponse(
        res.transaction_hash as string,
        res.authorization_result === 'true',
        res.reason as string | undefined
      );
    }
    
    // Set from and to fields
    if (typed.from) {
      message.from = typed.from as string;
    }
    
    if (typed.to) {
      message.to = Array.isArray(typed.to) ? typed.to : [typed.to as string];
    }
    
    return message;
  }
  
  /**
   * Convert to JSON
   * 
   * @returns JSON representation of the message
   */
  toJSON(): Record<string, unknown> {
    // Basic properties
    const result: Record<string, unknown> = {
      id: this.id,
      type: this.type,
      ledgerId: this.ledgerId,
    };
    
    // Optional properties
    if (this.from) result.from = this.from;
    if (this.to?.length) result.to = this.to;
    if (this.created) result.created = this.created;
    if (this.expires) result.expires = this.expires;
    if (this.threadId) result.threadId = this.threadId;
    if (this.correlation) result.correlation = this.correlation;
    if (this.customData) result.customData = this.customData;
    
    // Authorization data
    if (this.authorizationRequest) {
      result.authorization_request = this.authorizationRequest;
    }
    
    if (this._data.authorization_response) {
      result.authorization_response = this._data.authorization_response;
    }
    
    return result;
  }
}

/**
 * Type for message handler functions
 */
export type MessageHandler = (message: Message, metadata?: MessageMetadata) => Promise<void>;

/**
 * Type for message subscriber functions
 */
export type MessageSubscriber = (message: Message, metadata?: MessageMetadata) => void;

/**
 * Generate a UUID
 * 
 * @returns A UUID string
 */
function generateUuid(): string {
  return crypto.randomUUID();
}
