import { readUiScale, snapUiScale, stepUiScale, UI_SCALE_OPTIONS, UI_SCALE_STORAGE_KEY } from './uiScale';

function same(actual: unknown, expected: unknown, message: string) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`${message}: ${JSON.stringify(actual)} !== ${JSON.stringify(expected)}`);
  }
}

same(snapUiScale(Number.NaN), 1, 'A non-number falls back to 100%');
same(snapUiScale(1.12), 1.1, 'A value between steps snaps to the nearer step');
same(snapUiScale(1.2), 1.25, 'The upper neighbour wins when it is closer');
same(snapUiScale(9), 1.75, 'A huge stored value snaps to the largest step');
same(snapUiScale(0.01), 0.9, 'A tiny stored value snaps to the smallest step');
same(stepUiScale(1, 1), 1.1, 'Ctrl+Plus moves to the next larger step');
same(stepUiScale(1, -1), 0.9, 'Ctrl+Minus moves to the next smaller step');
same(stepUiScale(0.9, -1), 0.9, 'The smallest step does not wrap');
same(stepUiScale(1.75, 1), 1.75, 'The largest step does not wrap');
same(UI_SCALE_OPTIONS.length, 6, 'The dialog and the shortcuts share one step list');

const memory = new Map<string, string>();
const storage = {
  getItem: (key: string) => memory.get(key) ?? null,
  setItem: (key: string, value: string) => { memory.set(key, value); },
  removeItem: (key: string) => { memory.delete(key); },
  clear: () => { memory.clear(); },
  key: (index: number) => [...memory.keys()][index] ?? null,
  get length() { return memory.size; },
};
Object.defineProperty(globalThis, 'localStorage', { configurable: true, value: storage });
Object.defineProperty(globalThis, 'window', { configurable: true, value: globalThis });
memory.set(UI_SCALE_STORAGE_KEY, '1.33');
same(readUiScale(), 1.25, 'A hand-edited preference snaps onto a real step');

console.log('Interface size steps passed.');
