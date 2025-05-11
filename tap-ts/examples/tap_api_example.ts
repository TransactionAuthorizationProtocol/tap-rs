/**
 * TAP-TS Example using the new TAP specification API
 * 
 * This example demonstrates how to use the new TAP API to create
 * and process messages for a transfer flow.
 */

import {
  TAPMessage,
  TAPAgent,
  TAPNode,
  MessageTypes,
  createTAPAgent,
  createTAPNode,
  TAPMessages
} from "../src/tap-mod.ts";

import type {
  Transfer,
  DID,
  Amount,
  CAIP19,
  Participant
} from "../src/tap-mod.ts";

// Initialize the WASM engine
async function initializeWasm() {
  // In a real application, you would import from the published package
  // import { wasmLoader } from "@notabene/tap-ts";
  const { wasmLoader } = await import("../src/tap-mod.ts");
  
  // Enable mock mode for this example (no real WASM loading)
  wasmLoader.setUseMock(true);
  await wasmLoader.load();
  
  return wasmLoader;
}

// Run the example
async function runExample() {
  console.log("Initializing TAP-TS example...");
  await initializeWasm();
  
  // Create agents for the originator and beneficiary
  const originatorAgent = createTAPAgent({
    nickname: "Originator VASP",
    debug: true
  });
  
  const beneficiaryAgent = createTAPAgent({
    nickname: "Beneficiary VASP",
    debug: true
  });
  
  console.log(`Originator DID: ${originatorAgent.getDID()}`);
  console.log(`Beneficiary DID: ${beneficiaryAgent.getDID()}`);
  
  // Create a node to manage message routing
  const node = createTAPNode({ debug: true });
  
  // Register the agents with the node
  node.registerAgent(originatorAgent);
  node.registerAgent(beneficiaryAgent);
  
  // Set up message handlers
  beneficiaryAgent.handleMessage(MessageTypes.TRANSFER, async (message, metadata) => {
    console.log("Beneficiary received a Transfer message");
    
    // Get the transfer data
    const transfer = message.getTransfer();
    if (!transfer) {
      console.error("Invalid Transfer message");
      return;
    }
    
    console.log(`Transfer amount: ${transfer.amount} ${transfer.asset}`);
    
    // Create an authorization response
    const authorizeMessage = TAPMessages.createAuthorize(
      {
        reason: "Approved by compliance checks",
        settlementAddress: "eip155:1:0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
      },
      message.id, // Use the transfer message ID as the thread ID
      {
        from: beneficiaryAgent.getDID(),
        to: [message.from!]
      }
    );
    
    // Sign the message
    beneficiaryAgent.signMessage(authorizeMessage);
    
    // Process through the node (which will deliver it to the originator)
    await node.processMessage(authorizeMessage);
  });
  
  originatorAgent.handleMessage(MessageTypes.AUTHORIZE, (message, metadata) => {
    console.log("Originator received an Authorize message");
    
    // Get the authorize data
    const authorize = message.getAuthorize();
    if (!authorize) {
      console.error("Invalid Authorize message");
      return;
    }
    
    console.log(`Authorization reason: ${authorize.reason}`);
    console.log(`Settlement address: ${authorize.settlementAddress}`);
    
    // In a real implementation, this would trigger the settlement process
    console.log("Starting settlement process...");
  });
  
  // Create a transfer message
  const transfer: Transfer = {
    "@context": "https://tap.rsvp/schema/1.0",
    "@type": "Transfer",
    asset: "eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48" as CAIP19,
    amount: "100.00" as Amount,
    originator: {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Party",
      "@id": originatorAgent.getDID() as DID,
      role: "originator",
      name: "Alice Smith"
    } as Participant<"Party">,
    beneficiary: {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Party",
      "@id": beneficiaryAgent.getDID() as DID,
      role: "beneficiary",
      name: "Bob Jones"
    } as Participant<"Party">,
    agents: [
      {
        "@context": "https://tap.rsvp/schema/1.0",
        "@type": "Agent",
        "@id": originatorAgent.getDID() as DID,
        role: "originator_vasp",
        for: originatorAgent.getDID() as DID
      } as Participant<"Agent">,
      {
        "@context": "https://tap.rsvp/schema/1.0",
        "@type": "Agent",
        "@id": beneficiaryAgent.getDID() as DID,
        role: "beneficiary_vasp",
        for: beneficiaryAgent.getDID() as DID
      } as Participant<"Agent">
    ],
    memo: "Payment for services"
  };
  
  // Create the transfer message
  const transferMessage = TAPMessages.createTransfer(
    transfer,
    {
      from: originatorAgent.getDID() as DID,
      to: [beneficiaryAgent.getDID() as DID]
    }
  );
  
  // Sign the message with the originator's key
  originatorAgent.signMessage(transferMessage);
  
  // Send the message through the node
  console.log("Sending transfer message...");
  await node.processMessage(transferMessage);
  
  console.log("Example completed");
}

// Run the example and handle errors
runExample().catch(error => {
  console.error("Error in example:", error);
});