/**
 * Performance benchmarks for the TAP-TS library
 * 
 * Run with: deno run --allow-read --allow-net --allow-env benchmark/benchmark.ts
 */

import { wasmLoader, Message, MessageType, Agent, TapNode } from "../src/mod.ts";

// Define a simple benchmarking function
async function benchmark(name: string, fn: () => void | Promise<void>, iterations = 1000): Promise<number> {
  console.log(`Running benchmark: ${name} (${iterations} iterations)`);
  
  const start = performance.now();
  
  for (let i = 0; i < iterations; i++) {
    const result = fn();
    if (result instanceof Promise) {
      await result;
    }
  }
  
  const end = performance.now();
  const duration = end - start;
  const opsPerSec = Math.floor((iterations / duration) * 1000);
  
  console.log(`  Completed in ${duration.toFixed(2)}ms (${opsPerSec.toLocaleString()} ops/sec)`);
  return opsPerSec;
}

// Helpers
async function runBenchmarks() {
  console.log('TAP-TS Performance Benchmarks');
  console.log('=============================');
  
  // Wait for WASM to load
  console.log('Loading WASM module...');
  await wasmLoader.load();
  console.log('WASM module loaded\n');
  
  // Run each benchmark suite
  await runMessageSuite();
  await runAgentSuite();
  await runNodeSuite();
  
  console.log('\nAll benchmarks completed');
}

// Message Benchmarking Suite
async function runMessageSuite() {
  console.log('Message Benchmarks:');
  console.log('-----------------');
  
  // Setup test data
  const aliceDID = "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH";
  const bobDID = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
  
  // Create message instances for benchmarking
  const authRequest = new Message({
    type: MessageType.AUTHORIZATION_REQUEST,
  });
  
  authRequest.setAuthorizationRequestData({
    transactionHash: "0x1234567890abcdef",
    sender: "0xAliceSender",
    receiver: "0xBobReceiver",
    amount: "100.0",
    asset: "BTC",
  });
  
  const transfer = new Message({
    type: MessageType.TRANSFER,
  });
  
  transfer.setTransferData({
    asset: "eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F",
    amount: "100.0",
    originatorDid: aliceDID,
    beneficiaryDid: bobDID,
    memo: "Payment for services"
  });
  
  // Benchmarks
  await benchmark('Create message', () => {
    const msg = new Message({
      type: MessageType.AUTHORIZATION_REQUEST,
    });
    msg.setAuthorizationRequestData({
      transactionHash: "0x1234567890abcdef",
      sender: "0xSender",
      receiver: "0xReceiver",
      amount: "100.0",
      asset: "BTC",
    });
  });
  
  await benchmark('Serialize message to JSON', () => {
    const json = authRequest.toJSON();
  });
  
  const jsonStr = authRequest.toJSON();
  await benchmark('Deserialize message from JSON', () => {
    const msg = Message.fromJSON(jsonStr);
  });
  
  await benchmark('Create transfer message', () => {
    const msg = new Message({
      type: MessageType.TRANSFER,
    });
    msg.setTransferData({
      asset: "eip155:1/erc20:0xToken",
      amount: "10.0",
      originatorDid: aliceDID,
      beneficiaryDid: bobDID,
    });
  });
}

// Agent Benchmarking Suite
async function runAgentSuite() {
  console.log('\nAgent Benchmarks:');
  console.log('----------------');
  
  // Setup test data
  const alice = new Agent({
    nickname: "Alice"
  });
  
  const bob = new Agent({
    nickname: "Bob"
  });
  
  const authRequest = new Message({
    type: MessageType.AUTHORIZATION_REQUEST,
  });
  
  authRequest.setAuthorizationRequestData({
    transactionHash: "0x1234567890abcdef",
    sender: "0xAliceSender",
    receiver: "0xBobReceiver",
    amount: "100.0",
    asset: "BTC",
  });
  
  // Benchmarks
  await benchmark('Create agent', () => {
    const agent = new Agent({
      nickname: "TestAgent"
    });
  });
  
  await benchmark('Get agent DID', () => {
    const did = alice.did;
  });
  
  // Mock send message (don't actually send)
  await benchmark('Prepare message', () => {
    // We can't directly call private method, so simulate it
    const msg = authRequest.toJSON();
    const parsed = JSON.parse(msg);
  });
}

// Node Benchmarking Suite
async function runNodeSuite() {
  console.log('\nNode Benchmarks:');
  console.log('---------------');
  
  // Setup test data
  const node = new TapNode({
    debug: false
  });
  
  const alice = new Agent({
    nickname: "Alice"
  });
  
  node.registerAgent(alice);
  
  // Benchmarks
  await benchmark('Create node', () => {
    const n = new TapNode({
      debug: false
    });
  });
  
  // This benchmark will run fewer iterations since it does more work
  await benchmark('Register and unregister agent', async () => {
    const a = new Agent({
      nickname: "TestAgent"
    });
    node.registerAgent(a);
    node.unregisterAgent(a.did);
  }, 100);
}

// Run all benchmarks
await runBenchmarks();
