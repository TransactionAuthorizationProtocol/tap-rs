/**
 * This script copies the WASM files from the tap-wasm/pkg directory
 * to the appropriate location in the tap-ts distribution folder
 */

const fs = require('fs');
const path = require('path');

// Source and destination paths
const sourceWasmDir = path.resolve(__dirname, '../../tap-wasm/pkg');
const destWasmDir = path.resolve(__dirname, '../dist/wasm');

// Create destination directory if it doesn't exist
if (!fs.existsSync(destWasmDir)) {
  fs.mkdirSync(destWasmDir, { recursive: true });
}

// Files to copy
const filesToCopy = [
  'tap_wasm_bg.wasm',
  'tap_wasm.js',
  'tap_wasm.d.ts',
  'tap_wasm_bg.wasm.d.ts',
];

// Copy files
for (const file of filesToCopy) {
  const sourcePath = path.join(sourceWasmDir, file);
  const destPath = path.join(destWasmDir, file);
  
  if (fs.existsSync(sourcePath)) {
    fs.copyFileSync(sourcePath, destPath);
    console.log(`Copied ${file} to ${destWasmDir}`);
  } else {
    console.error(`Error: ${sourcePath} does not exist`);
    process.exit(1);
  }
}

// Create browser.js and node.js entry points
const browserJS = `
import * as wasm from './tap_wasm.js';

/**
 * Initialize the WASM module
 * @returns {Promise<void>}
 */
export async function initialize() {
  await wasm.default();
  wasm.init_tap_wasm();
  return wasm;
}

export default { initialize };
`;

const nodeJS = `
const wasm = require('./tap_wasm.js');

/**
 * Initialize the WASM module
 * @returns {Promise<void>}
 */
async function initialize() {
  await wasm.default();
  wasm.init_tap_wasm();
  return wasm;
}

module.exports = { initialize };
`;

fs.writeFileSync(path.join(destWasmDir, 'browser.js'), browserJS);
console.log('Created browser.js');

fs.writeFileSync(path.join(destWasmDir, 'node.js'), nodeJS);
console.log('Created node.js');

console.log('WASM files successfully prepared for distribution');