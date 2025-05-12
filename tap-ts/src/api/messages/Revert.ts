/**
 * Revert message class
 * Implements the Revert message type for TAP
 * This is a reply message requesting reversal of a settled transaction
 */

import { DIDCommMessageBase, MessageOptions } from './base';
import type {
  Revert as RevertBody,
  RevertMessage
} from '@taprsvp/types';
import { ValidationError } from '../../utils/errors';

/**
 * Revert message options
 * Extends the base message options with revert-specific fields
 */
export interface RevertMessageOptions extends MessageOptions {
  /** Required Thread ID for the message this replies to */
  thid: string;

  /** Settlement address for the revert */
  settlementAddress: string;

  /** Reason for the revert request */
  reason: string;
}

/**
 * Revert message implementation
 * Represents a Revert message in the TAP protocol
 */
export class Revert extends DIDCommMessageBase<any> implements RevertMessage {
  /** The message type URI for Revert messages */
  readonly type: "https://tap.rsvp/schema/1.0#Revert" = "https://tap.rsvp/schema/1.0#Revert";

  /** Thread ID linking this reply to the original message */
  readonly thid: string;

  // Required Revert interface properties
  readonly "@context": "https://tap.rsvp/schema/1.0" = "https://tap.rsvp/schema/1.0";
  readonly "@type": "Revert" = "Revert";
  settlementAddress: string;
  reason: string;

  /**
   * Create a new Revert message
   *
   * @param options Revert message options
   */
  constructor(options: RevertMessageOptions) {
    if (!options.thid) {
      throw new Error('Thread ID (thid) is required for Revert messages');
    }

    // Initialize with super
    super("https://tap.rsvp/schema/1.0#Revert", {}, options);

    // Set required properties
    this.thid = options.thid;
    this.settlementAddress = options.settlementAddress;
    this.reason = options.reason;
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
    if (!this.settlementAddress) {
      throw new ValidationError('Missing required field: settlementAddress', 'settlementAddress');
    }

    if (!this.reason) {
      throw new ValidationError('Missing required field: reason', 'reason');
    }

    if (!this.thid) {
      throw new ValidationError('Thread ID (thid) is required for Revert messages');
    }
  }
}