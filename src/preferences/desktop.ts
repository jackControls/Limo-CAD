import { PreferenceCoordinator, type LegacyPreferences, type SharedPreferences } from './shared';
import { LOCALE_STORAGE_KEY, SIX_DOF_SPEED_STORAGE_KEY, THEME_STORAGE_KEY } from './keys';

let coordinator: PreferenceCoordinator | null = null;
let error: string | null = null;
let snapshot: SharedPreferences | null = null;
const listeners = new Set<(values: SharedPreferences) => void>();
const errorListeners = new Set<() => void>();

// This tiny runtime check avoids importing the document engine into theme and
// locale initialization, which would create a cycle through the app store.
function desktop(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}
function legacy(): LegacyPreferences {
  const values: LegacyPreferences = {};
  for (const [field, key] of [
    ['theme', THEME_STORAGE_KEY],
    ['locale', LOCALE_STORAGE_KEY],
    ['six_dof_speed', SIX_DOF_SPEED_STORAGE_KEY],
  ] as const) {
    try {
      const value = window.localStorage.getItem(key);
      if (value !== null) values[field] = value;
    } catch { /* Locked WebView storage has no explicit legacy value. */ }
  }
  return values;
}
function instance(): PreferenceCoordinator | null {
  if (!desktop()) return null;
  if (coordinator) return coordinator;
  const invoke = async <T>(command: string, args?: Record<string, unknown>): Promise<T> =>
    (await import('@tauri-apps/api/core')).invoke<T>(command, args);
  coordinator = new PreferenceCoordinator({
    load: () => invoke('app_preferences_load'),
    importLegacy: values => invoke('app_preferences_import_legacy', {legacy: values}),
    patch: values => invoke('app_preferences_patch', {patch: values}),
    apply: values => {
      snapshot = values;
      // Keep the existing WebView cache useful during asynchronous startup and
      // for older desktop builds; this does not publish another shared write.
      for (const [field, key] of [
        ['theme', THEME_STORAGE_KEY],
        ['locale', LOCALE_STORAGE_KEY],
        ['six_dof_speed', SIX_DOF_SPEED_STORAGE_KEY],
      ] as const) {
        if (values[field] !== undefined) {
          try { window.localStorage.setItem(key, String(values[field])); } catch { /* Live state still works. */ }
        }
      }
      for (const listener of listeners) listener(values);
    },
    status: next => {
      if (error === next) return;
      error = next;
      for (const listener of errorListeners) listener();
    },
  });
  void coordinator.start(legacy());
  return coordinator;
}

/** Existing preference setters keep their synchronous browser behavior. */
export function persistDesktopPreference(patch: SharedPreferences): void {
  void instance()?.patch(patch);
}
export function retryDesktopPreferences(): void {
  void instance()?.retry();
}
export function desktopPreferenceError(): string | null {
  return error;
}
export function subscribeDesktopPreferenceError(listener: () => void): () => void {
  errorListeners.add(listener);
  return () => { errorListeners.delete(listener); };
}

/** Event payloads are invalidations, never ordered snapshots. Polling also
 * observes native-host/other-process writers sharing the same config folder. */
export function observeDesktopPreferences(apply: (values: SharedPreferences) => void): () => void {
  const current = instance();
  if (!current) return () => {};
  listeners.add(apply);
  if (snapshot) apply(snapshot);
  let disposed = false;
  let unlisten: (() => void) | null = null;
  const refresh = () => { if (!disposed) void current.refresh(); };
  const visible = () => { if (document.visibilityState !== 'hidden') refresh(); };
  const timer = window.setInterval(visible, 500);
  window.addEventListener('focus', refresh);
  document.addEventListener('visibilitychange', visible);
  void import('@tauri-apps/api/event').then(async ({listen}) => {
    const stop = await listen('app-preferences-changed', refresh);
    if (disposed) stop(); else unlisten = stop;
  }).catch(() => {
    // Polling/focus remain active when an event subscription is unavailable.
  });
  return () => {
    disposed = true;
    listeners.delete(apply);
    window.clearInterval(timer);
    window.removeEventListener('focus', refresh);
    document.removeEventListener('visibilitychange', visible);
    unlisten?.();
  };
}
