/**
 * Complete message class
 * Implements the Complete message type for TAP
 * This is a reply message to a payment for providing settlement instructions
 */

import { DIDCommMessageBase, MessageOptions } from './base';
import type {
  Complete as CompleteBody,
  CAIP10,
  Amount
} from '@taprsvp/types';
import { ValidationError } from '../../utils/errors';

/**
 * Complete message options
 * Extends the base message options with complete-specific fields
 */
export interface CompleteMessageOptions extends MessageOptions {
  /** Required thread ID for the message this replies to */
  thid: string;

  /** Settlement address where funds should be sent */
  settlementAddress: CAIP10;

  /** Optional final payment amount */
  amount?: Amount;

  /** Original payment amount for validation */
  originalAmount?: Amount;
}

/**
 * Complete message implementation
 * Represents a Complete message in the TAP protocol
 */
export class Complete extends DIDCommMessageBase<any> {
  /** The message type URI for Complete messages */
  readonly type: "https://tap.rsvp/schema/1.0#Complete" = "https://tap.rsvp/schema/1.0#Complete";

  /** Thread ID linking this reply to the original message */
  readonly thid: string;

  // Required Complete interface properties
  readonly "@context": "https://tap.rsvp/schema/1.0" = "https://tap.rsvp/schema/1.0";
  readonly "@type": "Complete" = "Complete";
  settlementAddress: CAIP10;

  // Optional Complete interface properties
  amount?: Amount;

  /** Original payment amount for validation */
  private originalAmount?: Amount;

  /**
   * Create a new Complete message
   *
   * @param options Complete message options
   */
  constructor(options: CompleteMessageOptions) {
    if (!options.thid) {
      throw new Error('Thread ID (thid) is required for Complete messages');
    }

    // Initialize with super
    super("https://tap.rsvp/schema/1.0#Complete", {}, options);

    // Set required properties
    this.thid = options.thid;
    this.settlementAddress = options.settlementAddress;

    // Set optional properties
    if (options.amount) {
      this.amount = options.amount;
    }

    // Store original amount for validation
    if (options.amount && options.originalAmount) {
      this.originalAmount = options.originalAmount;
    }
  }

  /**
   * Validate the Complete message
   * Checks that all required fields are present and valid
   *
   * @throws ValidationError if the message is invalid
   */
  _validate(): void {
    // Call parent validation
    super._validate();

    // Validate complete-specific fields
    if (!this.thid) {
      throw new ValidationError('Thread ID (thid) is required for Complete messages');
    }

    if (!this.settlementAddress) {
      throw new ValidationError('Missing required field: settlementAddress', 'settlementAddress');
    }

    // Validate amount if provided
    if (this.amount) {
      // Validate amount format
      if (!/^(\d+|\d+\.\d+)$/.test(this.amount)) {
        throw new ValidationError('Invalid amount format', 'amount');
      }

      // Validate against original amount if available
      if (this.originalAmount) {
        const finalAmount = parseFloat(this.amount);
        const origAmount = parseFloat(this.originalAmount);

        if (finalAmount > origAmount) {
          throw new ValidationError(
            `Complete amount (${finalAmount}) cannot be greater than original amount (${origAmount})`,
            'amount'
          );
        }
      }
    }
  }
}