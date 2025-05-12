/**
 * Basic usage example for @taprsvp/tap
 *
 * This example shows a complete transfer flow between two parties.
 */

import { TAPAgent, Transfer } from "@taprsvp/tap";
import type { Transfer as TransferInterface } from "@taprsvp/types";

// Example function to create a signer (in real code, you would use a proper key manager)
function createSigner(privateKey: string, did: string) {
  return {
    sign: async (data: Uint8Array): Promise<Uint8Array> => {
      console.log(
        `[Signing ${data.length} bytes with key ${privateKey.slice(0, 8)}...]`,
      );
      // In a real implementation, this would actually sign the data
      return new Uint8Array(64); // Fake signature
    },
    getDID: () => did,
  };
}

// Example transport layer (in real code, this would use HTTP, WebSockets, etc.)
async function sendMessage(message: any, endpoint: string) {
  console.log(`[Sending message ${message.id} to ${endpoint}]`);
  console.log(JSON.stringify(message, null, 2));
  return true;
}

async function main() {
  try {
    console.log("Starting TAP transfer flow example...");

    // Create agents for originator and beneficiary
    console.log("\n1. Creating agents...");

    const originatorDid =
      "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
    const beneficiaryDid =
      "did:key:z6MkgYAGxLBSJEm1JAHFuGVK7nzHBRXSkmGRRiZNqvz1N9GK";

    const originatorAgent = new TAPAgent({
      did: originatorDid,
      signer: createSigner("originator-private-key", originatorDid),
    });

    const beneficiaryAgent = new TAPAgent({
      did: beneficiaryDid,
      signer: createSigner("beneficiary-private-key", beneficiaryDid),
    });

    console.log(
      `Originator agent created with DID: ${originatorAgent.getDID()}`,
    );
    console.log(
      `Beneficiary agent created with DID: ${beneficiaryAgent.getDID()}`,
    );

    // Create a transfer message
    console.log("\n2. Creating transfer message...");

    const transfer = new Transfer({
      asset: "eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC on Ethereum
      amount: "100.50",
      originator: {
        "@id": originatorDid,
        "@type": "Party",
        role: "originator",
      },
      beneficiary: {
        "@id": beneficiaryDid,
        "@type": "Party",
        role: "beneficiary",
      },
      agents: [
        { "@id": originatorDid, "@type": "Agent" },
        { "@id": beneficiaryDid, "@type": "Agent" },
      ],
      memo: "Payment for services",
    });

    console.log(`Transfer created with ID: ${transfer.id}`);
    console.log(`Asset: ${transfer.body.asset}`);
    console.log(`Amount: ${transfer.body.amount}`);

    // Sign and send the transfer
    console.log("\n3. Signing and sending transfer...");

    await originatorAgent.sign(transfer);
    console.log("Transfer signed by originator");

    await sendMessage(transfer, "https://beneficiary.example/endpoint");

    // On the beneficiary side...
    console.log("\n4. Beneficiary receives and processes transfer...");

    // In a real scenario, the beneficiary would receive the message via their endpoint
    const receivedTransfer = transfer; // Simulating receipt

    const isValid = await beneficiaryAgent.verify(receivedTransfer);
    console.log(
      `Transfer verification result: ${isValid ? "Valid" : "Invalid"}`,
    );

    if (isValid) {
      // Create an authorization response
      console.log("\n5. Beneficiary authorizes transfer...");

      const authorize = receivedTransfer.authorize(
        "eip155:1:0x742d35Cc6634C0532925a3b844Bc454e4438f44e", // Settlement address
        "Compliance checks passed",
      );

      console.log(`Authorization created with ID: ${authorize.id}`);
      console.log(`Thread ID: ${authorize.thid}`);
      console.log(`Settlement Address: ${authorize.settlementAddress}`);

      // Sign and send the authorization
      await beneficiaryAgent.sign(authorize);
      console.log("Authorization signed by beneficiary");

      await sendMessage(authorize, "https://originator.example/endpoint");

      // Originator settles on-chain and sends settlement confirmation
      console.log("\n6. Originator settles on-chain and confirms...");

      // In a real scenario, the originator would execute an on-chain transaction
      // For this example, we'll just simulate the process
      console.log("[Simulating on-chain settlement transaction...]");

      const settlementTxId =
        "eip155:1/tx/0x4a563af33c4871b51a8b108aa2fe1dd5280a30dfb7236170ae5e5e7957eb6392";

      const settle = receivedTransfer.settle(
        settlementTxId,
        transfer.body.amount,
      );

      console.log(`Settlement created with ID: ${settle.id}`);
      console.log(`Thread ID: ${settle.thid}`);
      console.log(`Settlement Transaction: ${settle.settlementId}`);

      // Sign and send the settlement confirmation
      await originatorAgent.sign(settle);
      console.log("Settlement signed by originator");

      await sendMessage(settle, "https://beneficiary.example/endpoint");

      console.log("\nTransfer flow completed successfully!");
    } else {
      console.error("Transfer verification failed!");
    }
  } catch (error) {
    console.error("Error during transfer flow:", error);
  }
}

// Run the example
main().catch(console.error);
