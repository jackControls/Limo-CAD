import React, { useEffect } from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { BevyUiParityLab } from './dev/BevyUiParityLab';
import { I18nProvider } from './i18n';
import { useLocaleStore } from './i18n/localeStore';
import { startSessionBridge } from './sessionBridge';
import { useAppStore } from './store/appStore';
import { applyUiScale, DEFAULT_UI_SCALE } from './uiScale';
import './index.css';

// E2E/debug handle (harmless in production): lets automation read app state.
declare global {
  interface Window {
    __appStore?: typeof useAppStore;
  }
}
window.__appStore = useAppStore;
startSessionBridge();
// Apply the saved zoom before the first render so the shell does not appear at
// 100% and then jump.
const savedUiScale = useAppStore.getState().uiScale;
if (savedUiScale !== DEFAULT_UI_SCALE) {
  void applyUiScale(savedUiScale).catch((error) => {
    console.warn('Could not apply UI scale', error);
  });
}

const showBevyUiLab =
  import.meta.env.DEV &&
  new URLSearchParams(window.location.search).has('bevy-ui-lab');
const RootComponent = showBevyUiLab ? BevyUiParityLab : App;

function AppRoot() {
  const locale = useLocaleStore((s) => s.locale);
  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);
  return (
    <I18nProvider locale={locale}>
      <RootComponent />
    </I18nProvider>
  );
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <AppRoot />
  </React.StrictMode>,
);
