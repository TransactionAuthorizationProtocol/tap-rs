/**
 * Type declarations for Deno
 */

// Using augmentation to extend the Deno namespace without redeclaring the test function
declare namespace Deno {
  // The test function is already declared in lib.deno.ns.d.ts
  // We're just ensuring TestContext is available for our code
}

interface TestContext {
  name: string;
  step(name: string, fn: () => void | Promise<void>): Promise<void>;
}

declare module "https://deno.land/std@0.177.0/testing/asserts.ts" {
  export function assertEquals(actual: any, expected: any, msg?: string): void;
  export function assertExists(actual: any, msg?: string): void;
  export function assertThrows(fn: () => void, ErrorClass?: any, msgIncludes?: string, msg?: string): Error;
}
