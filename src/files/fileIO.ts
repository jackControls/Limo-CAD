

import { translate } from '../i18n';

const MAX_FILE_BYTES = 256 * 1024 * 1024;

export interface SaveType {
  description: string;
  /** Dotted i18n key; when present the picker/dialog label follows the locale. */
  descriptionKey?: string;
  extension: string;
  alternateExtensions?: string[];
  mime: string;
}

type BrowserTarget = { kind: 'browser'; handle: FileSystemFileHandle; name: string };
type DownloadTarget = { kind: 'download'; name: string };
export type SaveTarget = BrowserTarget | DownloadTarget;

export interface OpenedFile {
  name: string;
  bytes: Uint8Array;
  writableTarget: SaveTarget | null;
}

interface PickerWindow extends Window {
  showOpenFilePicker?: (options: {
    multiple?: boolean;
    types?: Array<{
      description: string;
      accept: Record<string, string[]>;
    }>;
  }) => Promise<FileSystemFileHandle[]>;
  showSaveFilePicker?: (options: {
    suggestedName?: string;
    types?: Array<{
      description: string;
      accept: Record<string, string[]>;
    }>;
  }) => Promise<FileSystemFileHandle>;
}

function withExtension(name: string, extension: string): string {
  return name.toLowerCase().endsWith(extension.toLowerCase()) ? name : `${name}${extension}`;
}

function saveTypeLabel(type: SaveType): string {
  return type.descriptionKey ? translate(type.descriptionKey) : type.description;
}

function pickerType(type: SaveType) {
  return {
    description: saveTypeLabel(type),
    accept: {
      [type.mime]: [type.extension, ...(type.alternateExtensions ?? [])],
    },
  };
}

export async function chooseSaveTarget(
  suggestedName: string,
  type: SaveType,
): Promise<SaveTarget | null> {
  const fileName = withExtension(suggestedName, type.extension);

  const picker = window as PickerWindow;
  if (picker.showSaveFilePicker) {
    try {
      const handle = await picker.showSaveFilePicker({
        suggestedName: fileName,
        types: [pickerType(type)],
      });
      return { kind: 'browser', handle, name: handle.name };
    } catch (error) {
      if (error instanceof DOMException && error.name === 'AbortError') return null;
      throw error;
    }
  }
  return { kind: 'download', name: fileName };
}

export async function writeSaveTarget(target: SaveTarget, bytes: Uint8Array): Promise<void> {
  if (bytes.byteLength > MAX_FILE_BYTES) {
    throw new Error(translate('file.errorFileTooLarge'));
  }

  // File APIs accept ArrayBuffer views, not shared memory. Preserve the view's
  // byte range without copying ordinary project buffers.
  const buffer = bytes.buffer;
  const fileBytes = buffer instanceof ArrayBuffer
    ? new Uint8Array(buffer, bytes.byteOffset, bytes.byteLength)
    : new Uint8Array(bytes);

  if (target.kind === 'browser') {
    const writable = await target.handle.createWritable();
    try {
      await writable.write(fileBytes);
      await writable.close();
    } catch (error) {
      await writable.abort().catch(() => undefined);
      throw error;
    }
    return;
  }
  const blob = new Blob([fileBytes], { type: 'application/octet-stream' });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = target.name;
  anchor.click();
  setTimeout(() => URL.revokeObjectURL(url), 1_000);
}

export async function chooseOpenFile(
  type: SaveType,
  pathOverride?: string,
): Promise<OpenedFile | null> {
  if (pathOverride) {
    const response = await fetch(pathOverride);
    if (!response.ok) {
      throw new Error(`Could not read ${pathOverride}`);
    }
    const bytes = new Uint8Array(await response.arrayBuffer());
    if (bytes.byteLength > MAX_FILE_BYTES) {
      throw new Error(translate('file.errorFileTooLarge'));
    }
    const name = pathOverride.split(/[/\\]/).pop() || pathOverride;
    return { name, bytes, writableTarget: null };
  }

  const picker = window as PickerWindow;
  if (picker.showOpenFilePicker) {
    try {
      const [handle] = await picker.showOpenFilePicker({
        multiple: false,
        types: [pickerType(type)],
      });
      if (!handle) return null;
      const file = await handle.getFile();
      if (file.size > MAX_FILE_BYTES) {
        throw new Error(translate('file.errorFileTooLarge'));
      }
      return {
        name: file.name,
        bytes: new Uint8Array(await file.arrayBuffer()),
        writableTarget: { kind: 'browser', handle, name: handle.name },
      };
    } catch (error) {
      if (error instanceof DOMException && error.name === 'AbortError') return null;
      throw error;
    }
  }

  return new Promise((resolve, reject) => {
    const input = document.createElement('input');
    let settled = false;
    let cancelTimer: number | null = null;
    const finish = (value: OpenedFile | null) => {
      if (settled) return;
      settled = true;
      if (cancelTimer !== null) window.clearTimeout(cancelTimer);
      window.removeEventListener('focus', onWindowFocus);
      input.remove();
      resolve(value);
    };
    const fail = (error: unknown) => {
      if (settled) return;
      settled = true;
      if (cancelTimer !== null) window.clearTimeout(cancelTimer);
      window.removeEventListener('focus', onWindowFocus);
      input.remove();
      reject(error);
    };
    const onWindowFocus = () => {
      // Older browsers do not emit the input `cancel` event. File chooser
      // selection dispatches `change` first; the short delay lets it win.
      cancelTimer = window.setTimeout(() => {
        if (!input.files?.length) finish(null);
      }, 100);
    };
    input.type = 'file';
    input.accept = [type.extension, ...(type.alternateExtensions ?? [])].join(',');
    input.hidden = true;
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) {
        finish(null);
        return;
      }
      if (file.size > MAX_FILE_BYTES) {
        fail(new Error(translate('file.errorFileTooLarge')));
        return;
      }
      try {
        finish({
          name: file.name,
          bytes: new Uint8Array(await file.arrayBuffer()),
          writableTarget: null,
        });
      } catch (error) {
        fail(error);
      }
    };
    input.oncancel = () => finish(null);
    window.addEventListener('focus', onWindowFocus);
    document.body.append(input);
    input.click();
  });
}
