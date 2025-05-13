#!/usr/bin/env node

const { execSync } = require('child_process');
const path = require('path');

// Path to the tap-wasm directory
const wasmDir = path.resolve(__dirname, '../../tap-wasm');

// Run wasm-pack build
function buildWasm() {
  console.log('Building tap-wasm package...');
  
  try {
    // Change to the tap-wasm directory
    process.chdir(wasmDir);
    
    // Run wasm-pack build
    execSync('wasm-pack build --target web', { stdio: 'inherit' });
    
    console.log('Successfully built tap-wasm package!');
  } catch (error) {
    console.error('Error building tap-wasm package:', error.message);
    process.exit(1);
  }
}

// Main
buildWasm();