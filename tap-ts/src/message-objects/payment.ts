import { TAPAgent } from "../agent";
import { BaseMessage } from "./base-message";
import { DID, EntityReference } from "../types";

/**
 * PaymentMessage - Represents a TAP Payment message
 */
export class PaymentMessage extends BaseMessage {
  /**
   * Create a new payment message
   */
  constructor(agent: TAPAgent, message: any) {
    super(agent, message);
  }

  /**
   * Get the asset for the payment
   */
  get asset(): string | undefined {
    return this.body.asset;
  }

  /**
   * Set the asset for the payment
   */
  setAsset(asset: string): this {
    this.body.asset = asset;
    return this;
  }

  /**
   * Get the currency for the payment
   */
  get currency(): string | undefined {
    return this.body.currency;
  }

  /**
   * Set the currency for the payment
   */
  setCurrency(currency: string): this {
    this.body.currency = currency;
    return this;
  }

  /**
   * Get the amount of the payment
   */
  get amount(): string {
    return this.body.amount;
  }

  /**
   * Set the amount of the payment
   */
  setAmount(amount: string): this {
    this.body.amount = amount;
    return this;
  }

  /**
   * Get the merchant for the payment
   */
  get merchant(): EntityReference {
    return this.body.merchant;
  }

  /**
   * Set the merchant for the payment
   */
  setMerchant(merchant: EntityReference): this {
    this.body.merchant = merchant;
    return this;
  }

  /**
   * Get the customer for the payment
   */
  get customer(): EntityReference | undefined {
    return this.body.customer;
  }

  /**
   * Set the customer for the payment
   */
  setCustomer(customer: EntityReference): this {
    this.body.customer = customer;
    if (customer['@id']) {
      this.setTo(customer['@id'] as DID);
    }
    return this;
  }

  /**
   * Get the invoice ID for the payment
   */
  get invoice(): string | undefined {
    return this.body.invoice;
  }

  /**
   * Set the invoice ID for the payment
   */
  setInvoice(invoice: string): this {
    this.body.invoice = invoice;
    return this;
  }

  /**
   * Get the expiry timestamp for the payment
   */
  get expiry(): string | undefined {
    return this.body.expiry;
  }

  /**
   * Set the expiry timestamp for the payment
   */
  setExpiry(expiry: string): this {
    this.body.expiry = expiry;
    return this;
  }

  /**
   * Get the supported assets for the payment
   */
  get supportedAssets(): string[] | undefined {
    return this.body.supportedAssets;
  }

  /**
   * Set the supported assets for the payment
   */
  setSupportedAssets(supportedAssets: string[]): this {
    this.body.supportedAssets = supportedAssets;
    return this;
  }

  /**
   * Get the agents involved in the payment
   */
  get agents(): EntityReference[] {
    return this.body.agents || [];
  }

  /**
   * Set the agents involved in the payment
   */
  setAgents(agents: EntityReference[]): this {
    this.body.agents = agents;
    return this;
  }
}