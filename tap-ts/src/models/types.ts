/**
 * Core Type Definitions for TAP
 * 
 * These types match the definitions in @prds/taips/packages/typescript/src/tap.ts
 * and are used throughout the TAP SDK.
 */

// Temporarily import from local file to make tests work
import { Purpose, CategoryPurpose } from "../temp";
// We'll need to implement our own currency types
export type IsoCurrency = string;
// We'll need to implement our own invoice types
export interface Invoice {
  id: string;
  // Additional invoice fields will go here
}

/**
 * Internationalized Resource Identifier (IRI)
 * A unique identifier that may contain international characters.
 * Used for identifying resources, particularly in JSON-LD contexts.
 */
export type IRI = `${string}:${string}`;

/**
 * Decentralized Identifier (DID)
 * A globally unique persistent identifier that doesn't require a centralized registration authority.
 *
 * Format: `did:method:method-specific-id`
 */
export type DID = `did:${string}:${string}`;

/**
 * TAP Context URI
 * Base URI for TAP schema version 1.0.
 * Used as the default context for all TAP messages.
 */
export type TAPContext = "https://tap.rsvp/schema/1.0";

/**
 * TAP Type URI
 * Fully qualified type identifier for TAP message types.
 * Combines the TAP context with a type-specific fragment.
 */
export type TAPType = `${TAPContext}#${string}`;

/**
 * Base interface for JSON-LD objects
 * Provides the core structure for JSON-LD compatible objects with type information.
 */
export interface JsonLdObject<T extends string> {
  "@context"?: IRI | Record<string, string>;
  "@type": T;
}

/**
 * Base interface for TAP message objects
 * Extends JsonLdObject with TAP-specific context and type requirements.
 */
export interface TapMessageObject<T extends string> extends JsonLdObject<T> {
  "@context": TAPContext | Record<string, IRI>;
  "@type": T;
}

/**
 * ISO 8601 DateTime string
 * Represents date and time in a standardized format.
 */
export type ISO8601DateTime = string;

/**
 * Chain Agnostic Blockchain Identifier (CAIP-2)
 * Represents a blockchain in a chain-agnostic way following the CAIP-2 specification.
 * The identifier consists of a namespace and reference separated by a colon.
 */
export type CAIP2 = `${string}:${string}`;

/**
 * Chain Agnostic Account Identifier (CAIP-10)
 * Represents an account/address on a specific blockchain following the CAIP-10 specification.
 * Extends CAIP-2 by adding the account address specific to that chain.
 */
export type CAIP10 = `${CAIP2}:${string}`;

/**
 * Chain Agnostic Asset Identifier (CAIP-19)
 * Represents an asset/token on a specific blockchain following the CAIP-19 specification.
 * Extends CAIP-2 by adding asset type and identifier information.
 */
export type CAIP19 = `${CAIP2}/${string}:${string}`;

/**
 * Digital Trust Identifier (DTI)
 * A standardized identifier for digital assets in traditional finance.
 */
export type DTI = string;

/**
 * Asset Identifier
 * Union type representing either a blockchain-based asset (CAIP-19) or a traditional finance asset (DTI).
 * Used to identify assets in a chain-agnostic way across different financial systems.
 */
export type Asset = CAIP19 | DTI;

/**
 * Decimal Amount
 * String representation of a decimal number.
 * Must be either a whole number or a decimal number with a period separator.
 */
export type Amount = `${number}.${number}` | `${number}`;

/**
 * Chain Agnostic Transaction Identifier (CAIP-220)
 * Represents a transaction on a specific blockchain in a chain-agnostic way.
 */
export type CAIP220 = string;

/**
 * Legal Entity Identifier (LEI)
 * A 20-character alphanumeric code that uniquely identifies legal entities globally.
 */
export type LEICode = string;

/**
 * ISO 20022 External Purpose Code
 * Standardized code indicating the purpose of a financial transaction.
 */
export type ISO20022PurposeCode = Purpose;

/**
 * ISO 20022 External Category Purpose Code
 * High-level classification of the purpose of a financial transaction.
 */
export type ISO20022CategoryPurposeCode = CategoryPurpose;

/**
 * Common DIDComm Message Structure
 * Base interface for all DIDComm messages in TAP.
 */
export interface DIDCommMessage<T = Record<string, unknown>> {
  /** Unique identifier for the message */
  id: string;

  /** Message type URI that identifies the message type and version */
  type: string;

  /** DID of the sender of the message */
  from: DID;

  /** Array of DIDs of the intended recipients */
  to: DID[];

  /** Optional thread ID to link related messages together */
  thid?: string;

  /** Optional parent thread ID for nested threads */
  pthid?: string;

  /** Unix timestamp when the message was created */
  created_time: number;

  /** Optional Unix timestamp when the message expires */
  expires_time?: number;

  /** Message body containing type-specific content */
  body: T;
}

/**
 * DIDComm reply message structure
 * Extends DIDComm message with required thread ID for responses.
 */
export interface DIDCommReply<T = Record<string, unknown>> extends DIDCommMessage<T> {
  /** Thread ID linking this reply to the original message */
  thid: string;
}

/**
 * Participant type enumeration
 */
export type ParticipantTypes = "Agent" | "Party";

/**
 * Participant in a TAP transaction
 * Represents either a party (originator/beneficiary) or an agent in the transaction.
 * Can include verification methods and policies.
 */
export interface Participant<T extends ParticipantTypes> extends JsonLdObject<T> {
  /**
   * Unique identifier for the participant
   * Can be either a DID or an IRI
   */
  "@id": DID | IRI;

  "@type": T;

  /**
   * Legal Entity Identifier code
   * Used to uniquely identify legal entities involved in financial transactions
   */
  "lei:leiCode"?: LEICode;

  /**
   * Human-readable name of the participant
   * Optional to support privacy requirements
   */
  name?: string;

  /**
   * SHA-256 hash of the normalized participant name
   * Used for privacy-preserving name matching per TAIP-12
   */
  nameHash?: string;

  /**
   * Role of the participant in the transaction
   * e.g., "originator", "beneficiary", "agent"
   */
  role?: string;

  /**
   * DID of the party this participant acts for
   * Used when participant is an agent acting on behalf of another party
   */
  for?: DID;

  /**
   * List of policies that apply to this participant
   * Defines requirements and constraints on the participant's actions
   */
  policies?: Policies[];

  /**
   * Merchant Category Code (ISO 18245)
   * Standard classification code for merchant types in payment transactions
   * Used primarily for merchants in payment requests
   */
  mcc?: string;
}

/**
 * Base interface for all TAP policy types.
 * Policies define requirements and constraints that must be satisfied during a transaction.
 * Each specific policy type extends this base interface with its own requirements.
 */
export interface Policy<T extends string> extends JsonLdObject<T> {
  /** The type identifier for this policy */
  "@type": T;

  /**
   * Optional DID of the party or agent required to fulfill this policy
   * Can be a single DID or an array of DIDs
   */
  from?: string;

  /**
   * Optional role of the party required to fulfill this policy
   * E.g. 'SettlementAddress' for TAIP-3
   */
  fromRole?: string;

  /**
   * Optional agent representing a party required to fulfill this policy
   * E.g. 'originator' or 'beneficiary' in TAIP-3
   */
  fromAgent?: string;

  /**
   * Optional human-readable description of the policy's purpose
   * Used to explain why this requirement exists
   */
  purpose?: string;
}

/**
 * Policy requiring authorization before proceeding
 * Used to ensure specific agents authorize a transaction.
 */
export interface RequireAuthorization extends Policy<"RequireAuthorization"> {}

/**
 * Policy requiring presentation of verifiable credentials
 * Used to request specific verifiable credentials from participants.
 */
export interface RequirePresentation extends Policy<"RequirePresentation"> {
  /**
   * Optional DID of the party the presentation is about
   * Used when requesting credentials about a specific party
   */
  aboutParty?: string;

  /**
   * Optional DID of the agent the presentation is about
   * Used when requesting credentials about a specific agent
   */
  aboutAgent?: string;

  /**
   * Presentation Exchange definition
   * Specifies the required credentials and constraints
   */
  presentationDefinition: string;

  /**
   * Optional credential type shorthand
   * Simplified way to request a specific credential type
   */
  credentialType?: string;
}

/**
 * Policy requiring relationship confirmation
 * Used to verify control of addresses and relationships between parties.
 */
export interface RequireRelationshipConfirmation
  extends Policy<"RequireRelationshipConfirmation"> {
  /**
   * Required nonce for signature
   * Prevents replay attacks
   */
  nonce: string;
}

/**
 * Policy requiring purpose codes for transactions
 * Used to enforce the inclusion of ISO 20022 purpose codes.
 */
export interface RequirePurpose extends Policy<"RequirePurpose"> {
  /**
   * Required purpose code fields
   * Specifies which purpose code types must be included
   */
  fields: ("purpose" | "categoryPurpose")[];
}

/**
 * Policy type definition
 * Union type of all possible policy types in TAP.
 */
export type Policies =
  | RequireAuthorization
  | RequirePresentation
  | RequireRelationshipConfirmation
  | RequirePurpose;

// Core TAP Message Bodies

/**
 * Transfer Message
 * Initiates a transfer of assets between parties.
 * Core message type for asset transfers in TAP.
 */
export interface Transfer extends TapMessageObject<"Transfer"> {
  /**
   * Asset being transferred
   * Can be either a blockchain asset (CAIP-19) or traditional finance asset (DTI)
   */
  asset: Asset;

  /**
   * Amount to transfer
   * String representation of the decimal amount
   */
  amount: Amount;

  /**
   * Optional ISO 20022 purpose code
   * Indicates the purpose of the transfer
   */
  purpose?: ISO20022PurposeCode;

  /**
   * Optional ISO 20022 category purpose code
   * High-level classification of the transfer purpose
   */
  categoryPurpose?: ISO20022CategoryPurposeCode;

  /**
   * Optional expiration timestamp
   * Indicates when the transfer request expires
   */
  expiry?: ISO8601DateTime;

  /**
   * Details of the transfer originator
   * The party initiating the transfer
   */
  originator: Participant<"Party">;

  /**
   * Optional details of the transfer beneficiary
   * The party receiving the transfer
   */
  beneficiary?: Participant<"Party">;

  /**
   * List of agents involved in the transfer
   * Includes compliance, custody, and other service providers
   */
  agents: Participant<"Agent">[];

  /**
   * Optional settlement transaction identifier
   * CAIP-220 identifier for the on-chain settlement
   */
  settlementId?: CAIP220;

  /**
   * Optional memo field
   * Additional information about the transfer
   */
  memo?: string;
}

/**
 * Payment Message
 * Requests payment from a customer, optionally specifying supported assets.
 * Used for merchant-initiated payment flows.
 */
export interface Payment extends TapMessageObject<"Payment"> {
  /**
   * Optional specific asset requested
   * CAIP-19 identifier for the requested blockchain asset
   * Either asset OR currency is required
   */
  asset?: CAIP19;

  /**
   * Optional ISO 4217 currency code
   * For fiat currency payment requests
   * Either asset OR currency is required
   */
  currency?: IsoCurrency;

  /**
   * Amount requested
   * String representation of the decimal amount
   */
  amount: Amount;

  /**
   * Optional list of acceptable assets
   * CAIP-19 identifiers for assets the merchant will accept
   * Used when currency is specified to indicate which crypto assets can be used
   */
  supportedAssets?: CAIP19[];

  /**
   * Optional Invoice object or URI to an invoice document
   * Provides additional details about the payment request
   */
  invoice?: Invoice | string;

  /**
   * Optional expiration time
   * When the payment request is no longer valid
   */
  expiry?: ISO8601DateTime;

  /**
   * Details of the merchant requesting payment
   * The party requesting to receive the payment
   */
  merchant: Participant<"Party">;

  /**
   * Optional details of the customer
   * The party from whom payment is requested
   */
  customer?: Participant<"Party">;

  /**
   * List of agents involved in the payment
   * Must include at least one merchant agent with policies
   */
  agents: Participant<"Agent">[];
}

/**
 * Authorization Message
 * Approves a transfer for execution after compliance checks.
 */
export interface Authorize extends TapMessageObject<"Authorize"> {
  /**
   * Optional reason for authorization
   */
  reason?: string;
  
  /**
   * Optional settlement address
   * The blockchain address where funds should be sent
   */
  settlementAddress?: CAIP10;
  
  /**
   * Optional expiration timestamp
   * Indicates when the authorization expires
   */
  expiry?: ISO8601DateTime;
}

/**
 * Connect Message
 * Requests a connection between agents with specified constraints.
 */
export interface Connect extends TapMessageObject<"Connect"> {
  /**
   * Details of the requesting agent
   * Includes identity and endpoints
   */
  agent?: Participant<"Agent"> & {
    /** Service URL */
    serviceUrl?: IRI;
  };

  /**
   * DID of the represented party
   * The party the agent acts on behalf of
   */
  for: DID;

  /**
   * Transaction constraints
   * Limits and allowed transaction types
   */
  constraints: TransactionConstraints;

  /**
   * Optional expiration timestamp
   * Indicates when the connection request expires
   */
  expiry?: ISO8601DateTime;
}

/**
 * Complete Message
 * Indicates that a transaction is ready for settlement, sent by the merchant's agent.
 * Used in the Payment flow to provide settlement address to the customer.
 */
export interface Complete extends TapMessageObject<"Complete"> {
  /**
   * Settlement address
   * The blockchain address where funds should be sent, specified in CAIP-10 format
   */
  settlementAddress: CAIP10;
  
  /**
   * Optional final payment amount
   * If specified, must be less than or equal to the amount in the original Payment message
   * If omitted, the full amount from the original Payment message is implied
   */
  amount?: Amount;
}

/**
 * Settlement Message
 * Confirms the on-chain settlement of a transfer.
 */
export interface Settle extends TapMessageObject<"Settle"> {
  /**
   * Settlement transaction identifier
   * CAIP-220 identifier for the on-chain settlement transaction
   */
  settlementId: CAIP220;
  
  /**
   * Optional settled amount
   * If specified, must be less than or equal to the amount in the original transaction
   * If a Complete message specified an amount, this must match that value
   */
  amount?: Amount;
}

/**
 * Rejection Message
 * Rejects a proposed transfer with a reason.
 */
export interface Reject extends TapMessageObject<"Reject"> {
  /**
   * Reason for rejection
   * Explanation of why the transfer was rejected
   */
  reason: string;
}

/**
 * Cancel Message
 * Terminates an existing transaction or connection.
 * Uses the thread ID to identify what is being cancelled.
 */
export interface Cancel extends TapMessageObject<"Cancel"> {
  /**
   * Optional reason for cancellation
   * Human readable explanation
   */
  reason?: string;
}

/**
 * Revert Message
 * Requests reversal of a settled transaction.
 * Used for dispute resolution or compliance-related reversals.
 */
export interface Revert extends TapMessageObject<"Revert"> {
  /**
   * Settlement address for the revert
   * CAIP-10 identifier for the revert destination
   */
  settlementAddress: string;

  /**
   * Reason for the revert request
   * Explanation of why the transfer needs to be reversed
   */
  reason: string;
}

/**
 * Update Agent Message
 * Updates the details or policies of an existing agent.
 */
export interface UpdateAgent extends TapMessageObject<"UpdateAgent"> {
  /**
   * Updated agent details
   * Complete agent information including any changes
   */
  agent: Participant<"Agent">;
}

/**
 * Update Party Message
 * Updates the details of a party in the transaction.
 */
export interface UpdateParty extends TapMessageObject<"UpdateParty"> {
  /**
   * Updated party details
   * Complete party information including any changes
   */
  party: Participant<"Party">;
}

/**
 * Add Agents Message
 * Adds new agents to the transaction.
 */
export interface AddAgents extends TapMessageObject<"AddAgents"> {
  /**
   * List of agents to add
   * Complete details for each new agent
   */
  agents: Participant<"Agent">[];
}

/**
 * Replace Agent Message
 * Replaces an existing agent with a new one.
 */
export interface ReplaceAgent extends TapMessageObject<"ReplaceAgent"> {
  /**
   * DID of the agent to replace
   * Identifies the existing agent
   */
  original: DID;

  /**
   * Details of the replacement agent
   * Complete information for the new agent
   */
  replacement: Participant<"Agent">;
}

/**
 * Remove Agent Message
 * Removes an agent from the transaction.
 */
export interface RemoveAgent extends TapMessageObject<"RemoveAgent"> {
  /**
   * DID of the agent to remove
   * Identifies the agent to be removed from the transaction
   */
  agent: DID;
}

/**
 * CACAO Attachment
 * Chain Agnostic CApability Object attachment for proving control of a DID.
 */
export interface CACAOAttachment {
  /**
   * Unique identifier for the attachment
   * Used to reference this attachment within the message
   */
  id: string;

  /**
   * Media type of the attachment
   * Must be "application/json" for CACAO attachments
   */
  media_type: "application/json";

  /**
   * Attachment data containing the CACAO proof
   * Includes signature, header, and proof details
   */
  data: {
    json: {
      /**
       * Header indicating the signature type
       * Must be "eth-personal-sign" for Ethereum personal signatures
       */
      h: "eth-personal-sign";

      /**
       * CACAO signature value
       * The cryptographic signature proving control
       */
      s: string;

      /**
       * Proof message that was signed
       * The message that was signed to create the signature
       */
      p: string;

      /**
       * Timestamp of the signature
       * When the proof was created
       */
      t: ISO8601DateTime;
    };
  };
}

/**
 * Confirm Relationship Message
 * Confirms a relationship between a party and an agent.
 */
export interface ConfirmRelationship extends TapMessageObject<"ConfirmRelationship"> {
  /**
   * DID of the agent
   * Identifies the agent in the relationship
   */
  "@id": DID;

  /**
   * Optional role of the agent
   * Describes the agent's function in the relationship
   */
  role?: string;

  /**
   * Optional DID of the party
   * Identifies the party the agent is related to
   */
  for?: DID;
}

/**
 * Update Policies Message
 * Updates the policies associated with an agent.
 */
export interface UpdatePolicies extends TapMessageObject<"UpdatePolicies"> {
  /**
   * List of updated policies
   * Complete set of policies that should apply
   */
  policies: Policies[];
}

/**
 * Transaction Constraints
 * Defines the allowed transaction parameters for a connection.
 */
export interface TransactionConstraints {
  /**
   * Allowed ISO 20022 purpose codes
   * Array of valid purpose codes for transactions
   */
  purposes?: ISO20022PurposeCode[];

  /**
   * Allowed ISO 20022 category purpose codes
   * Array of valid category purpose codes
   */
  categoryPurposes?: ISO20022CategoryPurposeCode[];

  /**
   * Transaction limits
   * Monetary limits for transactions
   */
  limits?: {
    /**
     * Maximum amount per transaction
     * Decimal string representation
     */
    per_transaction: Amount;

    /**
     * Maximum daily total
     * Decimal string representation
     */
    daily: Amount;

    /**
     * Currency for the limits
     * ISO 4217 currency code
     */
    currency: IsoCurrency;
  };
}

/**
 * Authorization Required Message
 * Response providing an authorization URL for connection approval.
 */
export interface AuthorizationRequired
  extends TapMessageObject<"AuthorizationRequired"> {
  /**
   * URL for connection authorization
   * Where the customer can review and approve
   */
  authorization_url: string;

  /**
   * Expiration timestamp
   * When the authorization URL expires
   */
  expires: ISO8601DateTime;
}

/**
 * Transaction Types
 * Union type of all transaction initiation messages in TAP.
 * Used for type-safe handling of transaction messages.
 */
export type Transactions = Transfer | Payment;

// DIDComm Message Wrappers

/**
 * Transfer Message Wrapper
 * DIDComm envelope for a Transfer message.
 */
export interface TransferMessage extends DIDCommMessage<Transfer> {
  type: "https://tap.rsvp/schema/1.0#Transfer";
}

/**
 * Payment Message Wrapper
 * DIDComm envelope for a Payment message.
 */
export interface PaymentMessage extends DIDCommMessage<Payment> {
  type: "https://tap.rsvp/schema/1.0#Payment";
}

/**
 * Authorization Message Wrapper
 * DIDComm envelope for an Authorization message.
 */
export interface AuthorizeMessage extends DIDCommReply<Authorize> {
  type: "https://tap.rsvp/schema/1.0#Authorize";
}

/**
 * Settlement Message Wrapper
 * DIDComm envelope for a Settlement message.
 */
export interface SettleMessage extends DIDCommReply<Settle> {
  type: "https://tap.rsvp/schema/1.0#Settle";
}

/**
 * Rejection Message Wrapper
 * DIDComm envelope for a Rejection message.
 */
export interface RejectMessage extends DIDCommReply<Reject> {
  type: "https://tap.rsvp/schema/1.0#Reject";
}

/**
 * Cancellation Message Wrapper
 * DIDComm envelope for a Cancellation message.
 */
export interface CancelMessage extends DIDCommReply<Cancel> {
  type: "https://tap.rsvp/schema/1.0#Cancel";
}

/**
 * Revert Message Wrapper
 * DIDComm envelope for a Revert message.
 */
export interface RevertMessage extends DIDCommReply<Revert> {
  type: "https://tap.rsvp/schema/1.0#Revert";
}

/**
 * Update Agent Message Wrapper
 * DIDComm envelope for an Update Agent message.
 */
export interface UpdateAgentMessage extends DIDCommReply<UpdateAgent> {
  type: "https://tap.rsvp/schema/1.0#UpdateAgent";
}

/**
 * Update Party Message Wrapper
 * DIDComm envelope for an Update Party message.
 */
export interface UpdatePartyMessage extends DIDCommReply<UpdateParty> {
  type: "https://tap.rsvp/schema/1.0#UpdateParty";
}

/**
 * Add Agents Message Wrapper
 * DIDComm envelope for an Add Agents message.
 */
export interface AddAgentsMessage extends DIDCommReply<AddAgents> {
  type: "https://tap.rsvp/schema/1.0#AddAgents";
}

/**
 * Replace Agent Message Wrapper
 * DIDComm envelope for a Replace Agent message.
 */
export interface ReplaceAgentMessage extends DIDCommReply<ReplaceAgent> {
  type: "https://tap.rsvp/schema/1.0#ReplaceAgent";
}

/**
 * Remove Agent Message Wrapper
 * DIDComm envelope for a Remove Agent message.
 */
export interface RemoveAgentMessage extends DIDCommReply<RemoveAgent> {
  type: "https://tap.rsvp/schema/1.0#RemoveAgent";
}

/**
 * Confirm Relationship Message Wrapper
 * DIDComm envelope for a Confirm Relationship message.
 */
export interface ConfirmRelationshipMessage extends DIDCommReply<ConfirmRelationship> {
  /**
   * Message type identifier
   * Must be "https://tap.rsvp/schema/1.0#ConfirmRelationship" for relationship confirmations
   */
  type: "https://tap.rsvp/schema/1.0#ConfirmRelationship";

  /**
   * Optional CACAO attachments
   * Proofs of DID control for the relationship
   */
  attachments?: [CACAOAttachment];
}

/**
 * Update Policies Message Wrapper
 * DIDComm envelope for an Update Policies message.
 */
export interface UpdatePoliciesMessage extends DIDCommReply<UpdatePolicies> {
  /**
   * Message type identifier
   * Must be "https://tap.rsvp/schema/1.0#UpdatePolicies" for policy updates
   */
  type: "https://tap.rsvp/schema/1.0#UpdatePolicies";
}

/**
 * Connect Message Wrapper
 * DIDComm envelope for a Connect message.
 */
export interface ConnectMessage extends DIDCommMessage<Connect> {
  type: "https://tap.rsvp/schema/1.0#Connect";
}

/**
 * Authorization Required Message Wrapper
 * DIDComm envelope for an Authorization Required message.
 */
export interface AuthorizationRequiredMessage
  extends DIDCommReply<AuthorizationRequired> {
  type: "https://tap.rsvp/schema/1.0#AuthorizationRequired";
}

/**
 * Complete Message Wrapper
 * DIDComm envelope for a Complete message.
 */
export interface CompleteMessage extends DIDCommReply<Complete> {
  type: "https://tap.rsvp/schema/1.0#Complete";
}

/**
 * TAP Message
 * Union type of all possible TAP messages.
 * Used for type-safe message handling in TAP implementations.
 * Includes all transaction, authorization, and management messages.
 */
export type TAPMessage =
  | TransferMessage
  | PaymentMessage
  | AuthorizeMessage
  | SettleMessage
  | RejectMessage
  | CancelMessage
  | RevertMessage
  | UpdateAgentMessage
  | UpdatePartyMessage
  | AddAgentsMessage
  | ReplaceAgentMessage
  | RemoveAgentMessage
  | ConfirmRelationshipMessage
  | UpdatePoliciesMessage
  | ConnectMessage
  | AuthorizationRequiredMessage
  | CompleteMessage;