/**
 * Cancel message class
 * Implements the Cancel message type for TAP
 * This is a reply message to cancel a transaction or connection
 */

import { DIDCommMessageBase, MessageOptions } from './base';
import type {
  Cancel as CancelBody,
  CancelMessage
} from '@taprsvp/types';
import { ValidationError } from '../../utils/errors';

/**
 * Cancel message options
 * Extends the base message options with cancel-specific fields
 */
export interface CancelMessageOptions extends MessageOptions {
  /** Required Thread ID for the message this replies to */
  thid: string;

  /** Optional reason for the cancellation */
  reason?: string;
}

/**
 * Cancel message implementation
 * Represents a Cancel message in the TAP protocol
 */
export class Cancel extends DIDCommMessageBase<any> implements CancelMessage {
  /** The message type URI for Cancel messages */
  readonly type: "https://tap.rsvp/schema/1.0#Cancel" = "https://tap.rsvp/schema/1.0#Cancel";

  /** Thread ID linking this reply to the original message */
  readonly thid: string;

  // Required properties for the Cancel interface
  readonly "@context": "https://tap.rsvp/schema/1.0" = "https://tap.rsvp/schema/1.0";
  readonly "@type": "Cancel" = "Cancel";

  // Optional properties
  reason?: string;

  /**
   * Create a new Cancel message
   *
   * @param options Cancel message options
   */
  constructor(options: CancelMessageOptions) {
    if (!options.thid) {
      throw new Error('Thread ID (thid) is required for Cancel messages');
    }

    // Initialize with super
    super("https://tap.rsvp/schema/1.0#Cancel", {}, options);

    // Set required properties
    this.thid = options.thid;

    // Set optional properties
    if (options.reason) {
      this.reason = options.reason;
    }
  }

  /**
   * Validate the Cancel message
   * Checks that all required fields are present and valid
   *
   * @throws ValidationError if the message is invalid
   */
  _validate(): void {
    // Call parent validation
    super._validate();

    // Cancel-specific validation
    if (!this.thid) {
      throw new ValidationError('Thread ID (thid) is required for Cancel messages');
    }
  }
}