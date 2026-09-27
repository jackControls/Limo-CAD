import { useAppStore } from '../store/appStore';
import { useLocaleStore } from '../i18n/localeStore';
import { applyThemePreference } from '../theme';
import { detectBrowserLocale } from '../i18n/locales';
import { DEFAULT_SIX_DOF_SPEED } from '../navigationPreferences';
import { observeDesktopPreferences } from './desktop';

/** Apply remote values without calling setters that would echo shared writes.
 * Application preferences neither load a model nor modify document history. */
export function startDesktopPreferences(): () => void {
  return observeDesktopPreferences(values => {
    const themePreference = values.theme ?? 'system';
    const resolvedTheme = applyThemePreference(themePreference);
    const sixDofSpeed = values.six_dof_speed ?? DEFAULT_SIX_DOF_SPEED;
    const state = useAppStore.getState();
    if (state.themePreference !== themePreference || state.resolvedTheme !== resolvedTheme || state.sixDofSpeed !== sixDofSpeed) {
      useAppStore.setState({themePreference, resolvedTheme, sixDofSpeed});
    }
    const locale = values.locale ?? detectBrowserLocale();
    if (useLocaleStore.getState().locale !== locale) useLocaleStore.setState({locale});
  });
}
