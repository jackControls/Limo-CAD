export const UI_SCALE_STORAGE_KEY = 'nbcad.uiScale';
export const DEFAULT_UI_SCALE = 1;
// Discrete steps instead of a slider: every step re-lays out the whole window,
// including the settings dialog, so a dragged thumb would jump under the cursor.
export const UI_SCALE_OPTIONS = [0.9, 1, 1.1, 1.25, 1.5, 1.75] as const;
const MIN_UI_SCALE = UI_SCALE_OPTIONS[0];
const MAX_UI_SCALE = UI_SCALE_OPTIONS[UI_SCALE_OPTIONS.length - 1];

export function clampUiScale(value: number): number {
  if (!Number.isFinite(value)) return DEFAULT_UI_SCALE;
  return Math.min(MAX_UI_SCALE, Math.max(MIN_UI_SCALE, value));
}

export function readUiScale(): number {
  if (typeof window === 'undefined') return DEFAULT_UI_SCALE;
  try {
    const stored = Number.parseFloat(
      window.localStorage.getItem(UI_SCALE_STORAGE_KEY) ?? '',
    );
    return Number.isFinite(stored) ? clampUiScale(stored) : DEFAULT_UI_SCALE;
  } catch {
    return DEFAULT_UI_SCALE;
  }
}

export function persistUiScale(value: number): number {
  const clamped = clampUiScale(value);
  if (typeof window === 'undefined') return clamped;
  try {
    window.localStorage.setItem(UI_SCALE_STORAGE_KEY, String(clamped));
  } catch {
    // A locked-down webview can deny storage. The live preference still works.
  }
  return clamped;
}

/**
 * The browser/WASM product has no embedded native viewport coordinate space.
 * Keep the saved preference API without importing the retired desktop bridge.
 */
export function currentUiScale(): number {
  return DEFAULT_UI_SCALE;
}

/**
 * Browser builds keep the browser's own zoom controls. The Bevy desktop has
 * its own native UI and does not consume browser/webview zoom preferences.
 */
export function applyUiScale(_value: number): Promise<void> {
  return Promise.resolve();
}
