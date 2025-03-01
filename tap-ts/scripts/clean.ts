/**
 * Clean script for TAP-TS
 * 
 * This script removes generated files and directories
 */

import { exists } from "@std/fs/mod.ts";
import { join } from "@std/path/mod.ts";

// Directories to clean
const dirsToClean = [
  "dist",
  "src/wasm/bindgen",
  "src/wasm/target",
  ".deno",
];

// Log function with timestamp
function log(message: string): void {
  console.log(`[${new Date().toISOString()}] ${message}`);
}

// Remove a directory or file if it exists
async function removeIfExists(path: string): Promise<void> {
  const fullPath = join(Deno.cwd(), path);
  if (await exists(fullPath)) {
    log(`Removing ${path}...`);
    try {
      await Deno.remove(fullPath, { recursive: true });
      log(`Successfully removed ${path}`);
    } catch (error) {
      console.error(`Error removing ${path}: ${error.message}`);
    }
  } else {
    log(`${path} does not exist, skipping`);
  }
}

// Main cleaning function
async function clean(): Promise<void> {
  log("Starting cleanup...");
  
  // Clean all directories in parallel
  await Promise.all(dirsToClean.map(removeIfExists));
  
  log("Cleanup completed");
}

// Run the clean function
await clean();
