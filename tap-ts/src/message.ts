/**
 * Message handling for TAP-TS
 * 
 * This module provides classes for creating and handling TAP messages.
 */

import { TapError, ErrorType } from './error.ts';
import {
  MessageType,
  AuthorizationRequest,
  AuthorizationResponse,
  MessageMetadata,
} from './types.ts';
import wasmLoader from './wasm/mod.ts';

/**
 * TAP Message class
 */
export class Message {
  private wasmMessage: any;
  
  /**
   * Create a new Message instance
   * 
   * @param messageType - Type of message to create
   * @param ledgerId - Ledger identifier
   */
  constructor(messageType: MessageType, ledgerId: string) {
    if (!wasmLoader.moduleIsLoaded()) {
      throw new TapError({
        type: ErrorType.WASM_NOT_LOADED,
        message: 'WASM module is not loaded',
      });
    }
    
    const module = wasmLoader.getModule();
    let wasmMessageType;
    
    switch (messageType) {
      case MessageType.AUTHORIZATION_REQUEST:
        wasmMessageType = module.MessageType.AuthorizationRequest;
        break;
      case MessageType.AUTHORIZATION_RESPONSE:
        wasmMessageType = module.MessageType.AuthorizationResponse;
        break;
      case MessageType.PING:
        wasmMessageType = module.MessageType.Ping;
        break;
      default:
        throw new TapError({
          type: ErrorType.INVALID_ARGUMENT,
          message: `Invalid message type: ${messageType}`,
        });
    }
    
    this.wasmMessage = new module.Message(wasmMessageType, ledgerId);
  }
  
  /**
   * Get the message ID
   * 
   * @returns Message ID
   */
  get id(): string {
    return this.wasmMessage.id;
  }
  
  /**
   * Get the message type
   * 
   * @returns Message type
   */
  get type(): MessageType {
    return this.wasmMessage.message_type as MessageType;
  }
  
  /**
   * Get the message version
   * 
   * @returns Message version
   */
  get version(): string {
    return this.wasmMessage.version;
  }
  
  /**
   * Get the ledger ID
   * 
   * @returns Ledger ID
   */
  get ledgerId(): string {
    return this.wasmMessage.ledger_id;
  }
  
  /**
   * Get the authorization request data (if any)
   * 
   * @returns Authorization request data or undefined
   */
  get authorizationRequest(): AuthorizationRequest | undefined {
    const request = this.wasmMessage.authorization_request;
    if (!request) {
      return undefined;
    }
    
    return {
      transactionHash: request.transaction_hash,
      sender: request.sender,
      receiver: request.receiver,
      amount: request.amount,
    };
  }
  
  /**
   * Get the authorization response data (if any)
   * 
   * @returns Authorization response data or undefined
   */
  get authorizationResponse(): AuthorizationResponse | undefined {
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
   * Set authorization request data
   * 
   * @param transactionHash - Transaction hash
   * @param sender - Sender address
   * @param receiver - Receiver address
   * @param amount - Transaction amount
   */
  setAuthorizationRequest(
    transactionHash: string,
    sender: string,
    receiver: string,
    amount: string
  ): void {
    this.wasmMessage.set_authorization_request(
      transactionHash,
      sender,
      receiver,
      amount
    );
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
   * Get the underlying WASM message
   * 
   * @returns WASM message
   */
  getWasmMessage(): any {
    return this.wasmMessage;
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
    
    const message = new Message(messageType, typed.ledger_id as string);
    
    // Set authorization request data if present
    if (typed.authorization_request && typeof typed.authorization_request === 'object') {
      const req = typed.authorization_request as Record<string, unknown>;
      
      if (!req.transaction_hash || typeof req.transaction_hash !== 'string') {
        throw new TapError({
          type: ErrorType.MESSAGE_INVALID,
          message: 'Invalid authorization request: missing or invalid transaction_hash',
        });
      }
      
      if (!req.sender || typeof req.sender !== 'string') {
        throw new TapError({
          type: ErrorType.MESSAGE_INVALID,
          message: 'Invalid authorization request: missing or invalid sender',
        });
      }
      
      if (!req.receiver || typeof req.receiver !== 'string') {
        throw new TapError({
          type: ErrorType.MESSAGE_INVALID,
          message: 'Invalid authorization request: missing or invalid receiver',
        });
      }
      
      if (!req.amount || typeof req.amount !== 'string') {
        throw new TapError({
          type: ErrorType.MESSAGE_INVALID,
          message: 'Invalid authorization request: missing or invalid amount',
        });
      }
      
      message.setAuthorizationRequest(
        req.transaction_hash as string,
        req.sender as string,
        req.receiver as string,
        req.amount as string
      );
    }
    
    // Set authorization response data if present
    if (typed.authorization_response && typeof typed.authorization_response === 'object') {
      const res = typed.authorization_response as Record<string, unknown>;
      
      if (!res.transaction_hash || typeof res.transaction_hash !== 'string') {
        throw new TapError({
          type: ErrorType.MESSAGE_INVALID,
          message: 'Invalid authorization response: missing or invalid transaction_hash',
        });
      }
      
      if (typeof res.authorization_result !== 'boolean') {
        throw new TapError({
          type: ErrorType.MESSAGE_INVALID,
          message: 'Invalid authorization response: missing or invalid authorization_result',
        });
      }
      
      message.setAuthorizationResponse(
        res.transaction_hash as string,
        res.authorization_result as boolean,
        res.reason as string | undefined
      );
    }
    
    return message;
  }
  
  /**
   * Convert the message to a JSON string
   * 
   * @returns JSON string representation of the message
   */
  toJSON(): string {
    // In a full implementation, we would convert the WASM message to a JSON string
    // For now, we'll create a simple representation
    const result: Record<string, unknown> = {
      id: this.id,
      message_type: this.type,
      version: this.version,
      ledger_id: this.ledgerId,
    };
    
    if (this.authorizationRequest) {
      result.authorization_request = {
        transaction_hash: this.authorizationRequest.transactionHash,
        sender: this.authorizationRequest.sender,
        receiver: this.authorizationRequest.receiver,
        amount: this.authorizationRequest.amount,
      };
    }
    
    if (this.authorizationResponse) {
      result.authorization_response = {
        transaction_hash: this.authorizationResponse.transactionHash,
        authorization_result: this.authorizationResponse.authorizationResult,
      };
      
      if (this.authorizationResponse.reason) {
        result.authorization_response.reason = this.authorizationResponse.reason;
      }
    }
    
    return JSON.stringify(result);
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
