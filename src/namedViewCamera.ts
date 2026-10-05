import type { ViewCameraDto } from './engine/types';

export interface NamedViewCameraApi {
  restore(camera: ViewCameraDto): void;
}

let pendingStop: (() => void) | null = null;

export function cancelNamedViewCameraRestore(): void {
  pendingStop?.();
  pendingStop = null;
}

export function armNamedViewCameraRestore(
  camera: ViewCameraDto,
  getCamera: () => NamedViewCameraApi | null,
  subscribe: (listener: () => void) => () => void,
  isCurrent: () => boolean = () => true,
): void {
  cancelNamedViewCameraRestore();
  if (!isCurrent()) return;
  const current = getCamera();
  if (current) {
    current.restore(camera);
    return;
  }
  let stop = () => {};
  let applied = false;
  stop = subscribe(() => {
    if (applied) return;
    if (!isCurrent()) {
      applied = true;
      stop();
      if (pendingStop === stop) pendingStop = null;
      return;
    }
    const api = getCamera();
    if (!api) return;
    applied = true;
    stop();
    if (pendingStop === stop) pendingStop = null;
    api.restore(camera);
  });
  pendingStop = stop;
}
