/**
 * Base error class for TAP errors
 */
export class TAPError extends Error {
  constructor(message: string, public code: string) {
    super(message);
    this.name = 'TAPError';
  }
}

/**
 * Error related to message signing operations
 */
export class SigningError extends TAPError {
  constructor(message: string) {
    super(message, 'SIGNING_ERROR');
    this.name = 'SigningError';
  }
}

/**
 * Error related to message validation
 */
export class ValidationError extends TAPError {
  constructor(message: string) {
    super(message, 'VALIDATION_ERROR');
    this.name = 'ValidationError';
  }
}

/**
 * Error related to network operations
 */
export class NetworkError extends TAPError {
  constructor(message: string) {
    super(message, 'NETWORK_ERROR');
    this.name = 'NetworkError';
  }
}

/**
 * Error related to configuration issues
 */
export class ConfigurationError extends TAPError {
  constructor(message: string) {
    super(message, 'CONFIGURATION_ERROR');
    this.name = 'ConfigurationError';
  }
}

/**
 * Error related to message processing
 */
export class ProcessingError extends TAPError {
  constructor(message: string) {
    super(message, 'PROCESSING_ERROR');
    this.name = 'ProcessingError';
  }
}