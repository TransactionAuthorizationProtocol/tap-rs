/**
 * Payment message class
 * Implements the Payment message type for TAP
 */

import { DIDCommMessageBase, MessageOptions } from './base';
import { 
  Payment as PaymentBody, 
  PaymentMessage,
  Participant,
  CAIP19,
  IsoCurrency,
  Amount,
  Complete,
  Settle,
  Cancel,
  DID,
  CAIP10,
  Invoice
} from '../../models/types';
import { ValidationError } from '../../utils/errors';

/**
 * Payment message options
 * Extends the base message options with payment-specific fields
 */
export interface PaymentMessageOptions extends MessageOptions {
  /** The amount being requested */
  amount: Amount;
  
  /** The merchant requesting payment */
  merchant: Participant<"Party">;
  
  /** The agents involved in the payment */
  agents: Participant<"Agent">[];
  
  /** Optional specific asset requested */
  asset?: CAIP19;
  
  /** Optional currency code for fiat payment requests */
  currency?: IsoCurrency;
  
  /** Optional supported assets */
  supportedAssets?: CAIP19[];
  
  /** Optional invoice details */
  invoice?: Invoice | string;
  
  /** Optional customer details */
  customer?: Participant<"Party">;
  
  /** Optional expiry timestamp */
  expiry?: string;
}

/**
 * Payment message implementation
 * Represents a Payment message in the TAP protocol
 */
export class Payment extends DIDCommMessageBase<PaymentBody> implements PaymentMessage {
  /** The message type URI for Payment messages */
  readonly type: "https://tap.rsvp/schema/1.0#Payment" = "https://tap.rsvp/schema/1.0#Payment";
  
  /**
   * Create a new Payment message
   * 
   * @param options Payment message options
   */
  constructor(options: PaymentMessageOptions) {
    // Create the message body
    const body: PaymentBody = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Payment",
      amount: options.amount,
      merchant: options.merchant,
      agents: options.agents
    };
    
    // Add optional fields if provided
    if (options.asset) body.asset = options.asset;
    if (options.currency) body.currency = options.currency;
    if (options.supportedAssets) body.supportedAssets = options.supportedAssets;
    if (options.invoice) body.invoice = options.invoice;
    if (options.customer) body.customer = options.customer;
    if (options.expiry) body.expiry = options.expiry;
    
    // Call the parent constructor
    super("https://tap.rsvp/schema/1.0#Payment", body, options);
  }
  
  /**
   * Validate the Payment message
   * Checks that all required fields are present and valid
   * 
   * @throws ValidationError if the message is invalid
   */
  _validate(): void {
    // Call parent validation
    super._validate();
    
    // Validate payment-specific fields
    if (!this.body.amount) {
      throw new ValidationError('Missing required field: amount', 'amount');
    }
    
    if (!this.body.merchant) {
      throw new ValidationError('Missing required field: merchant', 'merchant');
    }
    
    if (!this.body.agents || !this.body.agents.length) {
      throw new ValidationError('Missing required field: agents', 'agents');
    }
    
    // Either asset or currency must be provided
    if (!this.body.asset && !this.body.currency) {
      throw new ValidationError('Either asset or currency must be provided', 'asset/currency');
    }
    
    // Validate amount format
    if (!/^(\d+|\d+\.\d+)$/.test(this.body.amount)) {
      throw new ValidationError('Invalid amount format', 'amount');
    }
  }
  
  /**
   * Create a complete message for this payment
   * Used by the merchant's agent to provide settlement instructions
   * 
   * @param settlementAddress The address where funds should be sent
   * @param amount Optional final amount (must be <= original amount)
   * @returns A new Complete message
   */
  complete(
    settlementAddress: CAIP10,
    amount?: Amount
  ): Complete {
    // Create the complete body
    const completeBody: Complete = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Complete",
      settlementAddress
    };
    
    // Add optional amount
    if (amount) {
      // Validate amount is not greater than original
      const originalAmount = parseFloat(this.body.amount);
      const finalAmount = parseFloat(amount);
      
      if (finalAmount > originalAmount) {
        throw new ValidationError(
          `Complete amount (${finalAmount}) cannot be greater than original amount (${originalAmount})`,
          'amount'
        );
      }
      
      completeBody.amount = amount;
    }
    
    // Create and return the message
    const message = new DIDCommMessageBase<Complete>(
      "https://tap.rsvp/schema/1.0#Complete",
      completeBody,
      { thid: this.id }
    );
    
    return message as any;
  }
  
  /**
   * Create a settlement message for this payment
   * 
   * @param settlementId The settlement transaction ID
   * @param amount Optional settled amount
   * @returns A new Settle message
   */
  settle(settlementId: string, amount?: Amount): Settle {
    // Create the settle body
    const settleBody: Settle = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Settle",
      settlementId
    };
    
    // Add optional amount
    if (amount) settleBody.amount = amount;
    
    // Create and return the message
    const message = new DIDCommMessageBase<Settle>(
      "https://tap.rsvp/schema/1.0#Settle",
      settleBody,
      { thid: this.id }
    );
    
    return message as any;
  }
  
  /**
   * Create a cancellation message for this payment
   * 
   * @param reason Optional reason for cancellation
   * @returns A new Cancel message
   */
  cancel(reason?: string): Cancel {
    // Create the cancel body
    const cancelBody: Cancel = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Cancel"
    };
    
    // Add optional reason
    if (reason) cancelBody.reason = reason;
    
    // Create and return the message
    const message = new DIDCommMessageBase<Cancel>(
      "https://tap.rsvp/schema/1.0#Cancel",
      cancelBody,
      { thid: this.id }
    );
    
    return message as any;
  }
}