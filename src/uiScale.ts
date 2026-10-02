import { getCurrentWebview } from '@tauri-apps/api/webview';
import { isTauriRuntime } from './engine';

export const UI_SCALE_STORAGE_KEY = 'nbcad.uiScale';
export const DEFAULT_UI_SCALE = 1;
// Discrete steps instead of a slider: every step re-lays out the whole window,
// including the settings dialog, so a dragged thumb would jump under the cursor.
export const UI_SCALE_OPTIONS = [0.9, 1, 1.1, 1.25, 1.5, 1.75] as const;
const MIN_UI_SCALE = UI_SCALE_OPTIONS[0];
const MAX_UI_SCALE = UI_SCALE_OPTIONS[UI_SCALE_OPTIONS.length - 1];

let appliedUiScale = DEFAULT_UI_SCALE;
let pendingApply: Promise<void> = Promise.resolve();

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
 * Ratio of one CSS pixel to one native logical pixel. The native viewport
 * host positions its surface and draws its HUD in native logical pixels, so
 * every DOM rectangle it receives must be multiplied by this factor.
 */
export function currentUiScale(): number {
  return appliedUiScale;
}

/**
 * Zooms the whole desktop webview. Page zoom (unlike CSS `zoom`) shrinks the
 * CSS viewport, so media queries, `100vh` and hit testing stay consistent.
 * Browser builds keep the browser's own zoom controls.
 */
export function applyUiScale(value: number): Promise<void> {
  const scale = clampUiScale(value);
  if (!isTauriRuntime()) return Promise.resolve();
  // Serialise calls so a quick series of choices cannot finish out of order
  // and leave the native viewport using a factor the page no longer has.
  pendingApply = pendingApply
    .catch(() => undefined)
    .then(() => getCurrentWebview().setZoom(scale))
    .then(() => {
      appliedUiScale = scale;
      // The CSS geometry changes on the next layout pass. Ask the native
      // viewport bridge to resend it with the new factor afterwards.
      requestAnimationFrame(() =>
        document.dispatchEvent(new Event('nbcad:native-viewport-layout')),
      );
    });
  return pendingApply;
}
