/**
 * Authorize message class
 * Implements the Authorize message type for TAP
 * This is a reply message to a transfer for authorizing the transaction
 */

import { DIDCommMessageBase, MessageOptions } from './base';
import {
  Authorize as AuthorizeBody,
  AuthorizeMessage,
  CAIP10
} from '../../models/types';

/**
 * Authorize message options
 * Extends the base message options with authorize-specific fields
 */
export interface AuthorizeMessageOptions extends MessageOptions {
  /** Optional settlement address */
  settlementAddress?: CAIP10;
  
  /** Optional reason for the authorization */
  reason?: string;
  
  /** Optional expiry timestamp */
  expiry?: string;
}

/**
 * Authorize message implementation
 * Represents an Authorize message in the TAP protocol
 */
export class Authorize extends DIDCommMessageBase<AuthorizeBody> implements AuthorizeMessage {
  /** The message type URI for Authorize messages */
  readonly type: "https://tap.rsvp/schema/1.0#Authorize" = "https://tap.rsvp/schema/1.0#Authorize";
  
  /**
   * Create a new Authorize message
   * 
   * @param options Authorize message options
   */
  constructor(options: AuthorizeMessageOptions = {}) {
    // Create the message body
    const body: AuthorizeBody = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Authorize"
    };
    
    // Add optional fields if provided
    if (options.settlementAddress) body.settlementAddress = options.settlementAddress;
    if (options.reason) body.reason = options.reason;
    if (options.expiry) body.expiry = options.expiry;
    
    // Call the parent constructor
    super("https://tap.rsvp/schema/1.0#Authorize", body, options);
  }
}