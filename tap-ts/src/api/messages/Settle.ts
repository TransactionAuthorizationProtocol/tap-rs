/**
 * Settle message class
 * Implements the Settle message type for TAP
 * This is a reply message to confirm on-chain settlement
 */

import { DIDCommMessageBase, MessageOptions } from './base';
import {
  Settle as SettleBody,
  SettleMessage,
  CAIP220,
  Amount
} from '../../models/types';
import { ValidationError } from '../../utils/errors';

/**
 * Settle message options
 * Extends the base message options with settle-specific fields
 */
export interface SettleMessageOptions extends MessageOptions {
  /** Settlement transaction identifier */
  settlementId: CAIP220;
  
  /** Optional settled amount */
  amount?: Amount;
}

/**
 * Settle message implementation
 * Represents a Settle message in the TAP protocol
 */
export class Settle extends DIDCommMessageBase<SettleBody> implements SettleMessage {
  /** The message type URI for Settle messages */
  readonly type: "https://tap.rsvp/schema/1.0#Settle" = "https://tap.rsvp/schema/1.0#Settle";
  
  /**
   * Create a new Settle message
   * 
   * @param options Settle message options
   */
  constructor(options: SettleMessageOptions) {
    // Create the message body
    const body: SettleBody = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Settle",
      settlementId: options.settlementId
    };
    
    // Add optional amount if provided
    if (options.amount) {
      body.amount = options.amount;
    }
    
    // Call the parent constructor
    super("https://tap.rsvp/schema/1.0#Settle", body, options);
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
    if (!this.body.settlementId) {
      throw new ValidationError('Missing required field: settlementId', 'settlementId');
    }
    
    // Validate amount format if provided
    if (this.body.amount && !/^(\d+|\d+\.\d+)$/.test(this.body.amount)) {
      throw new ValidationError('Invalid amount format', 'amount');
    }
  }
}