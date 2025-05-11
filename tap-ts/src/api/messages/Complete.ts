/**
 * Complete message class
 * Implements the Complete message type for TAP
 * This is a reply message to a payment for providing settlement instructions
 */

import { DIDCommMessageBase, MessageOptions } from './base';
import {
  Complete as CompleteBody,
  CompleteMessage,
  CAIP10,
  Amount
} from '../../models/types';
import { ValidationError } from '../../utils/errors';

/**
 * Complete message options
 * Extends the base message options with complete-specific fields
 */
export interface CompleteMessageOptions extends MessageOptions {
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
export class Complete extends DIDCommMessageBase<CompleteBody> implements CompleteMessage {
  /** The message type URI for Complete messages */
  readonly type: "https://tap.rsvp/schema/1.0#Complete" = "https://tap.rsvp/schema/1.0#Complete";
  
  /** Original payment amount for validation */
  private originalAmount?: Amount;
  
  /**
   * Create a new Complete message
   * 
   * @param options Complete message options
   */
  constructor(options: CompleteMessageOptions) {
    // Create the message body
    const body: CompleteBody = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Complete",
      settlementAddress: options.settlementAddress
    };
    
    // Add optional amount if provided
    if (options.amount) {
      body.amount = options.amount;
      
      // Store original amount for validation
      if (options.originalAmount) {
        this.originalAmount = options.originalAmount;
      }
    }
    
    // Call the parent constructor
    super("https://tap.rsvp/schema/1.0#Complete", body, options);
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
    if (!this.body.settlementAddress) {
      throw new ValidationError('Missing required field: settlementAddress', 'settlementAddress');
    }
    
    // Validate amount if provided
    if (this.body.amount) {
      // Validate amount format
      if (!/^(\d+|\d+\.\d+)$/.test(this.body.amount)) {
        throw new ValidationError('Invalid amount format', 'amount');
      }
      
      // Validate against original amount if available
      if (this.originalAmount) {
        const finalAmount = parseFloat(this.body.amount);
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