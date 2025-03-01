/**
 * Build script for TAP-TS
 * 
 * This script builds the TypeScript code and WASM bindings
 */

import { ensureDir, emptyDir } from "@std/fs/mod.ts";
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
const bundleProcess = new Deno.Command(Deno.execPath(), {
  args: [
    "bundle",
    "--config", "./deno.json",
    "./src/mod.ts",
    "./dist/tap-ts.js",
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
const typesProcess = new Deno.Command(Deno.execPath(), {
  args: [
    "types",
    "--unstable",
    "./src/mod.ts",
  ],
  stdout: "piped",
  stderr: "inherit",
});

const typesOutput = await typesProcess.output();
if (!typesOutput.success) {
  console.error("Failed to generate type definitions");
  Deno.exit(1);
}

// Write the type definitions to a file
const typeDefs = new TextDecoder().decode(typesOutput.stdout);
await Deno.writeTextFile(join(distDir, "tap-ts.d.ts"), typeDefs);

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
