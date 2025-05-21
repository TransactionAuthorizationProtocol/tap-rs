import { TAPAgent } from "../agent";
import { BaseMessage } from "./base-message";
import { DID, EntityReference, Asset } from "../types";

/**
 * TransferMessage - Represents a TAP Transfer message
 */
export class TransferMessage extends BaseMessage {
  /**
   * Create a new transfer message
   */
  constructor(agent: TAPAgent, message: any) {
    super(agent, message);
  }

  /**
   * Get the asset being transferred
   */
  get asset(): Asset {
    return this.body.asset;
  }

  /**
   * Set the asset being transferred
   */
  setAsset(asset: Asset): this {
    this.body.asset = asset;
    return this;
  }

  /**
   * Get the amount being transferred
   */
  get amount(): string {
    return this.body.amount;
  }

  /**
   * Set the amount being transferred
   */
  setAmount(amount: string): this {
    this.body.amount = amount;
    return this;
  }

  /**
   * Get the originator of the transfer
   */
  get originator(): EntityReference {
    return this.body.originator;
  }

  /**
   * Set the originator of the transfer
   */
  setOriginator(originator: EntityReference): this {
    this.body.originator = originator;
    return this;
  }

  /**
   * Get the beneficiary of the transfer
   */
  get beneficiary(): EntityReference | undefined {
    return this.body.beneficiary;
  }

  /**
   * Set the beneficiary of the transfer
   */
  setBeneficiary(beneficiary: EntityReference): this {
    this.body.beneficiary = beneficiary;
    if (beneficiary['@id']) {
      this.setTo(beneficiary['@id'] as DID);
    }
    return this;
  }

  /**
   * Get the agents involved in the transfer
   */
  get agents(): EntityReference[] {
    return this.body.agents || [];
  }

  /**
   * Set the agents involved in the transfer
   */
  setAgents(agents: EntityReference[]): this {
    this.body.agents = agents;
    return this;
  }

  /**
   * Get the memo for the transfer
   */
  get memo(): string | undefined {
    return this.body.memo;
  }

  /**
   * Set the memo for the transfer
   */
  setMemo(memo: string): this {
    this.body.memo = memo;
    return this;
  }

  /**
   * Get the settlement ID for the transfer
   */
  get settlementId(): string | undefined {
    return this.body.settlementId;
  }

  /**
   * Set the settlement ID for the transfer
   */
  setSettlementId(settlementId: string): this {
    this.body.settlementId = settlementId;
    return this;
  }

  /**
   * Get the raw message for signing
   * @returns Raw message
   */
  getMessage(): any {
    return this.message;
  }
}