/**
 * TAP Agent
 * A class for managing identities and signing/verifying messages in TAP
 */

import { DID } from "../models/types";
import {
  DIDResolver,
  DIDResolutionResult,
  createDefaultResolver,
} from "./resolver";
import { getWasmModule, createAgent } from "../wasm/bridge";
import {
  ValidationError,
  CryptoError,
  VerificationError,
  DIDResolutionError,
} from "../utils/errors";

// Import TAP types from the standard @taprsvp/types package
import {
  DIDCommMessage,
  Transfer,
  TransferMessage,
  PaymentRequest,
  PaymentRequestMessage,
  Authorize,
  Reject,
  Settle,
  Cancel,
  Revert,
  Complete,
  Participant,
  CAIP10,
  CAIP19,
  Amount,
  Asset,
  IsoCurrency,
  ISO8601DateTime,
  ISO20022PurposeCode,
  ISO20022CategoryPurposeCode,
  Invoice,
  TapMessageObject,
} from '@taprsvp/types';

// Import our message wrapper for DIDComm envelope handling
import {
  MessageWrapper,
  MessageWrapperOptions,
  TransferWrapper,
  PaymentRequestWrapper,
  ReplyFactory
} from './MessageWrapper';

/**
 * Signer interface
 * Defines the methods required for signing messages
 */
export interface Signer {
  /**
   * Sign data with the private key
   *
   * @param data The data to sign
   * @returns Promise resolving to the signature
   */
  sign(data: Uint8Array): Promise<Uint8Array>;

  /**
   * Get the DID for this signer
   *
   * @returns The DID controlled by this signer
   */
  getDID(): DID;
}

/**
 * Key material interface
 * Represents the key material for a DID
 */
export interface KeyMaterial {
  /**
   * Private key (keep secure!)
   */
  privateKey: Uint8Array;

  /**
   * Public key
   */
  publicKey: Uint8Array;

  /**
   * DID associated with this key
   */
  did: DID;
}

/**
 * TAP Agent options
 * Configuration options for creating a TAP agent
 */
export interface TAPAgentOptions {
  /**
   * DID of the agent
   */
  did: DID;

  /**
   * Signer for the agent
   * Used to sign messages
   */
  signer: Signer;

  /**
   * DID resolver
   * Used to resolve DIDs to DID Documents
   * Default: basic resolver handling did:key and did:web
   */
  resolver?: DIDResolver;
}

/**
 * Options for creating a Transfer message
 */
export interface TransferOptions {
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
  purpose?: ISO20022PurposeCode;

  /** Optional category purpose code */
  categoryPurpose?: ISO20022CategoryPurposeCode;

  /** Optional expiry timestamp */
  expiry?: ISO8601DateTime;

  /** Additional message options */
  messageOptions?: MessageWrapperOptions;
}

/**
 * Options for creating a Payment Request message
 */
export interface PaymentRequestOptions {
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
  expiry?: ISO8601DateTime;

  /** Additional message options */
  messageOptions?: MessageWrapperOptions;
}

/**
 * Options for creating an Authorize message
 */
export interface AuthorizeOptions {
  /** Optional settlement address */
  settlementAddress?: CAIP10;

  /** Optional reason for the authorization */
  reason?: string;

  /** Optional expiry in seconds from now */
  expiryInSeconds?: number;

  /** Additional message options */
  messageOptions?: MessageWrapperOptions;
}

/**
 * Options for creating a Reject message
 */
export interface RejectOptions {
  /** Reason for the rejection */
  reason: string;

  /** Additional message options */
  messageOptions?: MessageWrapperOptions;
}

/**
 * Options for creating a Settle message
 */
export interface SettleOptions {
  /** Settlement transaction identifier */
  settlementId: string;

  /** Optional settled amount */
  amount?: Amount;

  /** Additional message options */
  messageOptions?: MessageWrapperOptions;
}

/**
 * Options for creating a Cancel message
 */
export interface CancelOptions {
  /** Optional reason for the cancellation */
  reason?: string;

  /** Additional message options */
  messageOptions?: MessageWrapperOptions;
}

/**
 * Options for creating a Revert message
 */
export interface RevertOptions {
  /** Settlement address for the revert */
  settlementAddress: string;

  /** Reason for the revert request */
  reason: string;

  /** Additional message options */
  messageOptions?: MessageWrapperOptions;
}

/**
 * Options for creating a Complete message
 */
export interface CompleteOptions {
  /** Settlement address where funds should be sent */
  settlementAddress: CAIP10;

  /** Optional final payment amount */
  amount?: Amount;

  /** Additional message options */
  messageOptions?: MessageWrapperOptions;
}

/**
 * TAP Agent implementation
 * Manages identities and handles message signing and verification in TAP
 */
export class TAPAgent {
  /** The agent's DID */
  private did: DID;

  /** The signer used for signing messages */
  private signer: Signer;

  /** The resolver used for resolving DIDs */
  private resolver: DIDResolver;

  /** The underlying WASM agent */
  private wasmAgent: any;

  /**
   * Create a new TAP agent
   *
   * @param options Agent configuration options
   */
  constructor(options: TAPAgentOptions) {
    this.did = options.did;
    this.signer = options.signer;
    this.resolver = options.resolver || createDefaultResolver();

    // Initialize the WASM agent
    this.initWasmAgent().catch((err) => {
      console.error("Failed to initialize WASM agent:", err);
    });
  }

  /**
   * Initialize the WASM agent
   * This happens asynchronously but we start it in the constructor
   */
  private async initWasmAgent(): Promise<void> {
    const wasm = await getWasmModule();
    this.wasmAgent = await createAgent(this.did, "placeholder-key");
  }

  /**
   * Get the agent's DID
   *
   * @returns The DID of the agent
   */
  getDID(): DID {
    return this.did;
  }

  /**
   * Create a new Transfer message
   *
   * @param options Options for the transfer
   * @returns A wrapper for the Transfer message
   */
  transfer(options: TransferOptions): TransferWrapper {
    // Create the Transfer body according to the TAP specification
    const transferBody: Transfer = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Transfer",
      asset: options.asset,
      amount: options.amount,
      originator: options.originator,
      agents: options.agents
    };

    // Add optional fields if provided
    if (options.beneficiary) transferBody.beneficiary = options.beneficiary;
    if (options.settlementId) transferBody.settlementId = options.settlementId;
    if (options.memo) transferBody.memo = options.memo;
    if (options.purpose) transferBody.purpose = options.purpose;
    if (options.categoryPurpose) transferBody.categoryPurpose = options.categoryPurpose;
    if (options.expiry) transferBody.expiry = options.expiry;

    // Create the wrapped message
    const wrapper = new TransferWrapper(transferBody, options.messageOptions);

    // Set this agent as the owner for enabling reply methods
    return wrapper.setAgent(this) as TransferWrapper;
  }

  /**
   * Create a new Payment Request message
   *
   * @param options Options for the payment request
   * @returns A wrapper for the Payment Request message
   */
  paymentRequest(options: PaymentRequestOptions): PaymentRequestWrapper {
    // Create the Payment Request body according to the TAP specification
    const paymentBody: PaymentRequest = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "PaymentRequest",
      amount: options.amount,
      merchant: options.merchant,
      agents: options.agents
    };

    // Add optional fields if provided
    if (options.asset) paymentBody.asset = options.asset;
    if (options.currency) paymentBody.currency = options.currency;
    if (options.supportedAssets) paymentBody.supportedAssets = options.supportedAssets;
    if (options.invoice) paymentBody.invoice = options.invoice;
    if (options.customer) paymentBody.customer = options.customer;
    if (options.expiry) paymentBody.expiry = options.expiry;

    // Create the wrapped message
    const wrapper = new PaymentRequestWrapper(paymentBody, options.messageOptions);

    // Set this agent as the owner for enabling reply methods
    return wrapper.setAgent(this) as PaymentRequestWrapper;
  }

  /**
   * Create an authorization message in response to a message
   *
   * @param parent The parent message this is replying to
   * @param options Options for the authorization
   * @returns A wrapper for the Authorize message
   */
  authorize(parent: MessageWrapper<any>, options: AuthorizeOptions): MessageWrapper<Authorize> {
    // Create the Authorize body according to the TAP specification
    const authorizeBody: Authorize = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Authorize",
      transfer: { "@id": parent.id },
    };

    // Add optional fields if provided
    if (options.reason) authorizeBody.reason = options.reason;
    if (options.settlementAddress) (authorizeBody as any).settlementAddress = options.settlementAddress;

    // Add expiry if specified
    if (options.expiryInSeconds) {
      const expiry = new Date(Date.now() + options.expiryInSeconds * 1000).toISOString();
      (authorizeBody as any).expiry = expiry;
    }

    // Create the reply
    const wrapper = ReplyFactory.createAuthorize(parent, authorizeBody, options.messageOptions);

    // Set this agent as the owner
    return wrapper.setAgent(this);
  }

  /**
   * Create a reject message in response to a message
   *
   * @param parent The parent message this is replying to
   * @param options Options for the rejection
   * @returns A wrapper for the Reject message
   */
  reject(parent: MessageWrapper<any>, options: RejectOptions): MessageWrapper<Reject> {
    // Create the Reject body according to the TAP specification
    const rejectBody: Reject = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Reject",
      transfer: { "@id": parent.id },
      reason: options.reason
    };

    // Create the reply
    const wrapper = ReplyFactory.createReject(parent, rejectBody, options.messageOptions);

    // Set this agent as the owner
    return wrapper.setAgent(this);
  }

  /**
   * Create a settlement message in response to a message
   *
   * @param parent The parent message this is replying to
   * @param options Options for the settlement
   * @returns A wrapper for the Settle message
   */
  settle(parent: MessageWrapper<any>, options: SettleOptions): MessageWrapper<Settle> {
    // Create the Settle body according to the TAP specification
    const settleBody: Settle = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Settle",
      transfer: { "@id": parent.id },
      settlementId: options.settlementId
    };

    // Add optional amount if provided
    if (options.amount) settleBody.amount = options.amount;

    // Create the reply
    const wrapper = ReplyFactory.createSettle(parent, settleBody, options.messageOptions);

    // Set this agent as the owner
    return wrapper.setAgent(this);
  }

  /**
   * Create a cancellation message in response to a message
   *
   * @param parent The parent message this is replying to
   * @param options Options for the cancellation
   * @returns A wrapper for the Cancel message
   */
  cancel(parent: MessageWrapper<any>, options: CancelOptions): MessageWrapper<Cancel> {
    // Create the Cancel body according to the TAP specification
    const cancelBody: Cancel = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Cancel"
    };

    // Add optional reason if provided
    if (options.reason) cancelBody.reason = options.reason;

    // Create the reply
    const wrapper = ReplyFactory.createCancel(parent, cancelBody, options.messageOptions);

    // Set this agent as the owner
    return wrapper.setAgent(this);
  }

  /**
   * Create a revert message in response to a message
   *
   * @param parent The parent message this is replying to
   * @param options Options for the revert
   * @returns A wrapper for the Revert message
   */
  revert(parent: MessageWrapper<any>, options: RevertOptions): MessageWrapper<Revert> {
    // Create the Revert body according to the TAP specification
    const revertBody: Revert = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Revert",
      settlementAddress: options.settlementAddress,
      reason: options.reason
    };

    // Create the reply
    const wrapper = ReplyFactory.createRevert(parent, revertBody, options.messageOptions);

    // Set this agent as the owner
    return wrapper.setAgent(this);
  }

  /**
   * Create a complete message in response to a payment request
   *
   * @param parent The parent message this is replying to
   * @param options Options for the complete message
   * @returns A wrapper for the Complete message
   */
  complete(parent: MessageWrapper<any>, options: CompleteOptions): MessageWrapper<Complete> {
    // Create the Complete body according to the TAP specification
    const completeBody: Complete = {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Complete",
      settlementAddress: options.settlementAddress
    };

    // Add optional amount if provided
    if (options.amount) completeBody.amount = options.amount;

    // Create the message with thread ID
    const wrapper = new MessageWrapper<Complete>(
      "https://tap.rsvp/schema/1.0#Complete",
      completeBody,
      { ...options.messageOptions, thid: parent.id }
    );

    // Set this agent as the owner
    return wrapper.setAgent(this);
  }

  /**
   * Sign a message
   * Prepares the message envelope and creates a signature
   *
   * @param message The message to sign
   * @returns Promise resolving to the signed message
   * @throws ValidationError if the message is invalid
   * @throws CryptoError if signing fails
   */
  async sign<T extends TapMessageObject<any>>(
    message: MessageWrapper<T>,
  ): Promise<MessageWrapper<T>> {
    // Prepare the envelope
    message.prepareEnvelope(this.did);

    try {
      // If the WASM agent is available, use it to sign the message
      if (this.wasmAgent) {
        // Convert the message to a format the WASM agent can understand
        // This depends on the exact WASM API
        await this.wasmAgent.sign(message);
      } else {
        // Otherwise use the signer directly
        // This is a simplified version; real implementation would need to
        // handle header, payload, etc. according to DIDComm spec
        const messageBytes = new TextEncoder().encode(JSON.stringify(message));
        const signature = await this.signer.sign(messageBytes);

        // In a real implementation, we would attach the signature to the message
        // For now, we'll just show what would happen
        console.log("Message signed with signature length:", signature.length);
      }

      return message;
    } catch (error) {
      throw new CryptoError(`Failed to sign message: ${error}`);
    }
  }

  /**
   * Verify a message signature
   *
   * @param message The message to verify
   * @returns Promise resolving to a boolean indicating if the signature is valid
   * @throws DIDResolutionError if the sender's DID cannot be resolved
   * @throws VerificationError if verification fails
   */
  async verify<T extends TapMessageObject<any>>(message: MessageWrapper<T>): Promise<boolean> {
    // Check if message has the required fields
    if (!message.from) {
      throw new ValidationError("Message has no sender (from field)");
    }

    try {
      // Resolve the sender's DID
      const resolution = await this.resolver.resolve(message.from);

      if (!resolution.didDocument) {
        throw new DIDResolutionError(
          message.from,
          resolution.didResolutionMetadata.error || "Unknown error",
        );
      }

      // If the WASM agent is available, use it to verify the message
      if (this.wasmAgent) {
        return await this.wasmAgent.verify(message);
      } else {
        // This is a placeholder for manual verification logic
        // In a real implementation, we would verify the message signature
        // using the verification methods in the DID Document
        console.log("Would verify message from:", message.from);
        console.log("Using DID Document:", resolution.didDocument);

        // For now, just return true (this is not secure!)
        return true;
      }
    } catch (error) {
      if (error instanceof DIDResolutionError) {
        throw error;
      }
      throw new VerificationError(`Failed to verify message: ${error}`);
    }
  }

  /**
   * Create a new agent with a generated key
   * Static factory method for easily creating new agents
   *
   * @returns Promise resolving to a new TAP agent with a generated key
   */
  static async create(): Promise<TAPAgent> {
    // This is a placeholder for generating new keys
    // In a real implementation, we would use a proper key generation library
    const wasm = await getWasmModule();
    const did = await wasm.create_did_key();

    // Create a simple signer that uses the WASM module
    const signer: Signer = {
      async sign(data: Uint8Array): Promise<Uint8Array> {
        // This would call into the WASM module to sign with the private key
        return new Uint8Array(0); // Placeholder
      },

      getDID(): DID {
        return did;
      },
    };

    // Create and return a new agent
    return new TAPAgent({
      did,
      signer,
      resolver: createDefaultResolver(),
    });
  }
}
