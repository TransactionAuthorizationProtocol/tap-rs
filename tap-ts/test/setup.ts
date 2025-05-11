// This file will be automatically loaded by Vitest before running tests
// We can use it to set up global mocks and test environment

import { vi } from 'vitest';

// Mock the WASM bridge functions
vi.mock('../src/wasm/bridge', () => ({
  initialize: vi.fn().mockResolvedValue({}),
  getWasmModule: vi.fn().mockResolvedValue({}),
  createMessage: vi.fn().mockImplementation((id, type, version) => ({ id, type, version })),
  createAgent: vi.fn().mockResolvedValue({
    sign: vi.fn(),
    verify: vi.fn().mockResolvedValue(true)
  }),
  generateUuid: vi.fn().mockResolvedValue('test-uuid-v4'),
  createDidKey: vi.fn().mockResolvedValue('did:key:test')
}));

// Mock any other external dependencies here