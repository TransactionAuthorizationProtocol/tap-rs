#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// Paths
const wasmPkgDir = path.resolve(__dirname, '../../tap-wasm/pkg');
const nodeModulesDir = path.resolve(__dirname, '../node_modules');
const wasmDistDir = path.resolve(nodeModulesDir, 'tap-wasm');

// Check if directories exist
if (!fs.existsSync(wasmPkgDir)) {
  console.error('Error: tap-wasm/pkg directory does not exist. Please build the WASM package first.');
  process.exit(1);
}

// Ensure node_modules directory exists
if (!fs.existsSync(nodeModulesDir)) {
  console.log('Creating node_modules directory...');
  fs.mkdirSync(nodeModulesDir, { recursive: true });
}

// Ensure tap-wasm directory exists in node_modules
if (!fs.existsSync(wasmDistDir)) {
  console.log('Creating tap-wasm directory in node_modules...');
  fs.mkdirSync(wasmDistDir, { recursive: true });
}

console.log('WASM directories exist and are ready to use.');