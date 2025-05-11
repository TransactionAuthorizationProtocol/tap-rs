/**
 * Revert message class
 * Implements the Revert message type for TAP
 * This is a reply message requesting reversal of a settled transaction
 */

import { DIDCommMessageBase, MessageOptions } from './base';
import {
  Revert as RevertBody,
  RevertMessage
} from '../../models/types';
import { ValidationError } from '../../utils/errors';

/**
 * Revert message options
 * Extends the base message options with revert-specific fields
 */
export interface RevertMessageOptions extends MessageOptions {
  /** Settlement address for the revert */
  settlementAddress: string;
  
  /** Reason for the revert request */
  reason: string;
}

/**
 * Revert message implementation
 * Represents a Revert message in the TAP protocol
 */
export class Revert extends DIDCommMessageBase<RevertBody> implements RevertMessage {
  /** The message type URI for Revert messages */
  readonly type: "https://tap.rsvp/schema/1.0#Revert" = "https://tap.rsvp/schema/1.0#Revert";
  
  /**
   * Create a new Revert message
   * 
   * @param options Revert message options
   */
  constructor(options: RevertMessageOptions) {
    // Create the message body
    const body: RevertBody = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Revert",
      settlementAddress: options.settlementAddress,
      reason: options.reason
    };
    
    // Call the parent constructor
    super("https://tap.rsvp/schema/1.0#Revert", body, options);
  }
  
  /**
   * Validate the Revert message
   * Checks that all required fields are present and valid
   * 
   * @throws ValidationError if the message is invalid
   */
  _validate(): void {
    // Call parent validation
    super._validate();
    
    // Validate revert-specific fields
    if (!this.body.settlementAddress) {
      throw new ValidationError('Missing required field: settlementAddress', 'settlementAddress');
    }
    
    if (!this.body.reason) {
      throw new ValidationError('Missing required field: reason', 'reason');
    }
  }
}