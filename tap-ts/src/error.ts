/**
 * Error handling for TAP-TS
 * 
 * This module provides error types and classes for TAP-TS.
 */

/**
 * TAP Error Types
 */
export enum ErrorType {
  UNKNOWN = "unknown",
  AGENT_NOT_FOUND = "agent_not_found",
  AGENT_ALREADY_EXISTS = "agent_already_exists",
  AGENT_ALREADY_REGISTERED = "agent_already_registered",
  INVALID_MESSAGE_TYPE = "invalid_message_type",
  INVALID_MESSAGE = "message_invalid",
  MESSAGE_INVALID = "message_invalid",
  INVALID_ARGUMENT = "invalid_argument",
  INVALID_DID = "did_invalid",
  DID_RESOLUTION_ERROR = "did_resolution_error",
  INTERNAL_ERROR = "internal_error",
  NOT_IMPLEMENTED = "not_implemented",
  NOT_SUPPORTED = "not_supported",
  VALIDATION_ERROR = "validation_error",
  WASM_ERROR = "wasm_error",
  WASM_NOT_LOADED = "wasm_not_loaded",
  WASM_LOAD_ERROR = "wasm_load_error",
  WASM_INIT_ERROR = "wasm_init_error",
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
  override readonly cause?: unknown;
  
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
  override toString(): string {
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
