/**
 * Transfer message class
 * Implements the Transfer message type for TAP
 */

import { DIDCommMessageBase, MessageOptions } from './base';
import { 
  Transfer as TransferBody, 
  TransferMessage,
  Participant,
  Asset,
  Amount,
  DID,
  CAIP10,
  Authorize,
  Reject,
  Settle,
  Cancel,
  Revert
} from '../../models/types';
import { ValidationError } from '../../utils/errors';

/**
 * Transfer message options
 * Extends the base message options with transfer-specific fields
 */
export interface TransferMessageOptions extends MessageOptions {
  /** The asset being transferred */
  asset: Asset;
  
  /** The amount being transferred */
  amount: Amount;
  
  /** The originator of the transfer */
  originator: Participant<"Party">;
  
  /** The beneficiary of the transfer (optional) */
  beneficiary?: Participant<"Party">;
  
  /** The agents involved in the transfer */
  agents: Participant<"Agent">[];
  
  /** Optional settlement ID for the transaction */
  settlementId?: string;
  
  /** Optional memo field */
  memo?: string;
  
  /** Optional purpose code */
  purpose?: string;
  
  /** Optional category purpose code */
  categoryPurpose?: string;
  
  /** Optional expiry timestamp */
  expiry?: string;
}

/**
 * Transfer message implementation
 * Represents a Transfer message in the TAP protocol
 */
export class Transfer extends DIDCommMessageBase<TransferBody> implements TransferMessage {
  /** The message type URI for Transfer messages */
  readonly type: "https://tap.rsvp/schema/1.0#Transfer" = "https://tap.rsvp/schema/1.0#Transfer";
  
  /**
   * Create a new Transfer message
   * 
   * @param options Transfer message options
   */
  constructor(options: TransferMessageOptions) {
    // Create the message body
    const body: TransferBody = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Transfer",
      asset: options.asset,
      amount: options.amount,
      originator: options.originator,
      agents: options.agents
    };
    
    // Add optional fields if provided
    if (options.beneficiary) body.beneficiary = options.beneficiary;
    if (options.settlementId) body.settlementId = options.settlementId;
    if (options.memo) body.memo = options.memo;
    if (options.purpose) body.purpose = options.purpose;
    if (options.categoryPurpose) body.categoryPurpose = options.categoryPurpose;
    if (options.expiry) body.expiry = options.expiry;
    
    // Call the parent constructor
    super("https://tap.rsvp/schema/1.0#Transfer", body, options);
  }
  
  /**
   * Validate the Transfer message
   * Checks that all required fields are present and valid
   * 
   * @throws ValidationError if the message is invalid
   */
  _validate(): void {
    // Call parent validation
    super._validate();
    
    // Validate transfer-specific fields
    if (!this.body.asset) {
      throw new ValidationError('Missing required field: asset', 'asset');
    }
    
    if (!this.body.amount) {
      throw new ValidationError('Missing required field: amount', 'amount');
    }
    
    if (!this.body.originator) {
      throw new ValidationError('Missing required field: originator', 'originator');
    }
    
    if (!this.body.agents || !this.body.agents.length) {
      throw new ValidationError('Missing required field: agents', 'agents');
    }
    
    // Validate amount format
    if (!/^(\d+|\d+\.\d+)$/.test(this.body.amount)) {
      throw new ValidationError('Invalid amount format', 'amount');
    }
  }
  
  /**
   * Create an authorization message for this transfer
   * 
   * @param settlementAddress Optional settlement address
   * @param reason Optional reason for authorization
   * @param expiryInSeconds Optional expiration time in seconds from now
   * @returns A new Authorize message
   */
  authorize(
    settlementAddress?: CAIP10,
    reason?: string,
    expiryInSeconds?: number
  ): Authorize {
    // Create the authorize body
    const authorizeBody: Authorize = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Authorize"
    };
    
    // Add optional fields
    if (settlementAddress) authorizeBody.settlementAddress = settlementAddress;
    if (reason) authorizeBody.reason = reason;
    if (expiryInSeconds) {
      // Convert to ISO string
      const expiry = new Date(Date.now() + expiryInSeconds * 1000).toISOString();
      authorizeBody.expiry = expiry;
    }
    
    // Create and return the message
    const message = new DIDCommMessageBase<Authorize>(
      "https://tap.rsvp/schema/1.0#Authorize",
      authorizeBody,
      { thid: this.id }
    );
    
    return message as any;
  }
  
  /**
   * Create a rejection message for this transfer
   * 
   * @param reason The reason for rejection
   * @returns A new Reject message
   */
  reject(reason: string): Reject {
    // Create the reject body
    const rejectBody: Reject = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Reject",
      reason
    };
    
    // Create and return the message
    const message = new DIDCommMessageBase<Reject>(
      "https://tap.rsvp/schema/1.0#Reject",
      rejectBody,
      { thid: this.id }
    );
    
    return message as any;
  }
  
  /**
   * Create a settlement message for this transfer
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
   * Create a cancellation message for this transfer
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
  
  /**
   * Create a revert message for this transfer
   * 
   * @param options Revert options
   * @param options.settlementAddress The address to return funds to
   * @param options.reason The reason for the revert
   * @returns A new Revert message
   */
  revert(options: { settlementAddress: string; reason: string }): Revert {
    // Create the revert body
    const revertBody: Revert = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Revert",
      settlementAddress: options.settlementAddress,
      reason: options.reason
    };
    
    // Create and return the message
    const message = new DIDCommMessageBase<Revert>(
      "https://tap.rsvp/schema/1.0#Revert",
      revertBody,
      { thid: this.id }
    );
    
    return message as any;
  }
}