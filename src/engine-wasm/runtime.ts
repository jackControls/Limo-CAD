type Bindings = typeof import('./pkg/nbcad_wasm');
let initialization: Promise<Bindings> | null = null;
let initialized: Bindings | null = null;

/** The engine, archive codec and triangulator share one WASM instance. */
export function ensureWasm(): Promise<Bindings> {
  initialization ??= import('./pkg/nbcad_wasm').then(async (bindings) => {
    await bindings.default();
    initialized = bindings;
    return bindings;
  }).catch((error: unknown) => {
    initialization = null;
    throw error;
  });
  return initialization;
}

/** Synchronous geometry helpers run only after the engine has initialized. */
export function wasmBindings(): Bindings {
  if (!initialized) throw new Error('The browser engine must initialize before triangulation');
  return initialized;
}
