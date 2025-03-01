/**
 * Tests for DID resolver integration
 */

import { assertEquals, assertExists } from "@std/assert/mod.ts";
import { resolveDID, canResolveDID, didResolver } from "../src/did/mod.ts";

Deno.test("DID Resolver Integration", async (t) => {
  await t.step("canResolveDID should return true for supported methods", () => {
    assertEquals(canResolveDID("did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH"), true);
    assertEquals(canResolveDID("did:web:example.com"), true);
    assertEquals(canResolveDID("did:pkh:eip155:1:0x1234567890123456789012345678901234567890"), true);
  });

  await t.step("canResolveDID should return false for unsupported methods", () => {
    assertEquals(canResolveDID("did:unsupported:test"), false);
    assertEquals(canResolveDID("invalid"), false);
  });

  await t.step("resolveDID should resolve did:key", async () => {
    const did = "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH";
    const result = await resolveDID(did);
    
    assertEquals(result.didDocument.id, did);
    assertEquals(result.didResolutionMetadata.error, undefined);
    assertExists(result.didDocument.verificationMethod);
    assertExists(result.didDocument.authentication);
  });

  await t.step("resolveDID should handle invalid DIDs", async () => {
    try {
      await resolveDID("invalid:did");
      Deno.test.fail("Should have thrown an error");
    } catch (error) {
      assertEquals(error.type, "DID_RESOLUTION_ERROR");
    }
  });

  await t.step("didResolver should be a valid Resolver instance", () => {
    assertExists(didResolver);
    assertExists(didResolver.resolve);
  });
});
