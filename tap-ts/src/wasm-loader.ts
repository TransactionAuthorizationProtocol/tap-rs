/**
 * WASM loader module that handles both development and production environments
 */

let wasmModule: any = null;
let wasmExports: any = null;

/**
 * Initialize the WASM module
 * In development, loads from tap-wasm package
 * In production, loads from bundled wasm directory
 */
export async function initWasm(): Promise<any> {
  if (wasmModule) {
    return wasmModule;
  }

  try {
    // Try development path first (tap-wasm package)
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
      
      const __filename = fileURLToPath(import.meta.url);
      const __dirname = dirname(__filename);
      const wasmPath = join(__dirname, '../../tap-wasm/pkg/tap_wasm_bg.wasm');
      
      const wasmBinary = readFileSync(wasmPath);
      wasmModule = await init(wasmBinary);
    }
    
    // Store the exports for later use
    wasmExports = tapWasmModule;
  } catch (devError) {
    // Fall back to production path (bundled WASM)
    try {
      const bundledModule = await import('../wasm/tap_wasm.js');
      const init = bundledModule.default;
      
      if (typeof window !== 'undefined') {
        // Browser environment
        wasmModule = await init();
      } else {
        // Node.js environment
        const { readFileSync } = await import('fs');
        const { join } = await import('path');
        const { fileURLToPath } = await import('url');
        const { dirname } = await import('path');
        
        const __filename = fileURLToPath(import.meta.url);
        const __dirname = dirname(__filename);
        const wasmPath = join(__dirname, '../wasm/tap_wasm_bg.wasm');
        
        const wasmBinary = readFileSync(wasmPath);
        wasmModule = await init(wasmBinary);
      }
      
      // Store the exports for later use
      wasmExports = bundledModule;
    } catch (prodError) {
      throw new Error(
        'Failed to load WASM module. Make sure tap-wasm is built or installed.\n' +
        `Development error: ${devError}\n` +
        `Production error: ${prodError}`
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