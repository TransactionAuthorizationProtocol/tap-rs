import { TAPAgent } from "../agent";
import { BaseMessage } from "./base-message";
import { Agent } from "../types";

/**
 * ConnectMessage - Represents a TAP Connect message
 */
export class ConnectMessage extends BaseMessage {
  /**
   * Create a new connect message
   */
  constructor(agent: TAPAgent, message: any) {
    super(agent, message);
  }

  /**
   * Get the agent making the connection
   */
  getConnectionAgent(): Agent | undefined {
    return this.body.agent;
  }

  /**
   * Set the agent making the connection
   */
  setConnectionAgent(agent: Agent): this {
    this.body.agent = agent;
    return this;
  }

  /**
   * Get what the connection is for
   */
  get for(): string {
    return this.body.for;
  }

  /**
   * Set what the connection is for
   */
  setFor(forValue: string): this {
    this.body.for = forValue;
    return this;
  }

  /**
   * Get the constraints for the connection
   */
  get constraints(): any {
    return this.body.constraints;
  }

  /**
   * Set the constraints for the connection
   */
  setConstraints(constraints: any): this {
    this.body.constraints = constraints;
    return this;
  }

  /**
   * Get the expiry timestamp for the connection
   */
  get expiry(): string | undefined {
    return this.body.expiry;
  }

  /**
   * Set the expiry timestamp for the connection
   */
  setExpiry(expiry: string): this {
    this.body.expiry = expiry;
    return this;
  }
}