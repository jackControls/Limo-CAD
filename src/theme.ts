import { persistDesktopPreference } from './preferences/desktop';
import { THEME_STORAGE_KEY } from './preferences/keys';
export { THEME_STORAGE_KEY } from './preferences/keys';

export type ThemePreference = 'system' | 'light' | 'dark';
export type ResolvedTheme = 'light' | 'dark';

export const SYSTEM_DARK_QUERY = '(prefers-color-scheme: dark)';

const isThemePreference = (value: string | null): value is ThemePreference =>
  value === 'system' || value === 'light' || value === 'dark';

/** Missing or invalid preferences intentionally fall back to the OS. */
export function readThemePreference(): ThemePreference {
  if (typeof window === 'undefined') return 'system';
  try {
    const stored = window.localStorage.getItem(THEME_STORAGE_KEY);
    return isThemePreference(stored) ? stored : 'system';
  } catch {
    return 'system';
  }
}

export function resolveTheme(
  preference: ThemePreference,
  systemDark = typeof window !== 'undefined' &&
    window.matchMedia?.(SYSTEM_DARK_QUERY).matches,
): ResolvedTheme {
  return preference === 'system' ? (systemDark ? 'dark' : 'light') : preference;
}

/** Apply both CSS token selection and native form/control color scheme. */
export function applyThemePreference(preference: ThemePreference): ResolvedTheme {
  const resolved = resolveTheme(preference);
  if (typeof document !== 'undefined') {
    const root = document.documentElement;
    // Shared preference polling must not trigger the native viewport's DOM
    // layout observer when the preference and resolved OS theme are unchanged.
    if (root.dataset.themePreference !== preference) root.dataset.themePreference = preference;
    if (root.dataset.theme !== resolved) root.dataset.theme = resolved;
    if (root.style.colorScheme !== resolved) root.style.colorScheme = resolved;
  }
  return resolved;
}

export function persistThemePreference(preference: ThemePreference): void {
  if (typeof window === 'undefined') return;
  try {
    window.localStorage.setItem(THEME_STORAGE_KEY, preference);
  } catch {
    // A locked-down webview can deny storage. The live preference still works.
  }
  persistDesktopPreference({theme: preference});
}
