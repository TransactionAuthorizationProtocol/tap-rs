/**
 * Connect message class
 * Implements the Connect message type for TAP
 */

import { DIDCommMessageBase, MessageOptions } from './base';
import { 
  Connect as ConnectBody, 
  ConnectMessage,
  DID,
  Participant,
  TransactionConstraints,
  IRI
} from '../../models/types';
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
  expiry?: string;
}

/**
 * Connect message implementation
 * Represents a Connect message in the TAP protocol
 */
export class Connect extends DIDCommMessageBase<ConnectBody> implements ConnectMessage {
  /** The message type URI for Connect messages */
  readonly type: "https://tap.rsvp/schema/1.0#Connect" = "https://tap.rsvp/schema/1.0#Connect";
  
  /**
   * Create a new Connect message
   * 
   * @param options Connect message options
   */
  constructor(options: ConnectMessageOptions) {
    // Create the message body
    const body: ConnectBody = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Connect",
      for: options.for,
      constraints: options.constraints
    };
    
    // Add optional fields if provided
    if (options.agent) body.agent = options.agent;
    if (options.expiry) body.expiry = options.expiry;
    
    // Call the parent constructor
    super("https://tap.rsvp/schema/1.0#Connect", body, options);
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
    if (!this.body.for) {
      throw new ValidationError('Missing required field: for', 'for');
    }
    
    if (!this.body.for.startsWith('did:')) {
      throw new ValidationError('Invalid for format: must be a DID', 'for');
    }
    
    if (!this.body.constraints) {
      throw new ValidationError('Missing required field: constraints', 'constraints');
    }
    
    // Validate agent if present
    if (this.body.agent) {
      if (!this.body.agent['@id']) {
        throw new ValidationError('Missing required field: agent.@id', 'agent.@id');
      }
      
      if (!this.body.agent['@id'].startsWith('did:')) {
        throw new ValidationError('Invalid agent.@id format: must be a DID', 'agent.@id');
      }
    }
  }
}