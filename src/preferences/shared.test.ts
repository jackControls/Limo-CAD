import { PreferenceCoordinator, type LegacyPreferences, type SharedPreferences } from './shared';
import { applyThemePreference, persistThemePreference, readThemePreference, THEME_STORAGE_KEY } from '../theme';
import { persistSixDofSpeed, readSixDofSpeed, SIX_DOF_SPEED_STORAGE_KEY } from '../navigationPreferences';
import { detectLocale, detectBrowserLocale, persistLocale, LOCALE_STORAGE_KEY } from '../i18n/locales';

function same(actual: unknown, expected: unknown, message: string) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`${message}: ${JSON.stringify(actual)} !== ${JSON.stringify(expected)}`);
  }
}
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>(done => { resolve = done; });
  return {promise, resolve};
}
const tick = async () => { await Promise.resolve(); await Promise.resolve(); };

function fixture(initial: SharedPreferences = {}) {
  const state = {
    saved: {...initial},
    applied: [] as SharedPreferences[],
    errors: [] as (string | null)[],
    writes: [] as SharedPreferences[],
    imported: [] as LegacyPreferences[],
    reads: 0,
    fail: false,
    failImport: false,
    load: null as Promise<SharedPreferences> | null,
    importing: null as Promise<SharedPreferences> | null,
    writing: null as Promise<void> | null,
  };
  const coordinator = new PreferenceCoordinator({
    async importLegacy(values) {
      state.imported.push({...values});
      if (state.failImport) throw new Error('Preference file is busy');
      return state.importing ? await state.importing : {...state.saved};
    },
    async load() {
      state.reads++;
      return state.load ? await state.load : {...state.saved};
    },
    async patch(values) {
      state.writes.push({...values});
      if (state.writing) await state.writing;
      if (state.fail) throw new Error('Preference destination is read only');
      state.saved = {...state.saved, ...values};
      return {...state.saved};
    },
    apply(values) { state.applied.push({...values}); },
    status(error) { state.errors.push(error); },
  });
  return {state, coordinator};
}

async function run() {
  {
    const oldWindow = Object.getOwnPropertyDescriptor(globalThis, 'window');
    const oldDocument = Object.getOwnPropertyDescriptor(globalThis, 'document');
    let writes = 0;
    let dark = false;
    const observed = () => new Proxy<Record<string, string>>({}, {
      set(target, key, value) { writes++; target[String(key)] = String(value); return true; },
    });
    const root = {dataset: observed(), style: observed()};
    Object.defineProperty(globalThis, 'window', {value: {matchMedia: () => ({matches: dark})}, configurable: true});
    Object.defineProperty(globalThis, 'document', {value: {documentElement: root}, configurable: true});
    try {
      same(applyThemePreference('system'), 'light', 'System theme starts from current OS');
      same(writes, 3, 'First theme installs preference, resolved token and color scheme');
      for (let poll = 0; poll < 20; poll++) applyThemePreference('system');
      same(writes, 3, 'Identical shared snapshots produce no DOM/layout-observer work');
      dark = true;
      same(applyThemePreference('system'), 'dark', 'Unchanged System preference still reacts to OS change');
      same(writes, 5, 'OS change updates only resolved theme and color scheme');
      applyThemePreference('dark');
      same(writes, 6, 'Explicit matching theme changes only stored preference attribute');
    } finally {
      if (oldWindow) Object.defineProperty(globalThis, 'window', oldWindow); else Reflect.deleteProperty(globalThis, 'window');
      if (oldDocument) Object.defineProperty(globalThis, 'document', oldDocument); else Reflect.deleteProperty(globalThis, 'document');
    }
  }
  {
    const {state, coordinator} = fixture({locale: 'es'});
    state.failImport = true;
    await coordinator.start({locale: 'es'});
    same(state.applied, [], 'Failed initial import does not replace explicit legacy choices with defaults');
    state.failImport = false;
    await coordinator.refresh();
    same(state.imported, [{locale: 'es'}, {locale: 'es'}], 'Transient startup failure retries missing-only import');
    same(state.reads, 0, 'Load cannot skip uncompleted legacy import');
    same(state.applied, [{locale: 'es'}], 'Recovered import applies saved preferences');
  }
  {
    const {state, coordinator} = fixture({theme: 'light', locale: 'es'});
    const importing = deferred<SharedPreferences>();
    state.importing = importing.promise;
    const boot = coordinator.start({theme: 'dark', locale: 'es'});
    const edit = coordinator.patch({theme: 'dark'});
    importing.resolve({theme: 'light', locale: 'es'});
    await boot;
    await edit;
    same(state.imported, [{theme: 'dark', locale: 'es'}], 'Startup forwards explicit legacy values once');
    same(state.applied.map(value => value.theme), ['dark', 'dark'], 'Delayed bootstrap cannot overwrite a new user choice');
    same(state.writes, [{theme: 'dark'}], 'User edit patches only its own field');
    await coordinator.start({theme: 'system'});
    same(state.imported.length, 1, 'Repeated startup cannot import old values again');
  }

  {
    const {state, coordinator} = fixture({theme: 'system', six_dof_speed: 1.5});
    await coordinator.start({});
    const writing = deferred<void>();
    state.writing = writing.promise;
    const save = coordinator.patch({six_dof_speed: 1});
    await tick();
    void coordinator.patch({six_dof_speed: 2});
    void coordinator.patch({six_dof_speed: 2.5, locale: 'de'});
    writing.resolve();
    await save;
    same(state.writes, [{six_dof_speed: 1}, {six_dof_speed: 2.5, locale: 'de'}], 'In-flight slider updates coalesce to the final field choices');
    same(state.applied.slice(1).map(value => value.six_dof_speed), [2.5, 2.5], 'Earlier write acknowledgement cannot flash an obsolete slider value');
  }
  {
    const {state, coordinator} = fixture({theme: 'system'});
    await coordinator.start({});
    const loading = deferred<SharedPreferences>();
    state.load = loading.promise;
    const refresh = coordinator.refresh();
    await tick();
    const edit = coordinator.patch({theme: 'dark'});
    loading.resolve({theme: 'light', locale: 'zh-CN'});
    await refresh;
    await edit;
    same(state.applied[1], {theme: 'dark', locale: 'zh-CN'}, 'Slow external refresh merges unrelated remote fields without discarding current local input');
  }
  {
    const {state, coordinator} = fixture({theme: 'light', locale: 'en'});
    await coordinator.start({});
    state.fail = true;
    await coordinator.patch({theme: 'dark'});
    same(state.writes.length, 1, 'Failed save is attempted once');
    state.saved.locale = 'de';
    await coordinator.refresh();
    same(state.applied[state.applied.length - 1], {theme: 'dark', locale: 'de'}, 'Polling retains unsaved live field while accepting unrelated remote change');
    same(state.writes.length, 1, 'Polling does not retry a failed write');
    if (!state.errors[state.errors.length - 1]?.includes('read only')) throw new Error('Read success concealed unsaved preference failure');
    state.fail = false;
    await coordinator.retry();
    same(state.saved, {theme: 'dark', locale: 'de'}, 'Explicit retry preserves the remote field');
    same(state.errors[state.errors.length - 1], null, 'Successful save clears the reported failure');
  }
  {
    const {state, coordinator} = fixture({theme: 'dark'});
    await coordinator.start({});
    const loading = deferred<SharedPreferences>();
    state.load = loading.promise;
    const refresh = coordinator.refresh();
    void coordinator.refresh();
    void coordinator.refresh();
    await tick();
    same(state.reads, 1, 'Repeated event/poll hints create one pending read');
    coordinator.dispose();
    loading.resolve({theme: 'light'});
    await refresh;
    await coordinator.patch({locale: 'es'});
    same(state.applied, [{theme: 'dark'}], 'Disposed observer cannot publish a late snapshot');
    same(state.writes.length, 0, 'Disposed observer cannot schedule writes');
  }
  {
    // Exercise the actual public browser APIs, not a second preference model.
    const oldWindow = Object.getOwnPropertyDescriptor(globalThis, 'window');
    const values = new Map<string, string>();
    const fake = {
      navigator: {language: 'de-DE'},
      localStorage: {
        getItem: (key: string) => values.get(key) ?? null,
        setItem: (key: string, value: string) => { values.set(key, value); },
      },
    };
    Object.defineProperty(globalThis, 'window', {value: fake, configurable: true});
    try {
      same(readThemePreference(), 'system', 'Browser default theme preserved');
      same(readSixDofSpeed(), 1.5, 'Browser default 6DoF speed preserved');
      same(detectLocale(), 'de', 'Browser language detection preserved');
      persistThemePreference('dark');
      persistSixDofSpeed(7);
      persistLocale('zh-CN');
      same([...values], [[THEME_STORAGE_KEY, 'dark'], [SIX_DOF_SPEED_STORAGE_KEY, '3'], [LOCALE_STORAGE_KEY, 'zh-CN']], 'Browser setters retain existing storage keys and normalization');
      same(readThemePreference(), 'dark', 'Browser stored theme read');
      same(readSixDofSpeed(), 3, 'Browser stored speed read');
      same(detectLocale(), 'zh-CN', 'Explicit locale wins');
      same(detectBrowserLocale(), 'de', 'Missing shared locale uses detection independently of legacy cache');
      fake.localStorage.getItem = () => { throw new Error('Storage blocked'); };
      fake.localStorage.setItem = () => { throw new Error('Storage blocked'); };
      persistThemePreference('light');
      persistLocale('es');
      same(persistSixDofSpeed(Number.NaN), 1.5, 'Blocked browser storage still permits live defaults');
      same(readThemePreference(), 'system', 'Blocked browser storage keeps existing fallback');
    } finally {
      if (oldWindow) Object.defineProperty(globalThis, 'window', oldWindow);
      else Reflect.deleteProperty(globalThis, 'window');
    }
  }
  console.log('Shared desktop preferences: bootstrap races, field merges, failed-save recovery, observers, and browser fallback passed.');
}

void run().catch(error => { console.error(error); throw error; });
