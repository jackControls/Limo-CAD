type Bindings = typeof import('./pkg/nbcad_wasm');
let initialization: Promise<Bindings> | null = null;

/** The engine, archive codec and triangulator share one WASM instance. */
export function ensureWasm(): Promise<Bindings> {
  initialization ??= import('./pkg/nbcad_wasm').then(async (bindings) => {
    await bindings.default();
    return bindings;
  }).catch((error: unknown) => {
    initialization = null;
    throw error;
  });
  return initialization;
}
