// Shared metadata for the native desktop packages. Cargo owns the version;
// packaging needs no frontend build, npm dependency tree, or web assets.
import { copyFileSync, mkdirSync, readFileSync, realpathSync } from 'node:fs';
import { join, resolve } from 'node:path';

export const projectRoot = realpathSync(join(import.meta.dirname, '..'));
export const desktopRoot = join(projectRoot, 'src-tauri');
export const version = readFileSync(join(desktopRoot, 'Cargo.toml'), 'utf8')
  .match(/^version = "([\d.]+(?:-[\w.]+)?)"$/m)?.[1];
if (!version) throw new Error('Native Cargo package has no valid version');
export const targetRoot = process.env.CARGO_TARGET_DIR
  ? resolve(projectRoot, process.env.CARGO_TARGET_DIR)
  : join(desktopRoot, 'target');

export function stageProjectNotices(directory) {
  mkdirSync(directory, { recursive: true });
  copyFileSync(join(projectRoot, 'LICENSE'), join(directory, 'noBS-CAD-LICENSE.txt'));
  copyFileSync(join(projectRoot, 'THIRD_PARTY_NOTICES.md'), join(directory, 'THIRD_PARTY_NOTICES.md'));
}
