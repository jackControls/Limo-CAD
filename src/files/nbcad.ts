import { ensureWasm } from '../engine-wasm/runtime';
import { translate } from '../i18n';

export const NBCAD_EXTENSION = '.nbcad';
export const NBCAD_FORMAT = 'nbcad-project';
export const NBCAD_CONTAINER_VERSION = 1;
export const LEGACY_PROJECT_EXTENSION = '.tfcad';
export const LEGACY_PROJECT_FORMAT = 'tfcad-project';
const APPLICATION_VERSION = '0.2.2';

export interface NbcadManifest {
  format: string;
  container_version: typeof NBCAD_CONTAINER_VERSION;
  model: 'model.json';
  model_schema_version?: number;
  application: string;
  application_version: string;
  saved_at: string;
}

/** ZIP limits, schema checks and codec are shared with the native desktop. */
export async function createNbcadArchive(modelJson: string): Promise<Uint8Array> {
  const { project_archive_encode } = await ensureWasm();
  try {
    return project_archive_encode(modelJson, APPLICATION_VERSION, new Date().toISOString());
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    throw new Error(detail.includes('the engine produced an invalid project model')
      ? translate('file.errorEngineInvalidModel') : detail);
  }
}

export async function readNbcadArchive(bytes: Uint8Array): Promise<{
  manifest: NbcadManifest;
  modelJson: string;
}> {
  const { project_archive_decode } = await ensureWasm();
  try {
    return JSON.parse(project_archive_decode(bytes)) as { manifest: NbcadManifest; modelJson: string };
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    throw new Error(translate('file.errorArchiveDamaged').replace('{detail}', detail));
  }
}
