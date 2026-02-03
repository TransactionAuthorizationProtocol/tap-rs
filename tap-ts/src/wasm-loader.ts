/**
 * WASM loader module that handles both development and production environments
 */

let wasmModule: any = null;
let wasmExports: any = null;

/**
 * Initialize the WASM module
 * In production, loads from bundled wasm directory (with inlined base64 WASM)
 * In development, falls back to tap-wasm package
 */
export async function initWasm(): Promise<any> {
  if (wasmModule) {
    return wasmModule;
  }

  try {
    // Try bundled WASM first (production / npm install path)
    // The bundled JS has the WASM binary inlined as base64, so init() works without arguments
    const bundledModule = await import('../wasm/tap_wasm.js');
    const init = bundledModule.default;
    wasmModule = await init();
    wasmExports = bundledModule;
  } catch (bundledError) {
    // Fall back to development path (tap-wasm package)
    try {
      const tapWasmModule = await import('tap-wasm');
      const init = tapWasmModule.default;

      if (typeof window !== 'undefined') {
        // Browser environment
        wasmModule = await init();
      } else {
        // Node.js environment - provide WASM binary
        const { readFileSync } = await import('fs');
        const { join } = await import('path');
        const { fileURLToPath } = await import('url');
        const { dirname } = await import('path');

        let __dirname: string;
        try {
          const __filename = fileURLToPath(import.meta.url);
          __dirname = dirname(__filename);
        } catch {
          __dirname = process.cwd();
        }

        const possiblePaths = [
          join(__dirname, '../../tap-wasm/pkg/tap_wasm_bg.wasm'),
          join(__dirname, '../node_modules/tap-wasm/tap_wasm_bg.wasm'),
          join(process.cwd(), '../tap-wasm/pkg/tap_wasm_bg.wasm'),
          join(process.cwd(), 'node_modules/tap-wasm/tap_wasm_bg.wasm'),
        ];

        let wasmBinary: Buffer | undefined;
        for (const wasmPath of possiblePaths) {
          try {
            wasmBinary = readFileSync(wasmPath);
            break;
          } catch {
            // Try next path
          }
        }

        if (!wasmBinary) {
          throw new Error('Could not find WASM file in any of the expected locations');
        }

        wasmModule = await init(wasmBinary);
      }

      wasmExports = tapWasmModule;
    } catch (devError) {
      throw new Error(
        'Failed to load WASM module. Make sure tap-wasm is built or installed.\n' +
        `Bundled error: ${bundledError}\n` +
        `Development error: ${devError}`
      );
    }
  }

  return wasmModule;
}

/**
 * Export all WASM exports for use in the application
 */
export async function getWasmExports() {
  await initWasm();

  if (!wasmExports) {
    throw new Error('WASM module not properly initialized');
  }

  return wasmExports;
}
