/**
 * Reject message class
 * Implements the Reject message type for TAP
 * This is a reply message to reject a transfer or payment request
 */

import { DIDCommMessageBase, MessageOptions } from './base';
import type {
  Reject as RejectBody,
  RejectMessage
} from '@taprsvp/types';
import { ValidationError } from '../../utils/errors';

/**
 * Reject message options
 * Extends the base message options with reject-specific fields
 */
export interface RejectMessageOptions extends MessageOptions {
  /** Required Thread ID for the message this replies to */
  thid: string;

  /** Reason for the rejection */
  reason: string;
}

/**
 * Reject message implementation
 * Represents a Reject message in the TAP protocol
 */
export class Reject extends DIDCommMessageBase<any> implements RejectMessage {
  /** The message type URI for Reject messages */
  readonly type: "https://tap.rsvp/schema/1.0#Reject" = "https://tap.rsvp/schema/1.0#Reject";

  /** Thread ID linking this reply to the original message */
  readonly thid: string;

  // Required properties for the Reject interface
  readonly "@context": "https://tap.rsvp/schema/1.0" = "https://tap.rsvp/schema/1.0";
  readonly "@type": "Reject" = "Reject";
  reason: string;
  transfer: { "@id": string };

  /**
   * Create a new Reject message
   *
   * @param options Reject message options
   */
  constructor(options: RejectMessageOptions) {
    if (!options.thid) {
      throw new Error('Thread ID (thid) is required for Reject messages');
    }

    // Initialize with super
    super("https://tap.rsvp/schema/1.0#Reject", {}, options);

    // Set required properties
    this.thid = options.thid;
    this.reason = options.reason;
    this.transfer = { "@id": options.thid };
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
    if (!this.reason) {
      throw new ValidationError('Missing required field: reason', 'reason');
    }

    if (!this.thid) {
      throw new ValidationError('Thread ID (thid) is required for Reject messages');
    }

    if (!this.transfer || !this.transfer["@id"]) {
      throw new ValidationError('Transfer reference is required');
    }
  }
}