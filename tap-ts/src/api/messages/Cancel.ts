/**
 * Cancel message class
 * Implements the Cancel message type for TAP
 * This is a reply message to cancel a transaction or connection
 */

import { DIDCommMessageBase, MessageOptions } from './base';
import {
  Cancel as CancelBody,
  CancelMessage
} from '../../models/types';

/**
 * Cancel message options
 * Extends the base message options with cancel-specific fields
 */
export interface CancelMessageOptions extends MessageOptions {
  /** Optional reason for the cancellation */
  reason?: string;
}

/**
 * Cancel message implementation
 * Represents a Cancel message in the TAP protocol
 */
export class Cancel extends DIDCommMessageBase<CancelBody> implements CancelMessage {
  /** The message type URI for Cancel messages */
  readonly type: "https://tap.rsvp/schema/1.0#Cancel" = "https://tap.rsvp/schema/1.0#Cancel";
  
  /**
   * Create a new Cancel message
   * 
   * @param options Cancel message options
   */
  constructor(options: CancelMessageOptions = {}) {
    // Create the message body
    const body: CancelBody = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Cancel"
    };
    
    // Add optional reason if provided
    if (options.reason) {
      body.reason = options.reason;
    }
    
    // Call the parent constructor
    super("https://tap.rsvp/schema/1.0#Cancel", body, options);
  }
}