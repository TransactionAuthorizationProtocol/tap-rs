/**
 * Custom error classes for TAP SDK
 */

/**
 * Base error class for all TAP-related errors
 */
export class TapError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'TapError';
    
    // This is needed for proper prototype chain in transpiled ES5
    Object.setPrototypeOf(this, TapError.prototype);
  }
}

/**
 * Error thrown when message validation fails
 */
export class ValidationError extends TapError {
  public field?: string;
  public code?: string;
  
  constructor(message: string, field?: string, code?: string) {
    super(message);
    this.name = 'ValidationError';
    this.field = field;
    this.code = code;
    
    Object.setPrototypeOf(this, ValidationError.prototype);
  }
}

/**
 * Error thrown when initialization fails
 */
export class InitializationError extends TapError {
  constructor(message: string) {
    super(message);
    this.name = 'InitializationError';
    
    Object.setPrototypeOf(this, InitializationError.prototype);
  }
}

/**
 * Error thrown when cryptographic operations fail
 */
export class CryptoError extends TapError {
  constructor(message: string) {
    super(message);
    this.name = 'CryptoError';
    
    Object.setPrototypeOf(this, CryptoError.prototype);
  }
}

/**
 * Error thrown when signature verification fails
 */
export class VerificationError extends CryptoError {
  constructor(message: string = 'Signature verification failed') {
    super(message);
    this.name = 'VerificationError';
    
    Object.setPrototypeOf(this, VerificationError.prototype);
  }
}

/**
 * Error thrown when DID resolution fails
 */
export class DIDResolutionError extends TapError {
  public did: string;
  
  constructor(did: string, message: string = 'DID resolution failed') {
    super(`${message}: ${did}`);
    this.name = 'DIDResolutionError';
    this.did = did;
    
    Object.setPrototypeOf(this, DIDResolutionError.prototype);
  }
}

/**
 * Error thrown when thread operations fail
 */
export class ThreadError extends TapError {
  public threadId?: string;
  
  constructor(message: string, threadId?: string) {
    super(threadId ? `${message} (thread: ${threadId})` : message);
    this.name = 'ThreadError';
    this.threadId = threadId;
    
    Object.setPrototypeOf(this, ThreadError.prototype);
  }
}

/**
 * Error thrown when the WASM bridge fails
 */
export class WasmBridgeError extends TapError {
  constructor(message: string) {
    super(message);
    this.name = 'WasmBridgeError';
    
    Object.setPrototypeOf(this, WasmBridgeError.prototype);
  }
}