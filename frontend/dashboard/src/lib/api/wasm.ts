/** RLMX WASM Runtime — @ruvector/ruvllm-wasm v2.0.2 integration */

export interface EdgeCapabilities {
  wasm: boolean;
  simd: boolean;
  webgpu: boolean;
  sharedMem: boolean;
  workers: boolean;
  mode: string;
}

let wasmMod: any = null;
let ruvllmInstance: any = null;

export async function detectCapabilities(): Promise<EdgeCapabilities> {
  const caps: EdgeCapabilities = {
    wasm: typeof WebAssembly !== 'undefined',
    simd: false,
    webgpu: false,
    sharedMem: typeof SharedArrayBuffer !== 'undefined',
    workers: typeof Worker !== 'undefined',
    mode: 'http',
  };

  if ((navigator as any).gpu) {
    try { caps.webgpu = !!(await (navigator as any).gpu.requestAdapter()); } catch {}
  }

  try {
    const simdTest = new Uint8Array([0,97,115,109,1,0,0,0,1,5,1,96,0,1,123,3,2,1,0,10,10,1,8,0,65,0,253,15,253,98,11]);
    caps.simd = WebAssembly.validate(simdTest);
  } catch {}

  caps.mode = caps.webgpu ? 'wasm-webgpu'
    : caps.simd ? 'wasm-cpu'
    : caps.wasm ? 'wasm-basic'
    : 'http';

  return caps;
}

export async function loadWasm(): Promise<boolean> {
  if (wasmMod) return true;
  try {
    // Dynamic import at runtime — bypasses Vite's static analysis
    const url = new URL('/ruvllm-wasm/ruvllm_wasm.js', window.location.origin).href;
    wasmMod = await Function('u', 'return import(u)')(url);
    if (typeof wasmMod.default === 'function') await wasmMod.default();
    return true;
  } catch {
    return false;
  }
}

export async function initRuvllm(): Promise<boolean> {
  if (!await loadWasm()) return false;
  try {
    ruvllmInstance = new wasmMod.RuvLLMWasm();
    ruvllmInstance.initialize();
    return true;
  } catch {
    return false;
  }
}

export function getWasmModule(): any { return wasmMod; }
export function getRuvllm(): any { return ruvllmInstance; }
