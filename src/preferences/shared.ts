import type { ThemePreference } from '../theme';
import type { SupportedLocale } from '../i18n/locales';

/** Optional fields distinguish an explicit choice from a detected default. */
export interface SharedPreferences {
  theme?: ThemePreference;
  locale?: SupportedLocale;
  six_dof_speed?: number;
}
export interface LegacyPreferences {
  theme?: string;
  locale?: string;
  six_dof_speed?: string;
}
export interface PreferencePorts {
  load(): Promise<SharedPreferences>;
  importLegacy(values: LegacyPreferences): Promise<SharedPreferences>;
  patch(values: SharedPreferences): Promise<SharedPreferences>;
  apply(values: SharedPreferences): void;
  status(error: string | null): void;
}

const fields = ['theme', 'locale', 'six_dof_speed'] as const;
type Field = typeof fields[number];
type Edit = {version: number; value: SharedPreferences[Field]};

/** Serialize shared reads/writes while keeping newer local edits above older
 * responses. A failed save remains a visible live choice until explicit retry;
 * polling never silently erases it or repeatedly retries a broken destination. */
export class PreferenceCoordinator {
  private queue: Promise<void> = Promise.resolve();
  private stopped = false;
  private started = false;
  private writing = false;
  private refreshing = false;
  private version = 0;
  private dirty = new Map<Field, Edit>();
  private saved: SharedPreferences | null = null;
  private legacy: LegacyPreferences | null = null;
  private readError: string | null = null;
  private writeError: string | null = null;

  constructor(private readonly ports: PreferencePorts) {}

  start(legacy: LegacyPreferences): Promise<void> {
    if (this.started || this.stopped) return this.queue;
    this.started = true;
    this.legacy = {...legacy};
    return this.enqueue(async () => {
      try {
        this.accept(await this.readShared());
      } catch (error) {
        this.readError = String(error);
        this.report();
      }
    });
  }

  patch(values: SharedPreferences): Promise<void> {
    if (this.stopped) return this.queue;
    for (const field of fields) {
      if (values[field] !== undefined) {
        this.dirty.set(field, {version: ++this.version, value: values[field]});
      }
    }
    return this.retry();
  }

  retry(): Promise<void> {
    if (this.stopped || this.writing) return this.queue;
    if (this.dirty.size === 0) return this.refresh();
    this.writing = true;
    return this.enqueue(async () => {
      try {
        while (!this.stopped && this.dirty.size > 0) {
          // Changes queued while a write is in flight collapse into the latest
          // value of each field, avoiding a stale slider-value write backlog.
          const captured = new Map(this.dirty);
          const patch = Object.fromEntries(
            [...captured].map(([field, edit]) => [field, edit.value]),
          ) as SharedPreferences;
          try {
            const saved = await this.ports.patch(patch);
            for (const [field, edit] of captured) {
              if (this.dirty.get(field)?.version === edit.version) this.dirty.delete(field);
            }
            if (this.dirty.size === 0) this.writeError = null;
            this.accept(saved);
          } catch (error) {
            this.writeError = String(error);
            this.report();
            break;
          }
        }
      } finally {
        this.writing = false;
      }
    });
  }

  refresh(): Promise<void> {
    if (this.stopped || this.refreshing) return this.queue;
    this.refreshing = true;
    return this.enqueue(async () => {
      try {
        this.accept(await this.readShared());
      } catch (error) {
        this.readError = String(error);
        this.report();
      } finally {
        this.refreshing = false;
      }
    });
  }

  dispose(): void {
    this.stopped = true;
  }

  private async readShared(): Promise<SharedPreferences> {
    if (this.legacy !== null) {
      // A transient first-launch lock/read error must not abandon migration
      // and replace explicit legacy choices with missing-field defaults.
      const saved = await this.ports.importLegacy(this.legacy);
      this.legacy = null;
      return saved;
    }
    return this.ports.load();
  }

  private enqueue(operation: () => Promise<void>): Promise<void> {
    this.queue = this.queue.then(async () => {
      if (!this.stopped) await operation();
    });
    return this.queue;
  }

  private accept(saved: SharedPreferences): void {
    if (this.stopped) return;
    this.saved = {...saved};
    this.readError = null;
    const visible = {...this.saved};
    for (const [field, edit] of this.dirty) {
      Object.assign(visible, {[field]: edit.value});
    }
    this.ports.apply(visible);
    this.report();
  }

  private report(): void {
    if (!this.stopped) this.ports.status(this.writeError ?? this.readError);
  }
}
