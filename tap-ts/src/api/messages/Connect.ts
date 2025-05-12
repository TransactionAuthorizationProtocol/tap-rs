/**
 * Connect message class
 * Implements the Connect message type for TAP
 */

import { DIDCommMessageBase, MessageOptions } from './base';
import type {
  Connect as ConnectBody,
  DID,
  Participant,
  TransactionConstraints,
  IRI,
  ISO8601DateTime
} from '@taprsvp/types';
import { ValidationError } from '../../utils/errors';

/**
 * Connect message options
 * Extends the base message options with connect-specific fields
 */
export interface ConnectMessageOptions extends MessageOptions {
  /** DID of the represented party */
  for: DID;

  /** Transaction constraints */
  constraints: TransactionConstraints;

  /** Optional agent details */
  agent?: Participant<"Agent"> & {
    /** Service URL */
    serviceUrl?: IRI;
  };

  /** Optional expiry timestamp */
  expiry?: ISO8601DateTime;
}

/**
 * Connect message implementation
 * Represents a Connect message in the TAP protocol
 */
export class Connect extends DIDCommMessageBase<any> {
  /** The message type URI for Connect messages */
  readonly type: "https://tap.rsvp/schema/1.0#Connect" = "https://tap.rsvp/schema/1.0#Connect";

  // Required properties
  readonly "@context": "https://tap.rsvp/schema/1.0" = "https://tap.rsvp/schema/1.0";
  readonly "@type": "Connect" = "Connect";
  for: DID;
  constraints: TransactionConstraints;

  // Optional properties
  agent?: Participant<"Agent"> & { serviceUrl?: IRI };
  expiry?: ISO8601DateTime;

  /**
   * Create a new Connect message
   *
   * @param options Connect message options
   */
  constructor(options: ConnectMessageOptions) {
    // Initialize with super
    super("https://tap.rsvp/schema/1.0#Connect", {}, options);

    // Set required properties
    this.for = options.for;
    this.constraints = options.constraints;

    // Add optional fields if provided
    if (options.agent) this.agent = options.agent;
    if (options.expiry) this.expiry = options.expiry;
  }

  /**
   * Validate the Connect message
   * Checks that all required fields are present and valid
   *
   * @throws ValidationError if the message is invalid
   */
  _validate(): void {
    // Call parent validation
    super._validate();

    // Validate connect-specific fields
    if (!this.for) {
      throw new ValidationError('Missing required field: for', 'for');
    }

    if (!this.for.startsWith('did:')) {
      throw new ValidationError('Invalid for format: must be a DID', 'for');
    }

    if (!this.constraints) {
      throw new ValidationError('Missing required field: constraints', 'constraints');
    }

    // Validate agent if present
    if (this.agent) {
      if (!this.agent['@id']) {
        throw new ValidationError('Missing required field: agent.@id', 'agent.@id');
      }

      if (!this.agent['@id'].startsWith('did:')) {
        throw new ValidationError('Invalid agent.@id format: must be a DID', 'agent.@id');
      }
    }
  }
}