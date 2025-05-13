#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// Paths
const wasmPkgDir = path.resolve(__dirname, '../../tap-wasm/pkg');
const wasmDistDir = path.resolve(__dirname, '../node_modules/tap-wasm');

// Files to copy
const filesToCopy = [
  'tap_wasm_bg.wasm',
  'tap_wasm.js',
  'tap_wasm.d.ts',
  'tap_wasm_bg.wasm.d.ts',
  'package.json'
];

// Create directories if they don't exist
function ensureDirectoryExists(directory) {
  if (!fs.existsSync(directory)) {
    fs.mkdirSync(directory, { recursive: true });
    console.log(`Created directory: ${directory}`);
  }
}

// Copy files
function copyFiles() {
  ensureDirectoryExists(wasmDistDir);

  filesToCopy.forEach(file => {
    const sourcePath = path.join(wasmPkgDir, file);
    const destPath = path.join(wasmDistDir, file);

    if (!fs.existsSync(sourcePath)) {
      console.error(`Source file does not exist: ${sourcePath}`);
      return;
    }

    try {
      fs.copyFileSync(sourcePath, destPath);
      console.log(`Copied: ${file}`);
    } catch (error) {
      console.error(`Error copying ${file}: ${error.message}`);
    }
  });
}

// Main
console.log('Copying WASM files to the tap-ts package...');
copyFiles();
console.log('Done!');