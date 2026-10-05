import { getCurrentWebview } from '@tauri-apps/api/webview';
import { isTauriRuntime } from './engine';

export const UI_SCALE_STORAGE_KEY = 'nbcad.uiScale';
export const DEFAULT_UI_SCALE = 1;
// Discrete steps instead of a slider: every step re-lays out the whole window,
// including the settings dialog, so a dragged thumb would jump under the cursor.
export const UI_SCALE_OPTIONS = [0.9, 1, 1.1, 1.25, 1.5, 1.75] as const;

let appliedUiScale = DEFAULT_UI_SCALE;
let pendingApply: Promise<void> = Promise.resolve();

/** Map any finite number onto a button the settings dialog can show as selected. */
export function snapUiScale(value: number): number {
  if (!Number.isFinite(value)) return DEFAULT_UI_SCALE;
  let best: number = DEFAULT_UI_SCALE;
  let bestDistance = Number.POSITIVE_INFINITY;
  for (const option of UI_SCALE_OPTIONS) {
    const distance = Math.abs(option - value);
    if (distance < bestDistance) {
      best = option;
      bestDistance = distance;
    }
  }
  return best;
}

export function stepUiScale(current: number, direction: -1 | 1): number {
  const snapped = snapUiScale(current);
  const index = UI_SCALE_OPTIONS.indexOf(snapped as (typeof UI_SCALE_OPTIONS)[number]);
  const next = index + direction;
  if (index < 0 || next < 0 || next >= UI_SCALE_OPTIONS.length) return snapped;
  return UI_SCALE_OPTIONS[next];
}

export function readUiScale(): number {
  if (typeof window === 'undefined') return DEFAULT_UI_SCALE;
  try {
    const stored = Number.parseFloat(
      window.localStorage.getItem(UI_SCALE_STORAGE_KEY) ?? '',
    );
    return Number.isFinite(stored) ? snapUiScale(stored) : DEFAULT_UI_SCALE;
  } catch {
    return DEFAULT_UI_SCALE;
  }
}

export function persistUiScale(value: number): number {
  const clamped = snapUiScale(value);
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
function refreshNativeViewportLayout(): void {
  document.dispatchEvent(new Event('nbcad:native-viewport-layout'));
}

/** Page zoom changes `clientWidth` only after the webview lays out again. */
function waitForZoomedWidth(expected: number): Promise<void> {
  if (!Number.isFinite(expected) || expected <= 0) return Promise.resolve();
  const tolerance = Math.max(2, Math.abs(expected) * 0.02);
  return new Promise((resolve) => {
    const start = performance.now();
    let settled = false;
    const finish = () => {
      if (settled) return;
      settled = true;
      resolve();
    };
    const check = () => {
      if (settled) return;
      const width = document.documentElement?.clientWidth ?? expected;
      if (Math.abs(width - expected) <= tolerance || performance.now() - start > 150) {
        finish();
        return;
      }
      requestAnimationFrame(check);
    };
    check();
    window.setTimeout(finish, 180);
  });
}

export function applyUiScale(value: number, force = false): Promise<void> {
  const scale = snapUiScale(value);
  if (!isTauriRuntime()) {
    appliedUiScale = scale;
    return Promise.resolve();
  }
  // Serialise calls so a quick series of choices cannot finish out of order
  // and leave the native viewport using a factor the page no longer has.
  pendingApply = pendingApply
    .catch(() => undefined)
    .then(async () => {
      const previous = appliedUiScale;
      const unchanged = Math.abs(previous - scale) < 0.001;
      if (unchanged && !force) return;
      const beforeWidth = document.documentElement?.clientWidth ?? 0;
      await getCurrentWebview().setZoom(scale);
      appliedUiScale = scale;
      if (!unchanged && beforeWidth > 0 && previous > 0) {
        await waitForZoomedWidth(beforeWidth * (previous / scale));
      }
      refreshNativeViewportLayout();
      requestAnimationFrame(() => {
        if (Math.abs(appliedUiScale - scale) < 0.001) refreshNativeViewportLayout();
      });
    });
  return pendingApply;
}
