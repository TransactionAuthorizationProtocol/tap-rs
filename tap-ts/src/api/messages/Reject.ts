/**
 * Reject message class
 * Implements the Reject message type for TAP
 * This is a reply message to reject a transfer or payment request
 */

import { DIDCommMessageBase, MessageOptions } from './base';
import {
  Reject as RejectBody,
  RejectMessage
} from '../../models/types';
import { ValidationError } from '../../utils/errors';

/**
 * Reject message options
 * Extends the base message options with reject-specific fields
 */
export interface RejectMessageOptions extends MessageOptions {
  /** Reason for the rejection */
  reason: string;
}

/**
 * Reject message implementation
 * Represents a Reject message in the TAP protocol
 */
export class Reject extends DIDCommMessageBase<RejectBody> implements RejectMessage {
  /** The message type URI for Reject messages */
  readonly type: "https://tap.rsvp/schema/1.0#Reject" = "https://tap.rsvp/schema/1.0#Reject";
  
  /**
   * Create a new Reject message
   * 
   * @param options Reject message options
   */
  constructor(options: RejectMessageOptions) {
    // Create the message body
    const body: RejectBody = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Reject",
      reason: options.reason
    };
    
    // Call the parent constructor
    super("https://tap.rsvp/schema/1.0#Reject", body, options);
  }
  
  /**
   * Validate the Reject message
   * Checks that all required fields are present and valid
   * 
   * @throws ValidationError if the message is invalid
   */
  _validate(): void {
    // Call parent validation
    super._validate();
    
    // Validate reject-specific fields
    if (!this.body.reason) {
      throw new ValidationError('Missing required field: reason', 'reason');
    }
  }
}