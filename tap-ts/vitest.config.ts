import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'node',
    include: ['**/*.{test,spec}.{js,ts}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: ['**/node_modules/**', '**/dist/**', '**/*.d.ts'],
    },
    // Increase timeout for tests since we're using real WASM
    testTimeout: 10000,
    // Ensure WASM files can be properly loaded during tests
    deps: {
      inline: [/tap-wasm/],
    },
  },
});