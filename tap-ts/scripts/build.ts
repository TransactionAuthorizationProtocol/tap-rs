/**
 * Build script for TAP-TS
 * 
 * This script builds the TypeScript code and WASM bindings
 */

import { ensureDir, emptyDir, exists } from "@std/fs/mod.ts";
import { join } from "@std/path/mod.ts";

// Ensure dist directory exists
const distDir = join(Deno.cwd(), "dist");
await ensureDir(distDir);
await emptyDir(distDir);

console.log("Building TAP-TS...");

// First, build the WASM module
console.log("Building WASM module...");
const wasmBuildProcess = new Deno.Command(Deno.execPath(), {
  args: ["task", "wasm"],
  stdout: "inherit",
  stderr: "inherit",
});

const wasmBuildStatus = await wasmBuildProcess.output();
if (!wasmBuildStatus.success) {
  console.error("Failed to build WASM module");
  Deno.exit(1);
}

// Now bundle the TypeScript code
console.log("Bundling TypeScript code...");
const bundleProcess = new Deno.Command("deno", {
  args: [
    "run",
    "--allow-read",
    "--allow-write",
    "--allow-env",
    "--allow-run",
    "npm:esbuild",
    "./src/mod.ts",
    "--bundle",
    "--outfile=./dist/tap-ts.js",
    "--format=esm",
  ],
  stdout: "inherit",
  stderr: "inherit",
});

const bundleStatus = await bundleProcess.output();
if (!bundleStatus.success) {
  console.error("Failed to bundle TypeScript code");
  Deno.exit(1);
}

// Create the TypeScript type definitions
console.log("Generating type definitions...");

// Copy type definitions from WASM bindgen output
const wasmBindgenDir = join(Deno.cwd(), "src", "wasm", "bindgen");
const wasmTypesPath = join(wasmBindgenDir, "tap_ts_wasm.d.ts");

if (await exists(wasmTypesPath)) {
  const wasmTypes = await Deno.readTextFile(wasmTypesPath);
  await Deno.writeTextFile(join(distDir, "tap-ts.d.ts"), wasmTypes);
} else {
  console.error("WASM types file not found at", wasmTypesPath);
  Deno.exit(1);
}

// Create a package.json for npm compatibility
console.log("Creating package.json...");
const packageJson = {
  name: "tap-ts",
  version: "0.1.0",
  description: "Transaction Authorization Protocol TypeScript Implementation",
  type: "module",
  main: "./dist/tap-ts.js",
  types: "./dist/tap-ts.d.ts",
  files: [
    "dist",
    "README.md",
    "LICENSE"
  ],
  author: "Notabene",
  license: "MIT",
};

await Deno.writeTextFile(
  join(Deno.cwd(), "package.json"),
  JSON.stringify(packageJson, null, 2)
);

console.log("Build completed successfully!");
