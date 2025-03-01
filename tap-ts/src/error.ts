/**
 * Error handling for TAP-TS
 * 
 * This module provides error types and classes for TAP-TS.
 */

/**
 * Error types for TAP-TS
 */
export enum ErrorType {
  // General errors
  UNKNOWN = 'unknown',
  NOT_IMPLEMENTED = 'not_implemented',
  INVALID_ARGUMENT = 'invalid_argument',
  INVALID_STATE = 'invalid_state',
  
  // WASM-related errors
  WASM_LOAD_ERROR = 'wasm_load_error',
  WASM_INIT_ERROR = 'wasm_init_error',
  WASM_NOT_LOADED = 'wasm_not_loaded',
  
  // DID-related errors
  DID_RESOLUTION_ERROR = 'did_resolution_error',
  DID_NOT_FOUND = 'did_not_found',
  DID_INVALID = 'did_invalid',
  
  // Message-related errors
  MESSAGE_INVALID = 'message_invalid',
  MESSAGE_SEND_ERROR = 'message_send_error',
  
  // Agent-related errors
  AGENT_NOT_FOUND = 'agent_not_found',
  AGENT_ALREADY_EXISTS = 'agent_already_exists',
  
  // Node-related errors
  NODE_NOT_INITIALIZED = 'node_not_initialized',
}

/**
 * Options for creating a TapError
 */
export interface TapErrorOptions {
  /** Error type */
  type: ErrorType;
  
  /** Error message */
  message: string;
  
  /** Optional underlying cause */
  cause?: unknown;
  
  /** Optional additional data */
  data?: Record<string, unknown>;
}

/**
 * TAP Error class
 */
export class TapError extends Error {
  /** Error type */
  readonly type: ErrorType;
  
  /** Optional underlying cause */
  readonly cause?: unknown;
  
  /** Optional additional data */
  readonly data?: Record<string, unknown>;
  
  /**
   * Create a new TAP error
   * 
   * @param options - Error options
   */
  constructor(options: TapErrorOptions) {
    super(options.message);
    
    this.name = 'TapError';
    this.type = options.type;
    this.cause = options.cause;
    this.data = options.data;
    
    // Capture stack trace
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, TapError);
    }
  }
  
  /**
   * Convert the error to a string
   * 
   * @returns String representation of the error
   */
  toString(): string {
    let result = `[${this.name}] ${this.type}: ${this.message}`;
    
    if (this.cause) {
      result += `\nCaused by: ${this.cause}`;
    }
    
    return result;
  }
  
  /**
   * Convert the error to a plain object
   * 
   * @returns Plain object representation of the error
   */
  toJSON(): Record<string, unknown> {
    return {
      name: this.name,
      type: this.type,
      message: this.message,
      cause: this.cause,
      data: this.data,
      stack: this.stack,
    };
  }
}
