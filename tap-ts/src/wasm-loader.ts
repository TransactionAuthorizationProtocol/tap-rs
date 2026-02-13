/**
 * WASM loader module that handles both development and production environments
 */

let wasmModule: any = null;
let wasmExports: any = null;

/**
 * Helper to load WASM binary from file system (Node.js only)
 */
async function loadWasmBinaryFromPaths(paths: string[]): Promise<Buffer | undefined> {
  const { readFileSync } = await import('fs');

  for (const wasmPath of paths) {
    try {
      return readFileSync(wasmPath);
    } catch {
      // Try next path
    }
  }
  return undefined;
}

/**
 * Helper to get the directory name for the current module
 */
async function getModuleDir(): Promise<string> {
  try {
    const { fileURLToPath } = await import('url');
    const { dirname } = await import('path');
    const __filename = fileURLToPath(import.meta.url);
    return dirname(__filename);
  } catch {
    return process.cwd();
  }
}

/**
 * Initialize the WASM module
 * In production, loads from bundled wasm directory
 * In development, falls back to tap-wasm package
 */
export async function initWasm(): Promise<any> {
  if (wasmModule) {
    return wasmModule;
  }

  const isNodeJs = typeof window === 'undefined';

  try {
    // Try bundled WASM first (production / npm install path)
    const bundledModule = await import('../wasm/tap_wasm.js');
    const init = bundledModule.default;

    if (isNodeJs) {
      // In Node.js, we need to pass the WASM binary since fetch() with import.meta.url doesn't work
      const { join } = await import('path');
      const __dirname = await getModuleDir();

      const possiblePaths = [
        join(__dirname, '../wasm/tap_wasm_bg.wasm'),
        join(process.cwd(), 'wasm/tap_wasm_bg.wasm'),
      ];

      const wasmBinary = await loadWasmBinaryFromPaths(possiblePaths);
      if (!wasmBinary) {
        throw new Error('Could not find bundled WASM file');
      }
      wasmModule = await init(wasmBinary);
    } else {
      // In browser, init() can fetch the WASM file itself
      wasmModule = await init();
    }
    wasmExports = bundledModule;
  } catch (bundledError) {
    // Fall back to development path (tap-wasm package)
    try {
      const tapWasmModule = await import('tap-wasm');
      const init = tapWasmModule.default;

      if (!isNodeJs) {
        // Browser environment
        wasmModule = await init();
      } else {
        // Node.js environment - provide WASM binary
        const { join } = await import('path');
        const __dirname = await getModuleDir();

        const possiblePaths = [
          join(__dirname, '../../tap-wasm/pkg/tap_wasm_bg.wasm'),
          join(__dirname, '../node_modules/tap-wasm/tap_wasm_bg.wasm'),
          join(process.cwd(), '../tap-wasm/pkg/tap_wasm_bg.wasm'),
          join(process.cwd(), 'node_modules/tap-wasm/tap_wasm_bg.wasm'),
        ];

        const wasmBinary = await loadWasmBinaryFromPaths(possiblePaths);
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
