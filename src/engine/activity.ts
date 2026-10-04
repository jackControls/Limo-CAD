export type EngineOperationOwner = symbol;
const pending = new Map<EngineOperationOwner, 'read' | 'mutation'>();
const listeners = new Set<() => void>();
export function subscribeEngineOperations(listener: () => void): () => void {
  listeners.add(listener);
  return () => { listeners.delete(listener); };
}
function changed(): void { for (const listener of listeners) listener(); }
export function pendingEngineOperations(owner?: EngineOperationOwner): number {
  return pending.size - (owner && pending.has(owner) ? 1 : 0);
}
export function pendingEngineMutations(owner?: EngineOperationOwner): number {
  let count = 0;
  for (const [operation, kind] of pending) {
    if (kind === 'mutation' && operation !== owner) count++;
  }
  return count;
}
export function trackEngineOperation<T>(operation: Promise<T> | ((owner: EngineOperationOwner) => Promise<T>)): Promise<T> {
  return track(operation, 'mutation');
}
export function trackEngineRead<T>(operation: Promise<T>): Promise<T> {
  return track(operation, 'read');
}
async function track<T>(operation: Promise<T> | ((owner: EngineOperationOwner) => Promise<T>), kind: 'read' | 'mutation'): Promise<T> {
  const owner = Symbol('engine operation');
  pending.set(owner, kind);
  changed();
  try { return await (typeof operation === 'function' ? operation(owner) : operation); }
  finally { pending.delete(owner); changed(); }
}
