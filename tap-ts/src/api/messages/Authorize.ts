/**
 * Authorize message class
 * Implements the Authorize message type for TAP
 * This is a reply message to a transfer for authorizing the transaction
 */

import { DIDCommMessageBase, MessageOptions } from './base';
import type {
  Authorize as AuthorizeBody,
  AuthorizeMessage,
  ISO8601DateTime,
  CAIP10
} from '@taprsvp/types';

/**
 * Authorize message options
 * Extends the base message options with authorize-specific fields
 */
export interface AuthorizeMessageOptions extends MessageOptions {
  /** Required thread ID for the message this replies to */
  thid: string;

  /** Optional settlement address */
  settlementAddress?: CAIP10;

  /** Optional reason for the authorization */
  reason?: string;

  /** Optional expiry timestamp */
  expiry?: ISO8601DateTime;
}

/**
 * Authorize message implementation
 * Represents an Authorize message in the TAP protocol
 */
export class Authorize extends DIDCommMessageBase<any> implements AuthorizeMessage {
  /** The message type URI for Authorize messages */
  readonly type: "https://tap.rsvp/schema/1.0#Authorize" = "https://tap.rsvp/schema/1.0#Authorize";

  /** Thread ID linking to the original message */
  readonly thid: string;

  // Required Authorize interface properties
  readonly "@context": "https://tap.rsvp/schema/1.0" = "https://tap.rsvp/schema/1.0";
  readonly "@type": "Authorize" = "Authorize";
  transfer: { "@id": string };

  // Optional Authorize interface properties
  reason?: string;
  settlementAddress?: CAIP10;
  expiry?: ISO8601DateTime;

  /**
   * Create a new Authorize message
   *
   * @param options Authorize message options
   */
  constructor(options: AuthorizeMessageOptions) {
    if (!options.thid) {
      throw new Error('Thread ID (thid) is required for Authorize messages');
    }

    // Initialize with super
    super("https://tap.rsvp/schema/1.0#Authorize", {}, options);

    // Set required properties
    this.thid = options.thid;
    this.transfer = { "@id": options.thid };

    // Set optional properties if provided
    if (options.settlementAddress) this.settlementAddress = options.settlementAddress;
    if (options.reason) this.reason = options.reason;
    if (options.expiry) this.expiry = options.expiry;
  }

  /**
   * Validate the Authorize message
   * Checks that all required fields are present and valid
   *
   * @throws ValidationError if the message is invalid
   */
  _validate(): void {
    // Call parent validation
    super._validate();

    // Authorize-specific validation
    if (!this.thid) {
      throw new Error('Thread ID (thid) is required for Authorize messages');
    }

    if (!this.transfer || !this.transfer["@id"]) {
      throw new Error('Transfer reference is required');
    }
  }
}