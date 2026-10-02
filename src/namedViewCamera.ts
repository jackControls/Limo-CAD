import type { ViewCameraDto } from './engine/types';

export interface NamedViewCameraApi {
  restore(camera: ViewCameraDto): void;
}

let pendingStop: (() => void) | null = null;

/** Apply this camera, cancelling a restore that is still waiting for the viewport. */
export function armNamedViewCameraRestore(
  camera: ViewCameraDto,
  getCamera: () => NamedViewCameraApi | null,
  subscribe: (listener: () => void) => () => void,
): void {
  pendingStop?.();
  pendingStop = null;
  const apply = () => {
    const api = getCamera();
    if (!api) return false;
    api.restore(camera);
    return true;
  };
  if (apply()) return;
  let stop = () => {};
  stop = subscribe(() => {
    if (!apply()) return;
    stop();
    if (pendingStop === stop) pendingStop = null;
  });
  pendingStop = stop;
}
