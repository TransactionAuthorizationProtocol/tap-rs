/**
 * Settle message class
 * Implements the Settle message type for TAP
 * This is a reply message to confirm on-chain settlement
 */

import { DIDCommMessageBase, MessageOptions } from './base';
import type {
  Settle as SettleBody,
  SettleMessage,
  CAIP220,
  Amount
} from '@taprsvp/types';
import { ValidationError } from '../../utils/errors';

/**
 * Settle message options
 * Extends the base message options with settle-specific fields
 */
export interface SettleMessageOptions extends MessageOptions {
  /** Required Thread ID for the message this replies to */
  thid: string;

  /** Settlement transaction identifier */
  settlementId: CAIP220;

  /** Optional settled amount */
  amount?: Amount;
}

/**
 * Settle message implementation
 * Represents a Settle message in the TAP protocol
 */
export class Settle extends DIDCommMessageBase<any> implements SettleMessage {
  /** The message type URI for Settle messages */
  readonly type: "https://tap.rsvp/schema/1.0#Settle" = "https://tap.rsvp/schema/1.0#Settle";

  /** Thread ID linking this reply to the original message */
  readonly thid: string;

  // Required Settle interface properties
  readonly "@context": "https://tap.rsvp/schema/1.0" = "https://tap.rsvp/schema/1.0";
  readonly "@type": "Settle" = "Settle";
  settlementId: CAIP220;
  transfer: { "@id": string };

  // Optional Settle interface properties
  amount?: Amount;

  /**
   * Create a new Settle message
   *
   * @param options Settle message options
   */
  constructor(options: SettleMessageOptions) {
    if (!options.thid) {
      throw new Error('Thread ID (thid) is required for Settle messages');
    }

    // Initialize with super
    super("https://tap.rsvp/schema/1.0#Settle", {}, options);

    // Set required properties
    this.thid = options.thid;
    this.settlementId = options.settlementId;
    this.transfer = { "@id": options.thid };

    // Set optional amount if provided
    if (options.amount) {
      this.amount = options.amount;
    }
  }

  /**
   * Validate the Settle message
   * Checks that all required fields are present and valid
   *
   * @throws ValidationError if the message is invalid
   */
  _validate(): void {
    // Call parent validation
    super._validate();

    // Validate settle-specific fields
    if (!this.settlementId) {
      throw new ValidationError('Missing required field: settlementId', 'settlementId');
    }

    if (!this.thid) {
      throw new ValidationError('Thread ID (thid) is required for Settle messages');
    }

    if (!this.transfer || !this.transfer["@id"]) {
      throw new ValidationError('Transfer reference is required');
    }

    // Validate amount format if provided
    if (this.amount && !/^(\d+|\d+\.\d+)$/.test(this.amount)) {
      throw new ValidationError('Invalid amount format', 'amount');
    }
  }
}